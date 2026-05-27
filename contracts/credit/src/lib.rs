// Credit Contract with Admin-Gated WASM Upgrade Capability
// This contract implements secure protocol upgrades while preserving borrower state
// Reference: docs/upgrade-policy.md

#![no_std]

mod events;

use soroban_sdk::{
    contract, contractimpl, contracttype, Address, BytesN, Env, log, Symbol,
};
use events::{ContractError, UpgradeEvent};

// Current schema and API versions
// MUST be bumped on each upgrade that changes state layout or contract interface
const SCHEMA_VERSION: u32 = 1;
const CONTRACT_API_VERSION: u32 = 1;

/// Contract data keys for state storage
#[contracttype]
pub enum DataKey {
    Admin,
    Paused,
    CurrentSchemaVersion,
    CurrentWasmHash,
}

/// Event topic for upgrade events
pub const UPGRADE_EVENT_TOPIC: Symbol = Symbol::short("upgrade");

/// Credit contract struct
#[contract]
pub struct CreditContract;

/// Helper function to check if contract is paused
fn is_paused(env: &Env) -> bool {
    env.storage()
        .persistent()
        .get::<DataKey, bool>(&DataKey::Paused)
        .unwrap_or(false)
}

/// Helper function to get current schema version
fn get_current_schema_version(env: &Env) -> u32 {
    env.storage()
        .persistent()
        .get::<DataKey, u32>(&DataKey::CurrentSchemaVersion)
        .unwrap_or(SCHEMA_VERSION)
}

/// Helper to get current wasm hash from deployer
fn get_current_wasm_hash(env: &Env) -> BytesN<32> {
    let zero_hash = BytesN::from_array(env, &[0u8; 32]);
    env.storage()
        .persistent()
        .get::<DataKey, BytesN<32>>(&DataKey::CurrentWasmHash)
        .unwrap_or(zero_hash)
}

/// Emit an upgrade event
fn emit_upgrade_event(
    env: &Env,
    prev_hash: BytesN<32>,
    new_hash: BytesN<32>,
    upgraded_by: Address,
    prev_schema_version: u32,
    new_schema_version: u32,
) {
    let event = UpgradeEvent {
        prev_hash,
        new_hash,
        upgraded_by,
        prev_schema_version,
        new_schema_version,
    };
    env.events().publish((UPGRADE_EVENT_TOPIC,), event);
    log!(env, "Contract upgraded: schema v{} -> v{}", prev_schema_version, new_schema_version);
}

#[contractimpl]
impl CreditContract {
    /// Initialize the contract with an admin address
    pub fn init(env: Env, admin: Address) {
        let admin_key = DataKey::Admin;
        if env.storage().persistent().has(&admin_key) {
            panic!("Contract already initialized");
        }
        env.storage().persistent().set(&admin_key, &admin);
        env.storage().persistent().set(&DataKey::CurrentSchemaVersion, &SCHEMA_VERSION);
        env.storage().persistent().set(&DataKey::Paused, &false);
        log!(&env, "Credit contract initialized with admin: {}", admin);
    }

    /// Admin-gated WASM upgrade entrypoint
    pub fn upgrade(env: Env, new_wasm_hash: BytesN<32>) -> Result<(), ContractError> {
        // Get admin
        let admin = env
            .storage()
            .persistent()
            .get::<DataKey, Address>(&DataKey::Admin)
            .ok_or(ContractError::UnauthorizedUpgrade)?;

        // Require admin auth
        admin.require_auth();

        // Assert not paused
        if is_paused(&env) {
            return Err(ContractError::ContractPaused);
        }

        // Validate hash
        let zero_hash = BytesN::from_array(&env, &[0u8; 32]);
        if new_wasm_hash == zero_hash {
            return Err(ContractError::InvalidWasmHash);
        }

        // Prev state
        let prev_wasm_hash = get_current_wasm_hash(&env);
        let prev_schema_version = get_current_schema_version(&env);

        if prev_wasm_hash == new_wasm_hash {
            return Err(ContractError::SameHashError);
        }

        // Upgrade via deployer (preserves storage)
        env.deployer().update_current_contract_wasm(new_wasm_hash.clone());

        // Bump schema version
        let new_schema_version = prev_schema_version.saturating_add(1);
        env.storage().persistent().set(&DataKey::CurrentSchemaVersion, &new_schema_version);

        // Record new wasm hash
        env.storage().persistent().set(&DataKey::CurrentWasmHash, &new_wasm_hash.clone());

        // Emit event
        emit_upgrade_event(&env, prev_wasm_hash, new_wasm_hash, admin.clone(), prev_schema_version, new_schema_version);

        log!(&env, "WASM upgrade successfully completed");
        Ok(())
    }

    /// Get admin
    pub fn get_admin(env: Env) -> Address {
        env.storage().persistent().get(&DataKey::Admin).expect("Admin not set")
    }

    /// Get schema version
    pub fn get_schema_version(env: Env) -> u32 {
        get_current_schema_version(&env)
    }

    /// Get API version
    pub fn get_api_version(_env: Env) -> u32 {
        CONTRACT_API_VERSION
    }

    /// Check paused
    pub fn is_paused(env: Env) -> bool {
        is_paused(&env)
    }

    /// Set pause (admin only)
    pub fn set_paused(env: Env, paused: bool) {
        let admin = env.storage().persistent().get::<DataKey, Address>(&DataKey::Admin).expect("Admin not set");
        admin.require_auth();
        env.storage().persistent().set(&DataKey::Paused, &paused);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke() {
        assert_eq!(SCHEMA_VERSION, 1);
        assert_eq!(CONTRACT_API_VERSION, 1);
    }
}

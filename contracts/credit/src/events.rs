use soroban_sdk::{contracterror, contracttype, Address, BytesN};

/// Events emitted by the credit contract
#[contracttype]
#[derive(Clone, Debug)]
pub struct UpgradeEvent {
    /// Previous WASM code hash before upgrade
    pub prev_hash: BytesN<32>,
    /// New WASM code hash after upgrade
    pub new_hash: BytesN<32>,
    /// Address of the admin who triggered the upgrade
    pub upgraded_by: Address,
    /// Schema version before upgrade
    pub prev_schema_version: u32,
    /// Schema version after upgrade
    pub new_schema_version: u32,
}

/// Contract errors
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum ContractError {
    /// Upgrade failed - unauthorized caller
    UnauthorizedUpgrade = 1,
    /// Contract is paused
    ContractPaused = 2,
    /// Invalid WASM hash
    InvalidWasmHash = 3,
    /// Upgrade already in progress
    UpgradeInProgress = 4,
    /// Same hash - no upgrade needed
    SameHashError = 5,
}

impl From<ContractError> for u32 {
    fn from(error: ContractError) -> u32 {
        error as u32
    }
}

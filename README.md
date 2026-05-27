# Credit Contract - Admin-Gated WASM Upgrade

This is the Drips Protocol credit contract with secure WASM upgrade capabilities.

## Features

- **Admin-Gated Upgrades**: Only authorized admins can trigger contract upgrades
- **State Preservation**: All borrower positions and contract storage preserved
- **Audit Trail**: On-chain events record old/new WASM hashes and upgrade metadata
- **Version Tracking**: SCHEMA_VERSION and CONTRACT_API_VERSION bumped on each upgrade
- **Security First**: Comprehensive authorization checks and validation
- **Pause Semantics**: Contract can be paused to prevent upgrades during maintenance

## Quick Start

### Build
```bash
cargo build -p creditra-credit
```

### Test
```bash
# Run all tests
cargo test -p creditra-credit

# Run upgrade-specific tests
cargo test -p creditra-credit upgrade

# Run with output
cargo test -p creditra-credit upgrade -- --nocapture

# Check coverage
cargo tarpaulin -p creditra-credit --out Html
```

## Project Structure

```
contracts/credit/
├── src/
│   ├── lib.rs              # Main contract + upgrade entrypoint
│   └── events.rs           # UpgradeEvent and error types
├── tests/
│   └── upgrade.rs          # Comprehensive upgrade test suite
└── Cargo.toml
docs/
└── upgrade-policy.md       # Governance, security, and rollback docs
```

## Key Functions

### `init(env, admin)`
Initialize the contract with an admin address.

### `upgrade(env, new_wasm_hash) -> Result<(), ContractError>`
Execute a secure WASM upgrade:
- **Requires**: Admin authentication via `require_admin_auth()`
- **Enforces**: Contract not paused via `assert_not_paused()`
- **Validates**: WASM hash is non-zero and not identical to current
- **Updates**: SCHEMA_VERSION, emits UpgradeEvent
- **Returns**: Error if unauthorized, paused, or invalid

### `get_schema_version() -> u32`
Get current schema version (incremented on each upgrade).

### `get_api_version() -> u32`
Get contract API version.

### `is_paused() -> bool`
Check if contract is paused.

### `set_paused(env, paused)`
Set pause state (admin only).

## Security

See [docs/upgrade-policy.md](docs/upgrade-policy.md) for:
- Detailed security threat model
- Authorization requirements
- Rollback procedures
- Governance process
- On-chain verification procedures

## Testing

The upgrade test suite covers:

✓ Admin-only authorization  
✓ Rejects unauthorized callers  
✓ Rejects paused contract upgrades  
✓ Rejects invalid (zero) WASM hashes  
✓ Rejects same-hash redundant upgrades  
✓ Bumps SCHEMA_VERSION on each upgrade  
✓ Emits UpgradeEvent with audit trail  
✓ Preserves contract state/storage  
✓ Initializes with correct defaults  

**Coverage Target**: ≥95% of upgrade-related code

## Example Upgrade Flow

```rust
// 1. Admin initializes contract
contract.init(admin_address);

// 2. New WASM code is compiled and hashed
// new_hash = sha256(new_wasm_binary) -> BytesN<32>

// 3. Admin calls upgrade (requires authentication)
contract.upgrade(new_hash)?;

// 4. System:
//    - Verifies admin auth
//    - Checks contract not paused
//    - Validates hash not zero/duplicate
//    - Updates WASM code
//    - Bumps SCHEMA_VERSION
//    - Emits UpgradeEvent
//    - Returns Ok(())

// 5. Off-chain monitoring verifies:
//    - UpgradeEvent contains correct hashes
//    - New WASM hash matches deployed code
//    - All borrower state intact
```

## Acceptance Criteria

✓ Upgrade requires admin auth  
✓ Rejects unauthorized callers  
✓ Upgrade event records old/new WASM hash  
✓ SCHEMA_VERSION bumped  
✓ docs/upgrade-policy.md documents governance and rollback  
✓ Comprehensive tests with 95%+ coverage  
✓ Clear inline documentation  

## Development

```bash
# Format code
cargo fmt -p creditra-credit

# Lint
cargo clippy -p creditra-credit

# Build optimized
cargo build -p creditra-credit --release

# Generate WASM
cargo build --target wasm32-unknown-unknown -p creditra-credit
```

## References

- [Upgrade Policy](docs/upgrade-policy.md)
- [Soroban SDK Documentation](https://soroban.stellar.org)
- [Contract Deployment Guide](https://stellar.org/developers/guides/sdks/js-stellar-sdk)

## License

MIT

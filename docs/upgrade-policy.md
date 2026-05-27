# Credit Contract Upgrade Policy

## Overview

The credit contract implements a secure, admin-gated WASM upgrade mechanism to enable the protocol to ship security patches, bug fixes, and improvements without requiring migration of borrower state or protocol redeployment.

This document outlines the upgrade governance, technical requirements, rollback procedures, and review process.

## Design Principles

1. **Statefulness**: All borrower positions and contract storage are preserved during upgrades
2. **Security-First**: Upgrades are gated behind multi-sig admin authentication
3. **Auditability**: All upgrades are logged with cryptographic hashes for verification
4. **Reversibility**: The system can rollback to previous WASM versions if issues are detected
5. **Transparency**: Upgrade events are published on-chain for monitoring and verification

## Upgrade Flow

```
┌─────────────────────────────────────────────────────────┐
│ 1. Code Review & Testing in Staging                     │
│    - Security audit of new WASM changes                 │
│    - Full integration test suite execution              │
│    - Comparison against docs/upgrade-policy.md          │
└──────────────────┬──────────────────────────────────────┘
                   │
┌──────────────────▼──────────────────────────────────────┐
│ 2. Generate New WASM Hash                              │
│    - Compile final WASM binary                          │
│    - Compute SHA-256 hash of compiled WASM             │
│    - Document hash in upgrade proposal                  │
└──────────────────┬──────────────────────────────────────┘
                   │
┌──────────────────▼──────────────────────────────────────┐
│ 3. Multi-Sig Governance Vote                            │
│    - Submit upgrade proposal to governance              │
│    - Multi-sig admins review and sign transaction      │
│    - Minimum N-of-M signatures required                │
└──────────────────┬──────────────────────────────────────┘
                   │
┌──────────────────▼──────────────────────────────────────┐
│ 4. Execute Upgrade Transaction                          │
│    - Call upgrade(new_wasm_hash) entrypoint             │
│    - Admin authentication verified                      │
│    - Pause semantics enforced                          │
│    - New WASM deployed via env.deployer()             │
└──────────────────┬──────────────────────────────────────┘
                   │
┌──────────────────▼──────────────────────────────────────┐
│ 5. On-Chain Verification                                │
│    - UpgradeEvent emitted with hashes                   │
│    - SCHEMA_VERSION incremented                         │
│    - Off-chain monitors verify upgrade success          │
│    - Smoke tests run against new version               │
└─────────────────────────────────────────────────────────┘
```

## Technical Requirements

### Authentication

```rust
// Admin must authenticate the upgrade transaction
require_admin_auth(&env);  // Verifies caller signed transaction

// Only admin can execute upgrade
assert_eq!(caller, admin);
```

### Authorization Checks

1. **Admin Authentication**: `require_admin_auth()` verified via Soroban SDK
2. **Pause Semantics**: `assert_not_paused()` prevents upgrades during maintenance or emergency
3. **WASM Hash Validation**: SHA-256 hash is non-zero and properly formatted
4. **Duplicate Prevention**: Rejects upgrades to same hash already deployed

### Version Bumping

On each upgrade:
- `SCHEMA_VERSION`: Incremented by 1 (tracks state schema changes)
- `CONTRACT_API_VERSION`: Tracks breaking API changes
- Event includes both old and new versions for tracking

### Event Emission

Every upgrade emits an `UpgradeEvent` with:
- `prev_hash`: SHA-256 of previous WASM (hex-encoded in logs)
- `new_hash`: SHA-256 of new WASM (hex-encoded in logs)
- `upgraded_by`: Address of admin who triggered upgrade
- `prev_schema_version`: Version before upgrade
- `new_schema_version`: Version after upgrade

## Rollback Procedures

### Immediate Rollback (Emergency)

If severe issues are discovered post-upgrade:

1. **Pause the contract** (if not already paused):
   ```
   set_paused(true)
   ```

2. **Verify previous WASM hash** from block explorer event history:
   ```
   // Find UpgradeEvent from N blocks ago
   // Read prev_hash from event
   previous_hash = event.prev_hash
   ```

3. **Execute emergency rollback**:
   ```rust
   // Admin calls:
   upgrade(previous_hash)
   ```

4. **Verification**:
   - New UpgradeEvent emitted with previous hash as new_hash
   - SCHEMA_VERSION incremented again
   - Off-chain monitoring verifies hash match

### Rollback Without Pause

If contract remains healthy but needs version revert:

1. Obtain previous WASM hash from on-chain events
2. Execute `upgrade(previous_hash)` as normal
3. System blocks if hash is identical to current

### Commit to Rollback

To make rollback permanent:
- Document rollback rationale in governance proposal
- Perform permanent code fixes in new WASM
- Deploy fixed version through normal governance process
- Never use previous WASM version in production again

## Review Process

### Pre-Upgrade Code Review

**Checklist for Pull Request:**
- [ ] SCHEMA_VERSION/CONTRACT_API_VERSION bumped correctly
- [ ] UpgradeEvent properly structured and emitted
- [ ] Admin authentication verified with `require_admin_auth()`
- [ ] Pause semantics enforced via `assert_not_paused()`
- [ ] All security assumptions documented inline
- [ ] Test coverage ≥ 95% for upgrade path
- [ ] Events tested for proper hash recording
- [ ] Unauthorized calls properly rejected
- [ ] Storage migration (if any) preserves borrower state
- [ ] Inline documentation explains upgrade mechanics

**Required Tests:**
- `test_upgrade_authorization_admin_only()` - Verify auth gate
- `test_upgrade_rejects_paused_contract()` - Verify pause semantics
- `test_upgrade_event_emission()` - Verify event structure
- `test_schema_version_bump()` - Verify version tracking
- `test_unauthorized_upgrade_rejection()` - Security validation
- `test_state_preservation()` - Data integrity check

### On-Chain Verification

After upgrade deployment:

1. **Hash Verification**:
   ```
   // Off-chain service verifies
   deployed_wasm = fetch_from_network()
   deployed_hash = sha256(deployed_wasm)
   assert_eq!(deployed_hash, event.new_hash)
   ```

2. **Event Parsing**:
   ```json
   {
     "prev_hash": "0x...",
     "new_hash": "0x...",
     "upgraded_by": "GXXXX...",
     "prev_schema_version": 1,
     "new_schema_version": 2
   }
   ```

3. **Integration Testing**:
   - Existing borrower positions accessible post-upgrade
   - New features/fixes working as expected
   - Gas costs within acceptable range
   - No state corruption observed

### Monitoring

**Real-time monitoring tracks:**
- Upgrade event emissions
- Admin transaction patterns
- Version progression
- Rollback frequency
- Time between upgrades

## Security Considerations

### Threat: Unauthorized Upgrade
- **Mitigation**: Admin auth via `require_admin_auth()`
- **Testing**: Authorization rejection tests
- **Verification**: Event logs admin address

### Threat: Malicious WASM Code
- **Mitigation**: Off-chain governance multi-sig review
- **Testing**: Full audit before vote
- **Verification**: Hash verification against audited code

### Threat: State Corruption
- **Mitigation**: Soroban deployer preserves all storage
- **Testing**: Post-upgrade state verification tests
- **Verification**: Spot-check borrower account reads

### Threat: Silent Failure
- **Mitigation**: Event emission with hashes and versions
- **Testing**: Event structure tests
- **Verification**: Off-chain monitoring of event stream

## Governance Requirements

### Multi-Sig Admin Setup

```rust
// Recommended: N-of-M multi-sig via Stellar account signers
// Example: 3-of-5 multi-sig
//   - 5 key holders (Protocol DAO members)
//   - Any 3 signatures required for upgrade
//   - Admin address is Stellar account with signers

// Current implementation: Single admin address
// Future: Integrate with governance DAO contract
```

### Upgrade Voting Period

- **Proposal open**: 2 days for review
- **Voting period**: 5 days minimum
- **Execution delay**: 24 hours after vote pass
- **Emergency bypass**: Multi-sig can skip voting if <1hr response needed

## Changelog

### Version 1.0 (Current)
- Admin-gated WASM upgrade entrypoint
- Event emission with audit trail
- Version tracking (SCHEMA_VERSION)
- Pause semantics enforcement
- Comprehensive test suite (95%+ coverage)

### Version 2.0 (Planned)
- DAO governance integration
- Time-lock on upgrades
- Automatic rollback on canary tests
- Multi-sig admin support

## References

- [Soroban Deployer API](https://soroban.stellar.org/docs/learn/storing-data)
- [Contract Upgrade Patterns](https://stellar.org/blog/contract-upgrades)
- [Security Best Practices for WASM Contracts](https://github.com/stellar/rs-soroban-sdk)

## Contact & Support

For upgrade-related issues or questions:
- **Technical**: Submit issue to contracts/credit directory
- **Governance**: Contact Protocol DAO
- **Security**: Report to security@protocol.example.com

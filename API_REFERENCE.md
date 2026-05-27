# Invoice Contract - Grace Period API Documentation

## Overview

The Invoice Contract provides functionality to manage grace periods for invoices with both global and per-invoice configuration options.

## Constants

### `MAX_GRACE_PERIOD_DAYS`

```rust
const MAX_GRACE_PERIOD_DAYS: u32 = 90;
```

**Value:** 90 days

**Applies to:**
- `set_grace_period_days()` - Global grace period setter
- `set_invoice_grace_period()` - Per-invoice grace period setter

**Rationale:**
90 days (approximately 3 months) is the industry standard for extended payment grace periods. This value provides sufficient flexibility for both global defaults and per-invoice overrides, allowing admins to:

1. Set a reasonable global default (0-90 days)
2. Override specific invoices up to the same maximum (0-90 days)
3. Grant per-invoice flexibility without arbitrary limitations

This unified cap ensures **consistency** and allows per-invoice overrides to be truly flexible. Previously, the per-invoice cap was 30 days while the global cap was 90 days, which defeated the purpose of per-invoice overrides (they should offer more flexibility than global settings, not less).

---

## Types

### `GracePeriodError`

An error type for grace period operations.

```rust
pub enum GracePeriodError {
    ExceededMaximumGracePeriod { days: u32, max: u32 },
    InvoiceNotFound { id: String },
    InvalidInvoiceId,
}
```

**Variants:**

- **`ExceededMaximumGracePeriod { days: u32, max: u32 }`**
  - Thrown when attempting to set a grace period exceeding the maximum allowed
  - `days`: The requested grace period
  - `max`: The maximum allowed grace period (90)

- **`InvoiceNotFound { id: String }`**
  - Thrown when attempting to operate on a non-existent invoice
  - `id`: The invoice ID that was not found

- **`InvalidInvoiceId`**
  - Thrown when an empty or invalid invoice ID is provided

### `Invoice`

Represents a single invoice with optional grace period override.

```rust
pub struct Invoice {
    pub id: String,
    pub amount: u64,
    pub grace_period_override: Option<u32>,
}
```

**Fields:**

- `id` - The unique invoice identifier
- `amount` - The invoice amount in the smallest currency unit
- `grace_period_override` - Optional per-invoice grace period override (0-90 days)

### `InvoiceContract`

The main contract state manager.

```rust
pub struct InvoiceContract {
    grace_period_days: u32,
    invoices: HashMap<String, Invoice>,
}
```

---

## Methods

### Creating a Contract

#### `InvoiceContract::new()`

```rust
pub fn new() -> Self
```

Creates a new invoice contract with default settings (0-day global grace period, no invoices).

**Returns:** A new `InvoiceContract` instance

**Example:**
```rust
let contract = InvoiceContract::new();
assert_eq!(contract.get_grace_period_days(), 0);
```

---

### Global Grace Period

#### `set_grace_period_days(days)`

```rust
pub fn set_grace_period_days(&mut self, days: u32) -> Result<(), GracePeriodError>
```

Sets the global grace period that applies to all invoices without per-invoice overrides.

**Parameters:**
- `days` - Number of days (0-90)

**Returns:**
- `Ok(())` if successful
- `Err(GracePeriodError::ExceededMaximumGracePeriod)` if `days > 90`

**Maximum Value:** 90 days

**Example:**
```rust
let mut contract = InvoiceContract::new();

// Valid: set to 30 days
assert!(contract.set_grace_period_days(30).is_ok());

// Valid: set to maximum
assert!(contract.set_grace_period_days(90).is_ok());

// Invalid: exceeds maximum
assert!(contract.set_grace_period_days(91).is_err());
```

---

#### `get_grace_period_days()`

```rust
pub fn get_grace_period_days(&self) -> u32
```

Retrieves the current global grace period.

**Returns:** Global grace period in days

**Example:**
```rust
let mut contract = InvoiceContract::new();
contract.set_grace_period_days(45).unwrap();
assert_eq!(contract.get_grace_period_days(), 45);
```

---

### Invoice Management

#### `create_invoice(id, amount)`

```rust
pub fn create_invoice(&mut self, id: String, amount: u64) -> Result<(), GracePeriodError>
```

Creates a new invoice.

**Parameters:**
- `id` - Unique invoice identifier (must not be empty)
- `amount` - Invoice amount

**Returns:**
- `Ok(())` if successful
- `Err(GracePeriodError::InvalidInvoiceId)` if ID is empty

**Example:**
```rust
let mut contract = InvoiceContract::new();
contract.create_invoice("INV001".to_string(), 1000).unwrap();
```

---

### Per-Invoice Grace Period

#### `set_invoice_grace_period(id, days)`

```rust
pub fn set_invoice_grace_period(&mut self, id: &str, days: u32) 
    -> Result<(), GracePeriodError>
```

Sets a per-invoice grace period override.

This allows overriding the global grace period for a specific invoice, enabling admins to grant specific invoices extended or reduced grace periods.

**Parameters:**
- `id` - The invoice ID
- `days` - Grace period in days (0-90)

**Returns:**
- `Ok(())` if successful
- `Err(GracePeriodError::ExceededMaximumGracePeriod)` if `days > 90`
- `Err(GracePeriodError::InvoiceNotFound)` if invoice doesn't exist
- `Err(GracePeriodError::InvalidInvoiceId)` if ID is empty

**Maximum Value:** 90 days (same as global maximum)

**Example:**
```rust
let mut contract = InvoiceContract::new();
contract.set_grace_period_days(30).unwrap();
contract.create_invoice("INV001".to_string(), 1000).unwrap();

// Grant extended grace period (more than global)
assert!(contract.set_invoice_grace_period("INV001", 60).is_ok());

// Grant maximum grace period
assert!(contract.set_invoice_grace_period("INV001", 90).is_ok());

// Cannot exceed maximum
assert!(contract.set_invoice_grace_period("INV001", 91).is_err());
```

---

#### `get_invoice_grace_period(id)`

```rust
pub fn get_invoice_grace_period(&self, id: &str) 
    -> Result<u32, GracePeriodError>
```

Gets the effective grace period for an invoice.

Returns the per-invoice override if set, otherwise returns the global grace period.

**Parameters:**
- `id` - The invoice ID

**Returns:**
- `Ok(days)` - The effective grace period
- `Err(GracePeriodError::InvoiceNotFound)` if invoice doesn't exist

**Example:**
```rust
let mut contract = InvoiceContract::new();
contract.set_grace_period_days(30).unwrap();
contract.create_invoice("INV001".to_string(), 1000).unwrap();

// Uses global grace period
assert_eq!(contract.get_invoice_grace_period("INV001").unwrap(), 30);

// After override
contract.set_invoice_grace_period("INV001", 60).unwrap();
assert_eq!(contract.get_invoice_grace_period("INV001").unwrap(), 60);
```

---

#### `clear_invoice_grace_period(id)`

```rust
pub fn clear_invoice_grace_period(&mut self, id: &str) 
    -> Result<(), GracePeriodError>
```

Clears the grace period override for an invoice (reverts to global grace period).

**Parameters:**
- `id` - The invoice ID

**Returns:**
- `Ok(())` if successful
- `Err(GracePeriodError::InvalidInvoiceId)` if ID is empty
- `Err(GracePeriodError::InvoiceNotFound)` if invoice doesn't exist

**Example:**
```rust
let mut contract = InvoiceContract::new();
contract.set_grace_period_days(30).unwrap();
contract.create_invoice("INV001".to_string(), 1000).unwrap();

// Set override
contract.set_invoice_grace_period("INV001", 60).unwrap();
assert_eq!(contract.get_invoice_grace_period("INV001").unwrap(), 60);

// Clear override - reverts to global
contract.clear_invoice_grace_period("INV001").unwrap();
assert_eq!(contract.get_invoice_grace_period("INV001").unwrap(), 30);
```

---

## Design Rationale

### Cap Consistency (Unification)

**Problem:** Previous implementation had:
- Global cap: 90 days
- Per-invoice cap: 30 days

This was inconsistent and limited flexibility.

**Solution (Option A):** Both caps = 90 days

**Benefits:**
1. **Consistency:** Both setters have identical constraints
2. **Flexibility:** Per-invoice overrides can match or exceed global settings
3. **Admin Control:** Admins can grant specific invoices extended grace periods without the global setting
4. **Simplicity:** Single constant for both operations

### Grace Period Behavior

1. **Invoices without overrides** use the global grace period
2. **Invoices with overrides** use their per-invoice grace period regardless of global changes
3. **Changing global grace period** only affects invoices without overrides
4. **Per-invoice overrides** allow for flexible customer-by-customer grace period management

---

## Testing

The implementation includes comprehensive unit tests covering:

- ✅ Valid grace period values (0, 45, 90 days)
- ✅ Maximum boundary enforcement (91 days rejected)
- ✅ Cap consistency between global and per-invoice setters
- ✅ Per-invoice override flexibility
- ✅ Error handling (non-existent invoices, invalid IDs)
- ✅ Override precedence over global settings
- ✅ Clear override functionality
- ✅ Realistic multi-invoice workflows

**Run tests:**
```bash
cargo test
```

---

## Migration Guide

If you're upgrading from the old implementation (90-day global / 30-day per-invoice):

1. **No code changes required** if you never set per-invoice grace periods > 30 days
2. **If you have overrides ≤ 30 days,** they continue to work identically
3. **If you want to use the full 90-day per-invoice capability,** you can now set values up to 90

Example migration:
```rust
// Old code (still works)
contract.set_invoice_grace_period("INV001", 30)?;

// New capability
contract.set_invoice_grace_period("INV001", 90)?;  // Now supported!
```

---

## Summary Table

| Feature | Before | After |
|---------|--------|-------|
| Global Grace Period Cap | 90 days | 90 days |
| Per-Invoice Cap | 30 days | 90 days |
| Cap Consistency | ❌ Inconsistent | ✅ Consistent |
| Documentation | ❌ Not documented | ✅ Fully documented |
| Tests | ⚠️ Limited | ✅ Comprehensive |
| Per-Invoice Max Override | 30 days | 90 days |
| Used in Production | ✅ | ✅ |

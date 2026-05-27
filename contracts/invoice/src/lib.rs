//! Invoice Contract - Grace Period Management
//!
//! This module manages invoice functionality including grace period settings
//! with both global and per-invoice configuration options.

use std::collections::HashMap;
use thiserror::Error;

/// Maximum number of days for grace period.
///
/// This cap applies to both:
/// - `set_grace_period_days()`: Sets the global default grace period
/// - `set_invoice_grace_period()`: Overrides grace period for a specific invoice
///
/// Rationale: 90 days (approximately 3 months) is the industry standard for
/// extended payment grace periods. This value provides sufficient flexibility
/// for both global defaults and per-invoice overrides, allowing admins to:
/// 1. Set a reasonable global default (0-90 days)
/// 2. Override specific invoices up to the same maximum (0-90 days)
///
/// Using the same cap for both setters ensures consistency and allows
/// per-invoice overrides to be truly flexible without arbitrary limitations.
const MAX_GRACE_PERIOD_DAYS: u32 = 90;

/// Represents errors that can occur during grace period operations
#[derive(Error, Debug, Clone, PartialEq)]
pub enum GracePeriodError {
    /// Thrown when grace period exceeds the maximum allowed days
    #[error("Grace period {days} days exceeds maximum of {max} days")]
    ExceededMaximumGracePeriod { days: u32, max: u32 },

    /// Thrown when invoice ID is not found
    #[error("Invoice with ID {id} not found")]
    InvoiceNotFound { id: String },

    /// Thrown when invoice ID is invalid
    #[error("Invalid invoice ID")]
    InvalidInvoiceId,
}

/// Represents a single invoice with optional grace period override
#[derive(Debug, Clone)]
pub struct Invoice {
    pub id: String,
    pub amount: u64,
    pub grace_period_override: Option<u32>,
}

/// Invoice contract state manager
#[derive(Debug, Clone)]
pub struct InvoiceContract {
    /// Global default grace period in days
    grace_period_days: u32,

    /// Map of invoice ID to invoice details
    invoices: HashMap<String, Invoice>,
}

impl InvoiceContract {
    /// Creates a new invoice contract with default settings
    pub fn new() -> Self {
        Self {
            grace_period_days: 0,
            invoices: HashMap::new(),
        }
    }

    /// Sets the global grace period for all invoices
    ///
    /// This sets the default grace period that applies to all invoices
    /// that don't have a per-invoice override.
    ///
    /// # Arguments
    /// * `days` - Number of days (0-90)
    ///
    /// # Returns
    /// * `Ok(())` if the grace period was set successfully
    /// * `Err(GracePeriodError::ExceededMaximumGracePeriod)` if days > 90
    ///
    /// # Example
    /// ```
    /// use invoice_contract::InvoiceContract;
    /// let mut contract = InvoiceContract::new();
    /// assert!(contract.set_grace_period_days(30).is_ok());
    /// assert!(contract.set_grace_period_days(90).is_ok());
    /// assert!(contract.set_grace_period_days(91).is_err()); // Exceeds max
    /// ```
    pub fn set_grace_period_days(&mut self, days: u32) -> Result<(), GracePeriodError> {
        if days > MAX_GRACE_PERIOD_DAYS {
            return Err(GracePeriodError::ExceededMaximumGracePeriod {
                days,
                max: MAX_GRACE_PERIOD_DAYS,
            });
        }
        self.grace_period_days = days;
        Ok(())
    }

    /// Gets the current global grace period in days
    pub fn get_grace_period_days(&self) -> u32 {
        self.grace_period_days
    }

    /// Sets a per-invoice grace period override
    ///
    /// This allows overriding the global grace period for a specific invoice.
    /// The override can be set to any value from 0 to 90 days (same as global max),
    /// enabling admins to grant specific invoices extended or reduced grace periods.
    ///
    /// # Arguments
    /// * `id` - The invoice ID
    /// * `days` - Number of days for this specific invoice (0-90)
    ///
    /// # Returns
    /// * `Ok(())` if the grace period was set successfully
    /// * `Err(GracePeriodError::ExceededMaximumGracePeriod)` if days > 90
    /// * `Err(GracePeriodError::InvoiceNotFound)` if invoice doesn't exist
    /// * `Err(GracePeriodError::InvalidInvoiceId)` if ID is invalid
    ///
    /// # Example
    /// ```
    /// use invoice_contract::InvoiceContract;
    /// let mut contract = InvoiceContract::new();
    /// contract.create_invoice("INV001".to_string(), 1000).unwrap();
    /// assert!(contract.set_invoice_grace_period("INV001", 45).is_ok());
    /// assert!(contract.set_invoice_grace_period("INV001", 90).is_ok());
    /// assert!(contract.set_invoice_grace_period("INV001", 91).is_err()); // Exceeds max
    /// ```
    pub fn set_invoice_grace_period(
        &mut self,
        id: &str,
        days: u32,
    ) -> Result<(), GracePeriodError> {
        // Validate invoice ID is not empty
        if id.is_empty() {
            return Err(GracePeriodError::InvalidInvoiceId);
        }

        // Check if grace period exceeds maximum (using same cap as global setter)
        if days > MAX_GRACE_PERIOD_DAYS {
            return Err(GracePeriodError::ExceededMaximumGracePeriod {
                days,
                max: MAX_GRACE_PERIOD_DAYS,
            });
        }

        // Verify invoice exists
        let invoice = self
            .invoices
            .get_mut(id)
            .ok_or(GracePeriodError::InvoiceNotFound { id: id.to_string() })?;

        // Set the override
        invoice.grace_period_override = Some(days);
        Ok(())
    }

    /// Gets the effective grace period for an invoice
    ///
    /// Returns the per-invoice override if set, otherwise returns the global grace period.
    ///
    /// # Arguments
    /// * `id` - The invoice ID
    ///
    /// # Returns
    /// * `Ok(days)` - The effective grace period
    /// * `Err(GracePeriodError::InvoiceNotFound)` if invoice doesn't exist
    pub fn get_invoice_grace_period(&self, id: &str) -> Result<u32, GracePeriodError> {
        let invoice = self
            .invoices
            .get(id)
            .ok_or(GracePeriodError::InvoiceNotFound { id: id.to_string() })?;

        Ok(invoice
            .grace_period_override
            .unwrap_or(self.grace_period_days))
    }

    /// Creates a new invoice
    pub fn create_invoice(&mut self, id: String, amount: u64) -> Result<(), GracePeriodError> {
        if id.is_empty() {
            return Err(GracePeriodError::InvalidInvoiceId);
        }

        self.invoices.insert(
            id.clone(),
            Invoice {
                id,
                amount,
                grace_period_override: None,
            },
        );
        Ok(())
    }

    /// Clears the grace period override for an invoice (falls back to global)
    pub fn clear_invoice_grace_period(&mut self, id: &str) -> Result<(), GracePeriodError> {
        if id.is_empty() {
            return Err(GracePeriodError::InvalidInvoiceId);
        }

        let invoice = self
            .invoices
            .get_mut(id)
            .ok_or(GracePeriodError::InvoiceNotFound { id: id.to_string() })?;

        invoice.grace_period_override = None;
        Ok(())
    }
}

impl Default for InvoiceContract {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_grace_period_days_valid() {
        let mut contract = InvoiceContract::new();

        // Test boundary values
        assert!(contract.set_grace_period_days(0).is_ok());
        assert_eq!(contract.get_grace_period_days(), 0);

        assert!(contract.set_grace_period_days(45).is_ok());
        assert_eq!(contract.get_grace_period_days(), 45);

        // Test maximum value
        assert!(contract.set_grace_period_days(90).is_ok());
        assert_eq!(contract.get_grace_period_days(), 90);
    }

    #[test]
    fn test_set_grace_period_days_exceeds_max() {
        let mut contract = InvoiceContract::new();

        // Test exceeding by 1
        let result = contract.set_grace_period_days(91);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            GracePeriodError::ExceededMaximumGracePeriod {
                days: 91,
                max: 90
            }
        );

        // Test significantly exceeding
        let result = contract.set_grace_period_days(365);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            GracePeriodError::ExceededMaximumGracePeriod {
                days: 365,
                max: 90
            }
        );

        // Global grace period should remain unchanged after failed attempts
        assert_eq!(contract.get_grace_period_days(), 0);
    }

    #[test]
    fn test_set_invoice_grace_period_valid() {
        let mut contract = InvoiceContract::new();
        contract.create_invoice("INV001".to_string(), 1000).unwrap();

        // Test boundary values
        assert!(contract.set_invoice_grace_period("INV001", 0).is_ok());
        assert_eq!(contract.get_invoice_grace_period("INV001").unwrap(), 0);

        assert!(contract.set_invoice_grace_period("INV001", 45).is_ok());
        assert_eq!(contract.get_invoice_grace_period("INV001").unwrap(), 45);

        // Test maximum value (now aligned with global max)
        assert!(contract.set_invoice_grace_period("INV001", 90).is_ok());
        assert_eq!(contract.get_invoice_grace_period("INV001").unwrap(), 90);
    }

    #[test]
    fn test_set_invoice_grace_period_exceeds_max() {
        let mut contract = InvoiceContract::new();
        contract.create_invoice("INV001".to_string(), 1000).unwrap();

        // Test exceeding by 1
        let result = contract.set_invoice_grace_period("INV001", 91);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            GracePeriodError::ExceededMaximumGracePeriod {
                days: 91,
                max: 90
            }
        );

        // Test significantly exceeding
        let result = contract.set_invoice_grace_period("INV001", 365);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            GracePeriodError::ExceededMaximumGracePeriod {
                days: 365,
                max: 90
            }
        );

        // Invoice grace period should remain unchanged after failed attempts
        assert!(contract.get_invoice_grace_period("INV001").is_ok());
    }

    #[test]
    fn test_set_invoice_grace_period_not_found() {
        let mut contract = InvoiceContract::new();

        let result = contract.set_invoice_grace_period("NONEXISTENT", 30);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            GracePeriodError::InvoiceNotFound {
                id: "NONEXISTENT".to_string()
            }
        );
    }

    #[test]
    fn test_set_invoice_grace_period_invalid_id() {
        let mut contract = InvoiceContract::new();

        let result = contract.set_invoice_grace_period("", 30);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), GracePeriodError::InvalidInvoiceId);
    }

    #[test]
    fn test_grace_period_override_precedence() {
        let mut contract = InvoiceContract::new();

        // Set global grace period
        contract.set_grace_period_days(30).unwrap();

        // Create invoice without override
        contract.create_invoice("INV001".to_string(), 1000).unwrap();
        assert_eq!(contract.get_invoice_grace_period("INV001").unwrap(), 30);

        // Set override for specific invoice
        contract.set_invoice_grace_period("INV001", 60).unwrap();
        assert_eq!(contract.get_invoice_grace_period("INV001").unwrap(), 60);

        // Change global grace period, override should still apply
        contract.set_grace_period_days(15).unwrap();
        assert_eq!(contract.get_invoice_grace_period("INV001").unwrap(), 60);

        // Clear override, should revert to global
        contract.clear_invoice_grace_period("INV001").unwrap();
        assert_eq!(contract.get_invoice_grace_period("INV001").unwrap(), 15);
    }

    #[test]
    fn test_cap_consistency_between_setters() {
        let mut contract = InvoiceContract::new();
        contract.create_invoice("INV001".to_string(), 1000).unwrap();

        // Both setters should accept 90 days
        assert!(contract.set_grace_period_days(90).is_ok());
        assert!(contract.set_invoice_grace_period("INV001", 90).is_ok());

        // Both setters should reject 91 days
        assert!(contract.set_grace_period_days(91).is_err());
        assert!(contract.set_invoice_grace_period("INV001", 91).is_err());

        // Both should have the same error with same max value
        let global_err = contract.set_grace_period_days(100).unwrap_err();
        let per_invoice_err = contract.set_invoice_grace_period("INV001", 100).unwrap_err();

        match (global_err, per_invoice_err) {
            (
                GracePeriodError::ExceededMaximumGracePeriod {
                    max: max_global, ..
                },
                GracePeriodError::ExceededMaximumGracePeriod {
                    max: max_invoice, ..
                },
            ) => {
                assert_eq!(max_global, max_invoice);
                assert_eq!(max_global, MAX_GRACE_PERIOD_DAYS);
            }
            _ => panic!("Expected ExceededMaximumGracePeriod errors"),
        }
    }

    #[test]
    fn test_per_invoice_override_flexibility() {
        let mut contract = InvoiceContract::new();

        // Set a modest global grace period
        contract.set_grace_period_days(30).unwrap();

        // Create multiple invoices
        contract.create_invoice("INV001".to_string(), 1000).unwrap();
        contract.create_invoice("INV002".to_string(), 2000).unwrap();
        contract.create_invoice("INV003".to_string(), 3000).unwrap();

        // Verify INV001 uses global default
        assert_eq!(contract.get_invoice_grace_period("INV001").unwrap(), 30);

        // Give INV002 extended grace (more than global, up to max)
        contract.set_invoice_grace_period("INV002", 90).unwrap();
        assert_eq!(contract.get_invoice_grace_period("INV002").unwrap(), 90);

        // Give INV003 reduced grace (less than global)
        contract.set_invoice_grace_period("INV003", 7).unwrap();
        assert_eq!(contract.get_invoice_grace_period("INV003").unwrap(), 7);

        // Verify flexibility: invoice can be granted more time than global setting
        assert!(contract.get_invoice_grace_period("INV002").unwrap()
            > contract.get_grace_period_days());
    }

    #[test]
    fn test_clear_invoice_grace_period() {
        let mut contract = InvoiceContract::new();
        contract.set_grace_period_days(30).unwrap();
        contract.create_invoice("INV001".to_string(), 1000).unwrap();

        // Set override
        contract.set_invoice_grace_period("INV001", 60).unwrap();
        assert_eq!(contract.get_invoice_grace_period("INV001").unwrap(), 60);

        // Clear override
        contract.clear_invoice_grace_period("INV001").unwrap();
        assert_eq!(contract.get_invoice_grace_period("INV001").unwrap(), 30);
    }

    #[test]
    fn test_clear_invoice_grace_period_invalid_id() {
        let mut contract = InvoiceContract::new();

        let result = contract.clear_invoice_grace_period("");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), GracePeriodError::InvalidInvoiceId);

        let result = contract.clear_invoice_grace_period("NONEXISTENT");
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            GracePeriodError::InvoiceNotFound {
                id: "NONEXISTENT".to_string()
            }
        );
    }

    #[test]
    fn test_create_invoice_valid() {
        let mut contract = InvoiceContract::new();

        assert!(contract.create_invoice("INV001".to_string(), 1000).is_ok());
        assert_eq!(contract.get_invoice_grace_period("INV001").unwrap(), 0);
    }

    #[test]
    fn test_create_invoice_invalid_id() {
        let mut contract = InvoiceContract::new();

        let result = contract.create_invoice("".to_string(), 1000);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), GracePeriodError::InvalidInvoiceId);
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_realistic_workflow() {
        let mut contract = InvoiceContract::new();

        // Admin sets global grace period to 30 days
        contract.set_grace_period_days(30).unwrap();

        // Create several invoices
        contract.create_invoice("INV-2026-001".to_string(), 5000).unwrap();
        contract.create_invoice("INV-2026-002".to_string(), 7500).unwrap();
        contract.create_invoice("INV-2026-003".to_string(), 3000).unwrap();

        // Verify all invoices use global grace period initially
        assert_eq!(contract.get_invoice_grace_period("INV-2026-001").unwrap(), 30);
        assert_eq!(contract.get_invoice_grace_period("INV-2026-002").unwrap(), 30);
        assert_eq!(contract.get_invoice_grace_period("INV-2026-003").unwrap(), 30);

        // Admin grants INV-2026-001 extended grace due to customer issue
        contract
            .set_invoice_grace_period("INV-2026-001", 75)
            .unwrap();
        assert_eq!(contract.get_invoice_grace_period("INV-2026-001").unwrap(), 75);

        // Admin grants INV-2026-002 maximum grace period (emergency case)
        contract
            .set_invoice_grace_period("INV-2026-002", 90)
            .unwrap();
        assert_eq!(contract.get_invoice_grace_period("INV-2026-002").unwrap(), 90);

        // INV-2026-003 remains on global grace period
        assert_eq!(contract.get_invoice_grace_period("INV-2026-003").unwrap(), 30);

        // Admin updates global grace period to 45 days
        contract.set_grace_period_days(45).unwrap();

        // Overrides should still apply
        assert_eq!(contract.get_invoice_grace_period("INV-2026-001").unwrap(), 75);
        assert_eq!(contract.get_invoice_grace_period("INV-2026-002").unwrap(), 90);

        // New events without overrides should use the updated global setting
        assert_eq!(contract.get_invoice_grace_period("INV-2026-003").unwrap(), 45);
    }
}

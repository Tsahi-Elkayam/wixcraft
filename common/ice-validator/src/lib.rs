//! ICE Validator - Cross-platform MSI validation
//!
//! This library provides ICE (Internal Consistency Evaluator) validation
//! for MSI installer packages without requiring Windows SDK.
//!
//! # Features
//!
//! - Load ICE rules from wixkb database
//! - Built-in subset of common ICE rules
//! - Validate MSI files against rules
//! - Cross-platform (Windows, macOS, Linux)
//!
//! # Example
//!
//! ```no_run
//! use ice_validator::Validator;
//!
//! // Create validator with built-in rules
//! let validator = Validator::with_builtin_rules();
//!
//! // Or load from wixkb database
//! // let validator = Validator::from_wixkb("~/.wixcraft/wixkb.db")?;
//!
//! // Validate an MSI file
//! let result = validator.validate("installer.msi").unwrap();
//!
//! println!("{}", result.summary());
//! for violation in &result.violations {
//!     println!("  {}", violation);
//! }
//! ```

pub mod rules;
pub mod types;
pub mod validator;

pub use types::{IceRule, Severity, ValidationResult, Violation};
pub use validator::Validator;

use thiserror::Error;

/// Errors that can occur during ICE validation
#[derive(Error, Debug)]
pub enum IceError {
    #[error("Failed to open MSI file: {0}")]
    MsiError(String),

    #[error("Database error: {0}")]
    DbError(#[from] rusqlite::Error),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Rule error: {0}")]
    RuleError(String),
}

pub type Result<T> = std::result::Result<T, IceError>;

/// Convenience function to validate an MSI with default settings
pub fn validate_msi<P: AsRef<std::path::Path>>(msi_path: P) -> Result<ValidationResult> {
    let validator = if let Some(db_path) = rules::default_wixkb_path() {
        if db_path.exists() {
            Validator::from_wixkb(db_path)?
        } else {
            Validator::with_builtin_rules()
        }
    } else {
        Validator::with_builtin_rules()
    };

    validator.validate(msi_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builtin_validation() {
        let validator = Validator::with_builtin_rules();
        assert!(validator.rules().len() >= 10);
    }
}

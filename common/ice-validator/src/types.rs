//! Common types for ICE validation

use serde::{Deserialize, Serialize};

/// Severity level for ICE violations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Error,
    Warning,
    Info,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::Error => write!(f, "error"),
            Severity::Warning => write!(f, "warning"),
            Severity::Info => write!(f, "info"),
        }
    }
}

impl std::str::FromStr for Severity {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "error" => Ok(Severity::Error),
            "warning" => Ok(Severity::Warning),
            "info" => Ok(Severity::Info),
            _ => Err(format!("Unknown severity: {}", s)),
        }
    }
}

/// An ICE rule definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IceRule {
    /// Rule code (e.g., ICE03, ICE09)
    pub code: String,
    /// Severity level
    pub severity: Severity,
    /// Human-readable description
    pub description: String,
    /// How to fix violations
    pub resolution: Option<String>,
    /// Tables this rule checks
    pub tables_affected: Vec<String>,
    /// Documentation URL
    pub documentation_url: Option<String>,
}

/// A validation violation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Violation {
    /// The ICE rule that was violated
    pub rule_code: String,
    /// Severity level
    pub severity: Severity,
    /// Human-readable message
    pub message: String,
    /// Table where the violation occurred
    pub table: Option<String>,
    /// Row identifier (primary key)
    pub row_key: Option<String>,
    /// Column name
    pub column: Option<String>,
    /// The problematic value
    pub value: Option<String>,
}

impl std::fmt::Display for Violation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}: {}", self.rule_code, self.severity, self.message)?;
        if let Some(ref table) = self.table {
            write!(f, " (table: {}", table)?;
            if let Some(ref key) = self.row_key {
                write!(f, ", key: {}", key)?;
            }
            write!(f, ")")?;
        }
        Ok(())
    }
}

/// Validation result for an MSI file
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Path to the validated file
    pub file_path: String,
    /// All violations found
    pub violations: Vec<Violation>,
    /// Number of rules checked
    pub rules_checked: usize,
    /// Validation duration in milliseconds
    pub duration_ms: u64,
}

impl ValidationResult {
    /// Check if validation passed (no errors)
    pub fn passed(&self) -> bool {
        !self.violations.iter().any(|v| v.severity == Severity::Error)
    }

    /// Count violations by severity
    pub fn count_by_severity(&self) -> (usize, usize, usize) {
        let mut errors = 0;
        let mut warnings = 0;
        let mut infos = 0;

        for v in &self.violations {
            match v.severity {
                Severity::Error => errors += 1,
                Severity::Warning => warnings += 1,
                Severity::Info => infos += 1,
            }
        }

        (errors, warnings, infos)
    }

    /// Get summary string
    pub fn summary(&self) -> String {
        let (errors, warnings, infos) = self.count_by_severity();
        format!(
            "{} errors, {} warnings, {} info ({} rules checked in {}ms)",
            errors, warnings, infos, self.rules_checked, self.duration_ms
        )
    }
}

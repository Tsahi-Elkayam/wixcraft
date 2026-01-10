//! ICE validation engine

use crate::types::{IceRule, Severity, ValidationResult, Violation};
use crate::Result;
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::time::Instant;

/// ICE Validator for MSI files
pub struct Validator {
    rules: Vec<IceRule>,
}

impl Validator {
    /// Create a new validator with the given rules
    pub fn new(rules: Vec<IceRule>) -> Self {
        Self { rules }
    }

    /// Create validator loading rules from wixkb
    pub fn from_wixkb<P: AsRef<Path>>(db_path: P) -> Result<Self> {
        let rules = crate::rules::load_from_wixkb(db_path)?;
        Ok(Self::new(rules))
    }

    /// Create validator with built-in rules only
    pub fn with_builtin_rules() -> Self {
        Self::new(crate::rules::builtin_rules())
    }

    /// Get the loaded rules
    pub fn rules(&self) -> &[IceRule] {
        &self.rules
    }

    /// Validate an MSI file
    pub fn validate<P: AsRef<Path>>(&self, msi_path: P) -> Result<ValidationResult> {
        let start = Instant::now();
        let path_str = msi_path.as_ref().display().to_string();

        let file = std::fs::File::open(&msi_path)?;
        let mut package = msi::Package::open(file)
            .map_err(|e| crate::IceError::MsiError(e.to_string()))?;

        let mut violations = Vec::new();

        // Run each implemented check
        self.check_ice03(&mut package, &mut violations);
        self.check_ice08(&mut package, &mut violations);
        self.check_ice18(&mut package, &mut violations);
        self.check_ice30(&mut package, &mut violations);
        self.check_foreign_keys(&mut package, &mut violations);
        self.check_required_tables(&package, &mut violations);

        let duration = start.elapsed();

        Ok(ValidationResult {
            file_path: path_str,
            violations,
            rules_checked: self.rules.len(),
            duration_ms: duration.as_millis() as u64,
        })
    }

    /// Helper to get string value from row
    fn get_str(row: &msi::Row, idx: usize) -> Option<String> {
        row[idx].as_str().map(|s| s.to_string())
    }

    /// Helper to get string value with default
    fn get_str_or_empty(row: &msi::Row, idx: usize) -> String {
        row[idx].as_str().map(|s| s.to_string()).unwrap_or_default()
    }

    /// ICE03: Basic data and foreign key validation
    fn check_ice03(&self, package: &mut msi::Package<std::fs::File>, violations: &mut Vec<Violation>) {
        // Check Component -> Directory_ references
        if let Some(comp_table) = package.tables().find(|t| t.name() == "Component") {
            let dir_col_idx = comp_table.columns().iter().position(|c| c.name() == "Directory_");

            if let Some(idx) = dir_col_idx {
                // Get all valid directory IDs
                let valid_dirs: HashSet<String> = self.collect_primary_keys(package, "Directory");

                if let Ok(rows) = package.select_rows(msi::Select::table("Component")) {
                    for row in rows {
                        let comp_id = Self::get_str_or_empty(&row, 0);
                        let dir_ref = Self::get_str_or_empty(&row, idx);

                        if !dir_ref.is_empty() && !valid_dirs.contains(&dir_ref) {
                            violations.push(Violation {
                                rule_code: "ICE03".into(),
                                severity: Severity::Error,
                                message: format!(
                                    "Component '{}' references non-existent Directory '{}'",
                                    comp_id, dir_ref
                                ),
                                table: Some("Component".into()),
                                row_key: Some(comp_id),
                                column: Some("Directory_".into()),
                                value: Some(dir_ref),
                            });
                        }
                    }
                }
            }
        }

        // Check File -> Component_ references
        if package.tables().any(|t| t.name() == "File") {
            let valid_comps: HashSet<String> = self.collect_primary_keys(package, "Component");

            if let Ok(rows) = package.select_rows(msi::Select::table("File")) {
                for row in rows {
                    let file_id = Self::get_str_or_empty(&row, 0);
                    let comp_ref = Self::get_str_or_empty(&row, 1);

                    if !comp_ref.is_empty() && !valid_comps.contains(&comp_ref) {
                        violations.push(Violation {
                            rule_code: "ICE03".into(),
                            severity: Severity::Error,
                            message: format!(
                                "File '{}' references non-existent Component '{}'",
                                file_id, comp_ref
                            ),
                            table: Some("File".into()),
                            row_key: Some(file_id),
                            column: Some("Component_".into()),
                            value: Some(comp_ref),
                        });
                    }
                }
            }
        }
    }

    /// ICE08: Duplicate GUIDs in Component table
    fn check_ice08(&self, package: &mut msi::Package<std::fs::File>, violations: &mut Vec<Violation>) {
        if let Some(comp_table) = package.tables().find(|t| t.name() == "Component") {
            let guid_col_idx = comp_table.columns().iter().position(|c| c.name() == "ComponentId");

            if let Some(idx) = guid_col_idx {
                let mut seen_guids: HashMap<String, String> = HashMap::new();

                if let Ok(rows) = package.select_rows(msi::Select::table("Component")) {
                    for row in rows {
                        let comp_id = Self::get_str_or_empty(&row, 0);
                        if let Some(guid) = Self::get_str(&row, idx) {
                            if !guid.is_empty() {
                                let guid_upper = guid.to_uppercase();
                                if let Some(first_comp) = seen_guids.get(&guid_upper) {
                                    violations.push(Violation {
                                        rule_code: "ICE08".into(),
                                        severity: Severity::Error,
                                        message: format!(
                                            "Duplicate ComponentId '{}' found in components '{}' and '{}'",
                                            guid, first_comp, comp_id
                                        ),
                                        table: Some("Component".into()),
                                        row_key: Some(comp_id.clone()),
                                        column: Some("ComponentId".into()),
                                        value: Some(guid),
                                    });
                                } else {
                                    seen_guids.insert(guid_upper, comp_id);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    /// ICE18: Empty KeyPath validation
    fn check_ice18(&self, package: &mut msi::Package<std::fs::File>, violations: &mut Vec<Violation>) {
        if let Some(comp_table) = package.tables().find(|t| t.name() == "Component") {
            let keypath_col_idx = comp_table.columns().iter().position(|c| c.name() == "KeyPath");

            if let Some(idx) = keypath_col_idx {
                if let Ok(rows) = package.select_rows(msi::Select::table("Component")) {
                    for row in rows {
                        let comp_id = Self::get_str_or_empty(&row, 0);
                        let keypath = Self::get_str_or_empty(&row, idx);

                        // KeyPath can be null for directory components, but warn if empty string
                        if keypath.is_empty() && !row[idx].is_null() {
                            violations.push(Violation {
                                rule_code: "ICE18".into(),
                                severity: Severity::Warning,
                                message: format!(
                                    "Component '{}' has an empty KeyPath",
                                    comp_id
                                ),
                                table: Some("Component".into()),
                                row_key: Some(comp_id),
                                column: Some("KeyPath".into()),
                                value: None,
                            });
                        }
                    }
                }
            }
        }
    }

    /// ICE30: Same file installed to multiple directories
    fn check_ice30(&self, package: &mut msi::Package<std::fs::File>, violations: &mut Vec<Violation>) {
        // Build component -> directory map
        let mut comp_to_dir: HashMap<String, String> = HashMap::new();
        if let Ok(rows) = package.select_rows(msi::Select::table("Component")) {
            for row in rows {
                let comp_id = Self::get_str_or_empty(&row, 0);
                let dir_id = Self::get_str_or_empty(&row, 2);
                comp_to_dir.insert(comp_id, dir_id);
            }
        }

        // Check for duplicate file names in different directories
        let mut file_locations: HashMap<String, Vec<(String, String)>> = HashMap::new();

        if let Ok(rows) = package.select_rows(msi::Select::table("File")) {
            for row in rows {
                let file_id = Self::get_str_or_empty(&row, 0);
                let comp_ref = Self::get_str_or_empty(&row, 1);
                let file_name = Self::get_str_or_empty(&row, 2);

                // Extract short name (before |) or use full name
                let short_name = file_name.split('|').next().unwrap_or(&file_name).to_string();

                if let Some(dir) = comp_to_dir.get(&comp_ref) {
                    file_locations
                        .entry(short_name)
                        .or_default()
                        .push((file_id, dir.clone()));
                }
            }
        }

        // Report files with same name in same directory from different components
        for (file_name, locations) in file_locations {
            if locations.len() > 1 {
                let mut by_dir: HashMap<&str, Vec<&str>> = HashMap::new();
                for (file_id, dir) in &locations {
                    by_dir.entry(dir.as_str()).or_default().push(file_id.as_str());
                }

                for (dir, files) in by_dir {
                    if files.len() > 1 {
                        violations.push(Violation {
                            rule_code: "ICE30".into(),
                            severity: Severity::Error,
                            message: format!(
                                "Multiple files named '{}' installed to directory '{}': {:?}",
                                file_name, dir, files
                            ),
                            table: Some("File".into()),
                            row_key: Some(files[0].to_string()),
                            column: Some("FileName".into()),
                            value: Some(file_name.clone()),
                        });
                    }
                }
            }
        }
    }

    /// Generic foreign key validation
    fn check_foreign_keys(&self, package: &mut msi::Package<std::fs::File>, violations: &mut Vec<Violation>) {
        let fk_checks = [
            ("FeatureComponents", "Feature_", "Feature", 0usize),
            ("FeatureComponents", "Component_", "Component", 1usize),
            ("Shortcut", "Component_", "Component", 3usize),
            ("Registry", "Component_", "Component", 5usize),
            ("ServiceInstall", "Component_", "Component", 11usize),
        ];

        for (from_table, from_col, to_table, col_idx) in fk_checks {
            self.check_single_fk(package, from_table, from_col, to_table, col_idx, violations);
        }
    }

    fn check_single_fk(
        &self,
        package: &mut msi::Package<std::fs::File>,
        from_table: &str,
        from_col: &str,
        to_table: &str,
        col_idx: usize,
        violations: &mut Vec<Violation>,
    ) {
        if !package.tables().any(|t| t.name() == from_table) {
            return;
        }

        // Get valid target IDs
        let valid_ids: HashSet<String> = self.collect_primary_keys(package, to_table);

        if valid_ids.is_empty() {
            return; // Target table doesn't exist or is empty
        }

        if let Ok(rows) = package.select_rows(msi::Select::table(from_table)) {
            for row in rows {
                let pk = Self::get_str_or_empty(&row, 0);
                let fk_val = Self::get_str_or_empty(&row, col_idx);

                if !fk_val.is_empty() && !valid_ids.contains(&fk_val) {
                    violations.push(Violation {
                        rule_code: "ICE03".into(),
                        severity: Severity::Error,
                        message: format!(
                            "{}.{} references non-existent {}: '{}'",
                            from_table, from_col, to_table, fk_val
                        ),
                        table: Some(from_table.to_string()),
                        row_key: Some(pk),
                        column: Some(from_col.to_string()),
                        value: Some(fk_val),
                    });
                }
            }
        }
    }

    /// Collect all primary keys from a table
    fn collect_primary_keys(&self, package: &mut msi::Package<std::fs::File>, table_name: &str) -> HashSet<String> {
        package.select_rows(msi::Select::table(table_name))
            .ok()
            .map(|rows| {
                rows.filter_map(|r| Self::get_str(&r, 0))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Check for required tables
    fn check_required_tables(&self, package: &msi::Package<std::fs::File>, violations: &mut Vec<Violation>) {
        let required_tables = ["Property", "Component", "Feature", "Directory"];
        let existing: HashSet<String> = package.tables().map(|t| t.name().to_string()).collect();

        for table in required_tables {
            if !existing.contains(table) {
                violations.push(Violation {
                    rule_code: "ICE06".into(),
                    severity: Severity::Warning,
                    message: format!("Required table '{}' is missing", table),
                    table: Some(table.to_string()),
                    row_key: None,
                    column: None,
                    value: None,
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validator_creation() {
        let validator = Validator::with_builtin_rules();
        assert!(!validator.rules().is_empty());
    }
}

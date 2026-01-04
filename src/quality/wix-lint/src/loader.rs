//! Rule loader - loads lint rules from wix-data JSON files

use crate::rules::Rule;
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LoaderError {
    #[error("Failed to read rules directory: {0}")]
    ReadDir(#[from] std::io::Error),
    #[error("Failed to parse rule file {file}: {source}")]
    ParseRule {
        file: PathBuf,
        source: serde_json::Error,
    },
    #[error("Rules directory not found: {0}")]
    NotFound(PathBuf),
}

/// Loads lint rules from wix-data JSON files
pub struct RuleLoader {
    /// Path to wix-data directory
    wix_data_path: PathBuf,
}

/// JSON structure for a rules file
#[derive(Debug, Deserialize)]
struct RulesFile {
    /// Schema version for this rules file (reserved for future use)
    #[serde(default)]
    #[allow(dead_code)]
    version: Option<String>,
    rules: Vec<RuleJson>,
}

/// JSON structure for a single rule
#[derive(Debug, Deserialize)]
struct RuleJson {
    id: String,
    name: String,
    description: String,
    severity: String,
    element: String,
    condition: String,
    message: String,
    #[serde(default)]
    fix: Option<FixJson>,
    /// Version when this rule was added
    #[serde(default)]
    since: Option<String>,
    /// Whether this rule is deprecated
    #[serde(default)]
    deprecated: bool,
    /// Message explaining deprecation
    #[serde(default, rename = "deprecatedMessage")]
    deprecated_message: Option<String>,
    /// Rule ID that replaces this deprecated rule
    #[serde(default, rename = "replacedBy")]
    replaced_by: Option<String>,
}

/// JSON structure for a fix suggestion
#[derive(Debug, Deserialize)]
struct FixJson {
    action: String,
    attribute: Option<String>,
    value: Option<String>,
}

impl RuleLoader {
    /// Create a new rule loader
    pub fn new(wix_data_path: &Path) -> Self {
        Self {
            wix_data_path: wix_data_path.to_path_buf(),
        }
    }

    /// Load all rules from the rules directory
    pub fn load_all(&self) -> Result<Vec<Rule>, LoaderError> {
        let rules_dir = self.wix_data_path.join("rules");

        if !rules_dir.exists() {
            return Err(LoaderError::NotFound(rules_dir));
        }

        let mut rules = Vec::new();

        for entry in fs::read_dir(&rules_dir)? {
            let entry = entry?;
            let path = entry.path();

            // Only process JSON files ending with -rules.json
            if path.extension().is_some_and(|ext| ext == "json") {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name.ends_with("-rules.json") {
                        let file_rules = self.load_file(&path)?;
                        rules.extend(file_rules);
                    }
                }
            }
        }

        Ok(rules)
    }

    /// Load rules from a single file
    fn load_file(&self, path: &Path) -> Result<Vec<Rule>, LoaderError> {
        let content = fs::read_to_string(path)?;
        let rules_file: RulesFile =
            serde_json::from_str(&content).map_err(|e| LoaderError::ParseRule {
                file: path.to_path_buf(),
                source: e,
            })?;

        Ok(rules_file
            .rules
            .into_iter()
            .map(|r| Rule {
                id: r.id,
                name: r.name,
                description: r.description,
                severity: r.severity.parse().unwrap_or_default(),
                element: r.element,
                condition: r.condition,
                message: r.message,
                fix: r.fix.map(|f| crate::rules::FixTemplate {
                    action: f.action,
                    attribute: f.attribute,
                    value: f.value,
                }),
                since: r.since,
                deprecated: r.deprecated,
                deprecated_message: r.deprecated_message,
                replaced_by: r.replaced_by,
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_parse_rule_json() {
        let json = r#"{
            "rules": [{
                "id": "test-rule",
                "name": "Test Rule",
                "description": "A test rule",
                "severity": "error",
                "element": "Package",
                "condition": "!attributes.Name",
                "message": "Package needs a name"
            }]
        }"#;

        let rules_file: RulesFile = serde_json::from_str(json).unwrap();
        assert_eq!(rules_file.rules.len(), 1);
        assert_eq!(rules_file.rules[0].id, "test-rule");
    }

    #[test]
    fn test_parse_rule_with_fix() {
        let json = r#"{
            "rules": [{
                "id": "test-rule",
                "name": "Test Rule",
                "description": "A test rule",
                "severity": "error",
                "element": "Component",
                "condition": "!attributes.Guid",
                "message": "Component needs Guid",
                "fix": {
                    "action": "addAttribute",
                    "attribute": "Guid",
                    "value": "*"
                }
            }]
        }"#;

        let rules_file: RulesFile = serde_json::from_str(json).unwrap();
        assert_eq!(rules_file.rules.len(), 1);
        assert!(rules_file.rules[0].fix.is_some());
        let fix = rules_file.rules[0].fix.as_ref().unwrap();
        assert_eq!(fix.action, "addAttribute");
        assert_eq!(fix.attribute, Some("Guid".to_string()));
        assert_eq!(fix.value, Some("*".to_string()));
    }

    #[test]
    fn test_rule_loader_new() {
        let loader = RuleLoader::new(Path::new("/some/path"));
        assert_eq!(loader.wix_data_path, PathBuf::from("/some/path"));
    }

    #[test]
    fn test_load_all_not_found() {
        let loader = RuleLoader::new(Path::new("/nonexistent/path"));
        let result = loader.load_all();
        assert!(result.is_err());
        match result {
            Err(LoaderError::NotFound(_)) => (),
            _ => panic!("Expected NotFound error"),
        }
    }

    #[test]
    fn test_load_rules_from_directory() {
        let temp_dir = TempDir::new().unwrap();
        let rules_dir = temp_dir.path().join("rules");
        fs::create_dir(&rules_dir).unwrap();

        // Create a rules file
        let rules_content = r#"{
            "rules": [{
                "id": "test-rule-1",
                "name": "Test Rule 1",
                "description": "First test rule",
                "severity": "error",
                "element": "Package",
                "condition": "!attributes.Name",
                "message": "Package needs a name"
            }, {
                "id": "test-rule-2",
                "name": "Test Rule 2",
                "description": "Second test rule",
                "severity": "warning",
                "element": "Component",
                "condition": "!attributes.Guid",
                "message": "Component needs Guid"
            }]
        }"#;
        fs::write(rules_dir.join("test-rules.json"), rules_content).unwrap();

        let loader = RuleLoader::new(temp_dir.path());
        let rules = loader.load_all().unwrap();

        assert_eq!(rules.len(), 2);
        assert!(rules.iter().any(|r| r.id == "test-rule-1"));
        assert!(rules.iter().any(|r| r.id == "test-rule-2"));
    }

    #[test]
    fn test_load_multiple_rule_files() {
        let temp_dir = TempDir::new().unwrap();
        let rules_dir = temp_dir.path().join("rules");
        fs::create_dir(&rules_dir).unwrap();

        // First rules file
        let rules1 = r#"{"rules": [{"id": "rule-a", "name": "A", "description": "A", "severity": "error", "element": "Package", "condition": "true", "message": "A"}]}"#;
        fs::write(rules_dir.join("a-rules.json"), rules1).unwrap();

        // Second rules file
        let rules2 = r#"{"rules": [{"id": "rule-b", "name": "B", "description": "B", "severity": "warning", "element": "File", "condition": "true", "message": "B"}]}"#;
        fs::write(rules_dir.join("b-rules.json"), rules2).unwrap();

        let loader = RuleLoader::new(temp_dir.path());
        let rules = loader.load_all().unwrap();

        assert_eq!(rules.len(), 2);
    }

    #[test]
    fn test_skip_non_rules_json_files() {
        let temp_dir = TempDir::new().unwrap();
        let rules_dir = temp_dir.path().join("rules");
        fs::create_dir(&rules_dir).unwrap();

        // Valid rules file
        let rules = r#"{"rules": [{"id": "valid", "name": "V", "description": "V", "severity": "error", "element": "Package", "condition": "true", "message": "V"}]}"#;
        fs::write(rules_dir.join("valid-rules.json"), rules).unwrap();

        // File that doesn't end with -rules.json
        let other = r#"{"rules": [{"id": "other", "name": "O", "description": "O", "severity": "error", "element": "Package", "condition": "true", "message": "O"}]}"#;
        fs::write(rules_dir.join("other.json"), other).unwrap();

        // Non-JSON file
        fs::write(rules_dir.join("readme.txt"), "This is not JSON").unwrap();

        let loader = RuleLoader::new(temp_dir.path());
        let rules = loader.load_all().unwrap();

        // Only the valid-rules.json should be loaded
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].id, "valid");
    }

    #[test]
    fn test_rule_severity_parsing() {
        let temp_dir = TempDir::new().unwrap();
        let rules_dir = temp_dir.path().join("rules");
        fs::create_dir(&rules_dir).unwrap();

        let rules = r#"{"rules": [
            {"id": "e", "name": "E", "description": "E", "severity": "error", "element": "A", "condition": "true", "message": "E"},
            {"id": "w", "name": "W", "description": "W", "severity": "warning", "element": "B", "condition": "true", "message": "W"},
            {"id": "i", "name": "I", "description": "I", "severity": "info", "element": "C", "condition": "true", "message": "I"}
        ]}"#;
        fs::write(rules_dir.join("sev-rules.json"), rules).unwrap();

        let loader = RuleLoader::new(temp_dir.path());
        let loaded = loader.load_all().unwrap();

        let error_rule = loaded.iter().find(|r| r.id == "e").unwrap();
        let warning_rule = loaded.iter().find(|r| r.id == "w").unwrap();
        let info_rule = loaded.iter().find(|r| r.id == "i").unwrap();

        assert_eq!(error_rule.severity, crate::Severity::Error);
        assert_eq!(warning_rule.severity, crate::Severity::Warning);
        assert_eq!(info_rule.severity, crate::Severity::Info);
    }

    #[test]
    fn test_empty_rules_directory() {
        let temp_dir = TempDir::new().unwrap();
        let rules_dir = temp_dir.path().join("rules");
        fs::create_dir(&rules_dir).unwrap();

        let loader = RuleLoader::new(temp_dir.path());
        let rules = loader.load_all().unwrap();

        assert!(rules.is_empty());
    }
}

//! Migration assistant for WiX v3 to v4 to v5
//!
//! Automatically migrates WXS files between WiX versions with detailed change reports.
//!
//! # Example
//!
//! ```
//! use wix_migrate::{Migrator, WixVersion};
//!
//! let migrator = Migrator::new(WixVersion::V3, WixVersion::V4);
//! let old_content = r#"<Wix xmlns="http://schemas.microsoft.com/wix/2006/wi">"#;
//! let result = migrator.migrate(old_content);
//! assert!(result.new_content.contains("wixtoolset.org"));
//! ```

use regex::Regex;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Migration errors
#[derive(Error, Debug)]
pub enum MigrateError {
    #[error("Unsupported version: {0}")]
    UnsupportedVersion(String),
    #[error("Invalid migration path: {0} to {1}")]
    InvalidPath(String, String),
    #[error("Parse error: {0}")]
    ParseError(String),
}

/// WiX version
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum WixVersion {
    V3,
    V4,
    V5,
}

impl WixVersion {
    pub fn as_str(&self) -> &'static str {
        match self {
            WixVersion::V3 => "v3",
            WixVersion::V4 => "v4",
            WixVersion::V5 => "v5",
        }
    }

    pub fn namespace(&self) -> &'static str {
        match self {
            WixVersion::V3 => "http://schemas.microsoft.com/wix/2006/wi",
            WixVersion::V4 => "http://wixtoolset.org/schemas/v4/wxs",
            WixVersion::V5 => "http://wixtoolset.org/schemas/v4/wxs",
        }
    }

    pub fn from_content(content: &str) -> Option<WixVersion> {
        if content.contains("schemas.microsoft.com/wix/2006/wi") {
            Some(WixVersion::V3)
        } else if content.contains("wixtoolset.org/schemas/v4/wxs") {
            // v4 and v5 use same namespace, check for v5-specific elements
            if content.contains("StandardDirectory")
                || content.contains("Bitness=")
                || content.contains("<Package ")
            {
                Some(WixVersion::V5)
            } else {
                Some(WixVersion::V4)
            }
        } else {
            None
        }
    }
}

/// A single migration change
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationChange {
    /// Type of change
    pub change_type: ChangeType,
    /// Original text
    pub original: String,
    /// Replacement text
    pub replacement: String,
    /// Description of the change
    pub description: String,
    /// Line number (if available)
    pub line: Option<usize>,
    /// Breaking change that requires manual review
    pub breaking: bool,
}

/// Type of migration change
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChangeType {
    Namespace,
    ElementRename,
    ElementRemove,
    AttributeRename,
    AttributeRemove,
    AttributeAdd,
    ValueChange,
    StructuralChange,
}

/// Result of migration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MigrationResult {
    /// Migrated content
    pub new_content: String,
    /// All changes made
    pub changes: Vec<MigrationChange>,
    /// Source version
    pub from_version: String,
    /// Target version
    pub to_version: String,
    /// Warnings for manual review
    pub warnings: Vec<String>,
    /// Has breaking changes
    pub has_breaking_changes: bool,
}

impl MigrationResult {
    pub fn change_count(&self) -> usize {
        self.changes.len()
    }

    pub fn breaking_count(&self) -> usize {
        self.changes.iter().filter(|c| c.breaking).count()
    }
}

/// Migration rule
struct MigrationRule {
    pattern: Regex,
    replacement: String,
    description: String,
    change_type: ChangeType,
    breaking: bool,
}

/// WiX version migrator
pub struct Migrator {
    from: WixVersion,
    to: WixVersion,
    rules: Vec<MigrationRule>,
}

impl Migrator {
    pub fn new(from: WixVersion, to: WixVersion) -> Self {
        let rules = Self::build_rules(from, to);
        Self { from, to, rules }
    }

    /// Auto-detect source version and migrate to target
    pub fn auto_migrate(content: &str, to: WixVersion) -> Result<MigrationResult, MigrateError> {
        let from = WixVersion::from_content(content)
            .ok_or_else(|| MigrateError::ParseError("Could not detect WiX version".to_string()))?;

        if from >= to {
            return Err(MigrateError::InvalidPath(
                from.as_str().to_string(),
                to.as_str().to_string(),
            ));
        }

        let migrator = Migrator::new(from, to);
        Ok(migrator.migrate(content))
    }

    /// Migrate content
    pub fn migrate(&self, content: &str) -> MigrationResult {
        let mut result = MigrationResult {
            new_content: content.to_string(),
            from_version: self.from.as_str().to_string(),
            to_version: self.to.as_str().to_string(),
            ..Default::default()
        };

        for rule in &self.rules {
            // Apply all matches for this rule
            loop {
                let current_content = result.new_content.clone();
                if let Some(m) = rule.pattern.find(&current_content) {
                    let original = m.as_str().to_string();
                    let replacement = rule.pattern.replace(&original, &rule.replacement);

                    if original != replacement {
                        let line = current_content[..m.start()]
                            .lines()
                            .count()
                            .saturating_add(1);

                        result.changes.push(MigrationChange {
                            change_type: rule.change_type,
                            original: original.clone(),
                            replacement: replacement.to_string(),
                            description: rule.description.clone(),
                            line: Some(line),
                            breaking: rule.breaking,
                        });

                        if rule.breaking {
                            result.has_breaking_changes = true;
                        }

                        result.new_content = format!(
                            "{}{}{}",
                            &current_content[..m.start()],
                            replacement,
                            &current_content[m.end()..]
                        );
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            }
        }

        // Add warnings for manual review items
        self.add_warnings(&mut result);

        result
    }

    fn build_rules(from: WixVersion, to: WixVersion) -> Vec<MigrationRule> {
        let mut rules = Vec::new();

        // Build rules for each version step
        if from == WixVersion::V3 && to >= WixVersion::V4 {
            rules.extend(Self::v3_to_v4_rules());
        }

        if from <= WixVersion::V4 && to >= WixVersion::V5 {
            rules.extend(Self::v4_to_v5_rules());
        }

        rules
    }

    fn v3_to_v4_rules() -> Vec<MigrationRule> {
        vec![
            // Namespace change
            MigrationRule {
                pattern: Regex::new(r#"xmlns="http://schemas\.microsoft\.com/wix/2006/wi""#)
                    .unwrap(),
                replacement: r#"xmlns="http://wixtoolset.org/schemas/v4/wxs""#.to_string(),
                description: "Update namespace from WiX v3 to v4".to_string(),
                change_type: ChangeType::Namespace,
                breaking: false,
            },
            // Product -> Package
            MigrationRule {
                pattern: Regex::new(r"<Product\s").unwrap(),
                replacement: "<Package ".to_string(),
                description: "Rename Product element to Package".to_string(),
                change_type: ChangeType::ElementRename,
                breaking: false,
            },
            MigrationRule {
                pattern: Regex::new(r"</Product>").unwrap(),
                replacement: "</Package>".to_string(),
                description: "Rename closing Product tag to Package".to_string(),
                change_type: ChangeType::ElementRename,
                breaking: false,
            },
            // Id attribute rename
            MigrationRule {
                pattern: Regex::new(r#"Id="\*""#).unwrap(),
                replacement: r#"Guid="*""#.to_string(),
                description: "Rename Id='*' to Guid='*' for auto-generated GUIDs".to_string(),
                change_type: ChangeType::AttributeRename,
                breaking: false,
            },
            // Win64 -> Bitness
            MigrationRule {
                pattern: Regex::new(r#"Win64="yes""#).unwrap(),
                replacement: r#"Bitness="always64""#.to_string(),
                description: "Replace Win64='yes' with Bitness='always64'".to_string(),
                change_type: ChangeType::AttributeRename,
                breaking: false,
            },
            MigrationRule {
                pattern: Regex::new(r#"Win64="no""#).unwrap(),
                replacement: r#"Bitness="always32""#.to_string(),
                description: "Replace Win64='no' with Bitness='always32'".to_string(),
                change_type: ChangeType::AttributeRename,
                breaking: false,
            },
            // Directory element changes
            MigrationRule {
                pattern: Regex::new(r#"<Directory\s+Id="TARGETDIR"\s+Name="SourceDir">"#).unwrap(),
                replacement: r#"<StandardDirectory Id="ProgramFilesFolder">"#.to_string(),
                description: "Replace TARGETDIR with StandardDirectory".to_string(),
                change_type: ChangeType::StructuralChange,
                breaking: true,
            },
            // RemoveFile -> RemoveFolder (for folders)
            MigrationRule {
                pattern: Regex::new(r#"<RemoveFile\s+Id="([^"]+)"\s+On="uninstall"\s+Name="\*\.\*"\s*/>"#).unwrap(),
                replacement: r#"<RemoveFolder Id="$1" On="uninstall" />"#.to_string(),
                description: "Convert RemoveFile with *.* to RemoveFolder".to_string(),
                change_type: ChangeType::ElementRename,
                breaking: false,
            },
            // CustomAction Schedule changes
            MigrationRule {
                pattern: Regex::new(r#"Execute="deferred""#).unwrap(),
                replacement: r#"Execute="deferred" Impersonate="no""#.to_string(),
                description: "Add Impersonate='no' to deferred custom actions".to_string(),
                change_type: ChangeType::AttributeAdd,
                breaking: true,
            },
            // util:XmlFile namespace
            MigrationRule {
                pattern: Regex::new(r#"xmlns:util="http://schemas\.microsoft\.com/wix/UtilExtension""#).unwrap(),
                replacement: r#"xmlns:util="http://wixtoolset.org/schemas/v4/wxs/util""#.to_string(),
                description: "Update Util extension namespace".to_string(),
                change_type: ChangeType::Namespace,
                breaking: false,
            },
        ]
    }

    fn v4_to_v5_rules() -> Vec<MigrationRule> {
        vec![
            // MajorUpgrade changes
            MigrationRule {
                pattern: Regex::new(r#"<MajorUpgrade\s+Schedule="afterInstallInitialize""#).unwrap(),
                replacement: r#"<MajorUpgrade Schedule="afterInstallValidate""#.to_string(),
                description: "Change MajorUpgrade schedule to afterInstallValidate".to_string(),
                change_type: ChangeType::ValueChange,
                breaking: true,
            },
            // Component Guid attribute for permanent components
            MigrationRule {
                pattern: Regex::new(r#"<Component([^>]*)\s+Permanent="yes"([^>]*)>"#).unwrap(),
                replacement: r#"<Component$1$2 Permanent="yes" Guid="*">"#.to_string(),
                description: "Add explicit Guid for permanent components".to_string(),
                change_type: ChangeType::AttributeAdd,
                breaking: false,
            },
            // StandardDirectory for common folders
            MigrationRule {
                pattern: Regex::new(r#"<Directory\s+Id="ProgramFilesFolder"\s*/>"#).unwrap(),
                replacement: r#"<StandardDirectory Id="ProgramFilesFolder" />"#.to_string(),
                description: "Use StandardDirectory for system folders".to_string(),
                change_type: ChangeType::ElementRename,
                breaking: false,
            },
            MigrationRule {
                pattern: Regex::new(r#"<Directory\s+Id="ProgramFiles64Folder"\s*/>"#).unwrap(),
                replacement: r#"<StandardDirectory Id="ProgramFiles64Folder" />"#.to_string(),
                description: "Use StandardDirectory for system folders".to_string(),
                change_type: ChangeType::ElementRename,
                breaking: false,
            },
        ]
    }

    fn add_warnings(&self, result: &mut MigrationResult) {
        // Check for custom actions that need review
        if result.new_content.contains("<CustomAction") {
            result.warnings.push(
                "Custom actions detected - review Execute and Impersonate attributes".to_string(),
            );
        }

        // Check for deprecated elements
        if result.new_content.contains("<Property Id=\"ARPNOMODIFY\"") {
            result.warnings.push(
                "ARPNOMODIFY property detected - consider using Package/@Modify attribute instead"
                    .to_string(),
            );
        }

        // Check for hardcoded GUIDs
        let guid_pattern =
            Regex::new(r#"Guid="[0-9A-Fa-f]{8}-[0-9A-Fa-f]{4}-[0-9A-Fa-f]{4}-[0-9A-Fa-f]{4}-[0-9A-Fa-f]{12}""#).unwrap();
        if guid_pattern.is_match(&result.new_content) {
            result.warnings.push(
                "Hardcoded GUIDs detected - consider using Guid='*' for auto-generation".to_string(),
            );
        }

        // Check for RegistrySearch
        if result.new_content.contains("<RegistrySearch") {
            result.warnings.push(
                "RegistrySearch elements detected - verify Root and Key attributes are correct"
                    .to_string(),
            );
        }
    }
}

/// Quick detection of WiX version
pub fn detect_version(content: &str) -> Option<WixVersion> {
    WixVersion::from_content(content)
}

/// List of breaking changes between versions
pub fn breaking_changes(from: WixVersion, to: WixVersion) -> Vec<&'static str> {
    let mut changes = Vec::new();

    if from == WixVersion::V3 && to >= WixVersion::V4 {
        changes.extend([
            "Product element renamed to Package",
            "Namespace changed from microsoft.com to wixtoolset.org",
            "Win64 attribute replaced with Bitness",
            "Custom action Execute='deferred' requires Impersonate attribute",
            "TARGETDIR/SourceDir replaced with StandardDirectory",
            "Extension namespaces changed",
        ]);
    }

    if from <= WixVersion::V4 && to >= WixVersion::V5 {
        changes.extend([
            "MajorUpgrade Schedule changed from afterInstallInitialize",
            "StandardDirectory required for system folders",
            "Package element structure changes",
        ]);
    }

    changes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_v3() {
        let content = r#"<Wix xmlns="http://schemas.microsoft.com/wix/2006/wi">"#;
        assert_eq!(detect_version(content), Some(WixVersion::V3));
    }

    #[test]
    fn test_detect_v4() {
        let content = r#"<Wix xmlns="http://wixtoolset.org/schemas/v4/wxs">
            <Product Id="..." Name="Test" />
        </Wix>"#;
        assert_eq!(detect_version(content), Some(WixVersion::V4));
    }

    #[test]
    fn test_detect_v5() {
        let content = r#"<Wix xmlns="http://wixtoolset.org/schemas/v4/wxs">
            <Package Name="Test" />
            <StandardDirectory Id="ProgramFilesFolder" />
        </Wix>"#;
        assert_eq!(detect_version(content), Some(WixVersion::V5));
    }

    #[test]
    fn test_namespace_migration() {
        let migrator = Migrator::new(WixVersion::V3, WixVersion::V4);
        let content = r#"<Wix xmlns="http://schemas.microsoft.com/wix/2006/wi">"#;

        let result = migrator.migrate(content);

        assert!(result.new_content.contains("wixtoolset.org/schemas/v4/wxs"));
        assert!(!result.changes.is_empty());
    }

    #[test]
    fn test_product_to_package() {
        let migrator = Migrator::new(WixVersion::V3, WixVersion::V4);
        let content = r#"<Product Id="*" Name="Test"></Product>"#;

        let result = migrator.migrate(content);

        assert!(result.new_content.contains("<Package"));
        assert!(result.new_content.contains("</Package>"));
    }

    #[test]
    fn test_win64_to_bitness() {
        let migrator = Migrator::new(WixVersion::V3, WixVersion::V4);
        let content = r#"<Component Win64="yes" />"#;

        let result = migrator.migrate(content);

        assert!(result.new_content.contains(r#"Bitness="always64""#));
    }

    #[test]
    fn test_auto_migrate() {
        let content = r#"<Wix xmlns="http://schemas.microsoft.com/wix/2006/wi">
            <Product Id="*" Name="Test">
                <Component Win64="yes" />
            </Product>
        </Wix>"#;

        let result = Migrator::auto_migrate(content, WixVersion::V4).unwrap();

        assert!(result.new_content.contains("wixtoolset.org"));
        assert!(result.new_content.contains("<Package"));
        assert!(result.new_content.contains("Bitness="));
    }

    #[test]
    fn test_auto_migrate_invalid_path() {
        let content = r#"<Wix xmlns="http://wixtoolset.org/schemas/v4/wxs">
            <Package Name="Test" />
        </Wix>"#;

        let result = Migrator::auto_migrate(content, WixVersion::V3);
        assert!(matches!(result, Err(MigrateError::InvalidPath(_, _))));
    }

    #[test]
    fn test_change_count() {
        let migrator = Migrator::new(WixVersion::V3, WixVersion::V4);
        let content = r#"<Wix xmlns="http://schemas.microsoft.com/wix/2006/wi">
            <Product Id="*" Name="Test" Win64="yes"></Product>
        </Wix>"#;

        let result = migrator.migrate(content);

        assert!(result.change_count() > 0);
    }

    #[test]
    fn test_breaking_changes_flag() {
        let migrator = Migrator::new(WixVersion::V3, WixVersion::V4);
        let content = r#"<Directory Id="TARGETDIR" Name="SourceDir">"#;

        let result = migrator.migrate(content);

        assert!(result.has_breaking_changes);
    }

    #[test]
    fn test_util_namespace() {
        let migrator = Migrator::new(WixVersion::V3, WixVersion::V4);
        let content = r#"xmlns:util="http://schemas.microsoft.com/wix/UtilExtension""#;

        let result = migrator.migrate(content);

        assert!(result.new_content.contains("wixtoolset.org/schemas/v4/wxs/util"));
    }

    #[test]
    fn test_warnings_custom_action() {
        let migrator = Migrator::new(WixVersion::V3, WixVersion::V4);
        let content = r#"<CustomAction Id="Test" />"#;

        let result = migrator.migrate(content);

        assert!(result.warnings.iter().any(|w| w.contains("Custom actions")));
    }

    #[test]
    fn test_warnings_hardcoded_guid() {
        let migrator = Migrator::new(WixVersion::V3, WixVersion::V4);
        let content = r#"<Component Guid="12345678-1234-1234-1234-123456789012" />"#;

        let result = migrator.migrate(content);

        assert!(result.warnings.iter().any(|w| w.contains("Hardcoded GUIDs")));
    }

    #[test]
    fn test_version_ordering() {
        assert!(WixVersion::V3 < WixVersion::V4);
        assert!(WixVersion::V4 < WixVersion::V5);
        assert!(WixVersion::V3 < WixVersion::V5);
    }

    #[test]
    fn test_breaking_changes_list() {
        let changes = breaking_changes(WixVersion::V3, WixVersion::V4);
        assert!(!changes.is_empty());
        assert!(changes.iter().any(|c| c.contains("Product")));
    }

    #[test]
    fn test_no_changes_needed() {
        let migrator = Migrator::new(WixVersion::V4, WixVersion::V5);
        let content = r#"<Wix xmlns="http://wixtoolset.org/schemas/v4/wxs">
            <Package Name="Already migrated" />
        </Wix>"#;

        let result = migrator.migrate(content);

        // Some changes might still be made (like StandardDirectory)
        // but content should remain valid
        assert!(!result.new_content.is_empty());
    }

    #[test]
    fn test_multiple_occurrences() {
        let migrator = Migrator::new(WixVersion::V3, WixVersion::V4);
        let content = r#"<Component Win64="yes" /><Component Win64="yes" />"#;

        let result = migrator.migrate(content);

        // Count occurrences of Bitness
        let bitness_count = result.new_content.matches("Bitness=").count();
        assert_eq!(bitness_count, 2);
    }

    #[test]
    fn test_change_line_numbers() {
        let migrator = Migrator::new(WixVersion::V3, WixVersion::V4);
        let content = r#"<Wix>
<Product Id="*">
</Product>
</Wix>"#;

        let result = migrator.migrate(content);

        // Check that line numbers are captured
        let product_change = result
            .changes
            .iter()
            .find(|c| c.original.contains("Product"));
        if let Some(change) = product_change {
            assert!(change.line.is_some());
        }
    }

    #[test]
    fn test_wix_version_as_str() {
        assert_eq!(WixVersion::V3.as_str(), "v3");
        assert_eq!(WixVersion::V4.as_str(), "v4");
        assert_eq!(WixVersion::V5.as_str(), "v5");
    }

    #[test]
    fn test_wix_version_namespace() {
        assert!(WixVersion::V3.namespace().contains("microsoft.com"));
        assert!(WixVersion::V4.namespace().contains("wixtoolset.org"));
    }

    #[test]
    fn test_v4_to_v5_standard_directory() {
        let migrator = Migrator::new(WixVersion::V4, WixVersion::V5);
        let content = r#"<Directory Id="ProgramFilesFolder" />"#;

        let result = migrator.migrate(content);

        assert!(result.new_content.contains("<StandardDirectory"));
    }

    #[test]
    fn test_migration_result_defaults() {
        let result = MigrationResult::default();

        assert!(result.new_content.is_empty());
        assert!(result.changes.is_empty());
        assert!(!result.has_breaking_changes);
        assert_eq!(result.change_count(), 0);
        assert_eq!(result.breaking_count(), 0);
    }
}

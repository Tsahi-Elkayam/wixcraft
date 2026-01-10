//! wix-upgrade - Upgrade path validator for WiX/MSI
//!
//! Validates upgrade compatibility between MSI versions by checking:
//! - Component GUID consistency
//! - Feature tree compatibility
//! - ProductCode/UpgradeCode rules
//! - Version number format
//! - Minor vs major upgrade requirements

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Component information extracted from WiX source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentInfo {
    pub id: String,
    pub guid: Option<String>,
    pub directory: Option<String>,
    pub key_path: Option<String>,
    pub files: Vec<String>,
    pub line: usize,
}

/// Feature information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureInfo {
    pub id: String,
    pub title: Option<String>,
    pub level: Option<u32>,
    pub components: Vec<String>,
    pub children: Vec<String>,
    pub line: usize,
}

/// Product/Package information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductInfo {
    pub name: Option<String>,
    pub version: Option<String>,
    pub manufacturer: Option<String>,
    pub product_code: Option<String>,
    pub upgrade_code: Option<String>,
}

/// Extracted project information
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProjectInfo {
    pub product: ProductInfo,
    pub components: Vec<ComponentInfo>,
    pub features: Vec<FeatureInfo>,
    pub source_file: String,
}

impl Default for ProductInfo {
    fn default() -> Self {
        Self {
            name: None,
            version: None,
            manufacturer: None,
            product_code: None,
            upgrade_code: None,
        }
    }
}

/// Upgrade validation issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpgradeIssue {
    pub id: String,
    pub severity: IssueSeverity,
    pub title: String,
    pub description: String,
    pub affected: String,
    pub suggestion: String,
    pub breaking: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IssueSeverity {
    Error,
    Warning,
    Info,
}

impl std::fmt::Display for IssueSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IssueSeverity::Error => write!(f, "ERROR"),
            IssueSeverity::Warning => write!(f, "WARNING"),
            IssueSeverity::Info => write!(f, "INFO"),
        }
    }
}

/// Upgrade type determination
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UpgradeType {
    SmallUpdate,   // Same ProductCode, same version
    MinorUpgrade,  // Same ProductCode, version change
    MajorUpgrade,  // Different ProductCode
    Unknown,
}

impl std::fmt::Display for UpgradeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UpgradeType::SmallUpdate => write!(f, "Small Update"),
            UpgradeType::MinorUpgrade => write!(f, "Minor Upgrade"),
            UpgradeType::MajorUpgrade => write!(f, "Major Upgrade"),
            UpgradeType::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Upgrade validation result
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UpgradeValidation {
    pub upgrade_type: Option<UpgradeType>,
    pub issues: Vec<UpgradeIssue>,
    pub error_count: usize,
    pub warning_count: usize,
    pub info_count: usize,
    pub compatible: bool,
}

impl UpgradeValidation {
    pub fn add_issue(&mut self, issue: UpgradeIssue) {
        match issue.severity {
            IssueSeverity::Error => self.error_count += 1,
            IssueSeverity::Warning => self.warning_count += 1,
            IssueSeverity::Info => self.info_count += 1,
        }
        if issue.breaking {
            self.compatible = false;
        }
        self.issues.push(issue);
    }
}

/// Upgrade validator
pub struct UpgradeValidator;

impl Default for UpgradeValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl UpgradeValidator {
    pub fn new() -> Self {
        Self
    }

    /// Extract project information from WiX source
    pub fn extract_info(&self, content: &str, filename: &str) -> ProjectInfo {
        let mut info = ProjectInfo {
            source_file: filename.to_string(),
            ..Default::default()
        };

        if let Ok(doc) = roxmltree::Document::parse(content) {
            // Extract Package/Product info
            for node in doc.descendants() {
                match node.tag_name().name() {
                    "Package" | "Product" => {
                        info.product.name = node.attribute("Name").map(String::from);
                        info.product.version = node.attribute("Version").map(String::from);
                        info.product.manufacturer = node.attribute("Manufacturer").map(String::from);
                        info.product.product_code = node.attribute("Id")
                            .or_else(|| node.attribute("ProductCode"))
                            .map(String::from);
                        info.product.upgrade_code = node.attribute("UpgradeCode").map(String::from);
                    }
                    "Component" => {
                        let comp = ComponentInfo {
                            id: node.attribute("Id").unwrap_or("").to_string(),
                            guid: node.attribute("Guid").map(String::from),
                            directory: node.parent()
                                .and_then(|p| p.attribute("Id"))
                                .map(String::from),
                            key_path: None, // Set below
                            files: Vec::new(), // Populated below
                            line: get_line_number(content, &node),
                        };
                        info.components.push(comp);
                    }
                    "Feature" => {
                        let feature = FeatureInfo {
                            id: node.attribute("Id").unwrap_or("").to_string(),
                            title: node.attribute("Title").map(String::from),
                            level: node.attribute("Level").and_then(|l| l.parse().ok()),
                            components: Vec::new(),
                            children: Vec::new(),
                            line: get_line_number(content, &node),
                        };
                        info.features.push(feature);
                    }
                    _ => {}
                }
            }

            // Second pass to get files and component refs
            for node in doc.descendants() {
                if node.tag_name().name() == "File" {
                    if let Some(parent) = node.parent() {
                        if parent.tag_name().name() == "Component" {
                            let comp_id = parent.attribute("Id").unwrap_or("");
                            let file_name = node.attribute("Source")
                                .or_else(|| node.attribute("Name"))
                                .unwrap_or("");
                            let is_keypath = node.attribute("KeyPath") == Some("yes");

                            if let Some(comp) = info.components.iter_mut().find(|c| c.id == comp_id) {
                                comp.files.push(file_name.to_string());
                                if is_keypath {
                                    comp.key_path = Some(file_name.to_string());
                                }
                            }
                        }
                    }
                }

                if node.tag_name().name() == "ComponentRef" {
                    if let Some(parent) = node.parent() {
                        if parent.tag_name().name() == "Feature" {
                            let feature_id = parent.attribute("Id").unwrap_or("");
                            let comp_id = node.attribute("Id").unwrap_or("");

                            if let Some(feature) = info.features.iter_mut().find(|f| f.id == feature_id) {
                                feature.components.push(comp_id.to_string());
                            }
                        }
                    }
                }
            }
        }

        info
    }

    /// Validate single project for upgrade readiness
    pub fn validate_single(&self, info: &ProjectInfo) -> UpgradeValidation {
        let mut result = UpgradeValidation {
            compatible: true,
            ..Default::default()
        };

        // Check version format
        if let Some(ref version) = info.product.version {
            self.check_version_format(version, &mut result);
        }

        // Check UpgradeCode presence
        if info.product.upgrade_code.is_none() {
            result.add_issue(UpgradeIssue {
                id: "UPG001".to_string(),
                severity: IssueSeverity::Warning,
                title: "Missing UpgradeCode".to_string(),
                description: "No UpgradeCode specified. This makes it impossible to detect and upgrade previous versions.".to_string(),
                affected: "Package".to_string(),
                suggestion: "Add UpgradeCode attribute with a stable GUID that never changes between versions.".to_string(),
                breaking: false,
            });
        }

        // Check for auto-generated GUIDs
        for comp in &info.components {
            if comp.guid.as_deref() == Some("*") {
                // Auto-GUID is fine
            } else if comp.guid.is_none() {
                result.add_issue(UpgradeIssue {
                    id: "UPG002".to_string(),
                    severity: IssueSeverity::Warning,
                    title: "Component without GUID".to_string(),
                    description: format!("Component '{}' has no GUID specified.", comp.id),
                    affected: comp.id.clone(),
                    suggestion: "Use Guid=\"*\" for auto-generation or specify a stable GUID.".to_string(),
                    breaking: false,
                });
            }
        }

        // Check for components without KeyPath
        for comp in &info.components {
            if comp.key_path.is_none() && !comp.files.is_empty() {
                result.add_issue(UpgradeIssue {
                    id: "UPG003".to_string(),
                    severity: IssueSeverity::Info,
                    title: "Component without explicit KeyPath".to_string(),
                    description: format!("Component '{}' has files but no explicit KeyPath.", comp.id),
                    affected: comp.id.clone(),
                    suggestion: "Add KeyPath=\"yes\" to the primary file to ensure proper component tracking.".to_string(),
                    breaking: false,
                });
            }
        }

        // Check for duplicate component GUIDs
        let mut guid_map: HashMap<String, Vec<String>> = HashMap::new();
        for comp in &info.components {
            if let Some(ref guid) = comp.guid {
                if guid != "*" {
                    guid_map.entry(guid.to_uppercase()).or_default().push(comp.id.clone());
                }
            }
        }

        for (guid, comps) in &guid_map {
            if comps.len() > 1 {
                result.add_issue(UpgradeIssue {
                    id: "UPG004".to_string(),
                    severity: IssueSeverity::Error,
                    title: "Duplicate Component GUID".to_string(),
                    description: format!(
                        "Components {} share the same GUID {}. This will cause unpredictable behavior.",
                        comps.join(", "), guid
                    ),
                    affected: comps.join(", "),
                    suggestion: "Each component must have a unique GUID. Use Guid=\"*\" for auto-generation.".to_string(),
                    breaking: true,
                });
            }
        }

        result
    }

    /// Compare two versions and validate upgrade path
    pub fn validate_upgrade(&self, old: &ProjectInfo, new: &ProjectInfo) -> UpgradeValidation {
        let mut result = UpgradeValidation {
            compatible: true,
            ..Default::default()
        };

        // Determine upgrade type
        result.upgrade_type = Some(self.determine_upgrade_type(old, new));

        // Check UpgradeCode consistency
        if old.product.upgrade_code != new.product.upgrade_code {
            if old.product.upgrade_code.is_some() && new.product.upgrade_code.is_some() {
                result.add_issue(UpgradeIssue {
                    id: "UPG010".to_string(),
                    severity: IssueSeverity::Error,
                    title: "UpgradeCode Changed".to_string(),
                    description: format!(
                        "UpgradeCode changed from '{}' to '{}'. This breaks the upgrade path.",
                        old.product.upgrade_code.as_deref().unwrap_or("none"),
                        new.product.upgrade_code.as_deref().unwrap_or("none")
                    ),
                    affected: "Package".to_string(),
                    suggestion: "UpgradeCode must remain constant across all versions. Revert to the original value.".to_string(),
                    breaking: true,
                });
            }
        }

        // Check version progression
        if let (Some(old_ver), Some(new_ver)) = (&old.product.version, &new.product.version) {
            self.check_version_progression(old_ver, new_ver, &mut result);
        }

        // Check component GUID changes
        self.check_component_guid_changes(old, new, &mut result);

        // Check for removed components
        self.check_removed_components(old, new, &mut result);

        // Check for feature tree changes (important for minor upgrades)
        if result.upgrade_type == Some(UpgradeType::MinorUpgrade) {
            self.check_feature_tree_changes(old, new, &mut result);
        }

        result
    }

    fn determine_upgrade_type(&self, old: &ProjectInfo, new: &ProjectInfo) -> UpgradeType {
        let old_code = old.product.product_code.as_deref();
        let new_code = new.product.product_code.as_deref();

        // If either uses "*" (auto-gen), assume major upgrade
        if old_code == Some("*") || new_code == Some("*") {
            return UpgradeType::MajorUpgrade;
        }

        if old_code != new_code {
            return UpgradeType::MajorUpgrade;
        }

        // Same ProductCode - check version
        match (&old.product.version, &new.product.version) {
            (Some(old_ver), Some(new_ver)) => {
                if old_ver == new_ver {
                    UpgradeType::SmallUpdate
                } else {
                    UpgradeType::MinorUpgrade
                }
            }
            _ => UpgradeType::Unknown,
        }
    }

    fn check_version_format(&self, version: &str, result: &mut UpgradeValidation) {
        let parts: Vec<&str> = version.split('.').collect();

        // Check for 4-part version
        if parts.len() >= 4 {
            result.add_issue(UpgradeIssue {
                id: "UPG005".to_string(),
                severity: IssueSeverity::Warning,
                title: "Four-Part Version Number".to_string(),
                description: format!(
                    "Version '{}' has 4 parts. Windows Installer ignores the 4th part for comparison.",
                    version
                ),
                affected: version.to_string(),
                suggestion: "Use only 3 parts (major.minor.build) or ensure 4th part is only for display.".to_string(),
                breaking: false,
            });
        }

        // Check individual part ranges
        for (i, part) in parts.iter().enumerate() {
            if let Ok(num) = part.parse::<u32>() {
                let max = if i == 2 { 65535 } else { 255 };
                if num > max {
                    result.add_issue(UpgradeIssue {
                        id: "UPG006".to_string(),
                        severity: IssueSeverity::Error,
                        title: "Version Part Out of Range".to_string(),
                        description: format!(
                            "Version part {} ({}) exceeds maximum value {}.",
                            i + 1, num, max
                        ),
                        affected: version.to_string(),
                        suggestion: format!("Version part {} must be <= {}.", i + 1, max),
                        breaking: true,
                    });
                }
            }
        }
    }

    fn check_version_progression(&self, old: &str, new: &str, result: &mut UpgradeValidation) {
        let old_parts: Vec<u32> = old.split('.').filter_map(|p| p.parse().ok()).collect();
        let new_parts: Vec<u32> = new.split('.').filter_map(|p| p.parse().ok()).collect();

        // Compare first 3 parts only (Windows Installer ignores 4th)
        let old_cmp: Vec<u32> = old_parts.iter().take(3).copied().collect();
        let new_cmp: Vec<u32> = new_parts.iter().take(3).copied().collect();

        if new_cmp <= old_cmp {
            result.add_issue(UpgradeIssue {
                id: "UPG007".to_string(),
                severity: IssueSeverity::Error,
                title: "Version Not Increased".to_string(),
                description: format!(
                    "New version '{}' is not greater than old version '{}'. Upgrades require increasing version.",
                    new, old
                ),
                affected: format!("{} -> {}", old, new),
                suggestion: "Increment at least one of the first three version parts.".to_string(),
                breaking: true,
            });
        }
    }

    fn check_component_guid_changes(&self, old: &ProjectInfo, new: &ProjectInfo, result: &mut UpgradeValidation) {
        let old_map: HashMap<String, &ComponentInfo> = old.components.iter()
            .map(|c| (c.id.clone(), c))
            .collect();

        for new_comp in &new.components {
            if let Some(old_comp) = old_map.get(&new_comp.id) {
                // Check if GUID changed (and neither is auto-gen)
                match (&old_comp.guid, &new_comp.guid) {
                    (Some(old_guid), Some(new_guid)) if old_guid != "*" && new_guid != "*" => {
                        if old_guid.to_uppercase() != new_guid.to_uppercase() {
                            result.add_issue(UpgradeIssue {
                                id: "UPG008".to_string(),
                                severity: IssueSeverity::Error,
                                title: "Component GUID Changed".to_string(),
                                description: format!(
                                    "Component '{}' GUID changed from '{}' to '{}'. This requires a major upgrade.",
                                    new_comp.id, old_guid, new_guid
                                ),
                                affected: new_comp.id.clone(),
                                suggestion: "Revert to the original GUID or ensure this is a major upgrade.".to_string(),
                                breaking: true,
                            });
                        }
                    }
                    _ => {}
                }

                // Check if KeyPath changed
                if old_comp.key_path != new_comp.key_path {
                    result.add_issue(UpgradeIssue {
                        id: "UPG009".to_string(),
                        severity: IssueSeverity::Warning,
                        title: "Component KeyPath Changed".to_string(),
                        description: format!(
                            "Component '{}' KeyPath changed from '{}' to '{}'.",
                            new_comp.id,
                            old_comp.key_path.as_deref().unwrap_or("none"),
                            new_comp.key_path.as_deref().unwrap_or("none")
                        ),
                        affected: new_comp.id.clone(),
                        suggestion: "KeyPath changes can affect component detection during repair.".to_string(),
                        breaking: false,
                    });
                }
            }
        }
    }

    fn check_removed_components(&self, old: &ProjectInfo, new: &ProjectInfo, result: &mut UpgradeValidation) {
        let new_ids: HashSet<_> = new.components.iter().map(|c| &c.id).collect();

        for old_comp in &old.components {
            if !new_ids.contains(&old_comp.id) {
                result.add_issue(UpgradeIssue {
                    id: "UPG011".to_string(),
                    severity: IssueSeverity::Warning,
                    title: "Component Removed".to_string(),
                    description: format!(
                        "Component '{}' was removed. Ensure proper cleanup in upgrade.",
                        old_comp.id
                    ),
                    affected: old_comp.id.clone(),
                    suggestion: "Use RemoveExistingProducts or ensure files are properly cleaned up.".to_string(),
                    breaking: false,
                });
            }
        }
    }

    fn check_feature_tree_changes(&self, old: &ProjectInfo, new: &ProjectInfo, result: &mut UpgradeValidation) {
        let old_features: HashSet<_> = old.features.iter().map(|f| &f.id).collect();
        let new_features: HashSet<_> = new.features.iter().map(|f| &f.id).collect();

        // Check for removed features
        for old_feature in &old.features {
            if !new_features.contains(&old_feature.id) {
                result.add_issue(UpgradeIssue {
                    id: "UPG012".to_string(),
                    severity: IssueSeverity::Error,
                    title: "Feature Removed in Minor Upgrade".to_string(),
                    description: format!(
                        "Feature '{}' was removed. Minor upgrades cannot remove features.",
                        old_feature.id
                    ),
                    affected: old_feature.id.clone(),
                    suggestion: "Use a major upgrade to remove features, or keep the feature.".to_string(),
                    breaking: true,
                });
            }
        }

        // Check for added features (warning only)
        for new_feature in &new.features {
            if !old_features.contains(&new_feature.id) {
                result.add_issue(UpgradeIssue {
                    id: "UPG013".to_string(),
                    severity: IssueSeverity::Info,
                    title: "New Feature Added".to_string(),
                    description: format!("Feature '{}' was added.", new_feature.id),
                    affected: new_feature.id.clone(),
                    suggestion: "Ensure new features have appropriate default install levels.".to_string(),
                    breaking: false,
                });
            }
        }
    }
}

fn get_line_number(content: &str, node: &roxmltree::Node) -> usize {
    let pos = node.range().start;
    content[..pos].matches('\n').count() + 1
}

/// Check if MajorUpgrade element is present
pub fn has_major_upgrade(content: &str) -> bool {
    if let Ok(doc) = roxmltree::Document::parse(content) {
        for node in doc.descendants() {
            if node.tag_name().name() == "MajorUpgrade" {
                return true;
            }
        }
    }
    false
}

/// Validate GUID format
pub fn validate_guid(guid: &str) -> bool {
    if guid == "*" {
        return true;
    }

    let guid_regex = Regex::new(
        r"(?i)^\{?[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}\}?$"
    ).unwrap();

    guid_regex.is_match(guid)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_basic_info() {
        let content = r#"
        <Wix xmlns="http://wixtoolset.org/schemas/v4/wxs">
            <Package Name="TestApp" Version="1.0.0" Manufacturer="Test" UpgradeCode="{12345678-1234-1234-1234-123456789012}">
                <Component Id="Comp1" Guid="*">
                    <File Source="app.exe" KeyPath="yes" />
                </Component>
            </Package>
        </Wix>
        "#;

        let validator = UpgradeValidator::new();
        let info = validator.extract_info(content, "test.wxs");

        assert_eq!(info.product.name, Some("TestApp".to_string()));
        assert_eq!(info.product.version, Some("1.0.0".to_string()));
        assert!(info.product.upgrade_code.is_some());
        assert_eq!(info.components.len(), 1);
        assert_eq!(info.components[0].id, "Comp1");
    }

    #[test]
    fn test_validate_version_format() {
        let validator = UpgradeValidator::new();

        // Test 4-part version
        let content = r#"
        <Wix xmlns="http://wixtoolset.org/schemas/v4/wxs">
            <Package Name="Test" Version="1.0.0.1" />
        </Wix>
        "#;
        let info = validator.extract_info(content, "test.wxs");
        let result = validator.validate_single(&info);
        assert!(result.issues.iter().any(|i| i.id == "UPG005"));
    }

    #[test]
    fn test_validate_missing_upgrade_code() {
        let validator = UpgradeValidator::new();

        let content = r#"
        <Wix xmlns="http://wixtoolset.org/schemas/v4/wxs">
            <Package Name="Test" Version="1.0.0" />
        </Wix>
        "#;
        let info = validator.extract_info(content, "test.wxs");
        let result = validator.validate_single(&info);
        assert!(result.issues.iter().any(|i| i.id == "UPG001"));
    }

    #[test]
    fn test_validate_duplicate_guids() {
        let validator = UpgradeValidator::new();

        let content = r#"
        <Wix xmlns="http://wixtoolset.org/schemas/v4/wxs">
            <Package Name="Test" Version="1.0.0">
                <Component Id="Comp1" Guid="{11111111-1111-1111-1111-111111111111}" />
                <Component Id="Comp2" Guid="{11111111-1111-1111-1111-111111111111}" />
            </Package>
        </Wix>
        "#;
        let info = validator.extract_info(content, "test.wxs");
        let result = validator.validate_single(&info);
        assert!(result.issues.iter().any(|i| i.id == "UPG004"));
        assert!(!result.compatible);
    }

    #[test]
    fn test_upgrade_type_detection() {
        let validator = UpgradeValidator::new();

        // Major upgrade (different ProductCode)
        let old = ProjectInfo {
            product: ProductInfo {
                product_code: Some("{11111111-1111-1111-1111-111111111111}".to_string()),
                version: Some("1.0.0".to_string()),
                ..Default::default()
            },
            ..Default::default()
        };
        let new = ProjectInfo {
            product: ProductInfo {
                product_code: Some("{22222222-2222-2222-2222-222222222222}".to_string()),
                version: Some("2.0.0".to_string()),
                ..Default::default()
            },
            ..Default::default()
        };

        let result = validator.validate_upgrade(&old, &new);
        assert_eq!(result.upgrade_type, Some(UpgradeType::MajorUpgrade));
    }

    #[test]
    fn test_minor_upgrade_detection() {
        let validator = UpgradeValidator::new();

        let old = ProjectInfo {
            product: ProductInfo {
                product_code: Some("{11111111-1111-1111-1111-111111111111}".to_string()),
                version: Some("1.0.0".to_string()),
                upgrade_code: Some("{00000000-0000-0000-0000-000000000000}".to_string()),
                ..Default::default()
            },
            ..Default::default()
        };
        let new = ProjectInfo {
            product: ProductInfo {
                product_code: Some("{11111111-1111-1111-1111-111111111111}".to_string()),
                version: Some("1.1.0".to_string()),
                upgrade_code: Some("{00000000-0000-0000-0000-000000000000}".to_string()),
                ..Default::default()
            },
            ..Default::default()
        };

        let result = validator.validate_upgrade(&old, &new);
        assert_eq!(result.upgrade_type, Some(UpgradeType::MinorUpgrade));
    }

    #[test]
    fn test_upgrade_code_change_error() {
        let validator = UpgradeValidator::new();

        let old = ProjectInfo {
            product: ProductInfo {
                upgrade_code: Some("{11111111-1111-1111-1111-111111111111}".to_string()),
                ..Default::default()
            },
            ..Default::default()
        };
        let new = ProjectInfo {
            product: ProductInfo {
                upgrade_code: Some("{22222222-2222-2222-2222-222222222222}".to_string()),
                ..Default::default()
            },
            ..Default::default()
        };

        let result = validator.validate_upgrade(&old, &new);
        assert!(result.issues.iter().any(|i| i.id == "UPG010"));
        assert!(!result.compatible);
    }

    #[test]
    fn test_component_guid_change() {
        let validator = UpgradeValidator::new();

        let old = ProjectInfo {
            components: vec![ComponentInfo {
                id: "Comp1".to_string(),
                guid: Some("{11111111-1111-1111-1111-111111111111}".to_string()),
                directory: None,
                key_path: None,
                files: Vec::new(),
                line: 1,
            }],
            ..Default::default()
        };
        let new = ProjectInfo {
            components: vec![ComponentInfo {
                id: "Comp1".to_string(),
                guid: Some("{22222222-2222-2222-2222-222222222222}".to_string()),
                directory: None,
                key_path: None,
                files: Vec::new(),
                line: 1,
            }],
            ..Default::default()
        };

        let result = validator.validate_upgrade(&old, &new);
        assert!(result.issues.iter().any(|i| i.id == "UPG008"));
    }

    #[test]
    fn test_validate_guid() {
        assert!(validate_guid("*"));
        assert!(validate_guid("{12345678-1234-1234-1234-123456789012}"));
        assert!(validate_guid("12345678-1234-1234-1234-123456789012"));
        assert!(!validate_guid("invalid"));
        assert!(!validate_guid("{12345678-1234}"));
    }

    #[test]
    fn test_has_major_upgrade() {
        let with_major = r#"
        <Wix><Package><MajorUpgrade DowngradeErrorMessage="..." /></Package></Wix>
        "#;
        let without_major = r#"
        <Wix><Package></Package></Wix>
        "#;

        assert!(has_major_upgrade(with_major));
        assert!(!has_major_upgrade(without_major));
    }

    #[test]
    fn test_version_progression_error() {
        let validator = UpgradeValidator::new();

        let old = ProjectInfo {
            product: ProductInfo {
                version: Some("2.0.0".to_string()),
                product_code: Some("{11111111-1111-1111-1111-111111111111}".to_string()),
                ..Default::default()
            },
            ..Default::default()
        };
        let new = ProjectInfo {
            product: ProductInfo {
                version: Some("1.0.0".to_string()),
                product_code: Some("{11111111-1111-1111-1111-111111111111}".to_string()),
                ..Default::default()
            },
            ..Default::default()
        };

        let result = validator.validate_upgrade(&old, &new);
        assert!(result.issues.iter().any(|i| i.id == "UPG007"));
    }

    #[test]
    fn test_severity_display() {
        assert_eq!(format!("{}", IssueSeverity::Error), "ERROR");
        assert_eq!(format!("{}", IssueSeverity::Warning), "WARNING");
        assert_eq!(format!("{}", IssueSeverity::Info), "INFO");
    }

    #[test]
    fn test_upgrade_type_display() {
        assert_eq!(format!("{}", UpgradeType::MajorUpgrade), "Major Upgrade");
        assert_eq!(format!("{}", UpgradeType::MinorUpgrade), "Minor Upgrade");
        assert_eq!(format!("{}", UpgradeType::SmallUpdate), "Small Update");
    }
}

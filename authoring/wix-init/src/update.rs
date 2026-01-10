//! wix-update - MSI update and patch manager
//!
//! Manages MSI updates, patches (MSP), and upgrade detection.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Update type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UpdateType {
    Patch,
    MinorUpgrade,
    MajorUpgrade,
    SmallUpdate,
}

/// Update status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UpdateStatus {
    Available,
    Downloading,
    Downloaded,
    Installing,
    Installed,
    Failed,
    NotApplicable,
}

/// Update information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateInfo {
    pub product_code: String,
    pub current_version: String,
    pub new_version: String,
    pub update_type: UpdateType,
    pub patch_path: Option<PathBuf>,
    pub description: Option<String>,
    pub kb_article: Option<String>,
    pub release_date: Option<String>,
}

impl UpdateInfo {
    pub fn new(product_code: &str, current: &str, new: &str, update_type: UpdateType) -> Self {
        Self {
            product_code: product_code.to_string(),
            current_version: current.to_string(),
            new_version: new.to_string(),
            update_type,
            patch_path: None,
            description: None,
            kb_article: None,
            release_date: None,
        }
    }

    pub fn with_patch(mut self, path: PathBuf) -> Self {
        self.patch_path = Some(path);
        self
    }

    pub fn with_description(mut self, desc: &str) -> Self {
        self.description = Some(desc.to_string());
        self
    }
}

/// Installed product information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledProduct {
    pub product_code: String,
    pub product_name: String,
    pub version: String,
    pub install_date: Option<String>,
    pub install_location: Option<String>,
    pub publisher: Option<String>,
    pub upgrade_code: Option<String>,
}

impl InstalledProduct {
    pub fn new(code: &str, name: &str, version: &str) -> Self {
        Self {
            product_code: code.to_string(),
            product_name: name.to_string(),
            version: version.to_string(),
            install_date: None,
            install_location: None,
            publisher: None,
            upgrade_code: None,
        }
    }
}

/// Version comparison helper
pub struct VersionCompare;

impl VersionCompare {
    /// Parse version string to components
    pub fn parse(version: &str) -> Vec<u32> {
        version
            .split('.')
            .filter_map(|s| s.parse().ok())
            .collect()
    }

    /// Compare two versions
    pub fn compare(v1: &str, v2: &str) -> std::cmp::Ordering {
        let parts1 = Self::parse(v1);
        let parts2 = Self::parse(v2);

        for i in 0..std::cmp::max(parts1.len(), parts2.len()) {
            let p1 = parts1.get(i).unwrap_or(&0);
            let p2 = parts2.get(i).unwrap_or(&0);
            match p1.cmp(p2) {
                std::cmp::Ordering::Equal => continue,
                other => return other,
            }
        }
        std::cmp::Ordering::Equal
    }

    /// Check if v1 is newer than v2
    pub fn is_newer(v1: &str, v2: &str) -> bool {
        Self::compare(v1, v2) == std::cmp::Ordering::Greater
    }

    /// Check if v1 is older than v2
    pub fn is_older(v1: &str, v2: &str) -> bool {
        Self::compare(v1, v2) == std::cmp::Ordering::Less
    }

    /// Determine update type based on version change
    pub fn determine_update_type(old: &str, new: &str) -> UpdateType {
        let old_parts = Self::parse(old);
        let new_parts = Self::parse(new);

        let old_major = old_parts.first().unwrap_or(&0);
        let new_major = new_parts.first().unwrap_or(&0);

        if new_major > old_major {
            return UpdateType::MajorUpgrade;
        }

        let old_minor = old_parts.get(1).unwrap_or(&0);
        let new_minor = new_parts.get(1).unwrap_or(&0);

        if new_minor > old_minor {
            return UpdateType::MinorUpgrade;
        }

        UpdateType::SmallUpdate
    }
}

/// Update command builder
pub struct UpdateCommand;

impl UpdateCommand {
    /// Build command for applying patch
    pub fn apply_patch(msp_path: &PathBuf, _product_code: &str) -> Vec<String> {
        vec![
            "/p".to_string(),
            msp_path.to_string_lossy().to_string(),
            format!("REINSTALL=ALL"),
            format!("REINSTALLMODE=omus"),
        ]
    }

    /// Build command for minor upgrade
    pub fn minor_upgrade(msi_path: &PathBuf) -> Vec<String> {
        vec![
            "/i".to_string(),
            msi_path.to_string_lossy().to_string(),
            "REINSTALL=ALL".to_string(),
            "REINSTALLMODE=vomus".to_string(),
        ]
    }

    /// Build command for major upgrade
    pub fn major_upgrade(msi_path: &PathBuf) -> Vec<String> {
        vec!["/i".to_string(), msi_path.to_string_lossy().to_string()]
    }
}

/// Update result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateResult {
    pub success: bool,
    pub status: UpdateStatus,
    pub old_version: String,
    pub new_version: String,
    pub error_message: Option<String>,
}

impl UpdateResult {
    pub fn success(old: &str, new: &str) -> Self {
        Self {
            success: true,
            status: UpdateStatus::Installed,
            old_version: old.to_string(),
            new_version: new.to_string(),
            error_message: None,
        }
    }

    pub fn failure(old: &str, error: &str) -> Self {
        Self {
            success: false,
            status: UpdateStatus::Failed,
            old_version: old.to_string(),
            new_version: String::new(),
            error_message: Some(error.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_update_info_new() {
        let info = UpdateInfo::new("{CODE}", "1.0", "1.1", UpdateType::MinorUpgrade);
        assert_eq!(info.current_version, "1.0");
        assert_eq!(info.new_version, "1.1");
    }

    #[test]
    fn test_update_info_with_patch() {
        let info = UpdateInfo::new("{CODE}", "1.0", "1.1", UpdateType::Patch)
            .with_patch(PathBuf::from("update.msp"));
        assert!(info.patch_path.is_some());
    }

    #[test]
    fn test_installed_product_new() {
        let product = InstalledProduct::new("{CODE}", "MyApp", "1.0.0");
        assert_eq!(product.product_name, "MyApp");
    }

    #[test]
    fn test_version_parse() {
        let parts = VersionCompare::parse("1.2.3");
        assert_eq!(parts, vec![1, 2, 3]);
    }

    #[test]
    fn test_version_compare_equal() {
        assert_eq!(VersionCompare::compare("1.0", "1.0"), std::cmp::Ordering::Equal);
        assert_eq!(VersionCompare::compare("1.0.0", "1.0"), std::cmp::Ordering::Equal);
    }

    #[test]
    fn test_version_compare_greater() {
        assert_eq!(VersionCompare::compare("1.1", "1.0"), std::cmp::Ordering::Greater);
        assert_eq!(VersionCompare::compare("2.0", "1.9"), std::cmp::Ordering::Greater);
    }

    #[test]
    fn test_version_compare_less() {
        assert_eq!(VersionCompare::compare("1.0", "1.1"), std::cmp::Ordering::Less);
        assert_eq!(VersionCompare::compare("1.9", "2.0"), std::cmp::Ordering::Less);
    }

    #[test]
    fn test_is_newer() {
        assert!(VersionCompare::is_newer("2.0", "1.0"));
        assert!(!VersionCompare::is_newer("1.0", "2.0"));
    }

    #[test]
    fn test_is_older() {
        assert!(VersionCompare::is_older("1.0", "2.0"));
        assert!(!VersionCompare::is_older("2.0", "1.0"));
    }

    #[test]
    fn test_determine_update_type_major() {
        let update_type = VersionCompare::determine_update_type("1.0", "2.0");
        assert_eq!(update_type, UpdateType::MajorUpgrade);
    }

    #[test]
    fn test_determine_update_type_minor() {
        let update_type = VersionCompare::determine_update_type("1.0", "1.1");
        assert_eq!(update_type, UpdateType::MinorUpgrade);
    }

    #[test]
    fn test_determine_update_type_small() {
        let update_type = VersionCompare::determine_update_type("1.0.0", "1.0.1");
        assert_eq!(update_type, UpdateType::SmallUpdate);
    }

    #[test]
    fn test_update_command_patch() {
        let cmd = UpdateCommand::apply_patch(&PathBuf::from("update.msp"), "{CODE}");
        assert!(cmd.contains(&"/p".to_string()));
    }

    #[test]
    fn test_update_command_minor() {
        let cmd = UpdateCommand::minor_upgrade(&PathBuf::from("app.msi"));
        assert!(cmd.contains(&"REINSTALL=ALL".to_string()));
    }

    #[test]
    fn test_update_command_major() {
        let cmd = UpdateCommand::major_upgrade(&PathBuf::from("app.msi"));
        assert!(cmd.contains(&"/i".to_string()));
    }

    #[test]
    fn test_update_result_success() {
        let result = UpdateResult::success("1.0", "2.0");
        assert!(result.success);
        assert_eq!(result.status, UpdateStatus::Installed);
    }

    #[test]
    fn test_update_result_failure() {
        let result = UpdateResult::failure("1.0", "Error occurred");
        assert!(!result.success);
        assert_eq!(result.status, UpdateStatus::Failed);
    }
}

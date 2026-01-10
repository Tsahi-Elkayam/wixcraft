//! wix-uninstall - Clean uninstaller for MSI packages
//!
//! Provides clean uninstallation with leftover cleanup.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Uninstall mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UninstallMode {
    /// Standard uninstall
    Normal,
    /// Force uninstall (ignore errors)
    Force,
    /// Complete cleanup including registry and files
    Clean,
}

/// Uninstall options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UninstallOptions {
    pub product_code: Option<String>,
    pub msi_path: Option<PathBuf>,
    pub mode: UninstallMode,
    pub silent: bool,
    pub no_restart: bool,
    pub log_path: Option<PathBuf>,
    pub cleanup_dirs: Vec<PathBuf>,
    pub cleanup_registry: Vec<String>,
}

impl UninstallOptions {
    pub fn new() -> Self {
        Self {
            product_code: None,
            msi_path: None,
            mode: UninstallMode::Normal,
            silent: false,
            no_restart: false,
            log_path: None,
            cleanup_dirs: Vec::new(),
            cleanup_registry: Vec::new(),
        }
    }

    pub fn by_product_code(mut self, code: &str) -> Self {
        self.product_code = Some(code.to_string());
        self
    }

    pub fn by_msi(mut self, path: PathBuf) -> Self {
        self.msi_path = Some(path);
        self
    }

    pub fn silent(mut self) -> Self {
        self.silent = true;
        self
    }

    pub fn force(mut self) -> Self {
        self.mode = UninstallMode::Force;
        self
    }

    pub fn clean(mut self) -> Self {
        self.mode = UninstallMode::Clean;
        self
    }

    pub fn no_restart(mut self) -> Self {
        self.no_restart = true;
        self
    }

    pub fn with_log(mut self, path: PathBuf) -> Self {
        self.log_path = Some(path);
        self
    }

    pub fn cleanup_dir(mut self, path: PathBuf) -> Self {
        self.cleanup_dirs.push(path);
        self
    }

    pub fn cleanup_registry_key(mut self, key: &str) -> Self {
        self.cleanup_registry.push(key.to_string());
        self
    }
}

impl Default for UninstallOptions {
    fn default() -> Self {
        Self::new()
    }
}

/// Uninstall command builder
pub struct UninstallCommand;

impl UninstallCommand {
    /// Build msiexec uninstall command
    pub fn build(options: &UninstallOptions) -> Vec<String> {
        let mut args = Vec::new();
        args.push("/x".to_string());

        // Target
        if let Some(ref code) = options.product_code {
            args.push(code.clone());
        } else if let Some(ref path) = options.msi_path {
            args.push(path.to_string_lossy().to_string());
        }

        // UI level
        if options.silent {
            args.push("/qn".to_string());
        }

        // Log
        if let Some(ref log) = options.log_path {
            args.push(format!("/l*v \"{}\"", log.to_string_lossy()));
        }

        // Restart
        if options.no_restart {
            args.push("/norestart".to_string());
        }

        args
    }

    /// Build as command string
    pub fn build_string(options: &UninstallOptions) -> String {
        let args = Self::build(options);
        format!("msiexec {}", args.join(" "))
    }
}

/// Leftover item types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LeftoverType {
    File,
    Directory,
    RegistryKey,
    RegistryValue,
    Service,
    ScheduledTask,
}

/// Leftover item from incomplete uninstall
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeftoverItem {
    pub item_type: LeftoverType,
    pub path: String,
    pub description: Option<String>,
    pub safe_to_remove: bool,
}

impl LeftoverItem {
    pub fn file(path: &str) -> Self {
        Self {
            item_type: LeftoverType::File,
            path: path.to_string(),
            description: None,
            safe_to_remove: true,
        }
    }

    pub fn directory(path: &str) -> Self {
        Self {
            item_type: LeftoverType::Directory,
            path: path.to_string(),
            description: None,
            safe_to_remove: true,
        }
    }

    pub fn registry(key: &str) -> Self {
        Self {
            item_type: LeftoverType::RegistryKey,
            path: key.to_string(),
            description: None,
            safe_to_remove: true,
        }
    }

    pub fn unsafe_to_remove(mut self) -> Self {
        self.safe_to_remove = false;
        self
    }

    pub fn with_description(mut self, desc: &str) -> Self {
        self.description = Some(desc.to_string());
        self
    }
}

/// Uninstall result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UninstallResult {
    pub success: bool,
    pub exit_code: i32,
    pub message: String,
    pub leftovers: Vec<LeftoverItem>,
    pub cleaned_items: usize,
}

impl UninstallResult {
    pub fn success() -> Self {
        Self {
            success: true,
            exit_code: 0,
            message: "Uninstall completed successfully".to_string(),
            leftovers: Vec::new(),
            cleaned_items: 0,
        }
    }

    pub fn failure(code: i32, message: &str) -> Self {
        Self {
            success: false,
            exit_code: code,
            message: message.to_string(),
            leftovers: Vec::new(),
            cleaned_items: 0,
        }
    }

    pub fn with_leftovers(mut self, leftovers: Vec<LeftoverItem>) -> Self {
        self.leftovers = leftovers;
        self
    }

    pub fn with_cleaned(mut self, count: usize) -> Self {
        self.cleaned_items = count;
        self
    }

    pub fn from_exit_code(code: i32) -> Self {
        let (success, message) = match code {
            0 => (true, "Uninstall completed successfully"),
            1602 => (false, "User cancelled uninstall"),
            1603 => (false, "Fatal error during uninstall"),
            1605 => (false, "Product is not installed"),
            1618 => (false, "Another installation is in progress"),
            3010 => (true, "Restart required to complete uninstall"),
            _ => (false, "Uninstall failed with unknown error"),
        };

        Self {
            success,
            exit_code: code,
            message: message.to_string(),
            leftovers: Vec::new(),
            cleaned_items: 0,
        }
    }
}

/// Cleanup scanner
pub struct CleanupScanner;

impl CleanupScanner {
    /// Common leftover locations
    pub fn common_leftover_paths(app_name: &str) -> Vec<PathBuf> {
        vec![
            PathBuf::from(format!("C:\\Program Files\\{}", app_name)),
            PathBuf::from(format!("C:\\Program Files (x86)\\{}", app_name)),
            PathBuf::from(format!("C:\\ProgramData\\{}", app_name)),
            PathBuf::from(format!("%APPDATA%\\{}", app_name)),
            PathBuf::from(format!("%LOCALAPPDATA%\\{}", app_name)),
        ]
    }

    /// Common leftover registry keys
    pub fn common_leftover_registry(app_name: &str) -> Vec<String> {
        vec![
            format!("HKLM\\SOFTWARE\\{}", app_name),
            format!("HKCU\\SOFTWARE\\{}", app_name),
            format!("HKLM\\SOFTWARE\\WOW6432Node\\{}", app_name),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uninstall_options_new() {
        let opts = UninstallOptions::new();
        assert_eq!(opts.mode, UninstallMode::Normal);
        assert!(!opts.silent);
    }

    #[test]
    fn test_uninstall_options_by_product_code() {
        let opts = UninstallOptions::new().by_product_code("{CODE}");
        assert_eq!(opts.product_code, Some("{CODE}".to_string()));
    }

    #[test]
    fn test_uninstall_options_by_msi() {
        let opts = UninstallOptions::new().by_msi(PathBuf::from("app.msi"));
        assert!(opts.msi_path.is_some());
    }

    #[test]
    fn test_uninstall_options_silent() {
        let opts = UninstallOptions::new().silent();
        assert!(opts.silent);
    }

    #[test]
    fn test_uninstall_options_force() {
        let opts = UninstallOptions::new().force();
        assert_eq!(opts.mode, UninstallMode::Force);
    }

    #[test]
    fn test_uninstall_options_clean() {
        let opts = UninstallOptions::new().clean();
        assert_eq!(opts.mode, UninstallMode::Clean);
    }

    #[test]
    fn test_uninstall_options_cleanup_dir() {
        let opts = UninstallOptions::new().cleanup_dir(PathBuf::from("C:\\App"));
        assert_eq!(opts.cleanup_dirs.len(), 1);
    }

    #[test]
    fn test_build_command() {
        let opts = UninstallOptions::new().by_product_code("{CODE}").silent();
        let args = UninstallCommand::build(&opts);
        assert!(args.contains(&"/x".to_string()));
        assert!(args.contains(&"/qn".to_string()));
    }

    #[test]
    fn test_build_command_string() {
        let opts = UninstallOptions::new().by_product_code("{CODE}");
        let cmd = UninstallCommand::build_string(&opts);
        assert!(cmd.starts_with("msiexec"));
    }

    #[test]
    fn test_leftover_file() {
        let item = LeftoverItem::file("C:\\App\\file.txt");
        assert_eq!(item.item_type, LeftoverType::File);
        assert!(item.safe_to_remove);
    }

    #[test]
    fn test_leftover_directory() {
        let item = LeftoverItem::directory("C:\\App");
        assert_eq!(item.item_type, LeftoverType::Directory);
    }

    #[test]
    fn test_leftover_registry() {
        let item = LeftoverItem::registry("HKLM\\SOFTWARE\\App");
        assert_eq!(item.item_type, LeftoverType::RegistryKey);
    }

    #[test]
    fn test_leftover_unsafe() {
        let item = LeftoverItem::file("C:\\important.dll").unsafe_to_remove();
        assert!(!item.safe_to_remove);
    }

    #[test]
    fn test_uninstall_result_success() {
        let result = UninstallResult::success();
        assert!(result.success);
        assert_eq!(result.exit_code, 0);
    }

    #[test]
    fn test_uninstall_result_failure() {
        let result = UninstallResult::failure(1603, "Error");
        assert!(!result.success);
    }

    #[test]
    fn test_uninstall_result_from_exit_code() {
        let success = UninstallResult::from_exit_code(0);
        assert!(success.success);

        let not_installed = UninstallResult::from_exit_code(1605);
        assert!(!not_installed.success);
    }

    #[test]
    fn test_uninstall_result_with_leftovers() {
        let leftovers = vec![LeftoverItem::file("test.txt")];
        let result = UninstallResult::success().with_leftovers(leftovers);
        assert_eq!(result.leftovers.len(), 1);
    }

    #[test]
    fn test_cleanup_scanner_paths() {
        let paths = CleanupScanner::common_leftover_paths("MyApp");
        assert!(!paths.is_empty());
        assert!(paths.iter().any(|p| p.to_string_lossy().contains("MyApp")));
    }

    #[test]
    fn test_cleanup_scanner_registry() {
        let keys = CleanupScanner::common_leftover_registry("MyApp");
        assert!(!keys.is_empty());
        assert!(keys.iter().any(|k| k.contains("MyApp")));
    }
}

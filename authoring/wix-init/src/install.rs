//! wix-install - MSI installer runner with enhanced options
//!
//! Provides a convenient wrapper around msiexec with common installation scenarios.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Installation mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InstallMode {
    Install,
    Administrative,
    Repair,
    Uninstall,
    Patch,
}

/// UI Level for installation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum UILevel {
    None,
    Basic,
    Reduced,
    #[default]
    Full,
}

impl UILevel {
    pub fn to_msiexec_flag(&self) -> &'static str {
        match self {
            UILevel::None => "/qn",
            UILevel::Basic => "/qb",
            UILevel::Reduced => "/qr",
            UILevel::Full => "/qf",
        }
    }
}

/// Installation options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallOptions {
    pub msi_path: PathBuf,
    pub mode: InstallMode,
    pub ui_level: UILevel,
    pub properties: HashMap<String, String>,
    pub log_path: Option<PathBuf>,
    pub transforms: Vec<PathBuf>,
    pub target_dir: Option<String>,
    pub no_restart: bool,
    pub force_restart: bool,
}

impl InstallOptions {
    pub fn new(msi_path: PathBuf) -> Self {
        Self {
            msi_path,
            mode: InstallMode::Install,
            ui_level: UILevel::Full,
            properties: HashMap::new(),
            log_path: None,
            transforms: Vec::new(),
            target_dir: None,
            no_restart: false,
            force_restart: false,
        }
    }

    pub fn silent(mut self) -> Self {
        self.ui_level = UILevel::None;
        self
    }

    pub fn with_property(mut self, name: &str, value: &str) -> Self {
        self.properties.insert(name.to_string(), value.to_string());
        self
    }

    pub fn with_log(mut self, path: PathBuf) -> Self {
        self.log_path = Some(path);
        self
    }

    pub fn with_target_dir(mut self, dir: &str) -> Self {
        self.target_dir = Some(dir.to_string());
        self.properties.insert("TARGETDIR".to_string(), dir.to_string());
        self
    }

    pub fn with_transform(mut self, path: PathBuf) -> Self {
        self.transforms.push(path);
        self
    }

    pub fn repair(mut self) -> Self {
        self.mode = InstallMode::Repair;
        self
    }

    pub fn uninstall(mut self) -> Self {
        self.mode = InstallMode::Uninstall;
        self
    }

    pub fn no_restart(mut self) -> Self {
        self.no_restart = true;
        self.properties.insert("REBOOT".to_string(), "ReallySuppress".to_string());
        self
    }
}

/// Command builder and executor for msiexec
pub struct MsiExecCommand;

impl MsiExecCommand {
    /// Execute msiexec command (Windows only)
    #[cfg(target_os = "windows")]
    pub fn execute(options: &InstallOptions) -> InstallResult {
        use std::process::Command;

        let args = Self::build(options);

        match Command::new("msiexec").args(&args).status() {
            Ok(status) => {
                let code = status.code().unwrap_or(-1);
                let mut result = InstallResult::from_exit_code(code);
                result.log_path = options.log_path.clone();
                result
            }
            Err(e) => InstallResult::failure(-1, &format!("Failed to execute msiexec: {}", e)),
        }
    }

    /// Execute msiexec command (non-Windows stub)
    #[cfg(not(target_os = "windows"))]
    pub fn execute(_options: &InstallOptions) -> InstallResult {
        InstallResult::failure(-1, "MSI installation is only supported on Windows")
    }

    /// Build msiexec command line
    pub fn build(options: &InstallOptions) -> Vec<String> {
        let mut args = Vec::new();

        // Mode flag
        match options.mode {
            InstallMode::Install => args.push("/i".to_string()),
            InstallMode::Administrative => args.push("/a".to_string()),
            InstallMode::Repair => args.push("/f".to_string()),
            InstallMode::Uninstall => args.push("/x".to_string()),
            InstallMode::Patch => args.push("/p".to_string()),
        }

        // MSI path
        args.push(options.msi_path.to_string_lossy().to_string());

        // UI Level
        args.push(options.ui_level.to_msiexec_flag().to_string());

        // Log file
        if let Some(ref log) = options.log_path {
            args.push(format!("/l*v \"{}\"", log.to_string_lossy()));
        }

        // Transforms
        for transform in &options.transforms {
            args.push(format!("TRANSFORMS=\"{}\"", transform.to_string_lossy()));
        }

        // Properties
        for (key, value) in &options.properties {
            args.push(format!("{}=\"{}\"", key, value));
        }

        // Restart options
        if options.no_restart {
            args.push("/norestart".to_string());
        } else if options.force_restart {
            args.push("/forcerestart".to_string());
        }

        args
    }

    /// Build command as string
    pub fn build_string(options: &InstallOptions) -> String {
        let args = Self::build(options);
        format!("msiexec {}", args.join(" "))
    }
}

/// Installation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallResult {
    pub success: bool,
    pub exit_code: i32,
    pub message: String,
    pub log_path: Option<PathBuf>,
}

impl InstallResult {
    pub fn success() -> Self {
        Self {
            success: true,
            exit_code: 0,
            message: "Installation completed successfully".to_string(),
            log_path: None,
        }
    }

    pub fn failure(code: i32, message: &str) -> Self {
        Self {
            success: false,
            exit_code: code,
            message: message.to_string(),
            log_path: None,
        }
    }

    pub fn from_exit_code(code: i32) -> Self {
        let message = match code {
            0 => "Installation completed successfully",
            1602 => "User cancelled installation",
            1603 => "Fatal error during installation",
            1618 => "Another installation is in progress",
            1619 => "Installation package could not be opened",
            1620 => "Installation package could not be found",
            1622 => "Error opening installation log file",
            1625 => "Installation prohibited by system policy",
            1638 => "Another version of this product is already installed",
            3010 => "Restart required to complete installation",
            _ => "Installation failed with unknown error",
        };

        Self {
            success: code == 0 || code == 3010,
            exit_code: code,
            message: message.to_string(),
            log_path: None,
        }
    }
}

/// Common installation presets
pub struct InstallPresets;

impl InstallPresets {
    /// Silent install
    pub fn silent_install(msi_path: PathBuf) -> InstallOptions {
        InstallOptions::new(msi_path).silent().no_restart()
    }

    /// Interactive install with logging
    pub fn logged_install(msi_path: PathBuf, log_path: PathBuf) -> InstallOptions {
        InstallOptions::new(msi_path).with_log(log_path)
    }

    /// Per-user install
    pub fn per_user_install(msi_path: PathBuf) -> InstallOptions {
        InstallOptions::new(msi_path)
            .with_property("ALLUSERS", "")
            .with_property("MSIINSTALLPERUSER", "1")
    }

    /// All users install
    pub fn all_users_install(msi_path: PathBuf) -> InstallOptions {
        InstallOptions::new(msi_path).with_property("ALLUSERS", "1")
    }

    /// Custom install directory
    pub fn custom_dir_install(msi_path: PathBuf, target_dir: &str) -> InstallOptions {
        InstallOptions::new(msi_path).with_target_dir(target_dir)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_install_options_new() {
        let opts = InstallOptions::new(PathBuf::from("test.msi"));
        assert_eq!(opts.mode, InstallMode::Install);
        assert_eq!(opts.ui_level, UILevel::Full);
    }

    #[test]
    fn test_install_options_silent() {
        let opts = InstallOptions::new(PathBuf::from("test.msi")).silent();
        assert_eq!(opts.ui_level, UILevel::None);
    }

    #[test]
    fn test_install_options_with_property() {
        let opts = InstallOptions::new(PathBuf::from("test.msi"))
            .with_property("KEY", "VALUE");
        assert_eq!(opts.properties.get("KEY"), Some(&"VALUE".to_string()));
    }

    #[test]
    fn test_install_options_with_log() {
        let opts = InstallOptions::new(PathBuf::from("test.msi"))
            .with_log(PathBuf::from("install.log"));
        assert_eq!(opts.log_path, Some(PathBuf::from("install.log")));
    }

    #[test]
    fn test_install_options_with_target_dir() {
        let opts = InstallOptions::new(PathBuf::from("test.msi"))
            .with_target_dir("C:\\App");
        assert_eq!(opts.target_dir, Some("C:\\App".to_string()));
    }

    #[test]
    fn test_install_options_repair() {
        let opts = InstallOptions::new(PathBuf::from("test.msi")).repair();
        assert_eq!(opts.mode, InstallMode::Repair);
    }

    #[test]
    fn test_install_options_uninstall() {
        let opts = InstallOptions::new(PathBuf::from("test.msi")).uninstall();
        assert_eq!(opts.mode, InstallMode::Uninstall);
    }

    #[test]
    fn test_install_options_no_restart() {
        let opts = InstallOptions::new(PathBuf::from("test.msi")).no_restart();
        assert!(opts.no_restart);
        assert_eq!(opts.properties.get("REBOOT"), Some(&"ReallySuppress".to_string()));
    }

    #[test]
    fn test_ui_level_flag() {
        assert_eq!(UILevel::None.to_msiexec_flag(), "/qn");
        assert_eq!(UILevel::Basic.to_msiexec_flag(), "/qb");
        assert_eq!(UILevel::Full.to_msiexec_flag(), "/qf");
    }

    #[test]
    fn test_build_command() {
        let opts = InstallOptions::new(PathBuf::from("test.msi"));
        let args = MsiExecCommand::build(&opts);
        assert!(args.contains(&"/i".to_string()));
        assert!(args.contains(&"test.msi".to_string()));
    }

    #[test]
    fn test_build_command_string() {
        let opts = InstallOptions::new(PathBuf::from("test.msi")).silent();
        let cmd = MsiExecCommand::build_string(&opts);
        assert!(cmd.starts_with("msiexec"));
        assert!(cmd.contains("/qn"));
    }

    #[test]
    fn test_install_result_success() {
        let result = InstallResult::success();
        assert!(result.success);
        assert_eq!(result.exit_code, 0);
    }

    #[test]
    fn test_install_result_failure() {
        let result = InstallResult::failure(1603, "Fatal error");
        assert!(!result.success);
        assert_eq!(result.exit_code, 1603);
    }

    #[test]
    fn test_install_result_from_exit_code() {
        let success = InstallResult::from_exit_code(0);
        assert!(success.success);

        let cancel = InstallResult::from_exit_code(1602);
        assert!(!cancel.success);
        assert!(cancel.message.contains("cancelled"));

        let restart = InstallResult::from_exit_code(3010);
        assert!(restart.success);
    }

    #[test]
    fn test_presets_silent() {
        let opts = InstallPresets::silent_install(PathBuf::from("test.msi"));
        assert_eq!(opts.ui_level, UILevel::None);
        assert!(opts.no_restart);
    }

    #[test]
    fn test_presets_logged() {
        let opts = InstallPresets::logged_install(
            PathBuf::from("test.msi"),
            PathBuf::from("install.log"),
        );
        assert!(opts.log_path.is_some());
    }

    #[test]
    fn test_presets_per_user() {
        let opts = InstallPresets::per_user_install(PathBuf::from("test.msi"));
        assert_eq!(opts.properties.get("MSIINSTALLPERUSER"), Some(&"1".to_string()));
    }

    #[test]
    fn test_presets_all_users() {
        let opts = InstallPresets::all_users_install(PathBuf::from("test.msi"));
        assert_eq!(opts.properties.get("ALLUSERS"), Some(&"1".to_string()));
    }
}

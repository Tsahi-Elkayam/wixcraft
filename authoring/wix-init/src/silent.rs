//! wix-silent - Silent installation configuration generator
//!
//! Generates and validates silent installation configurations.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Silent installation mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SilentMode {
    /// No UI at all
    None,
    /// Basic UI (progress bar)
    Basic,
    /// Reduced UI
    Reduced,
    /// Passive (no user input, shows progress)
    Passive,
}

impl SilentMode {
    pub fn to_msiexec_flag(&self) -> &'static str {
        match self {
            SilentMode::None => "/qn",
            SilentMode::Basic => "/qb",
            SilentMode::Reduced => "/qr",
            SilentMode::Passive => "/passive",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            SilentMode::None => "Completely silent, no UI",
            SilentMode::Basic => "Basic progress bar",
            SilentMode::Reduced => "Reduced UI with minimal interaction",
            SilentMode::Passive => "Progress only, no user input",
        }
    }
}

/// Silent installation property
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SilentProperty {
    pub name: String,
    pub value: String,
    pub required: bool,
    pub description: Option<String>,
}

impl SilentProperty {
    pub fn new(name: &str, value: &str) -> Self {
        Self {
            name: name.to_string(),
            value: value.to_string(),
            required: false,
            description: None,
        }
    }

    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }

    pub fn with_description(mut self, desc: &str) -> Self {
        self.description = Some(desc.to_string());
        self
    }

    pub fn to_arg(&self) -> String {
        format!("{}=\"{}\"", self.name, self.value)
    }
}

/// Silent installation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SilentConfig {
    pub msi_path: Option<PathBuf>,
    pub product_code: Option<String>,
    pub mode: SilentMode,
    pub properties: Vec<SilentProperty>,
    pub log_path: Option<PathBuf>,
    pub target_dir: Option<String>,
    pub features: Vec<String>,
    pub suppress_restart: bool,
    pub force_restart: bool,
}

impl Default for SilentConfig {
    fn default() -> Self {
        Self {
            msi_path: None,
            product_code: None,
            mode: SilentMode::None,
            properties: Vec::new(),
            log_path: None,
            target_dir: None,
            features: Vec::new(),
            suppress_restart: false,
            force_restart: false,
        }
    }
}

impl SilentConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_msi(mut self, path: PathBuf) -> Self {
        self.msi_path = Some(path);
        self
    }

    pub fn with_product_code(mut self, code: &str) -> Self {
        self.product_code = Some(code.to_string());
        self
    }

    pub fn with_mode(mut self, mode: SilentMode) -> Self {
        self.mode = mode;
        self
    }

    pub fn with_property(mut self, name: &str, value: &str) -> Self {
        self.properties.push(SilentProperty::new(name, value));
        self
    }

    pub fn with_log(mut self, path: PathBuf) -> Self {
        self.log_path = Some(path);
        self
    }

    pub fn with_target_dir(mut self, dir: &str) -> Self {
        self.target_dir = Some(dir.to_string());
        self
    }

    pub fn with_features(mut self, features: Vec<String>) -> Self {
        self.features = features;
        self
    }

    pub fn suppress_restart(mut self) -> Self {
        self.suppress_restart = true;
        self
    }

    pub fn force_restart(mut self) -> Self {
        self.force_restart = true;
        self
    }
}

/// Silent command generator
pub struct SilentCommandGenerator;

impl SilentCommandGenerator {
    /// Build msiexec command for silent installation
    pub fn build_install_command(config: &SilentConfig) -> Vec<String> {
        let mut args = vec!["/i".to_string()];

        if let Some(ref path) = config.msi_path {
            args.push(format!("\"{}\"", path.display()));
        }

        args.push(config.mode.to_msiexec_flag().to_string());

        // Add properties
        for prop in &config.properties {
            args.push(prop.to_arg());
        }

        // Add target directory
        if let Some(ref target_dir) = config.target_dir {
            args.push(format!("INSTALLDIR=\"{}\"", target_dir));
        }

        // Add features
        if !config.features.is_empty() {
            args.push(format!("ADDLOCAL={}", config.features.join(",")));
        }

        // Add log
        if let Some(ref log_path) = config.log_path {
            args.push(format!("/l*v \"{}\"", log_path.display()));
        }

        // Restart handling
        if config.suppress_restart {
            args.push("/norestart".to_string());
        } else if config.force_restart {
            args.push("/forcerestart".to_string());
        }

        args
    }

    /// Build uninstall command
    pub fn build_uninstall_command(config: &SilentConfig) -> Vec<String> {
        let mut args = vec!["/x".to_string()];

        if let Some(ref code) = config.product_code {
            args.push(code.clone());
        } else if let Some(ref path) = config.msi_path {
            args.push(format!("\"{}\"", path.display()));
        }

        args.push(config.mode.to_msiexec_flag().to_string());

        if let Some(ref log_path) = config.log_path {
            args.push(format!("/l*v \"{}\"", log_path.display()));
        }

        if config.suppress_restart {
            args.push("/norestart".to_string());
        }

        args
    }

    /// Build as string
    pub fn build_string(config: &SilentConfig) -> String {
        format!(
            "msiexec {}",
            Self::build_install_command(config).join(" ")
        )
    }
}

/// Transform file for silent configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformConfig {
    pub name: String,
    pub properties: HashMap<String, String>,
    pub features_to_add: Vec<String>,
    pub features_to_remove: Vec<String>,
}

impl TransformConfig {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            properties: HashMap::new(),
            features_to_add: Vec::new(),
            features_to_remove: Vec::new(),
        }
    }

    pub fn with_property(mut self, name: &str, value: &str) -> Self {
        self.properties.insert(name.to_string(), value.to_string());
        self
    }

    pub fn add_feature(mut self, feature: &str) -> Self {
        self.features_to_add.push(feature.to_string());
        self
    }

    pub fn remove_feature(mut self, feature: &str) -> Self {
        self.features_to_remove.push(feature.to_string());
        self
    }
}

/// Response file generator
pub struct ResponseFileGenerator;

impl ResponseFileGenerator {
    /// Generate INI-style response file
    pub fn generate_ini(config: &SilentConfig) -> String {
        let mut output = String::new();
        output.push_str("[Install]\n");

        if let Some(ref target_dir) = config.target_dir {
            output.push_str(&format!("INSTALLDIR={}\n", target_dir));
        }

        for prop in &config.properties {
            output.push_str(&format!("{}={}\n", prop.name, prop.value));
        }

        if !config.features.is_empty() {
            output.push_str(&format!("ADDLOCAL={}\n", config.features.join(",")));
        }

        output
    }

    /// Generate batch file
    pub fn generate_batch(config: &SilentConfig) -> String {
        let mut output = String::new();
        output.push_str("@echo off\n");
        output.push_str("REM Silent installation script\n\n");
        output.push_str(&SilentCommandGenerator::build_string(config));
        output.push_str("\n\nif %ERRORLEVEL% neq 0 (\n");
        output.push_str("    echo Installation failed with error %ERRORLEVEL%\n");
        output.push_str("    exit /b %ERRORLEVEL%\n");
        output.push_str(")\n");
        output.push_str("echo Installation completed successfully\n");
        output
    }

    /// Generate PowerShell script
    pub fn generate_powershell(config: &SilentConfig) -> String {
        let mut output = String::new();
        output.push_str("# Silent installation script\n\n");
        output.push_str("$msiArgs = @(\n");

        let args = SilentCommandGenerator::build_install_command(config);
        for arg in &args {
            output.push_str(&format!("    \"{}\",\n", arg.replace('\"', "`\"")));
        }
        output.push_str(")\n\n");

        output.push_str("$process = Start-Process msiexec -ArgumentList $msiArgs -Wait -PassThru\n");
        output.push_str("if ($process.ExitCode -ne 0) {\n");
        output.push_str("    Write-Error \"Installation failed with exit code $($process.ExitCode)\"\n");
        output.push_str("    exit $process.ExitCode\n");
        output.push_str("}\n");
        output.push_str("Write-Host \"Installation completed successfully\"\n");

        output
    }
}

/// Configuration validator
pub struct ConfigValidator;

impl ConfigValidator {
    /// Validate silent configuration
    pub fn validate(config: &SilentConfig) -> Vec<String> {
        let mut errors = Vec::new();

        if config.msi_path.is_none() && config.product_code.is_none() {
            errors.push("Either MSI path or product code is required".to_string());
        }

        if config.force_restart && config.suppress_restart {
            errors.push("Cannot both force and suppress restart".to_string());
        }

        // Check for conflicting properties
        let prop_names: Vec<_> = config.properties.iter().map(|p| &p.name).collect();
        let mut seen = std::collections::HashSet::new();
        for name in prop_names {
            if !seen.insert(name) {
                errors.push(format!("Duplicate property: {}", name));
            }
        }

        errors
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_silent_mode_to_flag() {
        assert_eq!(SilentMode::None.to_msiexec_flag(), "/qn");
        assert_eq!(SilentMode::Basic.to_msiexec_flag(), "/qb");
        assert_eq!(SilentMode::Passive.to_msiexec_flag(), "/passive");
    }

    #[test]
    fn test_silent_mode_description() {
        let desc = SilentMode::None.description();
        assert!(!desc.is_empty());
    }

    #[test]
    fn test_silent_property_new() {
        let prop = SilentProperty::new("MYKEY", "myvalue");
        assert_eq!(prop.name, "MYKEY");
        assert_eq!(prop.value, "myvalue");
    }

    #[test]
    fn test_silent_property_required() {
        let prop = SilentProperty::new("KEY", "value").required();
        assert!(prop.required);
    }

    #[test]
    fn test_silent_property_to_arg() {
        let prop = SilentProperty::new("KEY", "value");
        assert_eq!(prop.to_arg(), "KEY=\"value\"");
    }

    #[test]
    fn test_silent_config_new() {
        let config = SilentConfig::new();
        assert_eq!(config.mode, SilentMode::None);
    }

    #[test]
    fn test_silent_config_with_msi() {
        let config = SilentConfig::new().with_msi(PathBuf::from("app.msi"));
        assert!(config.msi_path.is_some());
    }

    #[test]
    fn test_silent_config_with_mode() {
        let config = SilentConfig::new().with_mode(SilentMode::Basic);
        assert_eq!(config.mode, SilentMode::Basic);
    }

    #[test]
    fn test_silent_config_with_property() {
        let config = SilentConfig::new().with_property("KEY", "value");
        assert_eq!(config.properties.len(), 1);
    }

    #[test]
    fn test_silent_config_with_target_dir() {
        let config = SilentConfig::new().with_target_dir("C:\\Apps\\MyApp");
        assert_eq!(config.target_dir, Some("C:\\Apps\\MyApp".to_string()));
    }

    #[test]
    fn test_silent_config_suppress_restart() {
        let config = SilentConfig::new().suppress_restart();
        assert!(config.suppress_restart);
    }

    #[test]
    fn test_silent_command_build_install() {
        let config = SilentConfig::new()
            .with_msi(PathBuf::from("app.msi"))
            .with_mode(SilentMode::None);
        let cmd = SilentCommandGenerator::build_install_command(&config);
        assert!(cmd.contains(&"/i".to_string()));
        assert!(cmd.contains(&"/qn".to_string()));
    }

    #[test]
    fn test_silent_command_build_uninstall() {
        let config = SilentConfig::new()
            .with_product_code("{CODE}")
            .with_mode(SilentMode::None);
        let cmd = SilentCommandGenerator::build_uninstall_command(&config);
        assert!(cmd.contains(&"/x".to_string()));
        assert!(cmd.contains(&"{CODE}".to_string()));
    }

    #[test]
    fn test_silent_command_build_string() {
        let config = SilentConfig::new().with_msi(PathBuf::from("app.msi"));
        let cmd = SilentCommandGenerator::build_string(&config);
        assert!(cmd.starts_with("msiexec"));
    }

    #[test]
    fn test_transform_config_new() {
        let transform = TransformConfig::new("enterprise");
        assert_eq!(transform.name, "enterprise");
    }

    #[test]
    fn test_transform_config_with_property() {
        let transform = TransformConfig::new("test").with_property("KEY", "value");
        assert_eq!(transform.properties.get("KEY"), Some(&"value".to_string()));
    }

    #[test]
    fn test_transform_config_add_feature() {
        let transform = TransformConfig::new("test").add_feature("Feature1");
        assert_eq!(transform.features_to_add, vec!["Feature1"]);
    }

    #[test]
    fn test_response_file_generate_ini() {
        let config = SilentConfig::new()
            .with_target_dir("C:\\Apps")
            .with_property("LICENSE", "key123");
        let ini = ResponseFileGenerator::generate_ini(&config);
        assert!(ini.contains("[Install]"));
        assert!(ini.contains("INSTALLDIR=C:\\Apps"));
    }

    #[test]
    fn test_response_file_generate_batch() {
        let config = SilentConfig::new().with_msi(PathBuf::from("app.msi"));
        let batch = ResponseFileGenerator::generate_batch(&config);
        assert!(batch.contains("@echo off"));
        assert!(batch.contains("msiexec"));
    }

    #[test]
    fn test_response_file_generate_powershell() {
        let config = SilentConfig::new().with_msi(PathBuf::from("app.msi"));
        let ps = ResponseFileGenerator::generate_powershell(&config);
        assert!(ps.contains("Start-Process msiexec"));
    }

    #[test]
    fn test_config_validator_valid() {
        let config = SilentConfig::new().with_msi(PathBuf::from("app.msi"));
        let errors = ConfigValidator::validate(&config);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_config_validator_missing_source() {
        let config = SilentConfig::new();
        let errors = ConfigValidator::validate(&config);
        assert!(!errors.is_empty());
    }

    #[test]
    fn test_config_validator_conflicting_restart() {
        let config = SilentConfig::new()
            .with_msi(PathBuf::from("app.msi"))
            .suppress_restart()
            .force_restart();
        let errors = ConfigValidator::validate(&config);
        assert!(errors.iter().any(|e| e.contains("restart")));
    }
}

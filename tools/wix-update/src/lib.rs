//! wix-update - Update script generator for MSI packages
//!
//! Generates update scripts for:
//! - Minor upgrades (same ProductCode)
//! - Major upgrades (different ProductCode)
//! - Patch updates (MSP)

use serde::{Deserialize, Serialize};

/// Update type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UpdateType {
    /// Minor upgrade - same ProductCode, new version
    Minor,
    /// Major upgrade - new ProductCode
    Major,
    /// Patch - MSP file
    Patch,
    /// Reinstall - repair/refresh
    Reinstall,
}

impl std::fmt::Display for UpdateType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UpdateType::Minor => write!(f, "Minor Upgrade"),
            UpdateType::Major => write!(f, "Major Upgrade"),
            UpdateType::Patch => write!(f, "Patch"),
            UpdateType::Reinstall => write!(f, "Reinstall"),
        }
    }
}

/// Update configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateConfig {
    pub update_type: UpdateType,
    pub msi_path: String,
    pub msp_path: Option<String>,
    pub log_file: Option<String>,
    pub silent: bool,
    pub no_restart: bool,
    pub properties: Vec<(String, String)>,
}

impl Default for UpdateConfig {
    fn default() -> Self {
        Self {
            update_type: UpdateType::Major,
            msi_path: String::new(),
            msp_path: None,
            log_file: Some("update.log".to_string()),
            silent: true,
            no_restart: false,
            properties: Vec::new(),
        }
    }
}

/// Update script generator
pub struct UpdateGenerator;

impl UpdateGenerator {
    /// Generate Windows batch script for update
    pub fn generate_batch(config: &UpdateConfig) -> String {
        let mut script = String::new();

        script.push_str("@echo off\n");
        script.push_str("setlocal enabledelayedexpansion\n\n");
        script.push_str("echo ========================================\n");
        script.push_str(&format!("echo {} Update Script\n", config.update_type));
        script.push_str("echo ========================================\n\n");

        // Check for admin privileges
        script.push_str(":: Check for administrator privileges\n");
        script.push_str("net session >nul 2>&1\n");
        script.push_str("if %errorlevel% neq 0 (\n");
        script.push_str("    echo ERROR: This script requires administrator privileges.\n");
        script.push_str("    echo Please run as Administrator.\n");
        script.push_str("    pause\n");
        script.push_str("    exit /b 1\n");
        script.push_str(")\n\n");

        // Check if MSI exists
        script.push_str(&format!("if not exist \"{}\" (\n", config.msi_path));
        script.push_str(&format!(
            "    echo ERROR: Update file not found: {}\n",
            config.msi_path
        ));
        script.push_str("    exit /b 1\n");
        script.push_str(")\n\n");

        // Generate msiexec command
        let cmd = Self::generate_msiexec_command(config);
        script.push_str(&format!("echo Running: {}\n", cmd));
        script.push_str(&format!("{}\n\n", cmd));

        // Check result
        script.push_str("set RESULT=%errorlevel%\n\n");
        script.push_str("if %RESULT% equ 0 (\n");
        script.push_str("    echo Update completed successfully.\n");
        script.push_str(") else if %RESULT% equ 3010 (\n");
        script.push_str("    echo Update completed. Restart required.\n");
        script.push_str(") else (\n");
        script.push_str("    echo Update failed with error code: %RESULT%\n");
        script.push_str(")\n\n");

        if let Some(ref log) = config.log_file {
            script.push_str(&format!("echo Log file: {}\n", log));
        }

        script.push_str("\nexit /b %RESULT%\n");
        script
    }

    /// Generate PowerShell script for update
    pub fn generate_powershell(config: &UpdateConfig) -> String {
        let mut script = String::new();

        script.push_str("# Update Script\n");
        script.push_str("#Requires -RunAsAdministrator\n\n");

        script.push_str(&format!(
            "$UpdateType = \"{}\"\n",
            config.update_type
        ));
        script.push_str(&format!("$MsiPath = \"{}\"\n", config.msi_path));
        if let Some(ref log) = config.log_file {
            script.push_str(&format!("$LogFile = \"{}\"\n", log));
        }
        script.push_str("\n");

        // Check file exists
        script.push_str("if (-not (Test-Path $MsiPath)) {\n");
        script.push_str("    Write-Error \"Update file not found: $MsiPath\"\n");
        script.push_str("    exit 1\n");
        script.push_str("}\n\n");

        script.push_str("Write-Host \"Starting $UpdateType...\" -ForegroundColor Cyan\n\n");

        // Build arguments
        let args = Self::generate_msiexec_args(config);
        script.push_str(&format!("$Arguments = @({})\n\n", args));

        script.push_str("$Process = Start-Process -FilePath \"msiexec.exe\" -ArgumentList $Arguments -Wait -PassThru\n\n");

        script.push_str("switch ($Process.ExitCode) {\n");
        script.push_str("    0 { Write-Host \"Update completed successfully.\" -ForegroundColor Green }\n");
        script.push_str("    3010 { Write-Host \"Update completed. Restart required.\" -ForegroundColor Yellow }\n");
        script.push_str("    default { Write-Error \"Update failed with code: $($Process.ExitCode)\" }\n");
        script.push_str("}\n\n");

        script.push_str("exit $Process.ExitCode\n");
        script
    }

    /// Generate msiexec command line
    pub fn generate_msiexec_command(config: &UpdateConfig) -> String {
        let mut cmd = String::from("msiexec");

        match config.update_type {
            UpdateType::Minor => {
                cmd.push_str(&format!(" /i \"{}\"", config.msi_path));
                cmd.push_str(" REINSTALLMODE=vomus REINSTALL=ALL");
            }
            UpdateType::Major => {
                cmd.push_str(&format!(" /i \"{}\"", config.msi_path));
            }
            UpdateType::Patch => {
                cmd.push_str(&format!(" /p \"{}\"", config.msp_path.as_deref().unwrap_or(&config.msi_path)));
            }
            UpdateType::Reinstall => {
                cmd.push_str(&format!(" /f \"{}\"", config.msi_path));
            }
        }

        // UI level
        if config.silent {
            cmd.push_str(" /qn");
        } else {
            cmd.push_str(" /qb");
        }

        // Logging
        if let Some(ref log) = config.log_file {
            cmd.push_str(&format!(" /l*v \"{}\"", log));
        }

        // Reboot
        if config.no_restart {
            cmd.push_str(" /norestart");
        }

        // Custom properties
        for (name, value) in &config.properties {
            if value.contains(' ') {
                cmd.push_str(&format!(" {}=\"{}\"", name, value));
            } else {
                cmd.push_str(&format!(" {}={}", name, value));
            }
        }

        cmd
    }

    fn generate_msiexec_args(config: &UpdateConfig) -> String {
        let mut args = Vec::new();

        match config.update_type {
            UpdateType::Minor => {
                args.push(format!("\"/i\""));
                args.push(format!("\"{}\"", config.msi_path));
                args.push("\"REINSTALLMODE=vomus\"".to_string());
                args.push("\"REINSTALL=ALL\"".to_string());
            }
            UpdateType::Major => {
                args.push("\"/i\"".to_string());
                args.push(format!("\"{}\"", config.msi_path));
            }
            UpdateType::Patch => {
                args.push("\"/p\"".to_string());
                args.push(format!(
                    "\"{}\"",
                    config.msp_path.as_deref().unwrap_or(&config.msi_path)
                ));
            }
            UpdateType::Reinstall => {
                args.push("\"/f\"".to_string());
                args.push(format!("\"{}\"", config.msi_path));
            }
        }

        if config.silent {
            args.push("\"/qn\"".to_string());
        }

        if let Some(ref log) = config.log_file {
            args.push(format!("\"/l*v\""));
            args.push(format!("\"{}\"", log));
        }

        if config.no_restart {
            args.push("\"/norestart\"".to_string());
        }

        args.join(", ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_major_upgrade_command() {
        let config = UpdateConfig {
            update_type: UpdateType::Major,
            msi_path: "update.msi".to_string(),
            log_file: Some("update.log".to_string()),
            silent: true,
            ..Default::default()
        };

        let cmd = UpdateGenerator::generate_msiexec_command(&config);
        assert!(cmd.contains("/i"));
        assert!(cmd.contains("update.msi"));
        assert!(cmd.contains("/qn"));
        assert!(cmd.contains("/l*v"));
    }

    #[test]
    fn test_generate_minor_upgrade_command() {
        let config = UpdateConfig {
            update_type: UpdateType::Minor,
            msi_path: "update.msi".to_string(),
            ..Default::default()
        };

        let cmd = UpdateGenerator::generate_msiexec_command(&config);
        assert!(cmd.contains("REINSTALLMODE=vomus"));
        assert!(cmd.contains("REINSTALL=ALL"));
    }

    #[test]
    fn test_generate_patch_command() {
        let config = UpdateConfig {
            update_type: UpdateType::Patch,
            msi_path: "base.msi".to_string(),
            msp_path: Some("patch.msp".to_string()),
            ..Default::default()
        };

        let cmd = UpdateGenerator::generate_msiexec_command(&config);
        assert!(cmd.contains("/p"));
        assert!(cmd.contains("patch.msp"));
    }

    #[test]
    fn test_generate_batch_script() {
        let config = UpdateConfig {
            msi_path: "update.msi".to_string(),
            ..Default::default()
        };

        let script = UpdateGenerator::generate_batch(&config);
        assert!(script.contains("@echo off"));
        assert!(script.contains("msiexec"));
        assert!(script.contains("administrator"));
    }

    #[test]
    fn test_generate_powershell_script() {
        let config = UpdateConfig {
            msi_path: "update.msi".to_string(),
            ..Default::default()
        };

        let script = UpdateGenerator::generate_powershell(&config);
        assert!(script.contains("#Requires -RunAsAdministrator"));
        assert!(script.contains("Start-Process"));
    }

    #[test]
    fn test_update_type_display() {
        assert_eq!(format!("{}", UpdateType::Major), "Major Upgrade");
        assert_eq!(format!("{}", UpdateType::Minor), "Minor Upgrade");
    }
}

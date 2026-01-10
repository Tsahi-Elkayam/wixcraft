//! wix-uninstall - Uninstall script generator for MSI packages
//!
//! Generates uninstall scripts and commands for MSI packages

use serde::{Deserialize, Serialize};

/// Uninstall method
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UninstallMethod {
    /// Uninstall by MSI file path
    MsiFile,
    /// Uninstall by ProductCode GUID
    ProductCode,
    /// Uninstall by product name (searches registry)
    ProductName,
    /// Uninstall by UpgradeCode GUID
    UpgradeCode,
}

impl std::fmt::Display for UninstallMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UninstallMethod::MsiFile => write!(f, "MSI File"),
            UninstallMethod::ProductCode => write!(f, "Product Code"),
            UninstallMethod::ProductName => write!(f, "Product Name"),
            UninstallMethod::UpgradeCode => write!(f, "Upgrade Code"),
        }
    }
}

/// Uninstall configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UninstallConfig {
    pub method: UninstallMethod,
    pub identifier: String,
    pub log_file: Option<String>,
    pub silent: bool,
    pub force: bool,
    pub cleanup: bool,
}

impl Default for UninstallConfig {
    fn default() -> Self {
        Self {
            method: UninstallMethod::ProductCode,
            identifier: String::new(),
            log_file: Some("uninstall.log".to_string()),
            silent: true,
            force: false,
            cleanup: false,
        }
    }
}

/// Uninstall script generator
pub struct UninstallGenerator;

impl UninstallGenerator {
    /// Generate msiexec command for uninstall
    pub fn generate_msiexec_command(config: &UninstallConfig) -> String {
        let mut cmd = String::from("msiexec");

        match config.method {
            UninstallMethod::MsiFile => {
                cmd.push_str(&format!(" /x \"{}\"", config.identifier));
            }
            UninstallMethod::ProductCode => {
                let code = if config.identifier.starts_with('{') {
                    config.identifier.clone()
                } else {
                    format!("{{{}}}", config.identifier)
                };
                cmd.push_str(&format!(" /x {}", code));
            }
            UninstallMethod::ProductName | UninstallMethod::UpgradeCode => {
                // These require lookup first, use placeholder
                cmd.push_str(" /x {PRODUCT_CODE}");
            }
        }

        if config.silent {
            cmd.push_str(" /qn");
        } else {
            cmd.push_str(" /qb");
        }

        if let Some(ref log) = config.log_file {
            cmd.push_str(&format!(" /l*v \"{}\"", log));
        }

        cmd
    }

    /// Generate Windows batch script for uninstall
    pub fn generate_batch(config: &UninstallConfig) -> String {
        let mut script = String::new();

        script.push_str("@echo off\n");
        script.push_str("setlocal enabledelayedexpansion\n\n");
        script.push_str("echo ========================================\n");
        script.push_str("echo Uninstall Script\n");
        script.push_str("echo ========================================\n\n");

        // Admin check
        script.push_str(":: Check for administrator privileges\n");
        script.push_str("net session >nul 2>&1\n");
        script.push_str("if %errorlevel% neq 0 (\n");
        script.push_str("    echo ERROR: This script requires administrator privileges.\n");
        script.push_str("    pause\n");
        script.push_str("    exit /b 1\n");
        script.push_str(")\n\n");

        match config.method {
            UninstallMethod::ProductName => {
                script.push_str(&format!(
                    "set PRODUCT_NAME={}\n\n",
                    config.identifier
                ));
                script.push_str(":: Find product code by name\n");
                script.push_str("for /f \"tokens=*\" %%a in ('wmic product where \"name like '%%!PRODUCT_NAME!%%'\" get IdentifyingNumber /value ^| find \"=\"') do (\n");
                script.push_str("    set %%a\n");
                script.push_str(")\n\n");
                script.push_str("if not defined IdentifyingNumber (\n");
                script.push_str("    echo Product not found: %PRODUCT_NAME%\n");
                script.push_str("    exit /b 1\n");
                script.push_str(")\n\n");
                script.push_str("echo Found product code: %IdentifyingNumber%\n");
                script.push_str("set PRODUCT_CODE=%IdentifyingNumber%\n\n");
            }
            UninstallMethod::ProductCode => {
                let code = if config.identifier.starts_with('{') {
                    config.identifier.clone()
                } else {
                    format!("{{{}}}", config.identifier)
                };
                script.push_str(&format!("set PRODUCT_CODE={}\n\n", code));
            }
            UninstallMethod::MsiFile => {
                script.push_str(&format!("set MSI_FILE={}\n\n", config.identifier));
            }
            _ => {}
        }

        // Uninstall command
        let ui = if config.silent { "/qn" } else { "/qb" };
        let log_arg = config
            .log_file
            .as_ref()
            .map(|l| format!(" /l*v \"{}\"", l))
            .unwrap_or_default();

        match config.method {
            UninstallMethod::MsiFile => {
                script.push_str(&format!(
                    "msiexec /x \"%MSI_FILE%\" {} {}\n\n",
                    ui, log_arg
                ));
            }
            _ => {
                script.push_str(&format!(
                    "msiexec /x %PRODUCT_CODE% {} {}\n\n",
                    ui, log_arg
                ));
            }
        }

        // Result handling
        script.push_str("set RESULT=%errorlevel%\n\n");
        script.push_str("if %RESULT% equ 0 (\n");
        script.push_str("    echo Uninstall completed successfully.\n");
        script.push_str(") else if %RESULT% equ 3010 (\n");
        script.push_str("    echo Uninstall completed. Restart required.\n");
        script.push_str(") else if %RESULT% equ 1605 (\n");
        script.push_str("    echo Product is not installed.\n");
        script.push_str(") else (\n");
        script.push_str("    echo Uninstall failed with error code: %RESULT%\n");
        script.push_str(")\n\n");

        // Optional cleanup
        if config.cleanup {
            script.push_str(":: Cleanup leftover files\n");
            script.push_str("echo Cleaning up...\n");
            script.push_str(":: Add cleanup commands here\n\n");
        }

        script.push_str("exit /b %RESULT%\n");
        script
    }

    /// Generate PowerShell script for uninstall
    pub fn generate_powershell(config: &UninstallConfig) -> String {
        let mut script = String::new();

        script.push_str("# Uninstall Script\n");
        script.push_str("#Requires -RunAsAdministrator\n\n");

        script.push_str("param(\n");
        script.push_str("    [switch]$Force,\n");
        script.push_str("    [switch]$Cleanup\n");
        script.push_str(")\n\n");

        match config.method {
            UninstallMethod::ProductName => {
                script.push_str(&format!(
                    "$ProductName = \"{}\"\n\n",
                    config.identifier
                ));
                script.push_str("# Find product by name\n");
                script.push_str("$Product = Get-CimInstance -ClassName Win32_Product | Where-Object { $_.Name -like \"*$ProductName*\" }\n\n");
                script.push_str("if (-not $Product) {\n");
                script.push_str("    Write-Error \"Product not found: $ProductName\"\n");
                script.push_str("    exit 1\n");
                script.push_str("}\n\n");
                script.push_str("$ProductCode = $Product.IdentifyingNumber\n");
                script.push_str("Write-Host \"Found: $($Product.Name) [$ProductCode]\"\n\n");
            }
            UninstallMethod::ProductCode => {
                let code = if config.identifier.starts_with('{') {
                    config.identifier.clone()
                } else {
                    format!("{{{}}}", config.identifier)
                };
                script.push_str(&format!("$ProductCode = \"{}\"\n\n", code));
            }
            UninstallMethod::MsiFile => {
                script.push_str(&format!("$MsiFile = \"{}\"\n\n", config.identifier));
            }
            _ => {}
        }

        // Build msiexec arguments
        let ui = if config.silent { "/qn" } else { "/qb" };
        script.push_str("Write-Host \"Starting uninstall...\" -ForegroundColor Cyan\n\n");

        match config.method {
            UninstallMethod::MsiFile => {
                script.push_str(&format!(
                    "$Arguments = @(\"/x\", \"`\"$MsiFile`\"\", \"{}\"",
                    ui
                ));
            }
            _ => {
                script.push_str(&format!(
                    "$Arguments = @(\"/x\", \"$ProductCode\", \"{}\"",
                    ui
                ));
            }
        }

        if let Some(ref log) = config.log_file {
            script.push_str(&format!(", \"/l*v\", \"{}\"", log));
        }
        script.push_str(")\n\n");

        script.push_str("$Process = Start-Process -FilePath \"msiexec.exe\" -ArgumentList $Arguments -Wait -PassThru\n\n");

        script.push_str("switch ($Process.ExitCode) {\n");
        script.push_str("    0 { Write-Host \"Uninstall completed successfully.\" -ForegroundColor Green }\n");
        script.push_str("    3010 { Write-Host \"Uninstall completed. Restart required.\" -ForegroundColor Yellow }\n");
        script.push_str("    1605 { Write-Host \"Product is not installed.\" -ForegroundColor Yellow }\n");
        script.push_str("    default { Write-Error \"Uninstall failed with code: $($Process.ExitCode)\" }\n");
        script.push_str("}\n\n");

        if config.cleanup {
            script.push_str("if ($Cleanup) {\n");
            script.push_str("    Write-Host \"Cleaning up leftover files...\"\n");
            script.push_str("    # Add cleanup logic here\n");
            script.push_str("}\n\n");
        }

        script.push_str("exit $Process.ExitCode\n");
        script
    }

    /// Generate PowerShell script to list installed products
    pub fn generate_list_products_script() -> String {
        r#"# List installed MSI products
#Requires -RunAsAdministrator

param(
    [string]$Filter = "*"
)

Write-Host "Searching for installed products..." -ForegroundColor Cyan
Write-Host ""

$Products = Get-CimInstance -ClassName Win32_Product |
    Where-Object { $_.Name -like $Filter } |
    Select-Object Name, Version, IdentifyingNumber, Vendor |
    Sort-Object Name

if ($Products) {
    $Products | Format-Table -AutoSize
    Write-Host ""
    Write-Host "Found $($Products.Count) product(s)" -ForegroundColor Green
} else {
    Write-Host "No products found matching: $Filter" -ForegroundColor Yellow
}
"#.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_command_by_code() {
        let config = UninstallConfig {
            method: UninstallMethod::ProductCode,
            identifier: "12345678-1234-1234-1234-123456789012".to_string(),
            ..Default::default()
        };

        let cmd = UninstallGenerator::generate_msiexec_command(&config);
        assert!(cmd.contains("/x"));
        assert!(cmd.contains("{12345678-1234-1234-1234-123456789012}"));
    }

    #[test]
    fn test_generate_command_by_file() {
        let config = UninstallConfig {
            method: UninstallMethod::MsiFile,
            identifier: "product.msi".to_string(),
            ..Default::default()
        };

        let cmd = UninstallGenerator::generate_msiexec_command(&config);
        assert!(cmd.contains("/x"));
        assert!(cmd.contains("product.msi"));
    }

    #[test]
    fn test_generate_batch_script() {
        let config = UninstallConfig {
            method: UninstallMethod::ProductCode,
            identifier: "{GUID}".to_string(),
            ..Default::default()
        };

        let script = UninstallGenerator::generate_batch(&config);
        assert!(script.contains("@echo off"));
        assert!(script.contains("msiexec"));
    }

    #[test]
    fn test_generate_powershell_script() {
        let config = UninstallConfig {
            method: UninstallMethod::ProductName,
            identifier: "MyApp".to_string(),
            ..Default::default()
        };

        let script = UninstallGenerator::generate_powershell(&config);
        assert!(script.contains("#Requires -RunAsAdministrator"));
        assert!(script.contains("Win32_Product"));
    }

    #[test]
    fn test_method_display() {
        assert_eq!(format!("{}", UninstallMethod::ProductCode), "Product Code");
        assert_eq!(format!("{}", UninstallMethod::MsiFile), "MSI File");
    }
}

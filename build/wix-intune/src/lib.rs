//! wix-intune - Microsoft Intune deployment package generator
//!
//! Generates .intunewin packages and deployment scripts for WiX MSI installers.
//! Provides:
//! - IntuneWin package metadata generation
//! - Install/uninstall command generation
//! - Detection rule generation
//! - Requirement rule generation

use serde::{Deserialize, Serialize};

/// Intune app type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IntuneAppType {
    /// Line-of-business MSI app
    MsiLob,
    /// Win32 app (wrapped in .intunewin)
    Win32,
}

/// Intune install behavior
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InstallBehavior {
    /// Install for system (all users)
    System,
    /// Install for current user only
    User,
}

/// Detection rule type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DetectionRuleType {
    /// MSI product code detection
    Msi,
    /// File existence detection
    File,
    /// Registry key detection
    Registry,
    /// Custom script detection
    Script,
}

/// Detection rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionRule {
    pub rule_type: DetectionRuleType,
    pub product_code: Option<String>,
    pub product_version: Option<String>,
    pub file_path: Option<String>,
    pub registry_key: Option<String>,
    pub registry_value: Option<String>,
    pub script_content: Option<String>,
}

/// Requirement rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequirementRule {
    pub os_architecture: String,
    pub minimum_os_version: String,
    pub disk_space_mb: Option<u64>,
    pub physical_memory_mb: Option<u64>,
    pub processor_speed_mhz: Option<u32>,
}

/// Intune deployment configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntuneConfig {
    pub display_name: String,
    pub description: Option<String>,
    pub publisher: String,
    pub version: String,
    pub app_type: IntuneAppType,
    pub install_behavior: InstallBehavior,
    pub msi_path: String,
    pub product_code: Option<String>,
    pub upgrade_code: Option<String>,
    pub install_command: String,
    pub uninstall_command: String,
    pub detection_rules: Vec<DetectionRule>,
    pub requirement_rules: Vec<RequirementRule>,
    pub return_codes: Vec<ReturnCode>,
}

/// Return code configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReturnCode {
    pub code: i32,
    pub result_type: String,
}

impl Default for IntuneConfig {
    fn default() -> Self {
        Self {
            display_name: String::new(),
            description: None,
            publisher: String::new(),
            version: "1.0.0".to_string(),
            app_type: IntuneAppType::MsiLob,
            install_behavior: InstallBehavior::System,
            msi_path: String::new(),
            product_code: None,
            upgrade_code: None,
            install_command: String::new(),
            uninstall_command: String::new(),
            detection_rules: Vec::new(),
            requirement_rules: vec![RequirementRule {
                os_architecture: "x64".to_string(),
                minimum_os_version: "10.0.17763.0".to_string(),
                disk_space_mb: None,
                physical_memory_mb: None,
                processor_speed_mhz: None,
            }],
            return_codes: vec![
                ReturnCode { code: 0, result_type: "success".to_string() },
                ReturnCode { code: 1707, result_type: "success".to_string() },
                ReturnCode { code: 3010, result_type: "softReboot".to_string() },
                ReturnCode { code: 1641, result_type: "hardReboot".to_string() },
                ReturnCode { code: 1618, result_type: "retry".to_string() },
            ],
        }
    }
}

/// Intune package generator
pub struct IntuneGenerator;

impl IntuneGenerator {
    /// Generate Intune configuration from WiX source
    pub fn from_wix(content: &str, msi_filename: &str) -> IntuneConfig {
        let mut config = IntuneConfig::default();
        config.msi_path = msi_filename.to_string();

        if let Ok(doc) = roxmltree::Document::parse(content) {
            for node in doc.descendants() {
                match node.tag_name().name() {
                    "Package" | "Product" => {
                        if let Some(name) = node.attribute("Name") {
                            config.display_name = name.to_string();
                        }
                        if let Some(version) = node.attribute("Version") {
                            config.version = version.to_string();
                        }
                        if let Some(manufacturer) = node.attribute("Manufacturer") {
                            config.publisher = manufacturer.to_string();
                        }
                        if let Some(id) = node.attribute("Id") {
                            if id != "*" {
                                config.product_code = Some(id.to_string());
                            }
                        }
                        if let Some(upgrade_code) = node.attribute("UpgradeCode") {
                            config.upgrade_code = Some(upgrade_code.to_string());
                        }
                        if let Some(desc) = node.attribute("Description") {
                            config.description = Some(desc.to_string());
                        }
                    }
                    _ => {}
                }
            }
        }

        // Generate install/uninstall commands
        config.install_command = Self::generate_install_command(&config);
        config.uninstall_command = Self::generate_uninstall_command(&config);

        // Generate detection rules
        if let Some(ref product_code) = config.product_code {
            config.detection_rules.push(DetectionRule {
                rule_type: DetectionRuleType::Msi,
                product_code: Some(product_code.clone()),
                product_version: Some(config.version.clone()),
                file_path: None,
                registry_key: None,
                registry_value: None,
                script_content: None,
            });
        }

        config
    }

    /// Generate msiexec install command
    pub fn generate_install_command(config: &IntuneConfig) -> String {
        format!(
            "msiexec /i \"{}\" /qn /norestart /l*v \"C:\\Windows\\Temp\\{}_install.log\"",
            config.msi_path,
            config.display_name.replace(' ', "_")
        )
    }

    /// Generate msiexec uninstall command
    pub fn generate_uninstall_command(config: &IntuneConfig) -> String {
        if let Some(ref product_code) = config.product_code {
            format!(
                "msiexec /x {} /qn /norestart",
                product_code
            )
        } else {
            format!(
                "msiexec /x \"{}\" /qn /norestart",
                config.msi_path
            )
        }
    }

    /// Generate PowerShell wrapper script for Win32 app deployment
    pub fn generate_install_script(config: &IntuneConfig) -> String {
        let mut script = String::new();

        script.push_str("# Intune Win32 App Install Script\n");
        script.push_str(&format!("# Application: {}\n", config.display_name));
        script.push_str(&format!("# Version: {}\n", config.version));
        script.push_str(&format!("# Publisher: {}\n\n", config.publisher));

        script.push_str("$ErrorActionPreference = \"Stop\"\n");
        script.push_str("$LogPath = \"$env:TEMP\\IntuneInstall.log\"\n\n");

        script.push_str("function Write-Log {\n");
        script.push_str("    param([string]$Message)\n");
        script.push_str("    $timestamp = Get-Date -Format \"yyyy-MM-dd HH:mm:ss\"\n");
        script.push_str("    \"$timestamp - $Message\" | Out-File -FilePath $LogPath -Append\n");
        script.push_str("}\n\n");

        script.push_str("try {\n");
        script.push_str(&format!("    Write-Log \"Installing {}\"\n", config.display_name));
        script.push_str("    $msiPath = Join-Path $PSScriptRoot \"");
        script.push_str(&config.msi_path);
        script.push_str("\"\n");
        script.push_str("    \n");
        script.push_str("    $arguments = @(\n");
        script.push_str("        \"/i\"\n");
        script.push_str("        \"`\"$msiPath`\"\"\n");
        script.push_str("        \"/qn\"\n");
        script.push_str("        \"/norestart\"\n");
        script.push_str("        \"/l*v\"\n");
        script.push_str("        \"`\"$env:TEMP\\MSIInstall.log`\"\"\n");
        script.push_str("    )\n");
        script.push_str("    \n");
        script.push_str("    $process = Start-Process -FilePath \"msiexec.exe\" -ArgumentList $arguments -Wait -PassThru\n");
        script.push_str("    Write-Log \"Exit code: $($process.ExitCode)\"\n");
        script.push_str("    \n");
        script.push_str("    switch ($process.ExitCode) {\n");
        script.push_str("        0       { Write-Log \"Installation successful\"; exit 0 }\n");
        script.push_str("        1707    { Write-Log \"Installation successful\"; exit 0 }\n");
        script.push_str("        3010    { Write-Log \"Reboot required\"; exit 3010 }\n");
        script.push_str("        1641    { Write-Log \"Reboot initiated\"; exit 1641 }\n");
        script.push_str("        default { Write-Log \"Installation failed\"; exit $process.ExitCode }\n");
        script.push_str("    }\n");
        script.push_str("}\n");
        script.push_str("catch {\n");
        script.push_str("    Write-Log \"Error: $_\"\n");
        script.push_str("    exit 1\n");
        script.push_str("}\n");

        script
    }

    /// Generate PowerShell uninstall script
    pub fn generate_uninstall_script(config: &IntuneConfig) -> String {
        let mut script = String::new();

        script.push_str("# Intune Win32 App Uninstall Script\n");
        script.push_str(&format!("# Application: {}\n\n", config.display_name));

        script.push_str("$ErrorActionPreference = \"Stop\"\n\n");

        script.push_str("try {\n");

        if let Some(ref product_code) = config.product_code {
            script.push_str(&format!(
                "    $arguments = \"/x {} /qn /norestart\"\n",
                product_code
            ));
        } else {
            script.push_str("    $msiPath = Join-Path $PSScriptRoot \"");
            script.push_str(&config.msi_path);
            script.push_str("\"\n");
            script.push_str("    $arguments = \"/x `\"$msiPath`\" /qn /norestart\"\n");
        }

        script.push_str("    $process = Start-Process -FilePath \"msiexec.exe\" -ArgumentList $arguments -Wait -PassThru\n");
        script.push_str("    exit $process.ExitCode\n");
        script.push_str("}\n");
        script.push_str("catch {\n");
        script.push_str("    exit 1\n");
        script.push_str("}\n");

        script
    }

    /// Generate PowerShell detection script
    pub fn generate_detection_script(config: &IntuneConfig) -> String {
        let mut script = String::new();

        script.push_str("# Intune Detection Script\n");
        script.push_str(&format!("# Application: {}\n\n", config.display_name));

        if let Some(ref product_code) = config.product_code {
            script.push_str(&format!(
                "$productCode = \"{}\"\n",
                product_code.trim_matches(|c| c == '{' || c == '}')
            ));
            script.push_str("$uninstallPaths = @(\n");
            script.push_str("    \"HKLM:\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall\"\n");
            script.push_str("    \"HKLM:\\SOFTWARE\\WOW6432Node\\Microsoft\\Windows\\CurrentVersion\\Uninstall\"\n");
            script.push_str(")\n\n");

            script.push_str("foreach ($path in $uninstallPaths) {\n");
            script.push_str("    $key = Get-ItemProperty -Path \"$path\\{$productCode}\" -ErrorAction SilentlyContinue\n");
            script.push_str("    if ($key) {\n");
            script.push_str("        Write-Host \"Installed\"\n");
            script.push_str("        exit 0\n");
            script.push_str("    }\n");
            script.push_str("}\n\n");

            script.push_str("exit 1\n");
        } else {
            script.push_str("# Product code not available, using display name detection\n");
            script.push_str(&format!(
                "$displayName = \"{}\"\n",
                config.display_name
            ));
            script.push_str("$app = Get-WmiObject -Class Win32_Product | Where-Object { $_.Name -eq $displayName }\n");
            script.push_str("if ($app) {\n");
            script.push_str("    Write-Host \"Installed\"\n");
            script.push_str("    exit 0\n");
            script.push_str("}\n");
            script.push_str("exit 1\n");
        }

        script
    }

    /// Generate IntuneWin content prep instructions
    pub fn generate_prep_instructions(config: &IntuneConfig) -> String {
        let mut instructions = String::new();

        instructions.push_str("Microsoft Intune Win32 Content Prep Instructions\n");
        instructions.push_str(&"=".repeat(50));
        instructions.push_str("\n\n");

        instructions.push_str(&format!("Application: {}\n", config.display_name));
        instructions.push_str(&format!("Version: {}\n", config.version));
        instructions.push_str(&format!("Publisher: {}\n\n", config.publisher));

        instructions.push_str("Step 1: Download the Win32 Content Prep Tool\n");
        instructions.push_str("-".repeat(50).as_str());
        instructions.push_str("\n");
        instructions.push_str("Download from: https://github.com/microsoft/Microsoft-Win32-Content-Prep-Tool\n\n");

        instructions.push_str("Step 2: Prepare Source Folder\n");
        instructions.push_str("-".repeat(50).as_str());
        instructions.push_str("\n");
        instructions.push_str("Create a folder with the following files:\n");
        instructions.push_str(&format!("  - {}\n", config.msi_path));
        instructions.push_str("  - Install.ps1 (optional wrapper script)\n");
        instructions.push_str("  - Uninstall.ps1 (optional wrapper script)\n\n");

        instructions.push_str("Step 3: Run Content Prep Tool\n");
        instructions.push_str("-".repeat(50).as_str());
        instructions.push_str("\n");
        instructions.push_str("IntuneWinAppUtil.exe \\\n");
        instructions.push_str("  -c \"<source_folder>\" \\\n");
        instructions.push_str(&format!("  -s \"{}\" \\\n", config.msi_path));
        instructions.push_str("  -o \"<output_folder>\"\n\n");

        instructions.push_str("Step 4: Upload to Intune\n");
        instructions.push_str("-".repeat(50).as_str());
        instructions.push_str("\n");
        instructions.push_str("1. Go to Microsoft Intune admin center\n");
        instructions.push_str("2. Apps > All Apps > Add\n");
        instructions.push_str("3. Select 'Windows app (Win32)'\n");
        instructions.push_str("4. Upload the generated .intunewin file\n\n");

        instructions.push_str("Step 5: Configure App Settings\n");
        instructions.push_str("-".repeat(50).as_str());
        instructions.push_str("\n");
        instructions.push_str(&format!("Name: {}\n", config.display_name));
        instructions.push_str(&format!("Publisher: {}\n", config.publisher));
        instructions.push_str(&format!("Version: {}\n\n", config.version));

        instructions.push_str("Install command:\n");
        instructions.push_str(&format!("  {}\n\n", config.install_command));

        instructions.push_str("Uninstall command:\n");
        instructions.push_str(&format!("  {}\n\n", config.uninstall_command));

        instructions.push_str("Detection rules:\n");
        if let Some(ref product_code) = config.product_code {
            instructions.push_str("  Type: MSI\n");
            instructions.push_str(&format!("  Product code: {}\n", product_code));
        }

        instructions
    }

    /// Generate Intune app manifest JSON
    pub fn generate_manifest(config: &IntuneConfig) -> String {
        let manifest = serde_json::json!({
            "@odata.type": "#microsoft.graph.win32LobApp",
            "displayName": config.display_name,
            "description": config.description,
            "publisher": config.publisher,
            "displayVersion": config.version,
            "installCommandLine": config.install_command,
            "uninstallCommandLine": config.uninstall_command,
            "installExperience": {
                "runAsAccount": match config.install_behavior {
                    InstallBehavior::System => "system",
                    InstallBehavior::User => "user",
                },
                "deviceRestartBehavior": "allow"
            },
            "detectionRules": config.detection_rules.iter().map(|r| {
                match r.rule_type {
                    DetectionRuleType::Msi => serde_json::json!({
                        "@odata.type": "#microsoft.graph.win32LobAppProductCodeDetection",
                        "productCode": r.product_code,
                        "productVersionOperator": "greaterThanOrEqual",
                        "productVersion": r.product_version
                    }),
                    DetectionRuleType::File => serde_json::json!({
                        "@odata.type": "#microsoft.graph.win32LobAppFileSystemDetection",
                        "path": r.file_path,
                        "detectionType": "exists"
                    }),
                    DetectionRuleType::Registry => serde_json::json!({
                        "@odata.type": "#microsoft.graph.win32LobAppRegistryDetection",
                        "keyPath": r.registry_key,
                        "valueName": r.registry_value,
                        "detectionType": "exists"
                    }),
                    DetectionRuleType::Script => serde_json::json!({
                        "@odata.type": "#microsoft.graph.win32LobAppPowerShellScriptDetection",
                        "scriptContent": r.script_content,
                        "enforceSignatureCheck": false,
                        "runAs32Bit": false
                    }),
                }
            }).collect::<Vec<_>>(),
            "requirementRules": config.requirement_rules.iter().map(|r| {
                serde_json::json!({
                    "@odata.type": "#microsoft.graph.win32LobAppRequirement",
                    "operator": "greaterThanOrEqual",
                    "detectionValue": r.minimum_os_version
                })
            }).collect::<Vec<_>>(),
            "returnCodes": config.return_codes.iter().map(|r| {
                serde_json::json!({
                    "returnCode": r.code,
                    "type": r.result_type
                })
            }).collect::<Vec<_>>()
        });

        serde_json::to_string_pretty(&manifest).unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_wix() {
        let content = r#"
        <Wix xmlns="http://wixtoolset.org/schemas/v4/wxs">
            <Package Name="TestApp" Version="1.2.3" Manufacturer="Acme Corp"
                     Id="{12345678-1234-1234-1234-123456789012}"
                     UpgradeCode="{AAAAAAAA-AAAA-AAAA-AAAA-AAAAAAAAAAAA}">
            </Package>
        </Wix>
        "#;

        let config = IntuneGenerator::from_wix(content, "TestApp.msi");
        assert_eq!(config.display_name, "TestApp");
        assert_eq!(config.version, "1.2.3");
        assert_eq!(config.publisher, "Acme Corp");
        assert!(config.product_code.is_some());
    }

    #[test]
    fn test_generate_install_command() {
        let config = IntuneConfig {
            display_name: "TestApp".to_string(),
            msi_path: "TestApp.msi".to_string(),
            ..Default::default()
        };

        let cmd = IntuneGenerator::generate_install_command(&config);
        assert!(cmd.contains("msiexec /i"));
        assert!(cmd.contains("/qn"));
        assert!(cmd.contains("TestApp.msi"));
    }

    #[test]
    fn test_generate_uninstall_command() {
        let config = IntuneConfig {
            product_code: Some("{12345678-1234-1234-1234-123456789012}".to_string()),
            msi_path: "TestApp.msi".to_string(),
            ..Default::default()
        };

        let cmd = IntuneGenerator::generate_uninstall_command(&config);
        assert!(cmd.contains("msiexec /x"));
        assert!(cmd.contains("{12345678-1234-1234-1234-123456789012}"));
    }

    #[test]
    fn test_generate_detection_script() {
        let config = IntuneConfig {
            display_name: "TestApp".to_string(),
            product_code: Some("{12345678-1234-1234-1234-123456789012}".to_string()),
            ..Default::default()
        };

        let script = IntuneGenerator::generate_detection_script(&config);
        assert!(script.contains("12345678-1234-1234-1234-123456789012"));
        assert!(script.contains("HKLM:"));
    }

    #[test]
    fn test_generate_manifest() {
        let config = IntuneConfig {
            display_name: "TestApp".to_string(),
            publisher: "Acme".to_string(),
            version: "1.0.0".to_string(),
            product_code: Some("{12345678-1234-1234-1234-123456789012}".to_string()),
            install_command: "msiexec /i test.msi /qn".to_string(),
            uninstall_command: "msiexec /x test.msi /qn".to_string(),
            detection_rules: vec![DetectionRule {
                rule_type: DetectionRuleType::Msi,
                product_code: Some("{12345678-1234-1234-1234-123456789012}".to_string()),
                product_version: Some("1.0.0".to_string()),
                file_path: None,
                registry_key: None,
                registry_value: None,
                script_content: None,
            }],
            ..Default::default()
        };

        let manifest = IntuneGenerator::generate_manifest(&config);
        assert!(manifest.contains("win32LobApp"));
        assert!(manifest.contains("TestApp"));
    }

    #[test]
    fn test_generate_prep_instructions() {
        let config = IntuneConfig {
            display_name: "TestApp".to_string(),
            publisher: "Acme".to_string(),
            version: "1.0.0".to_string(),
            msi_path: "TestApp.msi".to_string(),
            ..Default::default()
        };

        let instructions = IntuneGenerator::generate_prep_instructions(&config);
        assert!(instructions.contains("IntuneWinAppUtil"));
        assert!(instructions.contains("TestApp.msi"));
    }
}

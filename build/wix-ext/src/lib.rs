//! wix-ext - WiX extension manager with version pinning for CI/CD
//!
//! Manages WiX Toolset extensions with proper version control,
//! solving common issues with extension version mismatches in CI/CD pipelines.

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Known WiX extensions with their NuGet package names
pub static KNOWN_EXTENSIONS: &[(&str, &str, &str)] = &[
    ("bal", "WixToolset.Bal.wixext", "Burn Bootstrapper Application Layer"),
    ("dependency", "WixToolset.Dependency.wixext", "Dependency/Provides"),
    ("difxapp", "WixToolset.DifxApp.wixext", "Driver Install Frameworks (deprecated)"),
    ("directx", "WixToolset.DirectX.wixext", "DirectX detection"),
    ("firewall", "WixToolset.Firewall.wixext", "Windows Firewall rules"),
    ("http", "WixToolset.Http.wixext", "HTTP URL reservations"),
    ("iis", "WixToolset.Iis.wixext", "IIS web server"),
    ("msmq", "WixToolset.Msmq.wixext", "Microsoft Message Queuing"),
    ("netfx", "WixToolset.Netfx.wixext", ".NET Framework detection"),
    ("powershell", "WixToolset.PowerShell.wixext", "PowerShell snap-ins"),
    ("sql", "WixToolset.Sql.wixext", "SQL Server databases"),
    ("ui", "WixToolset.UI.wixext", "Standard UI dialogs"),
    ("util", "WixToolset.Util.wixext", "Utility elements"),
    ("vs", "WixToolset.VisualStudio.wixext", "Visual Studio detection"),
];

/// Extension configuration for a project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionConfig {
    /// WiX version to target
    pub wix_version: String,
    /// Extensions with pinned versions
    pub extensions: HashMap<String, ExtensionEntry>,
}

/// Single extension entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionEntry {
    /// NuGet package name
    pub package: String,
    /// Pinned version (semver)
    pub version: String,
    /// Whether it's a prerelease version
    #[serde(default)]
    pub prerelease: bool,
}

/// Installed extension info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledExtension {
    pub name: String,
    pub package: String,
    pub version: String,
    pub path: Option<PathBuf>,
}

/// Extension issue detected
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionIssue {
    pub extension: String,
    pub issue_type: IssueType,
    pub message: String,
    pub suggestion: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IssueType {
    VersionMismatch,
    NotInstalled,
    Prerelease,
    Deprecated,
    UnknownExtension,
}

impl Default for ExtensionConfig {
    fn default() -> Self {
        Self {
            wix_version: "5.0".to_string(),
            extensions: HashMap::new(),
        }
    }
}

impl ExtensionConfig {
    /// Load from TOML file
    pub fn from_file(path: &Path) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: ExtensionConfig = toml::from_str(&content)?;
        Ok(config)
    }

    /// Save to TOML file
    pub fn save(&self, path: &Path) -> anyhow::Result<()> {
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Add an extension with version
    pub fn add(&mut self, short_name: &str, version: &str) -> anyhow::Result<()> {
        let (_, package, _) = KNOWN_EXTENSIONS
            .iter()
            .find(|(name, _, _)| *name == short_name)
            .ok_or_else(|| anyhow::anyhow!("Unknown extension: {}", short_name))?;

        let prerelease = version.contains('-');

        self.extensions.insert(
            short_name.to_string(),
            ExtensionEntry {
                package: package.to_string(),
                version: version.to_string(),
                prerelease,
            },
        );

        Ok(())
    }

    /// Remove an extension
    pub fn remove(&mut self, short_name: &str) -> bool {
        self.extensions.remove(short_name).is_some()
    }
}

/// WiX Extension Manager
pub struct ExtensionManager {
    /// Path to wix executable
    wix_path: PathBuf,
    /// Global extension directory
    global_dir: Option<PathBuf>,
}

impl Default for ExtensionManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ExtensionManager {
    pub fn new() -> Self {
        Self {
            wix_path: PathBuf::from("wix"),
            global_dir: None,
        }
    }

    pub fn with_wix_path(mut self, path: PathBuf) -> Self {
        self.wix_path = path;
        self
    }

    /// Check WiX installation
    pub fn check_wix(&self) -> anyhow::Result<String> {
        let output = Command::new(&self.wix_path)
            .arg("--version")
            .output()?;

        if output.status.success() {
            let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
            Ok(version)
        } else {
            Err(anyhow::anyhow!("WiX not found or not working"))
        }
    }

    /// List installed extensions
    pub fn list_installed(&self) -> anyhow::Result<Vec<InstalledExtension>> {
        let output = Command::new(&self.wix_path)
            .args(["extension", "list"])
            .output()?;

        if !output.status.success() {
            return Err(anyhow::anyhow!(
                "Failed to list extensions: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        parse_extension_list(&stdout)
    }

    /// Add an extension
    pub fn add_extension(&self, package: &str, version: Option<&str>) -> anyhow::Result<()> {
        let mut args = vec!["extension", "add"];

        let package_spec = if let Some(v) = version {
            format!("{}/{}", package, v)
        } else {
            package.to_string()
        };

        args.push(&package_spec);

        let output = Command::new(&self.wix_path).args(&args).output()?;

        if output.status.success() {
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "Failed to add extension: {}",
                String::from_utf8_lossy(&output.stderr)
            ))
        }
    }

    /// Remove an extension
    pub fn remove_extension(&self, package: &str) -> anyhow::Result<()> {
        let output = Command::new(&self.wix_path)
            .args(["extension", "remove", package])
            .output()?;

        if output.status.success() {
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "Failed to remove extension: {}",
                String::from_utf8_lossy(&output.stderr)
            ))
        }
    }

    /// Sync extensions from config file
    pub fn sync(&self, config: &ExtensionConfig) -> anyhow::Result<Vec<String>> {
        let mut actions = Vec::new();
        let installed = self.list_installed()?;

        // Create lookup of installed extensions
        let installed_map: HashMap<String, &InstalledExtension> = installed
            .iter()
            .map(|e| (e.package.clone(), e))
            .collect();

        // Add/update extensions from config
        for (name, entry) in &config.extensions {
            if let Some(existing) = installed_map.get(&entry.package) {
                if existing.version != entry.version {
                    // Remove old version
                    self.remove_extension(&entry.package)?;
                    actions.push(format!("Removed {} v{}", name, existing.version));

                    // Add new version
                    self.add_extension(&entry.package, Some(&entry.version))?;
                    actions.push(format!("Added {} v{}", name, entry.version));
                }
            } else {
                // Add new extension
                self.add_extension(&entry.package, Some(&entry.version))?;
                actions.push(format!("Added {} v{}", name, entry.version));
            }
        }

        Ok(actions)
    }

    /// Check for issues with installed extensions
    pub fn check(&self, config: &ExtensionConfig) -> anyhow::Result<Vec<ExtensionIssue>> {
        let mut issues = Vec::new();
        let installed = self.list_installed()?;

        // Create lookup of installed extensions
        let installed_map: HashMap<String, &InstalledExtension> = installed
            .iter()
            .map(|e| (e.package.clone(), e))
            .collect();

        // Check config against installed
        for (name, entry) in &config.extensions {
            if let Some(existing) = installed_map.get(&entry.package) {
                if existing.version != entry.version {
                    issues.push(ExtensionIssue {
                        extension: name.clone(),
                        issue_type: IssueType::VersionMismatch,
                        message: format!(
                            "Installed: {} v{}, Config: v{}",
                            name, existing.version, entry.version
                        ),
                        suggestion: format!(
                            "Run 'wix-ext sync' to update to v{}",
                            entry.version
                        ),
                    });
                }

                if entry.prerelease {
                    issues.push(ExtensionIssue {
                        extension: name.clone(),
                        issue_type: IssueType::Prerelease,
                        message: format!("{} is using prerelease version {}", name, entry.version),
                        suggestion: "Consider pinning to a stable release for production".to_string(),
                    });
                }
            } else {
                issues.push(ExtensionIssue {
                    extension: name.clone(),
                    issue_type: IssueType::NotInstalled,
                    message: format!("{} ({}) is not installed", name, entry.package),
                    suggestion: "Run 'wix-ext sync' to install missing extensions".to_string(),
                });
            }
        }

        // Check for deprecated extensions
        for ext in &installed {
            if ext.package.to_lowercase().contains("difxapp") {
                issues.push(ExtensionIssue {
                    extension: "difxapp".to_string(),
                    issue_type: IssueType::Deprecated,
                    message: "DifxApp extension is deprecated".to_string(),
                    suggestion: "Use DIFx driver installation directly or consider alternatives".to_string(),
                });
            }
        }

        Ok(issues)
    }

    /// Generate lockfile content
    pub fn generate_lockfile(&self, config: &ExtensionConfig) -> String {
        let mut content = String::new();

        content.push_str("# WiX Extension Lockfile\n");
        content.push_str("# Generated by wix-ext\n");
        content.push_str("# DO NOT EDIT - run 'wix-ext lock' to regenerate\n\n");

        content.push_str(&format!("wix_version = \"{}\"\n\n", config.wix_version));

        content.push_str("[extensions]\n");
        for (name, entry) in &config.extensions {
            content.push_str(&format!(
                "{} = {{ package = \"{}\", version = \"{}\" }}\n",
                name, entry.package, entry.version
            ));
        }

        content
    }
}

/// Parse `wix extension list` output
fn parse_extension_list(output: &str) -> anyhow::Result<Vec<InstalledExtension>> {
    let mut extensions = Vec::new();

    // Pattern: package/version or package version
    let re = Regex::new(r"([A-Za-z0-9.]+(?:\.wixext)?)[/\s]+(\d+\.\d+\.\d+(?:\.\d+)?(?:-[a-z0-9.]+)?)")?;

    for line in output.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with("Global") || line.starts_with("No ") {
            continue;
        }

        if let Some(caps) = re.captures(line) {
            let package = caps.get(1).unwrap().as_str();
            let version = caps.get(2).unwrap().as_str();

            // Find short name
            let name = KNOWN_EXTENSIONS
                .iter()
                .find(|(_, pkg, _)| pkg.to_lowercase() == package.to_lowercase())
                .map(|(name, _, _)| name.to_string())
                .unwrap_or_else(|| package.to_string());

            extensions.push(InstalledExtension {
                name,
                package: package.to_string(),
                version: version.to_string(),
                path: None,
            });
        }
    }

    Ok(extensions)
}

/// Detect extensions used in WiX files
pub fn detect_used_extensions(wix_content: &str) -> Vec<String> {
    let mut used = Vec::new();

    // Check for extension-specific elements
    let extension_patterns = [
        (r"<Bal\.", "bal"),
        (r"<Bundle\s", "bal"),
        (r"<Dependency\b", "dependency"),
        (r"<Provides\b", "dependency"),
        (r"<DifxApp", "difxapp"),
        (r"<DirectX", "directx"),
        (r"<Firewall", "firewall"),
        (r"<Http", "http"),
        (r"<UrlReservation", "http"),
        (r"<Iis\b", "iis"),
        (r"<WebSite", "iis"),
        (r"<WebApplication", "iis"),
        (r"<MessageQueue", "msmq"),
        (r"<Netfx", "netfx"),
        (r"<NetFx", "netfx"),
        (r"<PowerShell", "powershell"),
        (r"<Sql", "sql"),
        (r"<SqlDatabase", "sql"),
        (r"<UI\b", "ui"),
        (r"<WixUI", "ui"),
        (r"<UIRef\b", "ui"),
        (r"<Util\b", "util"),
        (r"<User\b", "util"),
        (r"<Group\b", "util"),
        (r"<XmlFile\b", "util"),
        (r"<XmlConfig\b", "util"),
        (r"<InternetShortcut\b", "util"),
        (r"<VisualStudio", "vs"),
    ];

    for (pattern, ext) in extension_patterns {
        if let Ok(re) = Regex::new(pattern) {
            if re.is_match(wix_content) && !used.contains(&ext.to_string()) {
                used.push(ext.to_string());
            }
        }
    }

    used
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_extension_list() {
        let output = "WixToolset.UI.wixext/5.0.0
WixToolset.Util.wixext/5.0.0
WixToolset.Bal.wixext/5.0.0";

        let extensions = parse_extension_list(output).unwrap();
        assert_eq!(extensions.len(), 3);
        assert_eq!(extensions[0].name, "ui");
        assert_eq!(extensions[1].name, "util");
    }

    #[test]
    fn test_config_add_extension() {
        let mut config = ExtensionConfig::default();
        config.add("ui", "5.0.0").unwrap();

        assert!(config.extensions.contains_key("ui"));
        assert_eq!(config.extensions["ui"].version, "5.0.0");
    }

    #[test]
    fn test_config_prerelease_detection() {
        let mut config = ExtensionConfig::default();
        config.add("ui", "5.0.0-preview.1").unwrap();

        assert!(config.extensions["ui"].prerelease);
    }

    #[test]
    fn test_detect_used_extensions() {
        let wix = r#"
        <Wix xmlns="http://wixtoolset.org/schemas/v4/wxs">
            <Package>
                <UI>
                    <UIRef Id="WixUI_Minimal" />
                </UI>
                <Util:User Id="TestUser" />
            </Package>
        </Wix>
        "#;

        let used = detect_used_extensions(wix);
        assert!(used.contains(&"ui".to_string()));
        assert!(used.contains(&"util".to_string()));
    }

    #[test]
    fn test_unknown_extension_error() {
        let mut config = ExtensionConfig::default();
        let result = config.add("nonexistent", "1.0.0");
        assert!(result.is_err());
    }

    #[test]
    fn test_generate_lockfile() {
        let mut config = ExtensionConfig::default();
        config.wix_version = "5.0.0".to_string();
        config.add("ui", "5.0.0").unwrap();

        let manager = ExtensionManager::new();
        let lockfile = manager.generate_lockfile(&config);

        assert!(lockfile.contains("wix_version = \"5.0.0\""));
        assert!(lockfile.contains("ui"));
    }
}

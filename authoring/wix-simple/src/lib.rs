//! wix-simple - Simplified WiX installer generator
//!
//! Generate complete WiX installers from minimal configuration.
//! No XML knowledge required - just specify what to install.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Simple installer configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimpleConfig {
    /// Application name
    pub name: String,
    /// Version (e.g., "1.0.0")
    pub version: String,
    /// Manufacturer/company name
    pub manufacturer: String,
    /// Files to install
    pub files: Vec<FileConfig>,
    /// Create shortcuts
    #[serde(default)]
    pub shortcuts: Vec<ShortcutConfig>,
    /// Platform (x64 or x86, default x64)
    #[serde(default = "default_platform")]
    pub platform: String,
    /// Install directory name (under Program Files)
    #[serde(default)]
    pub install_dir: Option<String>,
    /// Upgrade code (auto-generated if not specified)
    #[serde(default)]
    pub upgrade_code: Option<String>,
}

fn default_platform() -> String {
    "x64".to_string()
}

/// File configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileConfig {
    /// Source path
    pub source: String,
    /// Target subdirectory (optional)
    #[serde(default)]
    pub subdir: Option<String>,
}

/// Shortcut configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortcutConfig {
    /// Shortcut name
    pub name: String,
    /// Target file (relative to install dir)
    pub target: String,
    /// Location (start_menu or desktop)
    #[serde(default = "default_location")]
    pub location: String,
}

fn default_location() -> String {
    "start_menu".to_string()
}

/// WiX generator
pub struct SimpleGenerator;

impl SimpleGenerator {
    /// Generate complete WiX source from simple config
    pub fn generate(config: &SimpleConfig) -> String {
        let mut wix = String::new();

        let upgrade_code = config.upgrade_code.clone()
            .unwrap_or_else(|| format!("{{{}}}", Uuid::new_v4().to_string().to_uppercase()));
        let install_dir = config.install_dir.clone()
            .unwrap_or_else(|| config.name.clone());
        let _safe_name = sanitize_id(&config.name);

        // Header
        wix.push_str(&format!(r#"<?xml version="1.0" encoding="UTF-8"?>
<Wix xmlns="http://wixtoolset.org/schemas/v4/wxs">
  <Package Name="{}"
           Version="{}"
           Manufacturer="{}"
           UpgradeCode="{}"
           Platform="{}"
           Compressed="yes">

    <MajorUpgrade DowngradeErrorMessage="A newer version of [ProductName] is already installed." />
    <MediaTemplate EmbedCab="yes" />

"#, config.name, config.version, config.manufacturer, upgrade_code, config.platform));

        // Directory structure
        let program_files = if config.platform == "x86" { "ProgramFilesFolder" } else { "ProgramFiles64Folder" };
        wix.push_str(&format!(r#"    <!-- Directory Structure -->
    <StandardDirectory Id="{}">
      <Directory Id="INSTALLDIR" Name="{}">
"#, program_files, install_dir));

        // Collect subdirectories
        let mut subdirs: std::collections::HashSet<String> = std::collections::HashSet::new();
        for file in &config.files {
            if let Some(ref subdir) = file.subdir {
                subdirs.insert(subdir.clone());
            }
        }

        // Generate subdirectory elements
        for subdir in &subdirs {
            let subdir_id = format!("Dir_{}", sanitize_id(subdir));
            wix.push_str(&format!(r#"        <Directory Id="{}" Name="{}">
        </Directory>
"#, subdir_id, subdir));
        }

        // Main component for files without subdir
        let main_files: Vec<_> = config.files.iter()
            .filter(|f| f.subdir.is_none())
            .collect();

        if !main_files.is_empty() {
            wix.push_str(r#"        <Component Id="MainComponent" Guid="*">
"#);
            for (i, file) in main_files.iter().enumerate() {
                let file_name = std::path::Path::new(&file.source)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("file");
                let file_id = format!("File_{}", sanitize_id(file_name));
                let key_path = if i == 0 { " KeyPath=\"yes\"" } else { "" };
                wix.push_str(&format!(r#"          <File Id="{}" Source="{}"{} />
"#, file_id, file.source, key_path));
            }
            wix.push_str(r#"        </Component>
"#);
        }

        wix.push_str(r#"      </Directory>
    </StandardDirectory>

"#);

        // Shortcuts
        if !config.shortcuts.is_empty() {
            let start_menu_shortcuts: Vec<_> = config.shortcuts.iter()
                .filter(|s| s.location == "start_menu")
                .collect();
            let desktop_shortcuts: Vec<_> = config.shortcuts.iter()
                .filter(|s| s.location == "desktop")
                .collect();

            if !start_menu_shortcuts.is_empty() {
                wix.push_str(&format!(r#"    <!-- Start Menu Shortcuts -->
    <StandardDirectory Id="ProgramMenuFolder">
      <Directory Id="ApplicationProgramsFolder" Name="{}">
        <Component Id="StartMenuShortcuts" Guid="*">
"#, config.name));

                for shortcut in &start_menu_shortcuts {
                    let shortcut_id = format!("Shortcut_{}", sanitize_id(&shortcut.name));
                    wix.push_str(&format!(r#"          <Shortcut Id="{}"
                    Name="{}"
                    Target="[INSTALLDIR]{}"
                    WorkingDirectory="INSTALLDIR" />
"#, shortcut_id, shortcut.name, shortcut.target));
                }

                wix.push_str(&format!(r#"          <RemoveFolder Id="CleanUpStartMenu" Directory="ApplicationProgramsFolder" On="uninstall" />
          <RegistryValue Root="HKCU" Key="Software\\{}\\{}" Name="installed" Type="integer" Value="1" KeyPath="yes" />
        </Component>
      </Directory>
    </StandardDirectory>

"#, config.manufacturer, config.name));
            }

            if !desktop_shortcuts.is_empty() {
                wix.push_str(r#"    <!-- Desktop Shortcuts -->
    <StandardDirectory Id="DesktopFolder">
      <Component Id="DesktopShortcuts" Guid="*">
"#);

                for shortcut in &desktop_shortcuts {
                    let shortcut_id = format!("DesktopShortcut_{}", sanitize_id(&shortcut.name));
                    wix.push_str(&format!(r#"        <Shortcut Id="{}"
                  Name="{}"
                  Target="[INSTALLDIR]{}"
                  WorkingDirectory="INSTALLDIR" />
"#, shortcut_id, shortcut.name, shortcut.target));
                }

                wix.push_str(&format!(r#"        <RegistryValue Root="HKCU" Key="Software\\{}\\{}\\Desktop" Name="installed" Type="integer" Value="1" KeyPath="yes" />
      </Component>
    </StandardDirectory>

"#, config.manufacturer, config.name));
            }
        }

        // Feature
        wix.push_str(&format!(r#"    <!-- Feature -->
    <Feature Id="MainFeature" Title="{}" Level="1">
"#, config.name));

        if !main_files.is_empty() {
            wix.push_str(r#"      <ComponentRef Id="MainComponent" />
"#);
        }

        if config.shortcuts.iter().any(|s| s.location == "start_menu") {
            wix.push_str(r#"      <ComponentRef Id="StartMenuShortcuts" />
"#);
        }

        if config.shortcuts.iter().any(|s| s.location == "desktop") {
            wix.push_str(r#"      <ComponentRef Id="DesktopShortcuts" />
"#);
        }

        wix.push_str(r#"    </Feature>

  </Package>
</Wix>
"#);

        wix
    }

    /// Generate from JSON config string
    pub fn generate_from_json(json: &str) -> anyhow::Result<String> {
        let config: SimpleConfig = serde_json::from_str(json)?;
        Ok(Self::generate(&config))
    }

    /// Create example config
    pub fn example_config() -> SimpleConfig {
        SimpleConfig {
            name: "MyApplication".to_string(),
            version: "1.0.0".to_string(),
            manufacturer: "My Company".to_string(),
            files: vec![
                FileConfig {
                    source: "bin\\myapp.exe".to_string(),
                    subdir: None,
                },
                FileConfig {
                    source: "bin\\mylib.dll".to_string(),
                    subdir: None,
                },
            ],
            shortcuts: vec![
                ShortcutConfig {
                    name: "My Application".to_string(),
                    target: "myapp.exe".to_string(),
                    location: "start_menu".to_string(),
                },
            ],
            platform: "x64".to_string(),
            install_dir: None,
            upgrade_code: None,
        }
    }
}

fn sanitize_id(s: &str) -> String {
    s.chars()
        .map(|c| if c.is_alphanumeric() { c } else { '_' })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_basic() {
        let config = SimpleConfig {
            name: "TestApp".to_string(),
            version: "1.0.0".to_string(),
            manufacturer: "Test Corp".to_string(),
            files: vec![
                FileConfig {
                    source: "app.exe".to_string(),
                    subdir: None,
                },
            ],
            shortcuts: vec![],
            platform: "x64".to_string(),
            install_dir: None,
            upgrade_code: Some("{11111111-1111-1111-1111-111111111111}".to_string()),
        };

        let wix = SimpleGenerator::generate(&config);
        assert!(wix.contains("TestApp"));
        assert!(wix.contains("Test Corp"));
        assert!(wix.contains("app.exe"));
        assert!(wix.contains("ProgramFiles64Folder"));
    }

    #[test]
    fn test_generate_with_shortcuts() {
        let config = SimpleConfig {
            name: "TestApp".to_string(),
            version: "1.0.0".to_string(),
            manufacturer: "Test Corp".to_string(),
            files: vec![
                FileConfig {
                    source: "app.exe".to_string(),
                    subdir: None,
                },
            ],
            shortcuts: vec![
                ShortcutConfig {
                    name: "Test App".to_string(),
                    target: "app.exe".to_string(),
                    location: "start_menu".to_string(),
                },
            ],
            platform: "x64".to_string(),
            install_dir: None,
            upgrade_code: None,
        };

        let wix = SimpleGenerator::generate(&config);
        assert!(wix.contains("StartMenuShortcuts"));
        assert!(wix.contains("Test App"));
    }

    #[test]
    fn test_generate_x86() {
        let config = SimpleConfig {
            name: "TestApp".to_string(),
            version: "1.0.0".to_string(),
            manufacturer: "Test Corp".to_string(),
            files: vec![],
            shortcuts: vec![],
            platform: "x86".to_string(),
            install_dir: None,
            upgrade_code: None,
        };

        let wix = SimpleGenerator::generate(&config);
        assert!(wix.contains("ProgramFilesFolder"));
        assert!(wix.contains("Platform=\"x86\""));
    }

    #[test]
    fn test_generate_from_json() {
        let json = r#"{
            "name": "JsonApp",
            "version": "2.0.0",
            "manufacturer": "JSON Corp",
            "files": [{"source": "app.exe"}],
            "shortcuts": []
        }"#;

        let wix = SimpleGenerator::generate_from_json(json).unwrap();
        assert!(wix.contains("JsonApp"));
        assert!(wix.contains("2.0.0"));
    }

    #[test]
    fn test_example_config() {
        let config = SimpleGenerator::example_config();
        let wix = SimpleGenerator::generate(&config);
        assert!(wix.contains("MyApplication"));
    }
}

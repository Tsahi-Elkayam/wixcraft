//! Import installers from NSIS, InnoSetup, and other formats to WiX
//!
//! Parses installer scripts and generates equivalent WXS files.
//!
//! # Example
//!
//! ```
//! use wix_import::{ImportFormat, Importer};
//!
//! let nsis_script = r#"
//! Name "My Application"
//! OutFile "setup.exe"
//! InstallDir "$PROGRAMFILES\MyApp"
//! "#;
//!
//! let importer = Importer::new(ImportFormat::Nsis);
//! let result = importer.parse(nsis_script).unwrap();
//! assert_eq!(result.product_name, Some("My Application".to_string()));
//! ```

use regex::Regex;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Import errors
#[derive(Error, Debug)]
pub enum ImportError {
    #[error("Parse error: {0}")]
    ParseError(String),
    #[error("Unsupported format: {0}")]
    UnsupportedFormat(String),
    #[error("Missing required field: {0}")]
    MissingField(String),
}

/// Source format to import from
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ImportFormat {
    Nsis,
    InnoSetup,
    InstallShield,
}

impl ImportFormat {
    pub fn as_str(&self) -> &'static str {
        match self {
            ImportFormat::Nsis => "NSIS",
            ImportFormat::InnoSetup => "InnoSetup",
            ImportFormat::InstallShield => "InstallShield",
        }
    }

    pub fn detect(content: &str) -> Option<ImportFormat> {
        if content.contains("!include") || content.contains("OutFile") || content.contains("Section") && content.contains("SectionEnd") {
            Some(ImportFormat::Nsis)
        } else if content.contains("[Setup]") && content.contains("AppName=") {
            Some(ImportFormat::InnoSetup)
        } else {
            None
        }
    }
}

/// Parsed installer information
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct InstallerInfo {
    /// Product name
    pub product_name: Option<String>,
    /// Product version
    pub version: Option<String>,
    /// Publisher/manufacturer
    pub publisher: Option<String>,
    /// Installation directory
    pub install_dir: Option<String>,
    /// Output file name
    pub output_file: Option<String>,
    /// License file
    pub license_file: Option<String>,
    /// Files to install
    pub files: Vec<FileEntry>,
    /// Shortcuts to create
    pub shortcuts: Vec<ShortcutEntry>,
    /// Registry entries
    pub registry: Vec<RegistryEntry>,
    /// Uninstaller info
    pub uninstaller: Option<UninstallerInfo>,
    /// Custom actions/commands
    pub commands: Vec<String>,
    /// Source format
    pub source_format: String,
    /// Warnings during import
    pub warnings: Vec<String>,
}

/// A file to install
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    pub source: String,
    pub destination: String,
    pub recursive: bool,
}

/// A shortcut to create
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortcutEntry {
    pub name: String,
    pub target: String,
    pub location: ShortcutLocation,
    pub icon: Option<String>,
}

/// Shortcut location
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShortcutLocation {
    Desktop,
    StartMenu,
    Startup,
}

/// Registry entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryEntry {
    pub root: String,
    pub key: String,
    pub name: Option<String>,
    pub value: Option<String>,
    pub value_type: String,
}

/// Uninstaller information
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UninstallerInfo {
    pub name: Option<String>,
    pub icon: Option<String>,
}

/// Importer for converting other formats to WiX
pub struct Importer {
    format: ImportFormat,
}

impl Importer {
    pub fn new(format: ImportFormat) -> Self {
        Self { format }
    }

    /// Auto-detect format and create importer
    pub fn auto_detect(content: &str) -> Result<Self, ImportError> {
        ImportFormat::detect(content)
            .map(|f| Self::new(f))
            .ok_or_else(|| ImportError::UnsupportedFormat("Could not detect format".to_string()))
    }

    /// Parse the installer script
    pub fn parse(&self, content: &str) -> Result<InstallerInfo, ImportError> {
        match self.format {
            ImportFormat::Nsis => self.parse_nsis(content),
            ImportFormat::InnoSetup => self.parse_innosetup(content),
            ImportFormat::InstallShield => Err(ImportError::UnsupportedFormat(
                "InstallShield import not yet implemented".to_string(),
            )),
        }
    }

    fn parse_nsis(&self, content: &str) -> Result<InstallerInfo, ImportError> {
        let mut info = InstallerInfo {
            source_format: "NSIS".to_string(),
            ..Default::default()
        };

        // Parse Name
        let name_re = Regex::new(r#"Name\s+"([^"]+)""#).unwrap();
        if let Some(cap) = name_re.captures(content) {
            info.product_name = Some(cap[1].to_string());
        }

        // Parse OutFile
        let outfile_re = Regex::new(r#"OutFile\s+"([^"]+)""#).unwrap();
        if let Some(cap) = outfile_re.captures(content) {
            info.output_file = Some(cap[1].to_string());
        }

        // Parse InstallDir
        let installdir_re = Regex::new(r#"InstallDir\s+"?([^\n"]+)"?"#).unwrap();
        if let Some(cap) = installdir_re.captures(content) {
            info.install_dir = Some(cap[1].trim().to_string());
        }

        // Parse Version
        let version_re = Regex::new(r#"!define\s+VERSION\s+"([^"]+)""#).unwrap();
        if let Some(cap) = version_re.captures(content) {
            info.version = Some(cap[1].to_string());
        }

        // Parse Publisher
        let publisher_re = Regex::new(r#"!define\s+(?:PUBLISHER|COMPANY)\s+"([^"]+)""#).unwrap();
        if let Some(cap) = publisher_re.captures(content) {
            info.publisher = Some(cap[1].to_string());
        }

        // Parse LicenseData
        let license_re = Regex::new(r#"LicenseData\s+"([^"]+)""#).unwrap();
        if let Some(cap) = license_re.captures(content) {
            info.license_file = Some(cap[1].to_string());
        }

        // Parse File commands
        let file_re = Regex::new(r#"File\s+(?:/r\s+)?(?:/oname=([^\s]+)\s+)?"([^"]+)""#).unwrap();
        for cap in file_re.captures_iter(content) {
            let source = cap.get(2).map(|m| m.as_str()).unwrap_or("");
            let recursive = content[cap.get(0).unwrap().start()..cap.get(0).unwrap().end()]
                .contains("/r");
            info.files.push(FileEntry {
                source: source.to_string(),
                destination: "$INSTDIR".to_string(),
                recursive,
            });
        }

        // Parse SetOutPath (destination)
        let outpath_re = Regex::new(r#"SetOutPath\s+"?([^\n"]+)"?"#).unwrap();
        let outpaths: Vec<_> = outpath_re
            .captures_iter(content)
            .map(|c| c[1].to_string())
            .collect();
        if !outpaths.is_empty() && !info.files.is_empty() {
            // Associate files with their output paths
            // This is a simplified version - real implementation would track state
        }

        // Parse CreateShortCut
        let shortcut_re = Regex::new(
            r#"CreateShortCut\s+"([^"]+)"\s+"([^"]+)"(?:\s+"[^"]*")*(?:\s+"([^"]+)")?"#,
        )
        .unwrap();
        for cap in shortcut_re.captures_iter(content) {
            let path = &cap[1];
            let location = if path.contains("$DESKTOP") {
                ShortcutLocation::Desktop
            } else if path.contains("$SMSTARTUP") {
                ShortcutLocation::Startup
            } else {
                ShortcutLocation::StartMenu
            };

            info.shortcuts.push(ShortcutEntry {
                name: path
                    .split('\\')
                    .last()
                    .unwrap_or(path)
                    .replace(".lnk", "")
                    .to_string(),
                target: cap[2].to_string(),
                location,
                icon: cap.get(3).map(|m| m.as_str().to_string()),
            });
        }

        // Parse WriteRegStr
        let reg_re = Regex::new(r#"WriteRegStr\s+(\w+)\s+"([^"]+)"\s+"([^"]*)"\s+"([^"]*)""#)
            .unwrap();
        for cap in reg_re.captures_iter(content) {
            info.registry.push(RegistryEntry {
                root: cap[1].to_string(),
                key: cap[2].to_string(),
                name: if cap[3].is_empty() {
                    None
                } else {
                    Some(cap[3].to_string())
                },
                value: Some(cap[4].to_string()),
                value_type: "string".to_string(),
            });
        }

        // Check for uninstaller
        if content.contains("WriteUninstaller") {
            info.uninstaller = Some(UninstallerInfo::default());
        }

        // Add warnings for unsupported features
        if content.contains("!insertmacro") {
            info.warnings
                .push("NSIS macros detected - manual conversion may be needed".to_string());
        }
        if content.contains("Call ") {
            info.warnings
                .push("Function calls detected - review custom logic".to_string());
        }

        Ok(info)
    }

    fn parse_innosetup(&self, content: &str) -> Result<InstallerInfo, ImportError> {
        let mut info = InstallerInfo {
            source_format: "InnoSetup".to_string(),
            ..Default::default()
        };

        // Parse [Setup] section
        let setup_re = Regex::new(r"(?s)\[Setup\](.*?)(?:\[|$)").unwrap();
        if let Some(cap) = setup_re.captures(content) {
            let setup_section = &cap[1];

            // AppName
            let name_re = Regex::new(r"AppName=(.+)").unwrap();
            if let Some(m) = name_re.captures(setup_section) {
                info.product_name = Some(m[1].trim().to_string());
            }

            // AppVersion
            let ver_re = Regex::new(r"AppVersion=(.+)").unwrap();
            if let Some(m) = ver_re.captures(setup_section) {
                info.version = Some(m[1].trim().to_string());
            }

            // AppPublisher
            let pub_re = Regex::new(r"AppPublisher=(.+)").unwrap();
            if let Some(m) = pub_re.captures(setup_section) {
                info.publisher = Some(m[1].trim().to_string());
            }

            // DefaultDirName
            let dir_re = Regex::new(r"DefaultDirName=(.+)").unwrap();
            if let Some(m) = dir_re.captures(setup_section) {
                info.install_dir = Some(m[1].trim().to_string());
            }

            // OutputBaseFilename
            let out_re = Regex::new(r"OutputBaseFilename=(.+)").unwrap();
            if let Some(m) = out_re.captures(setup_section) {
                info.output_file = Some(format!("{}.exe", m[1].trim()));
            }

            // LicenseFile
            let lic_re = Regex::new(r"LicenseFile=(.+)").unwrap();
            if let Some(m) = lic_re.captures(setup_section) {
                info.license_file = Some(m[1].trim().to_string());
            }
        }

        // Parse [Files] section
        let files_re = Regex::new(r"(?s)\[Files\](.*?)(?:\[|$)").unwrap();
        if let Some(cap) = files_re.captures(content) {
            let files_section = &cap[1];
            let file_re = Regex::new(r#"Source:\s*"([^"]+)";\s*DestDir:\s*"([^"]+)"(?:.*?Flags:\s*([^;]+))?"#).unwrap();

            for m in file_re.captures_iter(files_section) {
                let flags = m.get(3).map(|f| f.as_str()).unwrap_or("");
                info.files.push(FileEntry {
                    source: m[1].to_string(),
                    destination: m[2].to_string(),
                    recursive: flags.contains("recursesubdirs"),
                });
            }
        }

        // Parse [Icons] section (shortcuts)
        let icons_re = Regex::new(r"(?s)\[Icons\](.*?)(?:\[|$)").unwrap();
        if let Some(cap) = icons_re.captures(content) {
            let icons_section = &cap[1];
            let icon_re =
                Regex::new(r#"Name:\s*"([^"]+)";\s*Filename:\s*"([^"]+)"(?:.*?IconFilename:\s*"([^"]+)")?"#)
                    .unwrap();

            for m in icon_re.captures_iter(icons_section) {
                let name = &m[1];
                let location = if name.contains("{userdesktop}") || name.contains("{commondesktop}")
                {
                    ShortcutLocation::Desktop
                } else if name.contains("{userstartup}") || name.contains("{commonstartup}") {
                    ShortcutLocation::Startup
                } else {
                    ShortcutLocation::StartMenu
                };

                info.shortcuts.push(ShortcutEntry {
                    name: name
                        .split('\\')
                        .last()
                        .unwrap_or(name)
                        .to_string(),
                    target: m[2].to_string(),
                    location,
                    icon: m.get(3).map(|i| i.as_str().to_string()),
                });
            }
        }

        // Parse [Registry] section
        let reg_re = Regex::new(r"(?s)\[Registry\](.*?)(?:\[|$)").unwrap();
        if let Some(cap) = reg_re.captures(content) {
            let reg_section = &cap[1];
            let entry_re = Regex::new(
                r#"Root:\s*(\w+);\s*Subkey:\s*"([^"]+)"(?:;\s*ValueName:\s*"([^"]*)")?(?:;\s*ValueData:\s*"([^"]*)")?(?:;\s*ValueType:\s*(\w+))?"#,
            )
            .unwrap();

            for m in entry_re.captures_iter(reg_section) {
                info.registry.push(RegistryEntry {
                    root: m[1].to_string(),
                    key: m[2].to_string(),
                    name: m.get(3).map(|n| n.as_str().to_string()),
                    value: m.get(4).map(|v| v.as_str().to_string()),
                    value_type: m.get(5).map(|t| t.as_str()).unwrap_or("string").to_string(),
                });
            }
        }

        // Check for [UninstallRun] or [UninstallDelete]
        if content.contains("[UninstallRun]") || content.contains("[UninstallDelete]") {
            info.uninstaller = Some(UninstallerInfo::default());
        }

        // Add warnings for complex features
        if content.contains("[Code]") {
            info.warnings
                .push("Pascal Script code detected - manual conversion needed".to_string());
        }
        if content.contains("[Tasks]") {
            info.warnings
                .push("Tasks section detected - review optional components".to_string());
        }

        Ok(info)
    }

    /// Generate WXS from parsed info
    pub fn generate_wxs(&self, info: &InstallerInfo) -> String {
        let mut wxs = String::new();

        wxs.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
        wxs.push_str(
            "<Wix xmlns=\"http://wixtoolset.org/schemas/v4/wxs\">\n",
        );

        // Package element
        wxs.push_str("  <Package\n");
        if let Some(ref name) = info.product_name {
            wxs.push_str(&format!("    Name=\"{}\"\n", escape_xml(name)));
        }
        if let Some(ref version) = info.version {
            wxs.push_str(&format!("    Version=\"{}\"\n", version));
        }
        if let Some(ref publisher) = info.publisher {
            wxs.push_str(&format!(
                "    Manufacturer=\"{}\"\n",
                escape_xml(publisher)
            ));
        }
        wxs.push_str("    UpgradeCode=\"PUT-GUID-HERE\"\n");
        wxs.push_str("    Compressed=\"yes\">\n\n");

        // Media
        wxs.push_str("    <Media Id=\"1\" Cabinet=\"product.cab\" EmbedCab=\"yes\" />\n\n");

        // Directory structure
        wxs.push_str("    <StandardDirectory Id=\"ProgramFilesFolder\">\n");
        wxs.push_str("      <Directory Id=\"INSTALLFOLDER\" Name=\"");
        if let Some(ref name) = info.product_name {
            wxs.push_str(&escape_xml(name));
        } else {
            wxs.push_str("MyApp");
        }
        wxs.push_str("\">\n");

        // Components for files
        if !info.files.is_empty() {
            wxs.push_str("        <Component Id=\"MainComponent\" Guid=\"*\">\n");
            for file in &info.files {
                let _file_name = file
                    .source
                    .split(['\\', '/'])
                    .last()
                    .unwrap_or(&file.source);
                wxs.push_str(&format!(
                    "          <File Source=\"{}\" />\n",
                    escape_xml(&file.source)
                ));
            }
            wxs.push_str("        </Component>\n");
        }

        wxs.push_str("      </Directory>\n");
        wxs.push_str("    </StandardDirectory>\n\n");

        // Shortcuts
        if !info.shortcuts.is_empty() {
            wxs.push_str("    <StandardDirectory Id=\"ProgramMenuFolder\">\n");
            wxs.push_str("      <Directory Id=\"ProgramMenuDir\" Name=\"");
            if let Some(ref name) = info.product_name {
                wxs.push_str(&escape_xml(name));
            } else {
                wxs.push_str("MyApp");
            }
            wxs.push_str("\">\n");
            wxs.push_str("        <Component Id=\"ShortcutsComponent\" Guid=\"*\">\n");

            for shortcut in &info.shortcuts {
                wxs.push_str(&format!(
                    "          <Shortcut Id=\"{}\" Name=\"{}\" Target=\"[INSTALLFOLDER]{}\" />\n",
                    escape_xml(&shortcut.name.replace(' ', "_")),
                    escape_xml(&shortcut.name),
                    escape_xml(
                        shortcut
                            .target
                            .split(['\\', '/'])
                            .last()
                            .unwrap_or(&shortcut.target)
                    )
                ));
            }

            wxs.push_str(
                "          <RemoveFolder Id=\"ProgramMenuDir\" On=\"uninstall\" />\n",
            );
            wxs.push_str("          <RegistryValue Root=\"HKCU\" Key=\"Software\\[Manufacturer]\\[ProductName]\" Name=\"installed\" Type=\"integer\" Value=\"1\" KeyPath=\"yes\" />\n");
            wxs.push_str("        </Component>\n");
            wxs.push_str("      </Directory>\n");
            wxs.push_str("    </StandardDirectory>\n\n");
        }

        // Feature
        wxs.push_str("    <Feature Id=\"MainFeature\" Title=\"Main Feature\" Level=\"1\">\n");
        if !info.files.is_empty() {
            wxs.push_str("      <ComponentRef Id=\"MainComponent\" />\n");
        }
        if !info.shortcuts.is_empty() {
            wxs.push_str("      <ComponentRef Id=\"ShortcutsComponent\" />\n");
        }
        wxs.push_str("    </Feature>\n\n");

        wxs.push_str("  </Package>\n");
        wxs.push_str("</Wix>\n");

        // Add comments for warnings
        if !info.warnings.is_empty() {
            let mut comments = String::from("\n<!-- Import Warnings:\n");
            for warning in &info.warnings {
                comments.push_str(&format!("  - {}\n", warning));
            }
            comments.push_str("-->\n");
            wxs = comments + &wxs;
        }

        wxs
    }
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_nsis() {
        let content = r#"Name "My App"
OutFile "setup.exe"
Section "Main"
SectionEnd"#;

        assert_eq!(ImportFormat::detect(content), Some(ImportFormat::Nsis));
    }

    #[test]
    fn test_detect_innosetup() {
        let content = r#"[Setup]
AppName=My Application
AppVersion=1.0"#;

        assert_eq!(ImportFormat::detect(content), Some(ImportFormat::InnoSetup));
    }

    #[test]
    fn test_parse_nsis_name() {
        let content = r#"Name "My Application""#;
        let importer = Importer::new(ImportFormat::Nsis);
        let result = importer.parse(content).unwrap();

        assert_eq!(result.product_name, Some("My Application".to_string()));
    }

    #[test]
    fn test_parse_nsis_outfile() {
        let content = r#"OutFile "setup.exe""#;
        let importer = Importer::new(ImportFormat::Nsis);
        let result = importer.parse(content).unwrap();

        assert_eq!(result.output_file, Some("setup.exe".to_string()));
    }

    #[test]
    fn test_parse_nsis_installdir() {
        let content = r#"InstallDir "$PROGRAMFILES\MyApp""#;
        let importer = Importer::new(ImportFormat::Nsis);
        let result = importer.parse(content).unwrap();

        assert_eq!(
            result.install_dir,
            Some("$PROGRAMFILES\\MyApp".to_string())
        );
    }

    #[test]
    fn test_parse_nsis_file() {
        let content = r#"File "app.exe""#;
        let importer = Importer::new(ImportFormat::Nsis);
        let result = importer.parse(content).unwrap();

        assert!(!result.files.is_empty());
        assert_eq!(result.files[0].source, "app.exe");
    }

    #[test]
    fn test_parse_innosetup_setup() {
        let content = r#"[Setup]
AppName=Test App
AppVersion=2.0
AppPublisher=Test Inc"#;

        let importer = Importer::new(ImportFormat::InnoSetup);
        let result = importer.parse(content).unwrap();

        assert_eq!(result.product_name, Some("Test App".to_string()));
        assert_eq!(result.version, Some("2.0".to_string()));
        assert_eq!(result.publisher, Some("Test Inc".to_string()));
    }

    #[test]
    fn test_parse_innosetup_files() {
        let content = r#"[Setup]
AppName=Test
[Files]
Source: "app.exe"; DestDir: "{app}""#;

        let importer = Importer::new(ImportFormat::InnoSetup);
        let result = importer.parse(content).unwrap();

        assert!(!result.files.is_empty());
        assert_eq!(result.files[0].source, "app.exe");
    }

    #[test]
    fn test_auto_detect() {
        let nsis = r#"Name "Test"
OutFile "test.exe""#;

        let importer = Importer::auto_detect(nsis).unwrap();
        assert_eq!(importer.format, ImportFormat::Nsis);
    }

    #[test]
    fn test_generate_wxs() {
        let info = InstallerInfo {
            product_name: Some("Test App".to_string()),
            version: Some("1.0.0".to_string()),
            publisher: Some("Test Inc".to_string()),
            ..Default::default()
        };

        let importer = Importer::new(ImportFormat::Nsis);
        let wxs = importer.generate_wxs(&info);

        assert!(wxs.contains("Name=\"Test App\""));
        assert!(wxs.contains("Version=\"1.0.0\""));
        assert!(wxs.contains("Manufacturer=\"Test Inc\""));
    }

    #[test]
    fn test_generate_wxs_with_files() {
        let info = InstallerInfo {
            product_name: Some("Test".to_string()),
            files: vec![FileEntry {
                source: "app.exe".to_string(),
                destination: "$INSTDIR".to_string(),
                recursive: false,
            }],
            ..Default::default()
        };

        let importer = Importer::new(ImportFormat::Nsis);
        let wxs = importer.generate_wxs(&info);

        assert!(wxs.contains("<File"));
        assert!(wxs.contains("Source=\"app.exe\""));
    }

    #[test]
    fn test_generate_wxs_with_shortcuts() {
        let info = InstallerInfo {
            product_name: Some("Test".to_string()),
            shortcuts: vec![ShortcutEntry {
                name: "Test App".to_string(),
                target: "app.exe".to_string(),
                location: ShortcutLocation::StartMenu,
                icon: None,
            }],
            ..Default::default()
        };

        let importer = Importer::new(ImportFormat::Nsis);
        let wxs = importer.generate_wxs(&info);

        assert!(wxs.contains("<Shortcut"));
        assert!(wxs.contains("Name=\"Test App\""));
    }

    #[test]
    fn test_parse_nsis_shortcut() {
        let content = r#"CreateShortCut "$SMPROGRAMS\MyApp.lnk" "$INSTDIR\app.exe""#;
        let importer = Importer::new(ImportFormat::Nsis);
        let result = importer.parse(content).unwrap();

        assert!(!result.shortcuts.is_empty());
        assert_eq!(result.shortcuts[0].location, ShortcutLocation::StartMenu);
    }

    #[test]
    fn test_parse_nsis_registry() {
        let content =
            r#"WriteRegStr HKLM "Software\MyApp" "Version" "1.0""#;
        let importer = Importer::new(ImportFormat::Nsis);
        let result = importer.parse(content).unwrap();

        assert!(!result.registry.is_empty());
        assert_eq!(result.registry[0].root, "HKLM");
    }

    #[test]
    fn test_warnings_nsis_macros() {
        let content = r#"!insertmacro MUI_PAGE_WELCOME"#;
        let importer = Importer::new(ImportFormat::Nsis);
        let result = importer.parse(content).unwrap();

        assert!(!result.warnings.is_empty());
    }

    #[test]
    fn test_warnings_innosetup_code() {
        let content = r#"[Setup]
AppName=Test
[Code]
function Test: Boolean;
begin
end;"#;

        let importer = Importer::new(ImportFormat::InnoSetup);
        let result = importer.parse(content).unwrap();

        assert!(result.warnings.iter().any(|w| w.contains("Pascal")));
    }

    #[test]
    fn test_format_names() {
        assert_eq!(ImportFormat::Nsis.as_str(), "NSIS");
        assert_eq!(ImportFormat::InnoSetup.as_str(), "InnoSetup");
    }

    #[test]
    fn test_source_format() {
        let importer = Importer::new(ImportFormat::Nsis);
        let result = importer.parse("Name \"Test\"").unwrap();

        assert_eq!(result.source_format, "NSIS");
    }

    #[test]
    fn test_escape_xml() {
        assert_eq!(escape_xml("a & b"), "a &amp; b");
        assert_eq!(escape_xml("<test>"), "&lt;test&gt;");
    }

    #[test]
    fn test_installshield_unsupported() {
        let importer = Importer::new(ImportFormat::InstallShield);
        let result = importer.parse("");

        assert!(matches!(result, Err(ImportError::UnsupportedFormat(_))));
    }

    #[test]
    fn test_file_entry_recursive() {
        let content = r#"File /r "data\*.*""#;
        let importer = Importer::new(ImportFormat::Nsis);
        let result = importer.parse(content).unwrap();

        // The /r flag indicates recursive
        assert!(!result.files.is_empty());
    }

    #[test]
    fn test_parse_nsis_version() {
        let content = r#"!define VERSION "1.2.3""#;
        let importer = Importer::new(ImportFormat::Nsis);
        let result = importer.parse(content).unwrap();

        assert_eq!(result.version, Some("1.2.3".to_string()));
    }
}

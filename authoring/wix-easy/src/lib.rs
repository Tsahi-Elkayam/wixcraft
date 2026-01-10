//! wix-easy - Ansible-like YAML to WiX/MSI generator
//!
//! Define your installer in simple YAML, get WiX XML output.
//!
//! # Example YAML
//!
//! ```yaml
//! package:
//!   name: MyApp
//!   version: 1.0.0
//!   manufacturer: My Company
//!   description: My awesome application
//!
//! install:
//!   directory: ProgramFiles/MyCompany/MyApp
//!   files:
//!     - src: ./bin/*
//!     - src: ./config/default.json
//!       dest: config/
//!
//! shortcuts:
//!   - name: MyApp
//!     target: MyApp.exe
//!     location: desktop
//!
//! prerequisites:
//!   - dotnet: 8.0
//!   - vcredist: 2022
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use thiserror::Error;
use uuid::Uuid;

#[derive(Error, Debug)]
pub enum EasyError {
    #[error("Failed to parse YAML: {0}")]
    YamlError(#[from] serde_yaml::Error),

    #[error("Failed to read file: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Invalid configuration: {0}")]
    ConfigError(String),

    #[error("Missing required field: {0}")]
    MissingField(String),
}

pub type Result<T> = std::result::Result<T, EasyError>;

/// Main installer definition from YAML
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallerDef {
    /// Package metadata
    pub package: PackageDef,

    /// Installation configuration
    #[serde(default)]
    pub install: InstallDef,

    /// Features (optional, auto-generated if not specified)
    #[serde(default)]
    pub features: Vec<FeatureDef>,

    /// Shortcuts
    #[serde(default)]
    pub shortcuts: Vec<ShortcutDef>,

    /// Registry entries
    #[serde(default)]
    pub registry: Vec<RegistryDef>,

    /// Environment variables
    #[serde(default)]
    pub environment: Vec<EnvironmentDef>,

    /// Services
    #[serde(default)]
    pub services: Vec<ServiceDef>,

    /// Prerequisites
    #[serde(default)]
    pub prerequisites: Vec<PrerequisiteDef>,

    /// UI configuration
    #[serde(default)]
    pub ui: UiDef,

    /// Upgrade configuration
    #[serde(default)]
    pub upgrade: UpgradeDef,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageDef {
    /// Product name
    pub name: String,

    /// Product version (X.Y.Z format)
    pub version: String,

    /// Manufacturer/company name
    pub manufacturer: String,

    /// Product description
    #[serde(default)]
    pub description: String,

    /// Product GUID (auto-generated if not specified)
    #[serde(default)]
    pub product_code: Option<String>,

    /// Upgrade GUID (auto-generated if not specified, but should be stable)
    #[serde(default)]
    pub upgrade_code: Option<String>,

    /// Icon file path
    #[serde(default)]
    pub icon: Option<String>,

    /// License file path
    #[serde(default)]
    pub license: Option<String>,

    /// Install scope: per-user or per-machine
    #[serde(default = "default_scope")]
    pub scope: InstallScope,

    /// Target architecture
    #[serde(default = "default_arch")]
    pub architecture: Architecture,
}

fn default_scope() -> InstallScope {
    InstallScope::PerMachine
}

fn default_arch() -> Architecture {
    Architecture::X64
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum InstallScope {
    #[default]
    PerMachine,
    PerUser,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Architecture {
    X86,
    #[default]
    X64,
    Arm64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct InstallDef {
    /// Installation directory (e.g., "ProgramFiles/MyCompany/MyApp")
    #[serde(default = "default_directory")]
    pub directory: String,

    /// Files to install
    #[serde(default)]
    pub files: Vec<FileDef>,

    /// Directories to create (empty)
    #[serde(default)]
    pub directories: Vec<String>,
}

fn default_directory() -> String {
    "ProgramFiles/[Manufacturer]/[ProductName]".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileDef {
    /// Source file or glob pattern
    pub src: String,

    /// Destination subdirectory (relative to install dir)
    #[serde(default)]
    pub dest: String,

    /// Mark as vital (installation fails if missing)
    #[serde(default = "default_true")]
    pub vital: bool,

    /// File is the key path for the component
    #[serde(default)]
    pub key_path: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureDef {
    /// Feature ID
    pub id: String,

    /// Feature title (shown in UI)
    pub title: String,

    /// Feature description
    #[serde(default)]
    pub description: String,

    /// Install level (1 = default install, 0 = hidden)
    #[serde(default = "default_level")]
    pub level: i32,

    /// Files included in this feature (globs)
    #[serde(default)]
    pub files: Vec<String>,
}

fn default_level() -> i32 {
    1
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortcutDef {
    /// Shortcut name
    pub name: String,

    /// Target executable (relative to install dir)
    pub target: String,

    /// Shortcut location
    #[serde(default)]
    pub location: ShortcutLocation,

    /// Working directory (relative to install dir)
    #[serde(default)]
    pub working_dir: Option<String>,

    /// Command line arguments
    #[serde(default)]
    pub arguments: Option<String>,

    /// Icon file (relative to install dir)
    #[serde(default)]
    pub icon: Option<String>,

    /// Description/tooltip
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ShortcutLocation {
    #[default]
    StartMenu,
    Desktop,
    Both,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryDef {
    /// Registry key path (e.g., "HKCU/Software/MyCompany/MyApp")
    pub key: String,

    /// Registry values
    #[serde(default)]
    pub values: HashMap<String, RegistryValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RegistryValue {
    String(String),
    Dword(u32),
    Expand(ExpandString),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpandString {
    pub expand: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentDef {
    /// Environment variable name
    pub name: String,

    /// Value
    pub value: String,

    /// Action: set, append, prepend
    #[serde(default)]
    pub action: EnvAction,

    /// Scope: user or system
    #[serde(default)]
    pub scope: EnvScope,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum EnvAction {
    #[default]
    Set,
    Append,
    Prepend,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum EnvScope {
    #[default]
    User,
    System,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceDef {
    /// Service name
    pub name: String,

    /// Display name
    #[serde(default)]
    pub display_name: Option<String>,

    /// Service executable (relative to install dir)
    pub executable: String,

    /// Service description
    #[serde(default)]
    pub description: Option<String>,

    /// Start type
    #[serde(default)]
    pub start: ServiceStart,

    /// Service arguments
    #[serde(default)]
    pub arguments: Option<String>,

    /// Service account
    #[serde(default)]
    pub account: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ServiceStart {
    #[default]
    Auto,
    Manual,
    Disabled,
    Boot,
    System,
}

/// Prerequisite definition - supports multiple formats:
/// - Simple string: "dotnet:8.0"
/// - Map style: { dotnet: "8.0" }
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PrerequisiteDef {
    /// Map style: { dotnet: "8.0" }
    Map(HashMap<String, String>),
    /// Simple string: "dotnet:8.0"
    Simple(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UiDef {
    /// UI type: minimal, basic, full, or custom WXL reference
    #[serde(default)]
    pub style: UiStyle,

    /// Banner image (493x58)
    #[serde(default)]
    pub banner: Option<String>,

    /// Dialog background image (493x312)
    #[serde(default)]
    pub dialog: Option<String>,

    /// Custom EULA file
    #[serde(default)]
    pub eula: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum UiStyle {
    #[default]
    Minimal,
    Basic,
    Full,
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpgradeDef {
    /// Allow downgrade
    #[serde(default)]
    pub allow_downgrade: bool,

    /// Allow same version reinstall
    #[serde(default = "default_true")]
    pub allow_same_version: bool,

    /// Schedule: before or after InstallFiles
    #[serde(default)]
    pub schedule: UpgradeSchedule,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum UpgradeSchedule {
    #[default]
    Early,
    AfterInstallFiles,
}

impl InstallerDef {
    /// Parse from YAML string
    pub fn from_yaml(yaml: &str) -> Result<Self> {
        let def: InstallerDef = serde_yaml::from_str(yaml)?;
        def.validate()?;
        Ok(def)
    }

    /// Parse from YAML file
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        Self::from_yaml(&content)
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<()> {
        if self.package.name.is_empty() {
            return Err(EasyError::MissingField("package.name".into()));
        }
        if self.package.version.is_empty() {
            return Err(EasyError::MissingField("package.version".into()));
        }
        if self.package.manufacturer.is_empty() {
            return Err(EasyError::MissingField("package.manufacturer".into()));
        }
        Ok(())
    }

    /// Generate WiX XML
    pub fn generate_wix(&self, base_path: Option<&Path>) -> Result<String> {
        let generator = WixGenerator::new(self, base_path);
        generator.generate()
    }
}

/// WiX XML generator
struct WixGenerator<'a> {
    def: &'a InstallerDef,
    base_path: Option<&'a Path>,
    component_counter: std::cell::RefCell<u32>,
}

impl<'a> WixGenerator<'a> {
    fn new(def: &'a InstallerDef, base_path: Option<&'a Path>) -> Self {
        Self {
            def,
            base_path,
            component_counter: std::cell::RefCell::new(0),
        }
    }

    fn next_component_id(&self) -> String {
        let mut counter = self.component_counter.borrow_mut();
        *counter += 1;
        format!("Component_{}", counter)
    }

    fn generate(&self) -> Result<String> {
        let mut xml = String::new();

        xml.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
        xml.push_str("<!--\n");
        xml.push_str("  Generated by wix-easy\n");
        xml.push_str("  https://github.com/wixcraft/wixcraft\n");
        xml.push_str("-->\n");
        xml.push_str("<Wix xmlns=\"http://wixtoolset.org/schemas/v4/wxs\">\n\n");

        // Package element
        xml.push_str(&self.generate_package());
        xml.push_str("\n");

        // Directories
        xml.push_str(&self.generate_directories());

        // Components and Files
        xml.push_str(&self.generate_components());

        // Features
        xml.push_str(&self.generate_features());

        // UI
        xml.push_str(&self.generate_ui());

        xml.push_str("</Wix>\n");

        Ok(xml)
    }

    fn generate_package(&self) -> String {
        let pkg = &self.def.package;
        let upgrade_code = pkg.upgrade_code.clone()
            .unwrap_or_else(|| format!("{{{}}}", Uuid::new_v4().to_string().to_uppercase()));

        let scope = match pkg.scope {
            InstallScope::PerMachine => "perMachine",
            InstallScope::PerUser => "perUser",
        };

        let mut xml = String::new();
        xml.push_str(&format!("  <Package Name=\"{}\"\n", escape_xml(&pkg.name)));
        xml.push_str(&format!("           Manufacturer=\"{}\"\n", escape_xml(&pkg.manufacturer)));
        xml.push_str(&format!("           Version=\"{}\"\n", pkg.version));
        xml.push_str(&format!("           UpgradeCode=\"{}\"\n", upgrade_code));
        xml.push_str(&format!("           Scope=\"{}\">\n\n", scope));

        // MajorUpgrade
        let downgrade = if self.def.upgrade.allow_downgrade { "yes" } else { "no" };
        xml.push_str(&format!(
            "    <MajorUpgrade DowngradeErrorMessage=\"A newer version is already installed.\" AllowDowngrades=\"{}\" />\n\n",
            downgrade
        ));

        // Media
        xml.push_str("    <MediaTemplate EmbedCab=\"yes\" />\n\n");

        // Icon
        if let Some(ref icon) = pkg.icon {
            let icon_path = self.resolve_path(icon);
            xml.push_str(&format!("    <Icon Id=\"ProductIcon\" SourceFile=\"{}\" />\n", icon_path.display()));
            xml.push_str("    <Property Id=\"ARPPRODUCTICON\" Value=\"ProductIcon\" />\n\n");
        }

        xml
    }

    fn generate_directories(&self) -> String {
        let mut xml = String::new();

        // Parse the install directory path
        let dir_parts: Vec<&str> = self.def.install.directory
            .split('/')
            .filter(|s| !s.is_empty())
            .collect();

        xml.push_str("    <!-- Directory Structure -->\n");
        xml.push_str("    <StandardDirectory Id=\"ProgramFilesFolder\">\n");

        let mut indent = 6;
        for (i, part) in dir_parts.iter().skip(1).enumerate() {
            let resolved = self.resolve_placeholder(part);
            let id = if i == dir_parts.len() - 2 {
                "INSTALLFOLDER"
            } else {
                &format!("Dir_{}", i)
            };
            xml.push_str(&format!(
                "{}<Directory Id=\"{}\" Name=\"{}\">\n",
                " ".repeat(indent),
                id,
                escape_xml(&resolved)
            ));
            indent += 2;
        }

        // Close directories
        for _ in 0..dir_parts.len().saturating_sub(1) {
            indent -= 2;
            xml.push_str(&format!("{}</Directory>\n", " ".repeat(indent)));
        }

        xml.push_str("    </StandardDirectory>\n\n");

        // Shortcuts directories
        if !self.def.shortcuts.is_empty() {
            let has_desktop = self.def.shortcuts.iter().any(|s|
                matches!(s.location, ShortcutLocation::Desktop | ShortcutLocation::Both));
            let has_start = self.def.shortcuts.iter().any(|s|
                matches!(s.location, ShortcutLocation::StartMenu | ShortcutLocation::Both));

            if has_start {
                xml.push_str("    <StandardDirectory Id=\"ProgramMenuFolder\">\n");
                xml.push_str(&format!(
                    "      <Directory Id=\"ApplicationProgramsFolder\" Name=\"{}\" />\n",
                    escape_xml(&self.def.package.name)
                ));
                xml.push_str("    </StandardDirectory>\n\n");
            }
            if has_desktop {
                xml.push_str("    <StandardDirectory Id=\"DesktopFolder\" />\n\n");
            }
        }

        xml
    }

    fn generate_components(&self) -> String {
        let mut xml = String::new();
        xml.push_str("    <!-- Components -->\n");
        xml.push_str("    <ComponentGroup Id=\"ProductComponents\" Directory=\"INSTALLFOLDER\">\n");

        // Generate components for files
        for file in &self.def.install.files {
            let files = self.expand_glob(&file.src);
            for f in files {
                let comp_id = self.next_component_id();
                let file_name = f.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("file");
                let file_id = format!("File_{}", sanitize_id(file_name));

                xml.push_str(&format!("      <Component Id=\"{}\" Guid=\"*\">\n", comp_id));
                xml.push_str(&format!(
                    "        <File Id=\"{}\" Source=\"{}\" KeyPath=\"yes\" />\n",
                    file_id,
                    f.display()
                ));
                xml.push_str("      </Component>\n");
            }
        }

        // Shortcuts
        for shortcut in &self.def.shortcuts {
            let comp_id = self.next_component_id();
            let shortcut_id = format!("Shortcut_{}", sanitize_id(&shortcut.name));

            // Create shortcut component
            let dir = match shortcut.location {
                ShortcutLocation::Desktop => "DesktopFolder",
                ShortcutLocation::StartMenu | ShortcutLocation::Both => "ApplicationProgramsFolder",
            };

            xml.push_str(&format!(
                "      <Component Id=\"{}\" Guid=\"*\" Directory=\"{}\">\n",
                comp_id, dir
            ));
            xml.push_str(&format!(
                "        <Shortcut Id=\"{}\" Name=\"{}\" Target=\"[INSTALLFOLDER]{}\"",
                shortcut_id,
                escape_xml(&shortcut.name),
                escape_xml(&shortcut.target)
            ));

            if let Some(ref args) = shortcut.arguments {
                xml.push_str(&format!(" Arguments=\"{}\"", escape_xml(args)));
            }
            if let Some(ref desc) = shortcut.description {
                xml.push_str(&format!(" Description=\"{}\"", escape_xml(desc)));
            }
            xml.push_str(" WorkingDirectory=\"INSTALLFOLDER\" />\n");

            // Registry key for shortcut KeyPath
            xml.push_str(&format!(
                "        <RegistryValue Root=\"HKCU\" Key=\"Software\\{}\\{}\" Name=\"{}\" Type=\"integer\" Value=\"1\" KeyPath=\"yes\" />\n",
                escape_xml(&self.def.package.manufacturer),
                escape_xml(&self.def.package.name),
                escape_xml(&shortcut.name)
            ));

            xml.push_str("      </Component>\n");

            // If Both, add desktop shortcut too
            if matches!(shortcut.location, ShortcutLocation::Both) {
                let comp_id2 = self.next_component_id();
                let shortcut_id2 = format!("DesktopShortcut_{}", sanitize_id(&shortcut.name));

                xml.push_str(&format!(
                    "      <Component Id=\"{}\" Guid=\"*\" Directory=\"DesktopFolder\">\n",
                    comp_id2
                ));
                xml.push_str(&format!(
                    "        <Shortcut Id=\"{}\" Name=\"{}\" Target=\"[INSTALLFOLDER]{}\" WorkingDirectory=\"INSTALLFOLDER\" />\n",
                    shortcut_id2,
                    escape_xml(&shortcut.name),
                    escape_xml(&shortcut.target)
                ));
                xml.push_str(&format!(
                    "        <RegistryValue Root=\"HKCU\" Key=\"Software\\{}\\{}\" Name=\"Desktop_{}\" Type=\"integer\" Value=\"1\" KeyPath=\"yes\" />\n",
                    escape_xml(&self.def.package.manufacturer),
                    escape_xml(&self.def.package.name),
                    escape_xml(&shortcut.name)
                ));
                xml.push_str("      </Component>\n");
            }
        }

        // Registry entries
        for reg in &self.def.registry {
            let comp_id = self.next_component_id();
            let (root, key) = parse_registry_path(&reg.key);

            xml.push_str(&format!("      <Component Id=\"{}\" Guid=\"*\">\n", comp_id));

            for (name, value) in &reg.values {
                let (vtype, vstr) = match value {
                    RegistryValue::String(s) => ("string", s.clone()),
                    RegistryValue::Dword(d) => ("integer", d.to_string()),
                    RegistryValue::Expand(e) => ("expandable", e.expand.clone()),
                };

                if name.is_empty() || name == "@" {
                    xml.push_str(&format!(
                        "        <RegistryValue Root=\"{}\" Key=\"{}\" Type=\"{}\" Value=\"{}\" />\n",
                        root, escape_xml(&key), vtype, escape_xml(&vstr)
                    ));
                } else {
                    xml.push_str(&format!(
                        "        <RegistryValue Root=\"{}\" Key=\"{}\" Name=\"{}\" Type=\"{}\" Value=\"{}\" />\n",
                        root, escape_xml(&key), escape_xml(name), vtype, escape_xml(&vstr)
                    ));
                }
            }

            xml.push_str("      </Component>\n");
        }

        // Environment variables
        for env in &self.def.environment {
            let comp_id = self.next_component_id();
            let action = match env.action {
                EnvAction::Set => "set",
                EnvAction::Append => "set",
                EnvAction::Prepend => "set",
            };
            let part = match env.action {
                EnvAction::Set => "all",
                EnvAction::Append => "last",
                EnvAction::Prepend => "first",
            };
            let system = matches!(env.scope, EnvScope::System);

            xml.push_str(&format!("      <Component Id=\"{}\" Guid=\"*\">\n", comp_id));
            xml.push_str(&format!(
                "        <Environment Id=\"Env_{}\" Name=\"{}\" Value=\"{}\" Action=\"{}\" Part=\"{}\" System=\"{}\" />\n",
                sanitize_id(&env.name),
                escape_xml(&env.name),
                escape_xml(&env.value),
                action,
                part,
                if system { "yes" } else { "no" }
            ));
            xml.push_str("      </Component>\n");
        }

        // Services
        for service in &self.def.services {
            let comp_id = self.next_component_id();
            let display = service.display_name.as_ref().unwrap_or(&service.name);
            let start = match service.start {
                ServiceStart::Auto => "auto",
                ServiceStart::Manual => "demand",
                ServiceStart::Disabled => "disabled",
                ServiceStart::Boot => "boot",
                ServiceStart::System => "system",
            };

            xml.push_str(&format!("      <Component Id=\"{}\" Guid=\"*\">\n", comp_id));
            xml.push_str(&format!(
                "        <ServiceInstall Id=\"Service_{}\" Name=\"{}\" DisplayName=\"{}\" Start=\"{}\" Type=\"ownProcess\" ErrorControl=\"normal\"",
                sanitize_id(&service.name),
                escape_xml(&service.name),
                escape_xml(display),
                start
            ));

            if let Some(ref desc) = service.description {
                xml.push_str(&format!(" Description=\"{}\"", escape_xml(desc)));
            }
            if let Some(ref args) = service.arguments {
                xml.push_str(&format!(" Arguments=\"{}\"", escape_xml(args)));
            }
            if let Some(ref account) = service.account {
                xml.push_str(&format!(" Account=\"{}\"", escape_xml(account)));
            }

            xml.push_str(" />\n");
            xml.push_str(&format!(
                "        <ServiceControl Id=\"ServiceControl_{}\" Name=\"{}\" Start=\"install\" Stop=\"both\" Remove=\"uninstall\" Wait=\"yes\" />\n",
                sanitize_id(&service.name),
                escape_xml(&service.name)
            ));
            xml.push_str("      </Component>\n");
        }

        xml.push_str("    </ComponentGroup>\n\n");
        xml
    }

    fn generate_features(&self) -> String {
        let mut xml = String::new();

        if self.def.features.is_empty() {
            // Default single feature
            xml.push_str("    <!-- Features -->\n");
            xml.push_str(&format!(
                "    <Feature Id=\"MainFeature\" Title=\"{}\" Level=\"1\">\n",
                escape_xml(&self.def.package.name)
            ));
            xml.push_str("      <ComponentGroupRef Id=\"ProductComponents\" />\n");
            xml.push_str("    </Feature>\n\n");
        } else {
            xml.push_str("    <!-- Features -->\n");
            for feature in &self.def.features {
                xml.push_str(&format!(
                    "    <Feature Id=\"{}\" Title=\"{}\" Level=\"{}\"",
                    escape_xml(&feature.id),
                    escape_xml(&feature.title),
                    feature.level
                ));
                if !feature.description.is_empty() {
                    xml.push_str(&format!(" Description=\"{}\"", escape_xml(&feature.description)));
                }
                xml.push_str(">\n");
                xml.push_str("      <ComponentGroupRef Id=\"ProductComponents\" />\n");
                xml.push_str("    </Feature>\n");
            }
            xml.push('\n');
        }

        xml
    }

    fn generate_ui(&self) -> String {
        let mut xml = String::new();

        match self.def.ui.style {
            UiStyle::None => {}
            UiStyle::Minimal => {
                xml.push_str("    <!-- UI -->\n");
                xml.push_str("    <UI>\n");
                xml.push_str("      <UIRef Id=\"WixUI_Minimal\" />\n");
                xml.push_str("    </UI>\n\n");
            }
            UiStyle::Basic => {
                xml.push_str("    <!-- UI -->\n");
                xml.push_str("    <UI>\n");
                xml.push_str("      <UIRef Id=\"WixUI_InstallDir\" />\n");
                xml.push_str("      <Property Id=\"WIXUI_INSTALLDIR\" Value=\"INSTALLFOLDER\" />\n");
                xml.push_str("    </UI>\n\n");
            }
            UiStyle::Full => {
                xml.push_str("    <!-- UI -->\n");
                xml.push_str("    <UI>\n");
                xml.push_str("      <UIRef Id=\"WixUI_FeatureTree\" />\n");
                xml.push_str("    </UI>\n\n");
            }
        }

        // Close Package
        xml.push_str("  </Package>\n");

        xml
    }

    fn resolve_path(&self, path: &str) -> PathBuf {
        if let Some(base) = self.base_path {
            base.join(path)
        } else {
            PathBuf::from(path)
        }
    }

    fn resolve_placeholder(&self, s: &str) -> String {
        s.replace("[Manufacturer]", &self.def.package.manufacturer)
            .replace("[ProductName]", &self.def.package.name)
    }

    fn expand_glob(&self, pattern: &str) -> Vec<PathBuf> {
        let resolved = self.resolve_path(pattern);
        let pattern_str = resolved.to_string_lossy();

        if let Ok(paths) = glob::glob(&pattern_str) {
            paths.filter_map(|p| p.ok()).collect()
        } else {
            vec![resolved]
        }
    }
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

fn sanitize_id(s: &str) -> String {
    s.chars()
        .map(|c| if c.is_alphanumeric() || c == '_' { c } else { '_' })
        .collect()
}

fn parse_registry_path(path: &str) -> (&'static str, String) {
    let path = path.replace('\\', "/");
    let parts: Vec<&str> = path.splitn(2, '/').collect();

    let root = match parts.first().map(|s| s.to_uppercase()).as_deref() {
        Some("HKLM") | Some("HKEY_LOCAL_MACHINE") => "HKLM",
        Some("HKCU") | Some("HKEY_CURRENT_USER") => "HKCU",
        Some("HKCR") | Some("HKEY_CLASSES_ROOT") => "HKCR",
        Some("HKU") | Some("HKEY_USERS") => "HKU",
        _ => "HKCU",
    };

    let key = parts.get(1).unwrap_or(&"").to_string();
    (root, key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_yaml() {
        let yaml = r#"
package:
  name: MyApp
  version: 1.0.0
  manufacturer: My Company

install:
  directory: ProgramFiles/MyCompany/MyApp
  files:
    - src: ./bin/app.exe
"#;

        let def = InstallerDef::from_yaml(yaml).unwrap();
        assert_eq!(def.package.name, "MyApp");
        assert_eq!(def.package.version, "1.0.0");
        assert_eq!(def.package.manufacturer, "My Company");
    }

    #[test]
    fn test_parse_with_shortcuts() {
        let yaml = r#"
package:
  name: MyApp
  version: 1.0.0
  manufacturer: My Company

shortcuts:
  - name: MyApp
    target: app.exe
    location: both
"#;

        let def = InstallerDef::from_yaml(yaml).unwrap();
        assert_eq!(def.shortcuts.len(), 1);
        assert_eq!(def.shortcuts[0].name, "MyApp");
        assert!(matches!(def.shortcuts[0].location, ShortcutLocation::Both));
    }

    #[test]
    fn test_parse_with_registry() {
        let yaml = r#"
package:
  name: MyApp
  version: 1.0.0
  manufacturer: My Company

registry:
  - key: HKCU/Software/MyCompany/MyApp
    values:
      Version: "1.0.0"
      InstallPath: "[INSTALLFOLDER]"
"#;

        let def = InstallerDef::from_yaml(yaml).unwrap();
        assert_eq!(def.registry.len(), 1);
    }

    #[test]
    fn test_parse_with_services() {
        let yaml = r#"
package:
  name: MyApp
  version: 1.0.0
  manufacturer: My Company

services:
  - name: MyAppService
    executable: service.exe
    start: auto
    description: My application service
"#;

        let def = InstallerDef::from_yaml(yaml).unwrap();
        assert_eq!(def.services.len(), 1);
        assert_eq!(def.services[0].name, "MyAppService");
    }

    #[test]
    fn test_generate_wix() {
        let yaml = r#"
package:
  name: MyApp
  version: 1.0.0
  manufacturer: My Company

install:
  directory: ProgramFiles/MyCompany/MyApp
"#;

        let def = InstallerDef::from_yaml(yaml).unwrap();
        let wix = def.generate_wix(None).unwrap();

        assert!(wix.contains("<Wix"));
        assert!(wix.contains("<Package"));
        assert!(wix.contains("MyApp"));
        assert!(wix.contains("My Company"));
    }

    #[test]
    fn test_escape_xml() {
        assert_eq!(escape_xml("a < b"), "a &lt; b");
        assert_eq!(escape_xml("a & b"), "a &amp; b");
        assert_eq!(escape_xml("\"quoted\""), "&quot;quoted&quot;");
    }

    #[test]
    fn test_sanitize_id() {
        assert_eq!(sanitize_id("my-file.exe"), "my_file_exe");
        assert_eq!(sanitize_id("test 123"), "test_123");
    }

    #[test]
    fn test_parse_registry_path() {
        let (root, key) = parse_registry_path("HKLM/Software/MyApp");
        assert_eq!(root, "HKLM");
        assert_eq!(key, "Software/MyApp");

        let (root, key) = parse_registry_path("HKEY_CURRENT_USER\\Software\\Test");
        assert_eq!(root, "HKCU");
    }

    #[test]
    fn test_default_scope() {
        let yaml = r#"
package:
  name: MyApp
  version: 1.0.0
  manufacturer: My Company
"#;

        let def = InstallerDef::from_yaml(yaml).unwrap();
        assert!(matches!(def.package.scope, InstallScope::PerMachine));
    }

    #[test]
    fn test_per_user_scope() {
        let yaml = r#"
package:
  name: MyApp
  version: 1.0.0
  manufacturer: My Company
  scope: per-user
"#;

        let def = InstallerDef::from_yaml(yaml).unwrap();
        assert!(matches!(def.package.scope, InstallScope::PerUser));
    }

    #[test]
    fn test_missing_name_error() {
        let yaml = r#"
package:
  version: 1.0.0
  manufacturer: My Company
"#;

        let result = InstallerDef::from_yaml(yaml);
        assert!(result.is_err());
    }

    #[test]
    fn test_environment_variables() {
        let yaml = r#"
package:
  name: MyApp
  version: 1.0.0
  manufacturer: My Company

environment:
  - name: PATH
    value: "[INSTALLFOLDER]bin"
    action: append
    scope: user
"#;

        let def = InstallerDef::from_yaml(yaml).unwrap();
        assert_eq!(def.environment.len(), 1);
        assert_eq!(def.environment[0].name, "PATH");
        assert!(matches!(def.environment[0].action, EnvAction::Append));
    }

    #[test]
    fn test_prerequisites() {
        let yaml = r#"
package:
  name: MyApp
  version: 1.0.0
  manufacturer: My Company

prerequisites:
  - dotnet: "8.0"
  - vcredist: "2022"
"#;

        let def = InstallerDef::from_yaml(yaml).unwrap();
        assert_eq!(def.prerequisites.len(), 2);
    }
}

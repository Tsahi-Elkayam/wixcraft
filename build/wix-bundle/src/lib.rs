//! wix-bundle - Burn bootstrapper wizard for WiX installers
//!
//! Generates WiX Burn bootstrapper bundles with a fluent API.
//!
//! # Example
//!
//! ```
//! use wix_bundle::{Bundle, BundlePackage, BootstrapperUI};
//!
//! let bundle = Bundle::new("MySetup", "1.0.0")
//!     .manufacturer("My Company")
//!     .ui(BootstrapperUI::HyperlinkLicense)
//!     .package(BundlePackage::msi("MainProduct.msi").vital())
//!     .package(BundlePackage::exe("vcredist_x64.exe")
//!         .detect_condition("VCRUNTIME_X64")
//!         .install_args("/install /quiet /norestart"));
//!
//! let wxs = bundle.generate();
//! println!("{}", wxs);
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum BundleError {
    #[error("No packages defined in bundle")]
    NoPackages,

    #[error("Invalid package configuration: {0}")]
    InvalidPackage(String),

    #[error("Missing required field: {0}")]
    MissingField(String),
}

/// Bootstrapper UI type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BootstrapperUI {
    /// Standard UI with license agreement (hyperlink style)
    HyperlinkLicense,
    /// Standard UI with license agreement (sidebar style)
    HyperlinkSideLicense,
    /// Standard UI with license agreement (large dialog)
    HyperlinkLargeLicense,
    /// RtfLicense - displays RTF license text
    RtfLicense,
    /// RtfSideLicense - RTF license with sidebar
    RtfSideLicense,
    /// RtfLargeLicense - RTF license large
    RtfLargeLicense,
    /// No UI - silent installation
    None,
    /// Custom bootstrapper application
    Custom,
}

impl BootstrapperUI {
    pub fn as_theme(&self) -> Option<&'static str> {
        match self {
            BootstrapperUI::HyperlinkLicense => Some("WixStandardBootstrapperApplication.HyperlinkLicense"),
            BootstrapperUI::HyperlinkSideLicense => Some("WixStandardBootstrapperApplication.HyperlinkSideLicense"),
            BootstrapperUI::HyperlinkLargeLicense => Some("WixStandardBootstrapperApplication.HyperlinkLargeLicense"),
            BootstrapperUI::RtfLicense => Some("WixStandardBootstrapperApplication.RtfLicense"),
            BootstrapperUI::RtfSideLicense => Some("WixStandardBootstrapperApplication.RtfSideLicense"),
            BootstrapperUI::RtfLargeLicense => Some("WixStandardBootstrapperApplication.RtfLargeLicense"),
            BootstrapperUI::None => None,
            BootstrapperUI::Custom => None,
        }
    }
}

/// Package type in the bundle chain
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PackageType {
    Msi,
    Exe,
    Msp,
    Msu,
    Bundle,
}

impl PackageType {
    pub fn element_name(&self) -> &'static str {
        match self {
            PackageType::Msi => "MsiPackage",
            PackageType::Exe => "ExePackage",
            PackageType::Msp => "MspPackage",
            PackageType::Msu => "MsuPackage",
            PackageType::Bundle => "BundlePackage",
        }
    }
}

/// A package in the bundle chain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundlePackage {
    pub id: Option<String>,
    pub package_type: PackageType,
    pub source: String,
    pub name: Option<String>,
    pub download_url: Option<String>,
    pub install_condition: Option<String>,
    pub detect_condition: Option<String>,
    pub install_args: Option<String>,
    pub repair_args: Option<String>,
    pub uninstall_args: Option<String>,
    pub vital: bool,
    pub permanent: bool,
    pub visible: bool,
    pub cache: CacheType,
    pub compressed: Option<bool>,
    pub per_machine: Option<bool>,
}

impl BundlePackage {
    /// Create a new MSI package
    pub fn msi(source: impl Into<String>) -> Self {
        Self::new(PackageType::Msi, source)
    }

    /// Create a new EXE package
    pub fn exe(source: impl Into<String>) -> Self {
        Self::new(PackageType::Exe, source)
    }

    /// Create a new MSP (patch) package
    pub fn msp(source: impl Into<String>) -> Self {
        Self::new(PackageType::Msp, source)
    }

    /// Create a new MSU (Windows Update) package
    pub fn msu(source: impl Into<String>) -> Self {
        Self::new(PackageType::Msu, source)
    }

    /// Create a new nested bundle package
    pub fn bundle(source: impl Into<String>) -> Self {
        Self::new(PackageType::Bundle, source)
    }

    fn new(package_type: PackageType, source: impl Into<String>) -> Self {
        Self {
            id: None,
            package_type,
            source: source.into(),
            name: None,
            download_url: None,
            install_condition: None,
            detect_condition: None,
            install_args: None,
            repair_args: None,
            uninstall_args: None,
            vital: true,
            permanent: false,
            visible: true,
            cache: CacheType::Keep,
            compressed: None,
            per_machine: None,
        }
    }

    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn download_url(mut self, url: impl Into<String>) -> Self {
        self.download_url = Some(url.into());
        self
    }

    pub fn install_condition(mut self, condition: impl Into<String>) -> Self {
        self.install_condition = Some(condition.into());
        self
    }

    pub fn detect_condition(mut self, condition: impl Into<String>) -> Self {
        self.detect_condition = Some(condition.into());
        self
    }

    pub fn install_args(mut self, args: impl Into<String>) -> Self {
        self.install_args = Some(args.into());
        self
    }

    pub fn repair_args(mut self, args: impl Into<String>) -> Self {
        self.repair_args = Some(args.into());
        self
    }

    pub fn uninstall_args(mut self, args: impl Into<String>) -> Self {
        self.uninstall_args = Some(args.into());
        self
    }

    pub fn vital(mut self) -> Self {
        self.vital = true;
        self
    }

    pub fn non_vital(mut self) -> Self {
        self.vital = false;
        self
    }

    pub fn permanent(mut self) -> Self {
        self.permanent = true;
        self
    }

    pub fn hidden(mut self) -> Self {
        self.visible = false;
        self
    }

    pub fn cache(mut self, cache: CacheType) -> Self {
        self.cache = cache;
        self
    }

    pub fn compressed(mut self, compressed: bool) -> Self {
        self.compressed = Some(compressed);
        self
    }

    pub fn per_machine(mut self, per_machine: bool) -> Self {
        self.per_machine = Some(per_machine);
        self
    }

    /// Generate the package ID if not set
    pub fn get_id(&self) -> String {
        if let Some(ref id) = self.id {
            id.clone()
        } else {
            // Generate from source file name
            let name = self
                .source
                .split(['/', '\\'])
                .last()
                .unwrap_or(&self.source)
                .replace('.', "_")
                .replace('-', "_");
            format!("Pkg_{}", name)
        }
    }
}

/// Cache behavior for packages
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum CacheType {
    #[default]
    Keep,
    Remove,
    Force,
}

impl CacheType {
    pub fn as_str(&self) -> &'static str {
        match self {
            CacheType::Keep => "keep",
            CacheType::Remove => "remove",
            CacheType::Force => "force",
        }
    }
}

/// Bundle variable
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleVariable {
    pub name: String,
    pub value: Option<String>,
    pub variable_type: Option<VariableType>,
    pub persisted: bool,
    pub hidden: bool,
}

impl BundleVariable {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            value: None,
            variable_type: None,
            persisted: false,
            hidden: false,
        }
    }

    pub fn with_value(mut self, value: impl Into<String>) -> Self {
        self.value = Some(value.into());
        self
    }

    pub fn with_type(mut self, var_type: VariableType) -> Self {
        self.variable_type = Some(var_type);
        self
    }

    pub fn persisted(mut self) -> Self {
        self.persisted = true;
        self
    }

    pub fn hidden(mut self) -> Self {
        self.hidden = true;
        self
    }
}

/// Variable type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VariableType {
    String,
    Numeric,
    Version,
}

impl VariableType {
    pub fn as_str(&self) -> &'static str {
        match self {
            VariableType::String => "string",
            VariableType::Numeric => "numeric",
            VariableType::Version => "version",
        }
    }
}

/// Bundle configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bundle {
    pub name: String,
    pub version: String,
    pub manufacturer: Option<String>,
    pub upgrade_code: Option<String>,
    pub about_url: Option<String>,
    pub help_url: Option<String>,
    pub update_url: Option<String>,
    pub icon_source: Option<String>,
    pub splash_screen: Option<String>,
    pub license_file: Option<String>,
    pub ui: BootstrapperUI,
    pub packages: Vec<BundlePackage>,
    pub variables: Vec<BundleVariable>,
    pub package_groups: HashMap<String, Vec<BundlePackage>>,
    pub disable_modify: bool,
    pub disable_remove: bool,
    pub disable_repair: bool,
    pub per_machine: bool,
    pub condition: Option<String>,
}

impl Bundle {
    pub fn new(name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            manufacturer: None,
            upgrade_code: None,
            about_url: None,
            help_url: None,
            update_url: None,
            icon_source: None,
            splash_screen: None,
            license_file: None,
            ui: BootstrapperUI::HyperlinkLicense,
            packages: Vec::new(),
            variables: Vec::new(),
            package_groups: HashMap::new(),
            disable_modify: false,
            disable_remove: false,
            disable_repair: false,
            per_machine: true,
            condition: None,
        }
    }

    pub fn manufacturer(mut self, manufacturer: impl Into<String>) -> Self {
        self.manufacturer = Some(manufacturer.into());
        self
    }

    pub fn upgrade_code(mut self, code: impl Into<String>) -> Self {
        self.upgrade_code = Some(code.into());
        self
    }

    pub fn about_url(mut self, url: impl Into<String>) -> Self {
        self.about_url = Some(url.into());
        self
    }

    pub fn help_url(mut self, url: impl Into<String>) -> Self {
        self.help_url = Some(url.into());
        self
    }

    pub fn update_url(mut self, url: impl Into<String>) -> Self {
        self.update_url = Some(url.into());
        self
    }

    pub fn icon(mut self, path: impl Into<String>) -> Self {
        self.icon_source = Some(path.into());
        self
    }

    pub fn splash_screen(mut self, path: impl Into<String>) -> Self {
        self.splash_screen = Some(path.into());
        self
    }

    pub fn license_file(mut self, path: impl Into<String>) -> Self {
        self.license_file = Some(path.into());
        self
    }

    pub fn ui(mut self, ui: BootstrapperUI) -> Self {
        self.ui = ui;
        self
    }

    pub fn package(mut self, package: BundlePackage) -> Self {
        self.packages.push(package);
        self
    }

    pub fn packages<I>(mut self, packages: I) -> Self
    where
        I: IntoIterator<Item = BundlePackage>,
    {
        self.packages.extend(packages);
        self
    }

    pub fn variable(mut self, variable: BundleVariable) -> Self {
        self.variables.push(variable);
        self
    }

    pub fn package_group(mut self, name: impl Into<String>, packages: Vec<BundlePackage>) -> Self {
        self.package_groups.insert(name.into(), packages);
        self
    }

    pub fn disable_modify(mut self) -> Self {
        self.disable_modify = true;
        self
    }

    pub fn disable_remove(mut self) -> Self {
        self.disable_remove = true;
        self
    }

    pub fn disable_repair(mut self) -> Self {
        self.disable_repair = true;
        self
    }

    pub fn per_user(mut self) -> Self {
        self.per_machine = false;
        self
    }

    pub fn per_machine(mut self) -> Self {
        self.per_machine = true;
        self
    }

    pub fn condition(mut self, condition: impl Into<String>) -> Self {
        self.condition = Some(condition.into());
        self
    }

    /// Validate the bundle configuration
    pub fn validate(&self) -> Result<(), BundleError> {
        if self.packages.is_empty() && self.package_groups.is_empty() {
            return Err(BundleError::NoPackages);
        }

        for pkg in &self.packages {
            if pkg.source.is_empty() {
                return Err(BundleError::InvalidPackage(
                    "Package source cannot be empty".into(),
                ));
            }
        }

        Ok(())
    }

    /// Generate WiX bundle source
    pub fn generate(&self) -> String {
        let mut wxs = String::new();

        wxs.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
        wxs.push_str("<Wix xmlns=\"http://wixtoolset.org/schemas/v4/wxs\"\n");
        wxs.push_str("     xmlns:bal=\"http://wixtoolset.org/schemas/v4/wxs/bal\">\n\n");

        // Bundle element
        wxs.push_str("  <Bundle\n");
        wxs.push_str(&format!("    Name=\"{}\"\n", escape_xml(&self.name)));
        wxs.push_str(&format!("    Version=\"{}\"\n", escape_xml(&self.version)));

        if let Some(ref manufacturer) = self.manufacturer {
            wxs.push_str(&format!("    Manufacturer=\"{}\"\n", escape_xml(manufacturer)));
        }

        if let Some(ref code) = self.upgrade_code {
            wxs.push_str(&format!("    UpgradeCode=\"{}\"\n", escape_xml(code)));
        } else {
            wxs.push_str("    UpgradeCode=\"PUT-GUID-HERE\"\n");
        }

        if let Some(ref url) = self.about_url {
            wxs.push_str(&format!("    AboutUrl=\"{}\"\n", escape_xml(url)));
        }

        if let Some(ref url) = self.help_url {
            wxs.push_str(&format!("    HelpUrl=\"{}\"\n", escape_xml(url)));
        }

        if let Some(ref url) = self.update_url {
            wxs.push_str(&format!("    UpdateUrl=\"{}\"\n", escape_xml(url)));
        }

        if self.disable_modify {
            wxs.push_str("    DisableModify=\"yes\"\n");
        }

        if self.disable_remove {
            wxs.push_str("    DisableRemove=\"yes\"\n");
        }

        if self.disable_repair {
            wxs.push_str("    DisableRepair=\"yes\"\n");
        }

        wxs.push_str("  >\n\n");

        // Bootstrapper application
        wxs.push_str(&self.generate_bootstrapper_application());

        // Variables
        if !self.variables.is_empty() {
            wxs.push_str("    <!-- Variables -->\n");
            for var in &self.variables {
                wxs.push_str(&self.generate_variable(var));
            }
            wxs.push_str("\n");
        }

        // Condition
        if let Some(ref cond) = self.condition {
            wxs.push_str(&format!(
                "    <bal:Condition Message=\"This installation requires administrator privileges.\">\n\
                 \x20     {}\n\
                 \x20   </bal:Condition>\n\n",
                escape_xml(cond)
            ));
        }

        // Chain
        wxs.push_str("    <Chain>\n");

        // Package groups
        for (name, _) in &self.package_groups {
            wxs.push_str(&format!("      <PackageGroupRef Id=\"{}\" />\n", escape_xml(name)));
        }

        // Direct packages
        for pkg in &self.packages {
            wxs.push_str(&self.generate_package(pkg));
        }

        wxs.push_str("    </Chain>\n\n");

        wxs.push_str("  </Bundle>\n\n");

        // Package group fragments
        for (name, packages) in &self.package_groups {
            wxs.push_str(&format!(
                "  <Fragment>\n    <PackageGroup Id=\"{}\">\n",
                escape_xml(name)
            ));
            for pkg in packages {
                wxs.push_str(&self.generate_package(pkg));
            }
            wxs.push_str("    </PackageGroup>\n  </Fragment>\n\n");
        }

        wxs.push_str("</Wix>\n");

        wxs
    }

    fn generate_bootstrapper_application(&self) -> String {
        let mut app = String::new();
        app.push_str("    <BootstrapperApplication>\n");

        if let Some(theme) = self.ui.as_theme() {
            app.push_str(&format!(
                "      <bal:WixStandardBootstrapperApplication\n\
                 \x20       Theme=\"{}\"\n",
                theme.split('.').last().unwrap_or(theme)
            ));

            if let Some(ref license) = self.license_file {
                app.push_str(&format!(
                    "        LicenseFile=\"{}\"\n",
                    escape_xml(license)
                ));
            }

            if let Some(ref logo) = self.icon_source {
                app.push_str(&format!("        LogoFile=\"{}\"\n", escape_xml(logo)));
            }

            if let Some(ref splash) = self.splash_screen {
                app.push_str(&format!(
                    "        SplashScreenFile=\"{}\"\n",
                    escape_xml(splash)
                ));
            }

            app.push_str("      />\n");
        }

        app.push_str("    </BootstrapperApplication>\n\n");
        app
    }

    fn generate_variable(&self, var: &BundleVariable) -> String {
        let mut v = format!("    <Variable Name=\"{}\"", escape_xml(&var.name));

        if let Some(ref value) = var.value {
            v.push_str(&format!(" Value=\"{}\"", escape_xml(value)));
        }

        if let Some(ref var_type) = var.variable_type {
            v.push_str(&format!(" Type=\"{}\"", var_type.as_str()));
        }

        if var.persisted {
            v.push_str(" Persisted=\"yes\"");
        }

        if var.hidden {
            v.push_str(" Hidden=\"yes\"");
        }

        v.push_str(" />\n");
        v
    }

    fn generate_package(&self, pkg: &BundlePackage) -> String {
        let mut p = String::new();
        let element = pkg.package_type.element_name();
        let id = pkg.get_id();

        p.push_str(&format!("      <{}\n", element));
        p.push_str(&format!("        Id=\"{}\"\n", escape_xml(&id)));
        p.push_str(&format!("        SourceFile=\"{}\"\n", escape_xml(&pkg.source)));

        if let Some(ref name) = pkg.name {
            p.push_str(&format!("        Name=\"{}\"\n", escape_xml(name)));
        }

        if let Some(ref url) = pkg.download_url {
            p.push_str(&format!("        DownloadUrl=\"{}\"\n", escape_xml(url)));
        }

        if let Some(ref cond) = pkg.install_condition {
            p.push_str(&format!("        InstallCondition=\"{}\"\n", escape_xml(cond)));
        }

        if let Some(ref cond) = pkg.detect_condition {
            p.push_str(&format!("        DetectCondition=\"{}\"\n", escape_xml(cond)));
        }

        // EXE-specific attributes
        if pkg.package_type == PackageType::Exe {
            if let Some(ref args) = pkg.install_args {
                p.push_str(&format!(
                    "        InstallArguments=\"{}\"\n",
                    escape_xml(args)
                ));
            }
            if let Some(ref args) = pkg.repair_args {
                p.push_str(&format!(
                    "        RepairArguments=\"{}\"\n",
                    escape_xml(args)
                ));
            }
            if let Some(ref args) = pkg.uninstall_args {
                p.push_str(&format!(
                    "        UninstallArguments=\"{}\"\n",
                    escape_xml(args)
                ));
            }
        }

        p.push_str(&format!("        Vital=\"{}\"\n", if pkg.vital { "yes" } else { "no" }));

        if pkg.permanent {
            p.push_str("        Permanent=\"yes\"\n");
        }

        if !pkg.visible {
            p.push_str("        Visible=\"no\"\n");
        }

        p.push_str(&format!("        Cache=\"{}\"\n", pkg.cache.as_str()));

        if let Some(compressed) = pkg.compressed {
            p.push_str(&format!(
                "        Compressed=\"{}\"\n",
                if compressed { "yes" } else { "no" }
            ));
        }

        if let Some(per_machine) = pkg.per_machine {
            p.push_str(&format!(
                "        PerMachine=\"{}\"\n",
                if per_machine { "yes" } else { "no" }
            ));
        }

        p.push_str("      />\n");
        p
    }
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

/// Common bundle templates
pub struct BundleTemplates;

impl BundleTemplates {
    /// Create a simple setup bundle with one MSI
    pub fn simple_setup(name: &str, version: &str, msi_path: &str) -> Bundle {
        Bundle::new(name, version)
            .ui(BootstrapperUI::HyperlinkLicense)
            .package(BundlePackage::msi(msi_path).vital())
    }

    /// Create a bundle with .NET prerequisite
    pub fn with_dotnet(name: &str, version: &str, msi_path: &str, dotnet_version: &str) -> Bundle {
        Bundle::new(name, version)
            .ui(BootstrapperUI::HyperlinkLicense)
            .package(
                BundlePackage::exe(format!("dotnet-runtime-{}-win-x64.exe", dotnet_version))
                    .name(format!(".NET {} Runtime", dotnet_version))
                    .detect_condition(format!("NETCORERUNTIME{}", dotnet_version.split('.').next().unwrap_or("8")))
                    .install_args("/install /quiet /norestart")
                    .repair_args("/repair /quiet /norestart")
                    .uninstall_args("/uninstall /quiet /norestart"),
            )
            .package(BundlePackage::msi(msi_path).vital())
    }

    /// Create a bundle with VC++ redistributable
    pub fn with_vcredist(name: &str, version: &str, msi_path: &str) -> Bundle {
        Bundle::new(name, version)
            .ui(BootstrapperUI::HyperlinkLicense)
            .package(
                BundlePackage::exe("vc_redist.x64.exe")
                    .name("Visual C++ Redistributable")
                    .detect_condition("VCRUNTIME_X64")
                    .install_args("/install /quiet /norestart")
                    .repair_args("/repair /quiet /norestart")
                    .uninstall_args("/uninstall /quiet /norestart"),
            )
            .package(BundlePackage::msi(msi_path).vital())
    }

    /// Create a bundle with multiple prerequisites
    pub fn with_prerequisites(
        name: &str,
        version: &str,
        msi_path: &str,
        prereqs: Vec<BundlePackage>,
    ) -> Bundle {
        let mut bundle = Bundle::new(name, version).ui(BootstrapperUI::HyperlinkLicense);

        for prereq in prereqs {
            bundle = bundle.package(prereq);
        }

        bundle.package(BundlePackage::msi(msi_path).vital())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bundle_basic() {
        let bundle = Bundle::new("MySetup", "1.0.0")
            .manufacturer("My Company")
            .package(BundlePackage::msi("Product.msi"));

        let wxs = bundle.generate();
        assert!(wxs.contains("MySetup"));
        assert!(wxs.contains("1.0.0"));
        assert!(wxs.contains("My Company"));
        assert!(wxs.contains("Product.msi"));
    }

    #[test]
    fn test_bundle_with_exe_package() {
        let bundle = Bundle::new("Setup", "2.0.0").package(
            BundlePackage::exe("prereq.exe")
                .install_args("/install /quiet")
                .detect_condition("PREREQ_INSTALLED"),
        );

        let wxs = bundle.generate();
        assert!(wxs.contains("ExePackage"));
        assert!(wxs.contains("prereq.exe"));
        assert!(wxs.contains("/install /quiet"));
        assert!(wxs.contains("PREREQ_INSTALLED"));
    }

    #[test]
    fn test_bundle_ui_themes() {
        let bundle = Bundle::new("Setup", "1.0.0")
            .ui(BootstrapperUI::HyperlinkLargeLicense)
            .package(BundlePackage::msi("test.msi"));

        let wxs = bundle.generate();
        assert!(wxs.contains("HyperlinkLargeLicense"));
    }

    #[test]
    fn test_bundle_variables() {
        let bundle = Bundle::new("Setup", "1.0.0")
            .variable(
                BundleVariable::new("InstallPath")
                    .with_value("[ProgramFilesFolder]MyApp")
                    .persisted(),
            )
            .package(BundlePackage::msi("test.msi"));

        let wxs = bundle.generate();
        assert!(wxs.contains("Variable"));
        assert!(wxs.contains("InstallPath"));
        assert!(wxs.contains("Persisted=\"yes\""));
    }

    #[test]
    fn test_package_types() {
        assert_eq!(PackageType::Msi.element_name(), "MsiPackage");
        assert_eq!(PackageType::Exe.element_name(), "ExePackage");
        assert_eq!(PackageType::Msp.element_name(), "MspPackage");
        assert_eq!(PackageType::Msu.element_name(), "MsuPackage");
        assert_eq!(PackageType::Bundle.element_name(), "BundlePackage");
    }

    #[test]
    fn test_cache_types() {
        assert_eq!(CacheType::Keep.as_str(), "keep");
        assert_eq!(CacheType::Remove.as_str(), "remove");
        assert_eq!(CacheType::Force.as_str(), "force");
    }

    #[test]
    fn test_package_id_generation() {
        let pkg = BundlePackage::msi("Product.msi");
        assert_eq!(pkg.get_id(), "Pkg_Product_msi");

        let pkg = BundlePackage::exe("path/to/prereq-v1.0.exe");
        assert_eq!(pkg.get_id(), "Pkg_prereq_v1_0_exe");

        let pkg = BundlePackage::msi("Product.msi").id("CustomId");
        assert_eq!(pkg.get_id(), "CustomId");
    }

    #[test]
    fn test_bundle_validation() {
        let bundle = Bundle::new("Empty", "1.0.0");
        assert!(bundle.validate().is_err());

        let bundle = Bundle::new("WithPackage", "1.0.0").package(BundlePackage::msi("test.msi"));
        assert!(bundle.validate().is_ok());
    }

    #[test]
    fn test_package_options() {
        let pkg = BundlePackage::msi("test.msi")
            .vital()
            .permanent()
            .hidden()
            .cache(CacheType::Force)
            .compressed(true)
            .per_machine(true);

        assert!(pkg.vital);
        assert!(pkg.permanent);
        assert!(!pkg.visible);
        assert_eq!(pkg.cache, CacheType::Force);
        assert_eq!(pkg.compressed, Some(true));
        assert_eq!(pkg.per_machine, Some(true));
    }

    #[test]
    fn test_bundle_templates_simple() {
        let bundle = BundleTemplates::simple_setup("MyApp", "1.0.0", "MyApp.msi");

        assert_eq!(bundle.name, "MyApp");
        assert_eq!(bundle.packages.len(), 1);
        assert!(bundle.packages[0].vital);
    }

    #[test]
    fn test_bundle_templates_with_dotnet() {
        let bundle = BundleTemplates::with_dotnet("MyApp", "1.0.0", "MyApp.msi", "8.0");

        assert_eq!(bundle.packages.len(), 2);
        assert!(bundle.packages[0].source.contains("dotnet"));
        assert!(bundle.packages[1].source.contains("MyApp.msi"));
    }

    #[test]
    fn test_bundle_templates_with_vcredist() {
        let bundle = BundleTemplates::with_vcredist("MyApp", "1.0.0", "MyApp.msi");

        assert_eq!(bundle.packages.len(), 2);
        assert!(bundle.packages[0].source.contains("vc_redist"));
    }

    #[test]
    fn test_bundle_disable_options() {
        let bundle = Bundle::new("Setup", "1.0.0")
            .disable_modify()
            .disable_remove()
            .disable_repair()
            .package(BundlePackage::msi("test.msi"));

        let wxs = bundle.generate();
        assert!(wxs.contains("DisableModify=\"yes\""));
        assert!(wxs.contains("DisableRemove=\"yes\""));
        assert!(wxs.contains("DisableRepair=\"yes\""));
    }

    #[test]
    fn test_bundle_urls() {
        let bundle = Bundle::new("Setup", "1.0.0")
            .about_url("https://example.com/about")
            .help_url("https://example.com/help")
            .update_url("https://example.com/update")
            .package(BundlePackage::msi("test.msi"));

        let wxs = bundle.generate();
        assert!(wxs.contains("AboutUrl=\"https://example.com/about\""));
        assert!(wxs.contains("HelpUrl=\"https://example.com/help\""));
        assert!(wxs.contains("UpdateUrl=\"https://example.com/update\""));
    }

    #[test]
    fn test_bundle_package_groups() {
        let bundle = Bundle::new("Setup", "1.0.0")
            .package_group(
                "Prerequisites",
                vec![
                    BundlePackage::exe("prereq1.exe"),
                    BundlePackage::exe("prereq2.exe"),
                ],
            )
            .package(BundlePackage::msi("main.msi"));

        let wxs = bundle.generate();
        assert!(wxs.contains("PackageGroupRef Id=\"Prerequisites\""));
        assert!(wxs.contains("<PackageGroup Id=\"Prerequisites\">"));
    }

    #[test]
    fn test_variable_types() {
        assert_eq!(VariableType::String.as_str(), "string");
        assert_eq!(VariableType::Numeric.as_str(), "numeric");
        assert_eq!(VariableType::Version.as_str(), "version");
    }

    #[test]
    fn test_bootstrapper_ui_themes() {
        assert!(BootstrapperUI::HyperlinkLicense.as_theme().is_some());
        assert!(BootstrapperUI::RtfLicense.as_theme().is_some());
        assert!(BootstrapperUI::None.as_theme().is_none());
        assert!(BootstrapperUI::Custom.as_theme().is_none());
    }

    #[test]
    fn test_escape_xml() {
        assert_eq!(escape_xml("a < b"), "a &lt; b");
        assert_eq!(escape_xml("a & b"), "a &amp; b");
        assert_eq!(escape_xml("\"quoted\""), "&quot;quoted&quot;");
    }

    #[test]
    fn test_bundle_condition() {
        let bundle = Bundle::new("Setup", "1.0.0")
            .condition("VersionNT >= v6.1")
            .package(BundlePackage::msi("test.msi"));

        let wxs = bundle.generate();
        assert!(wxs.contains("bal:Condition"));
        assert!(wxs.contains("VersionNT"));
    }

    #[test]
    fn test_msp_package() {
        let pkg = BundlePackage::msp("patch.msp");
        assert_eq!(pkg.package_type, PackageType::Msp);
        assert_eq!(pkg.package_type.element_name(), "MspPackage");
    }

    #[test]
    fn test_msu_package() {
        let pkg = BundlePackage::msu("update.msu");
        assert_eq!(pkg.package_type, PackageType::Msu);
        assert_eq!(pkg.package_type.element_name(), "MsuPackage");
    }

    #[test]
    fn test_nested_bundle() {
        let pkg = BundlePackage::bundle("nested.exe");
        assert_eq!(pkg.package_type, PackageType::Bundle);
        assert_eq!(pkg.package_type.element_name(), "BundlePackage");
    }
}

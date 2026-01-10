//! wix-patch - Simplified patch/MSP generation for WiX installers
//!
//! Generates WiX patch source files for creating MSP patches.
//!
//! # Example
//!
//! ```
//! use wix_patch::{Patch, PatchFamily, PatchClassification};
//!
//! let patch = Patch::new("MyPatch", "1.0.1")
//!     .manufacturer("My Company")
//!     .classification(PatchClassification::Update)
//!     .target_product_code("{12345678-1234-1234-1234-123456789012}")
//!     .family(PatchFamily::new("MyPatchFamily")
//!         .supersede("1.0.0"));
//!
//! let wxs = patch.generate();
//! println!("{}", wxs);
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PatchError {
    #[error("No target product specified")]
    NoTarget,

    #[error("Missing required field: {0}")]
    MissingField(String),

    #[error("Invalid version format: {0}")]
    InvalidVersion(String),
}

/// Patch classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PatchClassification {
    /// Critical update - security or stability fix
    Critical,
    /// Hotfix - targeted fix for specific issue
    Hotfix,
    /// Security update
    Security,
    /// General update
    Update,
    /// Service pack
    ServicePack,
    /// Upgrade (minor version)
    Upgrade,
}

impl PatchClassification {
    pub fn as_str(&self) -> &'static str {
        match self {
            PatchClassification::Critical => "Critical",
            PatchClassification::Hotfix => "Hotfix",
            PatchClassification::Security => "Security",
            PatchClassification::Update => "Update",
            PatchClassification::ServicePack => "ServicePack",
            PatchClassification::Upgrade => "Upgrade",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            PatchClassification::Critical => "A critical update that must be applied",
            PatchClassification::Hotfix => "A targeted fix for a specific issue",
            PatchClassification::Security => "A security update that addresses vulnerabilities",
            PatchClassification::Update => "A general update with improvements",
            PatchClassification::ServicePack => "A collection of updates and fixes",
            PatchClassification::Upgrade => "An upgrade to a newer minor version",
        }
    }
}

/// Patch family for grouping related patches
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchFamily {
    pub name: String,
    pub version: Option<String>,
    pub superseded_versions: Vec<String>,
}

impl PatchFamily {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: None,
            superseded_versions: Vec::new(),
        }
    }

    pub fn version(mut self, version: impl Into<String>) -> Self {
        self.version = Some(version.into());
        self
    }

    pub fn supersede(mut self, version: impl Into<String>) -> Self {
        self.superseded_versions.push(version.into());
        self
    }

    pub fn supersede_all<I, S>(mut self, versions: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        for v in versions {
            self.superseded_versions.push(v.into());
        }
        self
    }
}

/// Target product definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchTarget {
    pub product_code: Option<String>,
    pub upgrade_code: Option<String>,
    pub min_version: Option<String>,
    pub max_version: Option<String>,
}

impl PatchTarget {
    pub fn new() -> Self {
        Self {
            product_code: None,
            upgrade_code: None,
            min_version: None,
            max_version: None,
        }
    }

    pub fn product_code(mut self, code: impl Into<String>) -> Self {
        self.product_code = Some(code.into());
        self
    }

    pub fn upgrade_code(mut self, code: impl Into<String>) -> Self {
        self.upgrade_code = Some(code.into());
        self
    }

    pub fn version_range(mut self, min: impl Into<String>, max: impl Into<String>) -> Self {
        self.min_version = Some(min.into());
        self.max_version = Some(max.into());
        self
    }
}

impl Default for PatchTarget {
    fn default() -> Self {
        Self::new()
    }
}

/// Patch sequence information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchSequence {
    pub family: String,
    pub sequence: u32,
    pub supersede: bool,
}

impl PatchSequence {
    pub fn new(family: impl Into<String>, sequence: u32) -> Self {
        Self {
            family: family.into(),
            sequence,
            supersede: true,
        }
    }

    pub fn no_supersede(mut self) -> Self {
        self.supersede = false;
        self
    }
}

/// Patch creation mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum PatchCreationMode {
    /// WiX default patching (transforms)
    #[default]
    Transform,
    /// Admin image patching
    AdminImage,
}

impl PatchCreationMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            PatchCreationMode::Transform => "transform",
            PatchCreationMode::AdminImage => "admin",
        }
    }
}

/// A file to include in the patch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchFile {
    pub source: String,
    pub target: String,
    pub component_id: Option<String>,
}

impl PatchFile {
    pub fn new(source: impl Into<String>, target: impl Into<String>) -> Self {
        Self {
            source: source.into(),
            target: target.into(),
            component_id: None,
        }
    }

    pub fn component(mut self, id: impl Into<String>) -> Self {
        self.component_id = Some(id.into());
        self
    }
}

/// Patch configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Patch {
    pub name: String,
    pub version: String,
    pub manufacturer: Option<String>,
    pub description: Option<String>,
    pub classification: PatchClassification,
    pub display_name: Option<String>,
    pub more_info_url: Option<String>,
    pub targets: Vec<PatchTarget>,
    pub families: Vec<PatchFamily>,
    pub sequences: Vec<PatchSequence>,
    pub files: Vec<PatchFile>,
    pub baseline_msi: Option<String>,
    pub updated_msi: Option<String>,
    pub creation_mode: PatchCreationMode,
    pub allow_remove: bool,
    pub optimize_size: bool,
    pub include_wholes_files: bool,
}

impl Patch {
    pub fn new(name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            manufacturer: None,
            description: None,
            classification: PatchClassification::Update,
            display_name: None,
            more_info_url: None,
            targets: Vec::new(),
            families: Vec::new(),
            sequences: Vec::new(),
            files: Vec::new(),
            baseline_msi: None,
            updated_msi: None,
            creation_mode: PatchCreationMode::Transform,
            allow_remove: true,
            optimize_size: true,
            include_wholes_files: false,
        }
    }

    pub fn manufacturer(mut self, manufacturer: impl Into<String>) -> Self {
        self.manufacturer = Some(manufacturer.into());
        self
    }

    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn classification(mut self, classification: PatchClassification) -> Self {
        self.classification = classification;
        self
    }

    pub fn display_name(mut self, name: impl Into<String>) -> Self {
        self.display_name = Some(name.into());
        self
    }

    pub fn more_info_url(mut self, url: impl Into<String>) -> Self {
        self.more_info_url = Some(url.into());
        self
    }

    pub fn target(mut self, target: PatchTarget) -> Self {
        self.targets.push(target);
        self
    }

    pub fn target_product_code(mut self, code: impl Into<String>) -> Self {
        self.targets.push(PatchTarget::new().product_code(code));
        self
    }

    pub fn target_upgrade_code(mut self, code: impl Into<String>) -> Self {
        self.targets.push(PatchTarget::new().upgrade_code(code));
        self
    }

    pub fn family(mut self, family: PatchFamily) -> Self {
        self.families.push(family);
        self
    }

    pub fn sequence(mut self, sequence: PatchSequence) -> Self {
        self.sequences.push(sequence);
        self
    }

    pub fn file(mut self, file: PatchFile) -> Self {
        self.files.push(file);
        self
    }

    pub fn baseline_msi(mut self, path: impl Into<String>) -> Self {
        self.baseline_msi = Some(path.into());
        self
    }

    pub fn updated_msi(mut self, path: impl Into<String>) -> Self {
        self.updated_msi = Some(path.into());
        self
    }

    pub fn creation_mode(mut self, mode: PatchCreationMode) -> Self {
        self.creation_mode = mode;
        self
    }

    pub fn allow_remove(mut self, allow: bool) -> Self {
        self.allow_remove = allow;
        self
    }

    pub fn optimize_size(mut self, optimize: bool) -> Self {
        self.optimize_size = optimize;
        self
    }

    pub fn include_whole_files(mut self, include: bool) -> Self {
        self.include_wholes_files = include;
        self
    }

    /// Validate the patch configuration
    pub fn validate(&self) -> Result<(), PatchError> {
        if self.targets.is_empty() {
            return Err(PatchError::NoTarget);
        }

        if self.families.is_empty() {
            return Err(PatchError::MissingField("patch family".into()));
        }

        // Validate version format
        let parts: Vec<&str> = self.version.split('.').collect();
        if parts.len() < 2 || parts.len() > 4 {
            return Err(PatchError::InvalidVersion(self.version.clone()));
        }

        for part in parts {
            if part.parse::<u32>().is_err() {
                return Err(PatchError::InvalidVersion(self.version.clone()));
            }
        }

        Ok(())
    }

    /// Generate WiX patch source
    pub fn generate(&self) -> String {
        let mut wxs = String::new();

        wxs.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
        wxs.push_str("<Wix xmlns=\"http://wixtoolset.org/schemas/v4/wxs\">\n\n");

        // Patch element
        wxs.push_str("  <Patch\n");
        wxs.push_str(&format!(
            "    Id=\"*\"\n"
        ));
        wxs.push_str(&format!(
            "    Classification=\"{}\"\n",
            self.classification.as_str()
        ));

        if let Some(ref desc) = self.description {
            wxs.push_str(&format!(
                "    Description=\"{}\"\n",
                escape_xml(desc)
            ));
        } else {
            wxs.push_str(&format!(
                "    Description=\"{} {} patch\"\n",
                escape_xml(&self.name),
                self.version
            ));
        }

        if let Some(ref display) = self.display_name {
            wxs.push_str(&format!(
                "    DisplayName=\"{}\"\n",
                escape_xml(display)
            ));
        } else {
            wxs.push_str(&format!(
                "    DisplayName=\"{} {}\"\n",
                escape_xml(&self.name),
                self.version
            ));
        }

        if let Some(ref manufacturer) = self.manufacturer {
            wxs.push_str(&format!(
                "    Manufacturer=\"{}\"\n",
                escape_xml(manufacturer)
            ));
        }

        if let Some(ref url) = self.more_info_url {
            wxs.push_str(&format!(
                "    MoreInfoURL=\"{}\"\n",
                escape_xml(url)
            ));
        }

        wxs.push_str(&format!(
            "    AllowRemoval=\"{}\"\n",
            if self.allow_remove { "yes" } else { "no" }
        ));

        if self.optimize_size {
            wxs.push_str("    OptimizePatchSizeForLargeFiles=\"yes\"\n");
        }

        wxs.push_str("  >\n\n");

        // Media element
        wxs.push_str("    <Media Id=\"1\" Cabinet=\"patch.cab\">\n");

        // Patch baselines
        if let (Some(ref baseline), Some(ref updated)) = (&self.baseline_msi, &self.updated_msi) {
            wxs.push_str(&format!(
                "      <PatchBaseline Id=\"Baseline\">\n\
                 \x20       <AdminImage SourceFile=\"{}\"\n\
                 \x20                   TargetFile=\"{}\" />\n\
                 \x20     </PatchBaseline>\n",
                escape_xml(baseline),
                escape_xml(updated)
            ));
        }

        wxs.push_str("    </Media>\n\n");

        // Patch families
        for family in &self.families {
            wxs.push_str(&format!(
                "    <PatchFamily Id=\"{}\"",
                escape_xml(&family.name)
            ));

            if let Some(ref version) = family.version {
                wxs.push_str(&format!(" Version=\"{}\"", version));
            }

            if family.superseded_versions.is_empty() {
                wxs.push_str(" />\n");
            } else {
                wxs.push_str(">\n");
                for sup in &family.superseded_versions {
                    wxs.push_str(&format!(
                        "      <Supersede Id=\"{}\" />\n",
                        escape_xml(sup)
                    ));
                }
                wxs.push_str("    </PatchFamily>\n");
            }
        }
        wxs.push_str("\n");

        // Target products
        for target in &self.targets {
            if let Some(ref code) = target.product_code {
                wxs.push_str(&format!(
                    "    <TargetProductCode Id=\"{}\"",
                    escape_xml(code)
                ));
                if let (Some(ref min), Some(ref max)) = (&target.min_version, &target.max_version) {
                    wxs.push_str(&format!(" MinVersion=\"{}\" MaxVersion=\"{}\"", min, max));
                }
                wxs.push_str(" />\n");
            }
            if let Some(ref code) = target.upgrade_code {
                wxs.push_str(&format!(
                    "    <TargetProductCodes UpgradeCode=\"{}\" />\n",
                    escape_xml(code)
                ));
            }
        }
        wxs.push_str("\n");

        // Patch sequences
        for seq in &self.sequences {
            wxs.push_str(&format!(
                "    <PatchSequence\n\
                 \x20     Family=\"{}\"\n\
                 \x20     Sequence=\"{}\"\n\
                 \x20     Supersede=\"{}\" />\n",
                escape_xml(&seq.family),
                seq.sequence,
                if seq.supersede { "yes" } else { "no" }
            ));
        }

        wxs.push_str("\n  </Patch>\n\n");

        // Component changes fragment
        if !self.files.is_empty() {
            wxs.push_str("  <!-- Component changes to be included in the patch -->\n");
            wxs.push_str("  <Fragment>\n");
            wxs.push_str("    <ComponentGroup Id=\"PatchComponents\">\n");

            let mut seen_components = HashSet::new();
            for file in &self.files {
                if let Some(ref comp_id) = file.component_id {
                    if seen_components.insert(comp_id.clone()) {
                        wxs.push_str(&format!(
                            "      <ComponentRef Id=\"{}\" />\n",
                            escape_xml(comp_id)
                        ));
                    }
                }
            }

            wxs.push_str("    </ComponentGroup>\n");
            wxs.push_str("  </Fragment>\n\n");
        }

        wxs.push_str("</Wix>\n");

        wxs
    }

    /// Generate a patch creation commands script
    pub fn generate_build_script(&self) -> String {
        let mut script = String::new();

        script.push_str("@echo off\n");
        script.push_str("REM Patch build script\n");
        script.push_str(&format!("REM Patch: {} {}\n\n", self.name, self.version));

        script.push_str("REM Variables\n");
        if let Some(ref baseline) = self.baseline_msi {
            script.push_str(&format!("set BASELINE_MSI={}\n", baseline));
        }
        if let Some(ref updated) = self.updated_msi {
            script.push_str(&format!("set UPDATED_MSI={}\n", updated));
        }
        script.push_str(&format!(
            "set PATCH_FILE={}-{}.msp\n\n",
            self.name.replace(' ', "_"),
            self.version
        ));

        script.push_str("REM Build the patch\n");
        script.push_str("wix build -pdbtype none patch.wxs -o %PATCH_FILE%\n\n");

        script.push_str("echo Patch created: %PATCH_FILE%\n");

        script
    }

    /// Generate a PowerShell patch creation script
    pub fn generate_powershell_script(&self) -> String {
        let mut script = String::new();

        script.push_str("# Patch build script\n");
        script.push_str(&format!("# Patch: {} {}\n\n", self.name, self.version));

        script.push_str("# Variables\n");
        if let Some(ref baseline) = self.baseline_msi {
            script.push_str(&format!("$BaselineMsi = \"{}\"\n", baseline));
        }
        if let Some(ref updated) = self.updated_msi {
            script.push_str(&format!("$UpdatedMsi = \"{}\"\n", updated));
        }
        script.push_str(&format!(
            "$PatchFile = \"{}-{}.msp\"\n\n",
            self.name.replace(' ', "_"),
            self.version
        ));

        script.push_str("# Build the patch\n");
        script.push_str("Write-Host \"Building patch...\"\n");
        script.push_str("wix build -pdbtype none patch.wxs -o $PatchFile\n\n");

        script.push_str("if ($LASTEXITCODE -eq 0) {\n");
        script.push_str("    Write-Host \"Patch created: $PatchFile\" -ForegroundColor Green\n");
        script.push_str("} else {\n");
        script.push_str("    Write-Host \"Patch build failed\" -ForegroundColor Red\n");
        script.push_str("    exit 1\n");
        script.push_str("}\n");

        script
    }
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

/// Common patch templates
pub struct PatchTemplates;

impl PatchTemplates {
    /// Create a simple hotfix patch
    pub fn hotfix(name: &str, version: &str, target_product: &str) -> Patch {
        Patch::new(name, version)
            .classification(PatchClassification::Hotfix)
            .target_product_code(target_product)
            .family(PatchFamily::new(format!("{}Family", name)))
    }

    /// Create a security update
    pub fn security_update(name: &str, version: &str, target_product: &str) -> Patch {
        Patch::new(name, version)
            .classification(PatchClassification::Security)
            .target_product_code(target_product)
            .family(PatchFamily::new(format!("{}SecurityFamily", name)))
            .allow_remove(false)
    }

    /// Create a cumulative update
    pub fn cumulative_update(
        name: &str,
        version: &str,
        target_product: &str,
        superseded: Vec<&str>,
    ) -> Patch {
        let family = PatchFamily::new(format!("{}Family", name))
            .supersede_all(superseded.iter().map(|s| s.to_string()));

        Patch::new(name, version)
            .classification(PatchClassification::Update)
            .target_product_code(target_product)
            .family(family)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_patch_basic() {
        let patch = Patch::new("TestPatch", "1.0.1")
            .manufacturer("Test Company")
            .target_product_code("{12345}")
            .family(PatchFamily::new("TestFamily"));

        let wxs = patch.generate();
        assert!(wxs.contains("TestPatch"));
        assert!(wxs.contains("1.0.1"));
        assert!(wxs.contains("Test Company"));
        assert!(wxs.contains("{12345}"));
        assert!(wxs.contains("TestFamily"));
    }

    #[test]
    fn test_patch_classification() {
        assert_eq!(PatchClassification::Critical.as_str(), "Critical");
        assert_eq!(PatchClassification::Hotfix.as_str(), "Hotfix");
        assert_eq!(PatchClassification::Security.as_str(), "Security");
        assert_eq!(PatchClassification::Update.as_str(), "Update");
        assert_eq!(PatchClassification::ServicePack.as_str(), "ServicePack");
        assert_eq!(PatchClassification::Upgrade.as_str(), "Upgrade");
    }

    #[test]
    fn test_patch_classification_in_output() {
        let patch = Patch::new("Test", "1.0.0")
            .classification(PatchClassification::Security)
            .target_product_code("{123}")
            .family(PatchFamily::new("TestFamily"));

        let wxs = patch.generate();
        assert!(wxs.contains("Classification=\"Security\""));
    }

    #[test]
    fn test_patch_family() {
        let family = PatchFamily::new("MyFamily")
            .version("1.0.0")
            .supersede("0.9.0")
            .supersede("0.8.0");

        assert_eq!(family.name, "MyFamily");
        assert_eq!(family.version, Some("1.0.0".to_string()));
        assert_eq!(family.superseded_versions.len(), 2);
    }

    #[test]
    fn test_patch_family_in_output() {
        let patch = Patch::new("Test", "1.0.0")
            .target_product_code("{123}")
            .family(
                PatchFamily::new("MainFamily")
                    .version("1.0.0")
                    .supersede("0.9.0"),
            );

        let wxs = patch.generate();
        assert!(wxs.contains("PatchFamily Id=\"MainFamily\""));
        assert!(wxs.contains("Supersede Id=\"0.9.0\""));
    }

    #[test]
    fn test_patch_target() {
        let target = PatchTarget::new()
            .product_code("{ABC}")
            .version_range("1.0.0", "2.0.0");

        assert_eq!(target.product_code, Some("{ABC}".to_string()));
        assert_eq!(target.min_version, Some("1.0.0".to_string()));
        assert_eq!(target.max_version, Some("2.0.0".to_string()));
    }

    #[test]
    fn test_patch_sequence() {
        let seq = PatchSequence::new("Family", 100);
        assert_eq!(seq.family, "Family");
        assert_eq!(seq.sequence, 100);
        assert!(seq.supersede);

        let seq = PatchSequence::new("Family", 100).no_supersede();
        assert!(!seq.supersede);
    }

    #[test]
    fn test_patch_validation() {
        let patch = Patch::new("Test", "1.0.0");
        assert!(patch.validate().is_err());

        let patch = Patch::new("Test", "1.0.0").target_product_code("{123}");
        assert!(patch.validate().is_err()); // Still missing family

        let patch = Patch::new("Test", "1.0.0")
            .target_product_code("{123}")
            .family(PatchFamily::new("Family"));
        assert!(patch.validate().is_ok());
    }

    #[test]
    fn test_patch_validation_version() {
        let patch = Patch::new("Test", "invalid")
            .target_product_code("{123}")
            .family(PatchFamily::new("Family"));
        assert!(patch.validate().is_err());

        let patch = Patch::new("Test", "1.0.0.0")
            .target_product_code("{123}")
            .family(PatchFamily::new("Family"));
        assert!(patch.validate().is_ok());
    }

    #[test]
    fn test_patch_files() {
        let file = PatchFile::new("new.dll", "INSTALLDIR/app.dll").component("MainComponent");

        assert_eq!(file.source, "new.dll");
        assert_eq!(file.target, "INSTALLDIR/app.dll");
        assert_eq!(file.component_id, Some("MainComponent".to_string()));
    }

    #[test]
    fn test_patch_with_files() {
        let patch = Patch::new("Test", "1.0.0")
            .target_product_code("{123}")
            .family(PatchFamily::new("Family"))
            .file(PatchFile::new("new.dll", "old.dll").component("Comp1"));

        let wxs = patch.generate();
        assert!(wxs.contains("ComponentRef Id=\"Comp1\""));
    }

    #[test]
    fn test_patch_msi_references() {
        let patch = Patch::new("Test", "1.0.0")
            .target_product_code("{123}")
            .family(PatchFamily::new("Family"))
            .baseline_msi("old/product.msi")
            .updated_msi("new/product.msi");

        let wxs = patch.generate();
        assert!(wxs.contains("old/product.msi"));
        assert!(wxs.contains("new/product.msi"));
    }

    #[test]
    fn test_patch_options() {
        let patch = Patch::new("Test", "1.0.0")
            .target_product_code("{123}")
            .family(PatchFamily::new("Family"))
            .allow_remove(false)
            .optimize_size(true);

        let wxs = patch.generate();
        assert!(wxs.contains("AllowRemoval=\"no\""));
        assert!(wxs.contains("OptimizePatchSizeForLargeFiles=\"yes\""));
    }

    #[test]
    fn test_patch_templates_hotfix() {
        let patch = PatchTemplates::hotfix("Fix1", "1.0.1", "{PRODUCT}");

        assert_eq!(patch.classification, PatchClassification::Hotfix);
        assert_eq!(patch.targets.len(), 1);
    }

    #[test]
    fn test_patch_templates_security() {
        let patch = PatchTemplates::security_update("SecFix", "1.0.1", "{PRODUCT}");

        assert_eq!(patch.classification, PatchClassification::Security);
        assert!(!patch.allow_remove);
    }

    #[test]
    fn test_patch_templates_cumulative() {
        let patch = PatchTemplates::cumulative_update(
            "CU1",
            "2.0.0",
            "{PRODUCT}",
            vec!["1.0.0", "1.0.1", "1.1.0"],
        );

        assert_eq!(patch.classification, PatchClassification::Update);
        assert_eq!(patch.families[0].superseded_versions.len(), 3);
    }

    #[test]
    fn test_build_script() {
        let patch = Patch::new("Test", "1.0.0")
            .baseline_msi("old.msi")
            .updated_msi("new.msi")
            .target_product_code("{123}")
            .family(PatchFamily::new("Family"));

        let script = patch.generate_build_script();
        assert!(script.contains("BASELINE_MSI=old.msi"));
        assert!(script.contains("UPDATED_MSI=new.msi"));
        assert!(script.contains("wix build"));
    }

    #[test]
    fn test_powershell_script() {
        let patch = Patch::new("Test", "1.0.0")
            .baseline_msi("old.msi")
            .updated_msi("new.msi")
            .target_product_code("{123}")
            .family(PatchFamily::new("Family"));

        let script = patch.generate_powershell_script();
        assert!(script.contains("$BaselineMsi"));
        assert!(script.contains("$UpdatedMsi"));
        assert!(script.contains("wix build"));
    }

    #[test]
    fn test_escape_xml() {
        assert_eq!(escape_xml("a < b"), "a &lt; b");
        assert_eq!(escape_xml("a & b"), "a &amp; b");
        assert_eq!(escape_xml("\"quoted\""), "&quot;quoted&quot;");
    }

    #[test]
    fn test_creation_mode() {
        assert_eq!(PatchCreationMode::Transform.as_str(), "transform");
        assert_eq!(PatchCreationMode::AdminImage.as_str(), "admin");
    }

    #[test]
    fn test_classification_description() {
        assert!(!PatchClassification::Critical.description().is_empty());
        assert!(!PatchClassification::Security.description().is_empty());
    }

    #[test]
    fn test_upgrade_code_target() {
        let patch = Patch::new("Test", "1.0.0")
            .target_upgrade_code("{UPGRADE-CODE}")
            .family(PatchFamily::new("Family"));

        let wxs = patch.generate();
        assert!(wxs.contains("UpgradeCode=\"{UPGRADE-CODE}\""));
    }

    #[test]
    fn test_more_info_url() {
        let patch = Patch::new("Test", "1.0.0")
            .target_product_code("{123}")
            .family(PatchFamily::new("Family"))
            .more_info_url("https://example.com/patch-info");

        let wxs = patch.generate();
        assert!(wxs.contains("MoreInfoURL=\"https://example.com/patch-info\""));
    }
}

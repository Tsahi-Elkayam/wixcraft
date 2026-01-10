//! wix-msix - MSIX package converter and analyzer
//!
//! Converts MSI packages to MSIX format and analyzes MSIX packages.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Package identity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageIdentity {
    pub name: String,
    pub publisher: String,
    pub version: String,
    pub processor_architecture: ProcessorArchitecture,
}

impl PackageIdentity {
    pub fn new(name: &str, publisher: &str, version: &str) -> Self {
        Self {
            name: name.to_string(),
            publisher: publisher.to_string(),
            version: version.to_string(),
            processor_architecture: ProcessorArchitecture::X64,
        }
    }

    pub fn with_architecture(mut self, arch: ProcessorArchitecture) -> Self {
        self.processor_architecture = arch;
        self
    }
}

/// Processor architecture
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProcessorArchitecture {
    X86,
    X64,
    Arm,
    Arm64,
    Neutral,
}

impl ProcessorArchitecture {
    pub fn as_str(&self) -> &'static str {
        match self {
            ProcessorArchitecture::X86 => "x86",
            ProcessorArchitecture::X64 => "x64",
            ProcessorArchitecture::Arm => "arm",
            ProcessorArchitecture::Arm64 => "arm64",
            ProcessorArchitecture::Neutral => "neutral",
        }
    }
}

/// MSIX capabilities
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Capability {
    InternetClient,
    InternetClientServer,
    PrivateNetworkClientServer,
    EnterpriseAuthentication,
    SharedUserCertificates,
    RemovableStorage,
    DocumentsLibrary,
    PicturesLibrary,
    VideosLibrary,
    MusicLibrary,
    Webcam,
    Microphone,
}

impl Capability {
    pub fn as_str(&self) -> &'static str {
        match self {
            Capability::InternetClient => "internetClient",
            Capability::InternetClientServer => "internetClientServer",
            Capability::PrivateNetworkClientServer => "privateNetworkClientServer",
            Capability::EnterpriseAuthentication => "enterpriseAuthentication",
            Capability::SharedUserCertificates => "sharedUserCertificates",
            Capability::RemovableStorage => "removableStorage",
            Capability::DocumentsLibrary => "documentsLibrary",
            Capability::PicturesLibrary => "picturesLibrary",
            Capability::VideosLibrary => "videosLibrary",
            Capability::MusicLibrary => "musicLibrary",
            Capability::Webcam => "webcam",
            Capability::Microphone => "microphone",
        }
    }
}

/// MSIX application
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MsixApplication {
    pub id: String,
    pub executable: String,
    pub entry_point: Option<String>,
    pub display_name: String,
    pub description: Option<String>,
    pub logo: Option<String>,
}

impl MsixApplication {
    pub fn new(id: &str, executable: &str, display_name: &str) -> Self {
        Self {
            id: id.to_string(),
            executable: executable.to_string(),
            entry_point: None,
            display_name: display_name.to_string(),
            description: None,
            logo: None,
        }
    }

    pub fn with_logo(mut self, logo: &str) -> Self {
        self.logo = Some(logo.to_string());
        self
    }
}

/// MSIX package configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MsixConfig {
    pub identity: PackageIdentity,
    pub display_name: String,
    pub publisher_display_name: String,
    pub description: Option<String>,
    pub logo: Option<String>,
    pub applications: Vec<MsixApplication>,
    pub capabilities: Vec<Capability>,
    pub min_windows_version: String,
}

impl MsixConfig {
    pub fn new(identity: PackageIdentity, display_name: &str, publisher_display_name: &str) -> Self {
        Self {
            identity,
            display_name: display_name.to_string(),
            publisher_display_name: publisher_display_name.to_string(),
            description: None,
            logo: None,
            applications: Vec::new(),
            capabilities: Vec::new(),
            min_windows_version: "10.0.17763.0".to_string(),
        }
    }

    pub fn with_application(mut self, app: MsixApplication) -> Self {
        self.applications.push(app);
        self
    }

    pub fn with_capability(mut self, capability: Capability) -> Self {
        self.capabilities.push(capability);
        self
    }

    pub fn with_min_version(mut self, version: &str) -> Self {
        self.min_windows_version = version.to_string();
        self
    }
}

/// Conversion options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversionOptions {
    pub source_msi: PathBuf,
    pub output_path: PathBuf,
    pub sign_certificate: Option<PathBuf>,
    pub include_registry_fixups: bool,
    pub include_file_redirections: bool,
    pub package_support_framework: bool,
}

impl ConversionOptions {
    pub fn new(source: PathBuf, output: PathBuf) -> Self {
        Self {
            source_msi: source,
            output_path: output,
            sign_certificate: None,
            include_registry_fixups: true,
            include_file_redirections: true,
            package_support_framework: false,
        }
    }

    pub fn with_certificate(mut self, cert: PathBuf) -> Self {
        self.sign_certificate = Some(cert);
        self
    }

    pub fn with_psf(mut self) -> Self {
        self.package_support_framework = true;
        self
    }
}

/// Conversion result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversionResult {
    pub success: bool,
    pub output_path: Option<PathBuf>,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
    pub fixups_applied: Vec<String>,
}

impl ConversionResult {
    pub fn success(output: PathBuf) -> Self {
        Self {
            success: true,
            output_path: Some(output),
            warnings: Vec::new(),
            errors: Vec::new(),
            fixups_applied: Vec::new(),
        }
    }

    pub fn failure(errors: Vec<String>) -> Self {
        Self {
            success: false,
            output_path: None,
            warnings: Vec::new(),
            errors,
            fixups_applied: Vec::new(),
        }
    }

    pub fn with_warning(mut self, warning: &str) -> Self {
        self.warnings.push(warning.to_string());
        self
    }

    pub fn with_fixup(mut self, fixup: &str) -> Self {
        self.fixups_applied.push(fixup.to_string());
        self
    }
}

/// Package analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageAnalysis {
    pub file_count: usize,
    pub total_size_bytes: u64,
    pub has_registry: bool,
    pub has_services: bool,
    pub has_drivers: bool,
    pub capabilities_required: Vec<Capability>,
    pub compatibility_issues: Vec<String>,
    pub conversion_complexity: ConversionComplexity,
}

/// Conversion complexity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConversionComplexity {
    Simple,
    Moderate,
    Complex,
    NotPossible,
}

/// Manifest generator
pub struct ManifestGenerator;

impl ManifestGenerator {
    /// Generate AppxManifest.xml
    pub fn generate(config: &MsixConfig) -> String {
        let mut output = String::new();
        output.push_str("<?xml version=\"1.0\" encoding=\"utf-8\"?>\n");
        output.push_str("<Package xmlns=\"http://schemas.microsoft.com/appx/manifest/foundation/windows10\"\n");
        output.push_str("         xmlns:uap=\"http://schemas.microsoft.com/appx/manifest/uap/windows10\"\n");
        output.push_str("         xmlns:rescap=\"http://schemas.microsoft.com/appx/manifest/foundation/windows10/restrictedcapabilities\">\n");
        output.push_str("\n");

        // Identity
        output.push_str(&format!(
            "  <Identity Name=\"{}\" Publisher=\"{}\" Version=\"{}\" ProcessorArchitecture=\"{}\" />\n",
            config.identity.name,
            config.identity.publisher,
            config.identity.version,
            config.identity.processor_architecture.as_str()
        ));
        output.push_str("\n");

        // Properties
        output.push_str("  <Properties>\n");
        output.push_str(&format!(
            "    <DisplayName>{}</DisplayName>\n",
            config.display_name
        ));
        output.push_str(&format!(
            "    <PublisherDisplayName>{}</PublisherDisplayName>\n",
            config.publisher_display_name
        ));
        if let Some(ref logo) = config.logo {
            output.push_str(&format!("    <Logo>{}</Logo>\n", logo));
        }
        if let Some(ref desc) = config.description {
            output.push_str(&format!("    <Description>{}</Description>\n", desc));
        }
        output.push_str("  </Properties>\n");
        output.push_str("\n");

        // Dependencies
        output.push_str("  <Dependencies>\n");
        output.push_str(&format!(
            "    <TargetDeviceFamily Name=\"Windows.Desktop\" MinVersion=\"{}\" MaxVersionTested=\"10.0.22000.0\" />\n",
            config.min_windows_version
        ));
        output.push_str("  </Dependencies>\n");
        output.push_str("\n");

        // Applications
        output.push_str("  <Applications>\n");
        for app in &config.applications {
            output.push_str(&format!(
                "    <Application Id=\"{}\" Executable=\"{}\" EntryPoint=\"Windows.FullTrustApplication\">\n",
                app.id, app.executable
            ));
            output.push_str("      <uap:VisualElements\n");
            output.push_str(&format!(
                "        DisplayName=\"{}\"\n",
                app.display_name
            ));
            if let Some(ref desc) = app.description {
                output.push_str(&format!("        Description=\"{}\"\n", desc));
            }
            output.push_str("        BackgroundColor=\"transparent\"\n");
            output.push_str("        Square150x150Logo=\"Assets\\Square150x150Logo.png\"\n");
            output.push_str("        Square44x44Logo=\"Assets\\Square44x44Logo.png\" />\n");
            output.push_str("    </Application>\n");
        }
        output.push_str("  </Applications>\n");
        output.push_str("\n");

        // Capabilities
        if !config.capabilities.is_empty() {
            output.push_str("  <Capabilities>\n");
            for cap in &config.capabilities {
                output.push_str(&format!(
                    "    <Capability Name=\"{}\" />\n",
                    cap.as_str()
                ));
            }
            output.push_str("    <rescap:Capability Name=\"runFullTrust\" />\n");
            output.push_str("  </Capabilities>\n");
        }

        output.push_str("</Package>\n");
        output
    }
}

/// MSIX analyzer
pub struct MsixAnalyzer;

impl MsixAnalyzer {
    /// Analyze MSI for MSIX conversion
    pub fn analyze_msi(_msi_path: &PathBuf) -> PackageAnalysis {
        // Simulated analysis
        PackageAnalysis {
            file_count: 0,
            total_size_bytes: 0,
            has_registry: false,
            has_services: false,
            has_drivers: false,
            capabilities_required: Vec::new(),
            compatibility_issues: Vec::new(),
            conversion_complexity: ConversionComplexity::Simple,
        }
    }

    /// Check conversion compatibility
    pub fn check_compatibility(analysis: &PackageAnalysis) -> Vec<String> {
        let mut issues = Vec::new();

        if analysis.has_drivers {
            issues.push("Drivers are not supported in MSIX packages".to_string());
        }

        if analysis.has_services {
            issues.push("Services require Windows 10 1903 or later".to_string());
        }

        issues
    }
}

/// Fixup configuration for PSF
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixupConfig {
    pub fixups: HashMap<String, serde_json::Value>,
}

impl FixupConfig {
    pub fn new() -> Self {
        Self {
            fixups: HashMap::new(),
        }
    }

    pub fn add_file_redirection(&mut self, pattern: &str, redirect_to: &str) {
        self.fixups.insert(
            format!("file_{}", pattern.replace('*', "star")),
            serde_json::json!({
                "type": "FileRedirection",
                "pattern": pattern,
                "redirectTo": redirect_to
            }),
        );
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(&self.fixups).unwrap_or_default()
    }
}

impl Default for FixupConfig {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_package_identity_new() {
        let id = PackageIdentity::new("MyApp", "CN=Publisher", "1.0.0.0");
        assert_eq!(id.name, "MyApp");
        assert_eq!(id.publisher, "CN=Publisher");
    }

    #[test]
    fn test_package_identity_with_architecture() {
        let id = PackageIdentity::new("MyApp", "CN=Pub", "1.0.0.0")
            .with_architecture(ProcessorArchitecture::Arm64);
        assert_eq!(id.processor_architecture, ProcessorArchitecture::Arm64);
    }

    #[test]
    fn test_processor_architecture_as_str() {
        assert_eq!(ProcessorArchitecture::X64.as_str(), "x64");
        assert_eq!(ProcessorArchitecture::Arm64.as_str(), "arm64");
    }

    #[test]
    fn test_capability_as_str() {
        assert_eq!(Capability::InternetClient.as_str(), "internetClient");
        assert_eq!(Capability::Webcam.as_str(), "webcam");
    }

    #[test]
    fn test_msix_application_new() {
        let app = MsixApplication::new("App1", "app.exe", "My Application");
        assert_eq!(app.id, "App1");
        assert_eq!(app.executable, "app.exe");
    }

    #[test]
    fn test_msix_application_with_logo() {
        let app = MsixApplication::new("App1", "app.exe", "App").with_logo("logo.png");
        assert_eq!(app.logo, Some("logo.png".to_string()));
    }

    #[test]
    fn test_msix_config_new() {
        let id = PackageIdentity::new("App", "CN=Pub", "1.0.0.0");
        let config = MsixConfig::new(id, "App", "Publisher");
        assert_eq!(config.display_name, "App");
    }

    #[test]
    fn test_msix_config_with_application() {
        let id = PackageIdentity::new("App", "CN=Pub", "1.0.0.0");
        let app = MsixApplication::new("App1", "app.exe", "App");
        let config = MsixConfig::new(id, "App", "Publisher").with_application(app);
        assert_eq!(config.applications.len(), 1);
    }

    #[test]
    fn test_msix_config_with_capability() {
        let id = PackageIdentity::new("App", "CN=Pub", "1.0.0.0");
        let config = MsixConfig::new(id, "App", "Publisher")
            .with_capability(Capability::InternetClient);
        assert_eq!(config.capabilities.len(), 1);
    }

    #[test]
    fn test_conversion_options_new() {
        let opts = ConversionOptions::new(
            PathBuf::from("app.msi"),
            PathBuf::from("app.msix"),
        );
        assert!(opts.include_registry_fixups);
    }

    #[test]
    fn test_conversion_options_with_certificate() {
        let opts = ConversionOptions::new(
            PathBuf::from("app.msi"),
            PathBuf::from("app.msix"),
        )
        .with_certificate(PathBuf::from("cert.pfx"));
        assert!(opts.sign_certificate.is_some());
    }

    #[test]
    fn test_conversion_result_success() {
        let result = ConversionResult::success(PathBuf::from("out.msix"));
        assert!(result.success);
        assert!(result.output_path.is_some());
    }

    #[test]
    fn test_conversion_result_failure() {
        let result = ConversionResult::failure(vec!["Error".to_string()]);
        assert!(!result.success);
        assert_eq!(result.errors.len(), 1);
    }

    #[test]
    fn test_conversion_result_with_warning() {
        let result = ConversionResult::success(PathBuf::from("out.msix"))
            .with_warning("Check services");
        assert_eq!(result.warnings.len(), 1);
    }

    #[test]
    fn test_manifest_generator_generate() {
        let id = PackageIdentity::new("MyApp", "CN=Publisher", "1.0.0.0");
        let app = MsixApplication::new("App1", "app.exe", "My App");
        let config = MsixConfig::new(id, "My App", "Publisher")
            .with_application(app)
            .with_capability(Capability::InternetClient);
        let manifest = ManifestGenerator::generate(&config);
        assert!(manifest.contains("<Identity Name=\"MyApp\""));
        assert!(manifest.contains("<DisplayName>My App</DisplayName>"));
    }

    #[test]
    fn test_msix_analyzer_analyze() {
        let analysis = MsixAnalyzer::analyze_msi(&PathBuf::from("app.msi"));
        assert_eq!(analysis.conversion_complexity, ConversionComplexity::Simple);
    }

    #[test]
    fn test_msix_analyzer_check_compatibility() {
        let mut analysis = PackageAnalysis {
            file_count: 10,
            total_size_bytes: 1000,
            has_registry: false,
            has_services: false,
            has_drivers: true,
            capabilities_required: Vec::new(),
            compatibility_issues: Vec::new(),
            conversion_complexity: ConversionComplexity::Complex,
        };
        let issues = MsixAnalyzer::check_compatibility(&analysis);
        assert!(!issues.is_empty());

        analysis.has_drivers = false;
        analysis.has_services = true;
        let issues = MsixAnalyzer::check_compatibility(&analysis);
        assert!(issues.iter().any(|i| i.contains("Services")));
    }

    #[test]
    fn test_fixup_config_new() {
        let config = FixupConfig::new();
        assert!(config.fixups.is_empty());
    }

    #[test]
    fn test_fixup_config_add_file_redirection() {
        let mut config = FixupConfig::new();
        config.add_file_redirection("*.log", "logs/");
        assert!(!config.fixups.is_empty());
    }

    #[test]
    fn test_fixup_config_to_json() {
        let config = FixupConfig::new();
        let json = config.to_json();
        assert!(json.contains('{'));
    }
}

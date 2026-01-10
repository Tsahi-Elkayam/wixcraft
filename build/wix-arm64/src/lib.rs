//! wix-arm64 - ARM64 support helper for WiX multi-platform builds
//!
//! Provides:
//! - ARM64 compatibility analysis
//! - Multi-platform build configuration
//! - Platform-specific variable handling
//! - Custom action compatibility checks

use serde::{Deserialize, Serialize};

/// Target platform
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Platform {
    X86,
    X64,
    Arm64,
}

impl Platform {
    pub fn as_str(&self) -> &'static str {
        match self {
            Platform::X86 => "x86",
            Platform::X64 => "x64",
            Platform::Arm64 => "arm64",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "x86" | "win32" | "intel" => Some(Platform::X86),
            "x64" | "amd64" | "intel64" => Some(Platform::X64),
            "arm64" | "aarch64" => Some(Platform::Arm64),
            _ => None,
        }
    }

    pub fn program_files_var(&self) -> &'static str {
        match self {
            Platform::X86 => "ProgramFilesFolder",
            Platform::X64 => "ProgramFiles64Folder",
            Platform::Arm64 => "ProgramFiles64Folder",
        }
    }

    pub fn system_folder_var(&self) -> &'static str {
        match self {
            Platform::X86 => "SystemFolder",
            Platform::X64 => "System64Folder",
            Platform::Arm64 => "System64Folder",
        }
    }
}

/// ARM64 compatibility issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Arm64Issue {
    pub element: Option<String>,
    pub message: String,
    pub severity: IssueSeverity,
    pub suggestion: String,
}

/// Issue severity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IssueSeverity {
    Error,
    Warning,
    Info,
}

/// ARM64 analysis result
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Arm64Analysis {
    pub current_platform: Option<String>,
    pub issues: Vec<Arm64Issue>,
    pub has_native_custom_actions: bool,
    pub has_driver_install: bool,
    pub has_32bit_components: bool,
    pub suggested_platforms: Vec<String>,
}

/// Multi-platform build configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiPlatformConfig {
    pub platforms: Vec<Platform>,
    pub common_source: String,
    pub platform_specific: Vec<PlatformOverride>,
}

/// Platform-specific override
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformOverride {
    pub platform: Platform,
    pub files: Vec<String>,
    pub variables: Vec<(String, String)>,
}

/// ARM64 analyzer
pub struct Arm64Analyzer;

impl Arm64Analyzer {
    /// Analyze WiX source for ARM64 compatibility
    pub fn analyze(content: &str) -> Arm64Analysis {
        let mut analysis = Arm64Analysis::default();

        if let Ok(doc) = roxmltree::Document::parse(content) {
            for node in doc.descendants() {
                match node.tag_name().name() {
                    "Package" | "Product" => {
                        if let Some(platform) = node.attribute("Platform") {
                            analysis.current_platform = Some(platform.to_string());

                            if platform.to_lowercase() != "arm64" {
                                analysis.suggested_platforms.push("arm64".to_string());
                            }
                        }
                    }

                    "CustomAction" => {
                        // Check for native DLL custom actions
                        if node.attribute("DllEntry").is_some() || node.attribute("BinaryKey").is_some() {
                            analysis.has_native_custom_actions = true;
                            analysis.issues.push(Arm64Issue {
                                element: node.attribute("Id").map(String::from),
                                message: "Native DLL custom action requires ARM64 build".to_string(),
                                severity: IssueSeverity::Warning,
                                suggestion: "Build custom action DLL for ARM64 target".to_string(),
                            });
                        }
                    }

                    "Binary" => {
                        if let Some(source) = node.attribute("SourceFile").or(node.attribute("Source")) {
                            let source_lower = source.to_lowercase();
                            // Check for x86/x64 specific paths
                            if source_lower.contains("x86") || source_lower.contains("win32") {
                                analysis.has_32bit_components = true;
                                analysis.issues.push(Arm64Issue {
                                    element: node.attribute("Id").map(String::from),
                                    message: format!("Binary '{}' appears to be x86-specific", source),
                                    severity: IssueSeverity::Warning,
                                    suggestion: "Provide ARM64 version or use platform variables".to_string(),
                                });
                            }
                        }
                    }

                    "Driver" | "difx:Driver" => {
                        analysis.has_driver_install = true;
                        analysis.issues.push(Arm64Issue {
                            element: node.attribute("Id").map(String::from),
                            message: "DifxApp driver installation not supported on ARM64".to_string(),
                            severity: IssueSeverity::Error,
                            suggestion: "Use pnputil.exe or DIFxAPI directly for ARM64".to_string(),
                        });
                    }

                    "File" => {
                        if let Some(source) = node.attribute("Source") {
                            let source_lower = source.to_lowercase();
                            if (source_lower.ends_with(".dll") || source_lower.ends_with(".exe"))
                                && (source_lower.contains("x86") || source_lower.contains("x64"))
                            {
                                analysis.issues.push(Arm64Issue {
                                    element: node.attribute("Id").map(String::from),
                                    message: format!("File '{}' may be platform-specific", source),
                                    severity: IssueSeverity::Info,
                                    suggestion: "Use platform variable like $(var.Platform) in path".to_string(),
                                });
                            }
                        }
                    }

                    _ => {}
                }
            }
        }

        analysis
    }

    /// Generate multi-platform WiX configuration
    pub fn generate_multiplatform_config(platforms: &[Platform]) -> String {
        let mut config = String::new();

        config.push_str("<!-- Multi-Platform Build Configuration -->\n");
        config.push_str("<!-- Use with: wix build -arch <platform> -->\n\n");

        config.push_str("<?define Platform = \"$(sys.BUILDARCH)\" ?>\n\n");

        config.push_str("<!-- Platform-specific variables -->\n");
        config.push_str("<?if $(var.Platform) = x86 ?>\n");
        config.push_str("  <?define ProgramFilesFolder = \"ProgramFilesFolder\" ?>\n");
        config.push_str("  <?define SystemFolder = \"SystemFolder\" ?>\n");
        config.push_str("  <?define Win64 = \"no\" ?>\n");
        config.push_str("<?elseif $(var.Platform) = x64 ?>\n");
        config.push_str("  <?define ProgramFilesFolder = \"ProgramFiles64Folder\" ?>\n");
        config.push_str("  <?define SystemFolder = \"System64Folder\" ?>\n");
        config.push_str("  <?define Win64 = \"yes\" ?>\n");
        config.push_str("<?elseif $(var.Platform) = arm64 ?>\n");
        config.push_str("  <?define ProgramFilesFolder = \"ProgramFiles64Folder\" ?>\n");
        config.push_str("  <?define SystemFolder = \"System64Folder\" ?>\n");
        config.push_str("  <?define Win64 = \"yes\" ?>\n");
        config.push_str("<?endif ?>\n\n");

        config.push_str("<!-- Binary paths by platform -->\n");
        config.push_str("<?define BinPath = \"..\\bin\\$(var.Platform)\" ?>\n\n");

        config.push_str("<!-- Package configuration -->\n");
        config.push_str("<Package Platform=\"$(var.Platform)\">\n");
        config.push_str("  <!-- Components will use $(var.Platform) for paths -->\n");
        config.push_str("</Package>\n");

        config
    }

    /// Generate ARM64-compatible Component
    pub fn generate_arm64_component(id: &str, files: &[&str]) -> String {
        let mut component = String::new();

        component.push_str(&format!("<Component Id=\"{}\" Win64=\"$(var.Win64)\">\n", id));
        for file in files {
            let file_id = file.replace('.', "_").replace('/', "_").replace('\\', "_");
            component.push_str(&format!(
                "  <File Id=\"{}\" Source=\"$(var.BinPath)\\{}\" />\n",
                file_id, file
            ));
        }
        component.push_str("</Component>\n");

        component
    }

    /// Generate build script for multiple platforms
    pub fn generate_build_script(platforms: &[Platform], project_name: &str) -> String {
        let mut script = String::new();

        script.push_str("# Multi-Platform Build Script\n");
        script.push_str(&format!("# Project: {}\n\n", project_name));

        script.push_str("param(\n");
        script.push_str("    [string[]]$Platforms = @(");
        script.push_str(&platforms.iter().map(|p| format!("\"{}\"", p.as_str())).collect::<Vec<_>>().join(", "));
        script.push_str(")\n");
        script.push_str(")\n\n");

        script.push_str("$ErrorActionPreference = \"Stop\"\n\n");

        script.push_str("foreach ($platform in $Platforms) {\n");
        script.push_str("    Write-Host \"Building for $platform...\" -ForegroundColor Cyan\n");
        script.push_str("    \n");
        script.push_str(&format!("    wix build -arch $platform -o \"{}.$platform.msi\" {}.wxs\n", project_name, project_name));
        script.push_str("    \n");
        script.push_str("    if ($LASTEXITCODE -ne 0) {\n");
        script.push_str("        Write-Host \"Build failed for $platform\" -ForegroundColor Red\n");
        script.push_str("        exit 1\n");
        script.push_str("    }\n");
        script.push_str("    \n");
        script.push_str("    Write-Host \"Successfully built $platform\" -ForegroundColor Green\n");
        script.push_str("}\n\n");

        script.push_str("Write-Host \"\"\n");
        script.push_str("Write-Host \"All platforms built successfully!\" -ForegroundColor Green\n");

        script
    }

    /// Generate ARM64 detection condition
    pub fn generate_arm64_condition() -> &'static str {
        r#"<!-- ARM64 Detection -->
<!-- Note: On ARM64, x64 apps run under emulation -->
<!-- ProcessorArchitecture: 0=x86, 9=x64, 12=ARM64 -->

<Property Id="ARM64">
  <RegistrySearch Id="ARM64Check" Root="HKLM"
                  Key="SYSTEM\CurrentControlSet\Control\Session Manager\Environment"
                  Name="PROCESSOR_ARCHITECTURE" Type="raw" />
</Property>

<Condition Message="This installer requires ARM64 Windows.">
  <![CDATA[Installed OR ARM64 = "ARM64"]]>
</Condition>"#
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_from_str() {
        assert_eq!(Platform::from_str("x86"), Some(Platform::X86));
        assert_eq!(Platform::from_str("x64"), Some(Platform::X64));
        assert_eq!(Platform::from_str("arm64"), Some(Platform::Arm64));
        assert_eq!(Platform::from_str("aarch64"), Some(Platform::Arm64));
        assert_eq!(Platform::from_str("unknown"), None);
    }

    #[test]
    fn test_analyze_native_custom_action() {
        let content = r#"
        <Wix xmlns="http://wixtoolset.org/schemas/v4/wxs">
            <Package Platform="x64">
                <CustomAction Id="MyCA" BinaryKey="MyDll" DllEntry="DoStuff" />
            </Package>
        </Wix>
        "#;

        let analysis = Arm64Analyzer::analyze(content);
        assert!(analysis.has_native_custom_actions);
        assert!(!analysis.issues.is_empty());
    }

    #[test]
    fn test_analyze_driver() {
        let content = r#"
        <Wix xmlns="http://wixtoolset.org/schemas/v4/wxs">
            <Package Platform="x64">
                <Component>
                    <Driver Id="MyDriver" />
                </Component>
            </Package>
        </Wix>
        "#;

        let analysis = Arm64Analyzer::analyze(content);
        assert!(analysis.has_driver_install);
    }

    #[test]
    fn test_generate_multiplatform_config() {
        let config = Arm64Analyzer::generate_multiplatform_config(&[Platform::X64, Platform::Arm64]);
        assert!(config.contains("arm64"));
        assert!(config.contains("ProgramFiles64Folder"));
    }

    #[test]
    fn test_generate_build_script() {
        let script = Arm64Analyzer::generate_build_script(
            &[Platform::X64, Platform::Arm64],
            "MyApp"
        );
        assert!(script.contains("arm64"));
        assert!(script.contains("MyApp"));
    }

    #[test]
    fn test_generate_arm64_component() {
        let component = Arm64Analyzer::generate_arm64_component("MainComp", &["app.exe", "lib.dll"]);
        assert!(component.contains("Win64=\"$(var.Win64)\""));
        assert!(component.contains("$(var.BinPath)"));
    }
}

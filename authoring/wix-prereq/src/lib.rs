//! wix-prereq - Prerequisites detection and bundle helper for WiX installers
//!
//! Detects application prerequisites and generates WiX bundle fragments.
//!
//! # Example
//!
//! ```
//! use wix_prereq::{PrereqDetector, Prerequisite, PrereqKind};
//!
//! let detector = PrereqDetector::new();
//!
//! // Detect from project files
//! let prereqs = detector.detect_from_content("<Project Sdk=\"Microsoft.NET.Sdk\">
//!   <PropertyGroup>
//!     <TargetFramework>net8.0</TargetFramework>
//!   </PropertyGroup>
//! </Project>", "app.csproj");
//!
//! // Generate bundle fragment
//! let generator = wix_prereq::BundleGenerator::new();
//! let wxs = generator.generate(&prereqs);
//! ```

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PrereqError {
    #[error("Failed to parse project file: {0}")]
    ParseError(String),

    #[error("Unknown prerequisite: {0}")]
    UnknownPrereq(String),

    #[error("Version not supported: {0}")]
    UnsupportedVersion(String),
}

/// Kind of prerequisite
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PrereqKind {
    DotNetFramework,
    DotNetCore,
    DotNet,
    VcRedist,
    Java,
    NodeJs,
    Python,
    DirectX,
    SqlServer,
    SqlServerExpress,
    WindowsInstaller,
    Msxml,
    VstoRuntime,
}

impl PrereqKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            PrereqKind::DotNetFramework => ".NET Framework",
            PrereqKind::DotNetCore => ".NET Core",
            PrereqKind::DotNet => ".NET",
            PrereqKind::VcRedist => "Visual C++ Redistributable",
            PrereqKind::Java => "Java Runtime",
            PrereqKind::NodeJs => "Node.js",
            PrereqKind::Python => "Python",
            PrereqKind::DirectX => "DirectX",
            PrereqKind::SqlServer => "SQL Server",
            PrereqKind::SqlServerExpress => "SQL Server Express",
            PrereqKind::WindowsInstaller => "Windows Installer",
            PrereqKind::Msxml => "MSXML",
            PrereqKind::VstoRuntime => "VSTO Runtime",
        }
    }

    pub fn download_url(&self, version: &str) -> Option<&'static str> {
        match self {
            PrereqKind::DotNetFramework => match version {
                "4.8" | "4.8.0" => Some("https://go.microsoft.com/fwlink/?LinkId=2085155"),
                "4.7.2" => Some("https://go.microsoft.com/fwlink/?LinkId=863262"),
                "4.6.2" => Some("https://go.microsoft.com/fwlink/?LinkId=780596"),
                _ => None,
            },
            PrereqKind::DotNet => match version {
                "8.0" | "8" => Some("https://dotnet.microsoft.com/download/dotnet/8.0"),
                "7.0" | "7" => Some("https://dotnet.microsoft.com/download/dotnet/7.0"),
                "6.0" | "6" => Some("https://dotnet.microsoft.com/download/dotnet/6.0"),
                _ => None,
            },
            PrereqKind::VcRedist => Some("https://aka.ms/vs/17/release/vc_redist.x64.exe"),
            PrereqKind::NodeJs => Some("https://nodejs.org/dist/v20.11.0/node-v20.11.0-x64.msi"),
            _ => None,
        }
    }
}

/// A detected prerequisite
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prerequisite {
    pub kind: PrereqKind,
    pub version: String,
    pub min_version: Option<String>,
    pub architecture: Option<Architecture>,
    pub required: bool,
    pub source: Option<String>,
    pub download_url: Option<String>,
    pub install_condition: Option<String>,
    pub detect_condition: Option<String>,
}

impl Prerequisite {
    pub fn new(kind: PrereqKind, version: impl Into<String>) -> Self {
        let version = version.into();
        let download_url = kind.download_url(&version).map(String::from);
        Self {
            kind,
            version,
            min_version: None,
            architecture: None,
            required: true,
            source: None,
            download_url,
            install_condition: None,
            detect_condition: None,
        }
    }

    pub fn with_architecture(mut self, arch: Architecture) -> Self {
        self.architecture = Some(arch);
        self
    }

    pub fn with_min_version(mut self, version: impl Into<String>) -> Self {
        self.min_version = Some(version.into());
        self
    }

    pub fn optional(mut self) -> Self {
        self.required = false;
        self
    }

    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source = Some(source.into());
        self
    }

    pub fn with_download_url(mut self, url: impl Into<String>) -> Self {
        self.download_url = Some(url.into());
        self
    }

    /// Get the WiX detection condition for this prerequisite
    pub fn get_detect_condition(&self) -> String {
        if let Some(ref cond) = self.detect_condition {
            return cond.clone();
        }

        match self.kind {
            PrereqKind::DotNetFramework => {
                let release = match self.version.as_str() {
                    "4.8" | "4.8.0" => 528040,
                    "4.8.1" => 533320,
                    "4.7.2" => 461808,
                    "4.7.1" => 461308,
                    "4.7" => 460798,
                    "4.6.2" => 394802,
                    "4.6.1" => 394254,
                    "4.6" => 393295,
                    "4.5.2" => 379893,
                    "4.5.1" => 378675,
                    "4.5" => 378389,
                    _ => 0,
                };
                if release > 0 {
                    format!(
                        "NETFRAMEWORK45 >= {}",
                        release
                    )
                } else {
                    format!("NETFRAMEWORK{}", self.version.replace('.', ""))
                }
            }
            PrereqKind::DotNet | PrereqKind::DotNetCore => {
                let major = self.version.split('.').next().unwrap_or("6");
                format!(
                    "NETCORERUNTIME{}",
                    major
                )
            }
            PrereqKind::VcRedist => {
                let arch = self.architecture.unwrap_or(Architecture::X64);
                match arch {
                    Architecture::X86 => "VCRUNTIME_X86".to_string(),
                    Architecture::X64 => "VCRUNTIME_X64".to_string(),
                    Architecture::Arm64 => "VCRUNTIME_ARM64".to_string(),
                }
            }
            PrereqKind::Java => format!("JAVA_VERSION >= \"{}\"", self.version),
            PrereqKind::NodeJs => format!("NODEJS_VERSION >= \"{}\"", self.version),
            PrereqKind::Python => format!("PYTHON_VERSION >= \"{}\"", self.version),
            _ => String::new(),
        }
    }

    /// Get the registry key for detecting this prerequisite
    pub fn get_registry_detection(&self) -> Option<RegistryDetection> {
        match self.kind {
            PrereqKind::DotNetFramework => Some(RegistryDetection {
                root: "HKLM".to_string(),
                key: r"SOFTWARE\Microsoft\NET Framework Setup\NDP\v4\Full".to_string(),
                value: "Release".to_string(),
                value_type: RegistryValueType::DWord,
            }),
            PrereqKind::DotNet | PrereqKind::DotNetCore => {
                let _major = self.version.split('.').next().unwrap_or("6");
                Some(RegistryDetection {
                    root: "HKLM".to_string(),
                    key: r"SOFTWARE\dotnet\Setup\InstalledVersions\x64\sharedhost".to_string(),
                    value: "Version".to_string(),
                    value_type: RegistryValueType::String,
                })
            }
            PrereqKind::VcRedist => Some(RegistryDetection {
                root: "HKLM".to_string(),
                key: r"SOFTWARE\Microsoft\VisualStudio\14.0\VC\Runtimes\X64".to_string(),
                value: "Installed".to_string(),
                value_type: RegistryValueType::DWord,
            }),
            PrereqKind::Java => Some(RegistryDetection {
                root: "HKLM".to_string(),
                key: r"SOFTWARE\JavaSoft\Java Runtime Environment".to_string(),
                value: "CurrentVersion".to_string(),
                value_type: RegistryValueType::String,
            }),
            PrereqKind::NodeJs => Some(RegistryDetection {
                root: "HKLM".to_string(),
                key: r"SOFTWARE\Node.js".to_string(),
                value: "Version".to_string(),
                value_type: RegistryValueType::String,
            }),
            PrereqKind::Python => Some(RegistryDetection {
                root: "HKLM".to_string(),
                key: format!(
                    r"SOFTWARE\Python\PythonCore\{}\InstallPath",
                    self.version.split('.').take(2).collect::<Vec<_>>().join(".")
                ),
                value: "".to_string(),
                value_type: RegistryValueType::String,
            }),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryDetection {
    pub root: String,
    pub key: String,
    pub value: String,
    pub value_type: RegistryValueType,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum RegistryValueType {
    String,
    DWord,
    QWord,
}

/// Target architecture
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Architecture {
    X86,
    X64,
    Arm64,
}

impl Architecture {
    pub fn as_str(&self) -> &'static str {
        match self {
            Architecture::X86 => "x86",
            Architecture::X64 => "x64",
            Architecture::Arm64 => "arm64",
        }
    }
}

/// Prerequisite detector
pub struct PrereqDetector {
    patterns: HashMap<&'static str, Vec<DetectionPattern>>,
}

struct DetectionPattern {
    regex: Regex,
    extractor: fn(&regex::Captures) -> Option<Prerequisite>,
}

impl PrereqDetector {
    pub fn new() -> Self {
        let mut patterns: HashMap<&'static str, Vec<DetectionPattern>> = HashMap::new();

        // .csproj patterns
        patterns.insert(
            "csproj",
            vec![
                // .NET / .NET Core
                DetectionPattern {
                    regex: Regex::new(r"<TargetFramework>net(\d+\.\d+)</TargetFramework>")
                        .unwrap(),
                    extractor: |caps| {
                        let version = caps.get(1)?.as_str();
                        Some(Prerequisite::new(PrereqKind::DotNet, version))
                    },
                },
                DetectionPattern {
                    regex: Regex::new(r"<TargetFramework>net(\d+)</TargetFramework>").unwrap(),
                    extractor: |caps| {
                        let version = caps.get(1)?.as_str();
                        Some(Prerequisite::new(PrereqKind::DotNet, format!("{}.0", version)))
                    },
                },
                // .NET Framework
                DetectionPattern {
                    regex: Regex::new(r"<TargetFrameworkVersion>v(\d+\.\d+\.?\d*)</TargetFrameworkVersion>")
                        .unwrap(),
                    extractor: |caps| {
                        let version = caps.get(1)?.as_str();
                        Some(Prerequisite::new(PrereqKind::DotNetFramework, version))
                    },
                },
                DetectionPattern {
                    regex: Regex::new(r"<TargetFramework>net(\d+)(\d)-windows</TargetFramework>")
                        .unwrap(),
                    extractor: |caps| {
                        let major = caps.get(1)?.as_str();
                        let minor = caps.get(2)?.as_str();
                        Some(Prerequisite::new(
                            PrereqKind::DotNetFramework,
                            format!("{}.{}", major, minor),
                        ))
                    },
                },
                // netcoreappX.X
                DetectionPattern {
                    regex: Regex::new(r"<TargetFramework>netcoreapp(\d+\.\d+)</TargetFramework>")
                        .unwrap(),
                    extractor: |caps| {
                        let version = caps.get(1)?.as_str();
                        Some(Prerequisite::new(PrereqKind::DotNetCore, version))
                    },
                },
            ],
        );

        // package.json patterns
        patterns.insert(
            "package.json",
            vec![
                DetectionPattern {
                    regex: Regex::new(r#""node"\s*:\s*"[>=^~]*(\d+)(?:\.\d+)*"#).unwrap(),
                    extractor: |caps| {
                        let version = caps.get(1)?.as_str();
                        Some(Prerequisite::new(PrereqKind::NodeJs, format!("{}.0", version)))
                    },
                },
                DetectionPattern {
                    regex: Regex::new(r#""engines"\s*:\s*\{[^}]*"node"\s*:\s*"[>=^~]*(\d+\.\d+)"#)
                        .unwrap(),
                    extractor: |caps| {
                        let version = caps.get(1)?.as_str();
                        Some(Prerequisite::new(PrereqKind::NodeJs, version))
                    },
                },
            ],
        );

        // requirements.txt patterns
        patterns.insert(
            "requirements.txt",
            vec![DetectionPattern {
                regex: Regex::new(r"python_requires\s*[>=]+\s*(\d+\.\d+)").unwrap(),
                extractor: |caps| {
                    let version = caps.get(1)?.as_str();
                    Some(Prerequisite::new(PrereqKind::Python, version))
                },
            }],
        );

        // pyproject.toml patterns
        patterns.insert(
            "pyproject.toml",
            vec![DetectionPattern {
                regex: Regex::new(r#"requires-python\s*=\s*"[>=]+(\d+\.\d+)"#).unwrap(),
                extractor: |caps| {
                    let version = caps.get(1)?.as_str();
                    Some(Prerequisite::new(PrereqKind::Python, version))
                },
            }],
        );

        // pom.xml patterns
        patterns.insert(
            "pom.xml",
            vec![
                DetectionPattern {
                    regex: Regex::new(r"<java\.version>(\d+)</java\.version>").unwrap(),
                    extractor: |caps| {
                        let version = caps.get(1)?.as_str();
                        Some(Prerequisite::new(PrereqKind::Java, version))
                    },
                },
                DetectionPattern {
                    regex: Regex::new(r"<maven\.compiler\.source>(\d+)</maven\.compiler\.source>")
                        .unwrap(),
                    extractor: |caps| {
                        let version = caps.get(1)?.as_str();
                        Some(Prerequisite::new(PrereqKind::Java, version))
                    },
                },
            ],
        );

        // build.gradle patterns
        patterns.insert(
            "build.gradle",
            vec![DetectionPattern {
                regex: Regex::new(r#"sourceCompatibility\s*=\s*['"]?(\d+)['"]?"#).unwrap(),
                extractor: |caps| {
                    let version = caps.get(1)?.as_str();
                    Some(Prerequisite::new(PrereqKind::Java, version))
                },
            }],
        );

        // C++ vcxproj patterns
        patterns.insert(
            "vcxproj",
            vec![DetectionPattern {
                regex: Regex::new(r"<PlatformToolset>v(\d+)</PlatformToolset>").unwrap(),
                extractor: |caps| {
                    let toolset = caps.get(1)?.as_str();
                    let year = match toolset {
                        "143" => "2022",
                        "142" => "2019",
                        "141" => "2017",
                        "140" => "2015",
                        _ => toolset,
                    };
                    Some(
                        Prerequisite::new(PrereqKind::VcRedist, year)
                            .with_architecture(Architecture::X64),
                    )
                },
            }],
        );

        Self { patterns }
    }

    /// Detect prerequisites from file content
    pub fn detect_from_content(&self, content: &str, filename: &str) -> Vec<Prerequisite> {
        let mut prereqs = Vec::new();

        // Determine file type from extension
        let ext = Path::new(filename)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        let file_type = match ext {
            "csproj" | "vbproj" | "fsproj" => "csproj",
            "vcxproj" => "vcxproj",
            "json" if filename.contains("package") => "package.json",
            "txt" if filename.contains("requirements") => "requirements.txt",
            "toml" if filename.contains("pyproject") => "pyproject.toml",
            "xml" if filename.contains("pom") => "pom.xml",
            "gradle" => "build.gradle",
            _ => return prereqs,
        };

        if let Some(patterns) = self.patterns.get(file_type) {
            for pattern in patterns {
                for caps in pattern.regex.captures_iter(content) {
                    if let Some(mut prereq) = (pattern.extractor)(&caps) {
                        prereq = prereq.with_source(filename.to_string());
                        prereqs.push(prereq);
                    }
                }
            }
        }

        // Deduplicate by kind
        let mut seen = std::collections::HashSet::new();
        prereqs.retain(|p| seen.insert((p.kind, p.version.clone())));

        prereqs
    }

    /// Detect prerequisites from multiple files
    pub fn detect_from_files(&self, files: &[(String, String)]) -> Vec<Prerequisite> {
        let mut all_prereqs = Vec::new();

        for (filename, content) in files {
            let prereqs = self.detect_from_content(content, filename);
            all_prereqs.extend(prereqs);
        }

        // Deduplicate
        let mut seen = std::collections::HashSet::new();
        all_prereqs.retain(|p| seen.insert((p.kind, p.version.clone())));

        all_prereqs
    }
}

impl Default for PrereqDetector {
    fn default() -> Self {
        Self::new()
    }
}

/// Known prerequisite packages with download info
pub struct PrereqCatalog;

impl PrereqCatalog {
    /// Get common .NET Framework prerequisites
    pub fn dotnet_framework() -> Vec<Prerequisite> {
        vec![
            Prerequisite::new(PrereqKind::DotNetFramework, "4.8.1")
                .with_download_url("https://go.microsoft.com/fwlink/?LinkId=2203304"),
            Prerequisite::new(PrereqKind::DotNetFramework, "4.8")
                .with_download_url("https://go.microsoft.com/fwlink/?LinkId=2085155"),
            Prerequisite::new(PrereqKind::DotNetFramework, "4.7.2")
                .with_download_url("https://go.microsoft.com/fwlink/?LinkId=863262"),
            Prerequisite::new(PrereqKind::DotNetFramework, "4.6.2")
                .with_download_url("https://go.microsoft.com/fwlink/?LinkId=780596"),
        ]
    }

    /// Get common .NET prerequisites
    pub fn dotnet() -> Vec<Prerequisite> {
        vec![
            Prerequisite::new(PrereqKind::DotNet, "9.0")
                .with_download_url("https://dotnet.microsoft.com/download/dotnet/9.0"),
            Prerequisite::new(PrereqKind::DotNet, "8.0")
                .with_download_url("https://dotnet.microsoft.com/download/dotnet/8.0"),
            Prerequisite::new(PrereqKind::DotNet, "7.0")
                .with_download_url("https://dotnet.microsoft.com/download/dotnet/7.0"),
            Prerequisite::new(PrereqKind::DotNet, "6.0")
                .with_download_url("https://dotnet.microsoft.com/download/dotnet/6.0"),
        ]
    }

    /// Get Visual C++ redistributables
    pub fn vc_redist() -> Vec<Prerequisite> {
        vec![
            Prerequisite::new(PrereqKind::VcRedist, "2022")
                .with_architecture(Architecture::X64)
                .with_download_url("https://aka.ms/vs/17/release/vc_redist.x64.exe"),
            Prerequisite::new(PrereqKind::VcRedist, "2022")
                .with_architecture(Architecture::X86)
                .with_download_url("https://aka.ms/vs/17/release/vc_redist.x86.exe"),
            Prerequisite::new(PrereqKind::VcRedist, "2019")
                .with_architecture(Architecture::X64)
                .with_download_url("https://aka.ms/vs/16/release/vc_redist.x64.exe"),
        ]
    }

    /// Get a prerequisite by kind and version
    pub fn get(kind: PrereqKind, version: &str) -> Option<Prerequisite> {
        match kind {
            PrereqKind::DotNetFramework => Self::dotnet_framework()
                .into_iter()
                .find(|p| p.version == version),
            PrereqKind::DotNet | PrereqKind::DotNetCore => {
                Self::dotnet().into_iter().find(|p| p.version == version)
            }
            PrereqKind::VcRedist => Self::vc_redist()
                .into_iter()
                .find(|p| p.version == version),
            _ => Some(Prerequisite::new(kind, version)),
        }
    }
}

/// Bundle fragment generator for WiX
pub struct BundleGenerator {
    pub bundle_name: String,
    pub bundle_version: String,
    pub manufacturer: String,
}

impl BundleGenerator {
    pub fn new() -> Self {
        Self {
            bundle_name: "Prerequisites".to_string(),
            bundle_version: "1.0.0".to_string(),
            manufacturer: "".to_string(),
        }
    }

    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.bundle_name = name.into();
        self
    }

    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.bundle_version = version.into();
        self
    }

    pub fn with_manufacturer(mut self, manufacturer: impl Into<String>) -> Self {
        self.manufacturer = manufacturer.into();
        self
    }

    /// Generate a WiX bundle fragment for prerequisites
    pub fn generate(&self, prereqs: &[Prerequisite]) -> String {
        let mut wxs = String::new();

        wxs.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
        wxs.push_str("<Wix xmlns=\"http://wixtoolset.org/schemas/v4/wxs\"\n");
        wxs.push_str("     xmlns:bal=\"http://wixtoolset.org/schemas/v4/wxs/bal\">\n\n");

        wxs.push_str("  <Fragment>\n");
        wxs.push_str("    <PackageGroup Id=\"Prerequisites\">\n");

        for prereq in prereqs {
            wxs.push_str(&self.generate_package(prereq));
        }

        wxs.push_str("    </PackageGroup>\n");
        wxs.push_str("  </Fragment>\n\n");

        // Generate detection fragments
        wxs.push_str("  <Fragment>\n");
        for prereq in prereqs {
            if let Some(reg) = prereq.get_registry_detection() {
                wxs.push_str(&self.generate_detection(&prereq.kind, &reg));
            }
        }
        wxs.push_str("  </Fragment>\n\n");

        wxs.push_str("</Wix>\n");

        wxs
    }

    fn generate_package(&self, prereq: &Prerequisite) -> String {
        let id = format!(
            "{}_{}_{}",
            prereq.kind.as_str().replace(' ', "").replace('.', ""),
            prereq.version.replace('.', "_"),
            prereq.architecture.map(|a| a.as_str()).unwrap_or("any")
        );

        let mut pkg = String::new();
        pkg.push_str(&format!("      <!-- {} {} -->\n", prereq.kind.as_str(), prereq.version));

        match prereq.kind {
            PrereqKind::DotNetFramework | PrereqKind::DotNet | PrereqKind::DotNetCore => {
                pkg.push_str(&format!(
                    "      <ExePackage Id=\"{}\"\n",
                    escape_xml(&id)
                ));
                pkg.push_str(&format!(
                    "                  Name=\"{} {}\"\n",
                    prereq.kind.as_str(),
                    prereq.version
                ));
                if let Some(ref url) = prereq.download_url {
                    pkg.push_str(&format!("                  DownloadUrl=\"{}\"\n", url));
                }
                let detect = prereq.get_detect_condition();
                if !detect.is_empty() {
                    pkg.push_str(&format!(
                        "                  DetectCondition=\"{}\"\n",
                        escape_xml(&detect)
                    ));
                }
                pkg.push_str("                  PerMachine=\"yes\"\n");
                pkg.push_str("                  Vital=\"yes\" />\n\n");
            }
            PrereqKind::VcRedist => {
                let arch = prereq.architecture.unwrap_or(Architecture::X64);
                pkg.push_str(&format!(
                    "      <ExePackage Id=\"{}\"\n",
                    escape_xml(&id)
                ));
                pkg.push_str(&format!(
                    "                  Name=\"VC++ {} Redistributable ({})\"\n",
                    prereq.version,
                    arch.as_str()
                ));
                if let Some(ref url) = prereq.download_url {
                    pkg.push_str(&format!("                  DownloadUrl=\"{}\"\n", url));
                }
                pkg.push_str("                  InstallArguments=\"/install /quiet /norestart\"\n");
                pkg.push_str("                  RepairArguments=\"/repair /quiet /norestart\"\n");
                pkg.push_str("                  UninstallArguments=\"/uninstall /quiet /norestart\"\n");
                let detect = prereq.get_detect_condition();
                if !detect.is_empty() {
                    pkg.push_str(&format!(
                        "                  DetectCondition=\"{}\"\n",
                        escape_xml(&detect)
                    ));
                }
                pkg.push_str("                  PerMachine=\"yes\"\n");
                pkg.push_str("                  Vital=\"yes\" />\n\n");
            }
            PrereqKind::NodeJs | PrereqKind::Python => {
                pkg.push_str(&format!(
                    "      <MsiPackage Id=\"{}\"\n",
                    escape_xml(&id)
                ));
                pkg.push_str(&format!(
                    "                  Name=\"{} {}\"\n",
                    prereq.kind.as_str(),
                    prereq.version
                ));
                if let Some(ref url) = prereq.download_url {
                    pkg.push_str(&format!("                  DownloadUrl=\"{}\"\n", url));
                }
                let detect = prereq.get_detect_condition();
                if !detect.is_empty() {
                    pkg.push_str(&format!(
                        "                  DetectCondition=\"{}\"\n",
                        escape_xml(&detect)
                    ));
                }
                pkg.push_str("                  PerMachine=\"yes\"\n");
                pkg.push_str("                  Vital=\"yes\" />\n\n");
            }
            _ => {
                pkg.push_str(&format!(
                    "      <ExePackage Id=\"{}\"\n",
                    escape_xml(&id)
                ));
                pkg.push_str(&format!(
                    "                  Name=\"{} {}\"\n",
                    prereq.kind.as_str(),
                    prereq.version
                ));
                pkg.push_str("                  Vital=\"yes\" />\n\n");
            }
        }

        pkg
    }

    fn generate_detection(&self, kind: &PrereqKind, reg: &RegistryDetection) -> String {
        let var_name = format!(
            "{}_DETECTED",
            kind.as_str().to_uppercase().replace(' ', "_")
        );

        format!(
            "    <util:RegistrySearch Id=\"{}Search\"\n\
             \x20                       Root=\"{}\"\n\
             \x20                       Key=\"{}\"\n\
             \x20                       Value=\"{}\"\n\
             \x20                       Variable=\"{}\" />\n",
            var_name,
            reg.root,
            escape_xml(&reg.key),
            reg.value,
            var_name
        )
    }

    /// Generate a simple check script (PowerShell)
    pub fn generate_check_script(&self, prereqs: &[Prerequisite]) -> String {
        let mut script = String::new();
        script.push_str("# Prerequisites Check Script\n");
        script.push_str("# Generated by wix-prereq\n\n");

        script.push_str("$missing = @()\n\n");

        for prereq in prereqs {
            script.push_str(&format!("# Check {} {}\n", prereq.kind.as_str(), prereq.version));

            if let Some(reg) = prereq.get_registry_detection() {
                script.push_str(&format!(
                    "$key = 'Registry::{}\\{}'\n",
                    reg.root, reg.key
                ));
                script.push_str("if (-not (Test-Path $key)) {\n");
                script.push_str(&format!(
                    "    $missing += '{} {}'\n",
                    prereq.kind.as_str(),
                    prereq.version
                ));
                script.push_str("}\n\n");
            }
        }

        script.push_str("if ($missing.Count -gt 0) {\n");
        script.push_str("    Write-Host 'Missing prerequisites:' -ForegroundColor Red\n");
        script.push_str("    $missing | ForEach-Object { Write-Host \"  - $_\" }\n");
        script.push_str("    exit 1\n");
        script.push_str("} else {\n");
        script.push_str("    Write-Host 'All prerequisites are installed.' -ForegroundColor Green\n");
        script.push_str("    exit 0\n");
        script.push_str("}\n");

        script
    }
}

impl Default for BundleGenerator {
    fn default() -> Self {
        Self::new()
    }
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_dotnet_sdk() {
        let detector = PrereqDetector::new();
        let content = r#"<Project Sdk="Microsoft.NET.Sdk">
  <PropertyGroup>
    <TargetFramework>net8.0</TargetFramework>
  </PropertyGroup>
</Project>"#;

        let prereqs = detector.detect_from_content(content, "app.csproj");
        assert_eq!(prereqs.len(), 1);
        assert_eq!(prereqs[0].kind, PrereqKind::DotNet);
        assert_eq!(prereqs[0].version, "8.0");
    }

    #[test]
    fn test_detect_dotnet_framework() {
        let detector = PrereqDetector::new();
        let content = r#"<Project>
  <PropertyGroup>
    <TargetFrameworkVersion>v4.8</TargetFrameworkVersion>
  </PropertyGroup>
</Project>"#;

        let prereqs = detector.detect_from_content(content, "app.csproj");
        assert_eq!(prereqs.len(), 1);
        assert_eq!(prereqs[0].kind, PrereqKind::DotNetFramework);
        assert_eq!(prereqs[0].version, "4.8");
    }

    #[test]
    fn test_detect_netcore() {
        let detector = PrereqDetector::new();
        let content = r#"<Project>
  <PropertyGroup>
    <TargetFramework>netcoreapp3.1</TargetFramework>
  </PropertyGroup>
</Project>"#;

        let prereqs = detector.detect_from_content(content, "app.csproj");
        assert_eq!(prereqs.len(), 1);
        assert_eq!(prereqs[0].kind, PrereqKind::DotNetCore);
        assert_eq!(prereqs[0].version, "3.1");
    }

    #[test]
    fn test_detect_nodejs() {
        let detector = PrereqDetector::new();
        let content = r#"{
  "name": "myapp",
  "engines": {
    "node": ">=18.0"
  }
}"#;

        let prereqs = detector.detect_from_content(content, "package.json");
        assert_eq!(prereqs.len(), 1);
        assert_eq!(prereqs[0].kind, PrereqKind::NodeJs);
        assert_eq!(prereqs[0].version, "18.0");
    }

    #[test]
    fn test_detect_java() {
        let detector = PrereqDetector::new();
        let content = r#"<project>
  <properties>
    <java.version>17</java.version>
  </properties>
</project>"#;

        let prereqs = detector.detect_from_content(content, "pom.xml");
        assert_eq!(prereqs.len(), 1);
        assert_eq!(prereqs[0].kind, PrereqKind::Java);
        assert_eq!(prereqs[0].version, "17");
    }

    #[test]
    fn test_detect_python() {
        let detector = PrereqDetector::new();
        let content = r#"[project]
requires-python = ">=3.10"
"#;

        let prereqs = detector.detect_from_content(content, "pyproject.toml");
        assert_eq!(prereqs.len(), 1);
        assert_eq!(prereqs[0].kind, PrereqKind::Python);
        assert_eq!(prereqs[0].version, "3.10");
    }

    #[test]
    fn test_detect_vcredist() {
        let detector = PrereqDetector::new();
        let content = r#"<Project>
  <PropertyGroup>
    <PlatformToolset>v143</PlatformToolset>
  </PropertyGroup>
</Project>"#;

        let prereqs = detector.detect_from_content(content, "app.vcxproj");
        assert_eq!(prereqs.len(), 1);
        assert_eq!(prereqs[0].kind, PrereqKind::VcRedist);
        assert_eq!(prereqs[0].version, "2022");
    }

    #[test]
    fn test_prereq_detect_condition() {
        let prereq = Prerequisite::new(PrereqKind::DotNetFramework, "4.8");
        let cond = prereq.get_detect_condition();
        assert!(cond.contains("NETFRAMEWORK45"));
        assert!(cond.contains("528040"));
    }

    #[test]
    fn test_prereq_registry_detection() {
        let prereq = Prerequisite::new(PrereqKind::DotNetFramework, "4.8");
        let reg = prereq.get_registry_detection().unwrap();
        assert_eq!(reg.root, "HKLM");
        assert!(reg.key.contains("NET Framework Setup"));
    }

    #[test]
    fn test_prereq_catalog() {
        let frameworks = PrereqCatalog::dotnet_framework();
        assert!(!frameworks.is_empty());
        assert!(frameworks.iter().any(|p| p.version == "4.8"));

        let dotnet = PrereqCatalog::dotnet();
        assert!(!dotnet.is_empty());
        assert!(dotnet.iter().any(|p| p.version == "8.0"));
    }

    #[test]
    fn test_bundle_generator() {
        let prereqs = vec![
            Prerequisite::new(PrereqKind::DotNet, "8.0"),
            Prerequisite::new(PrereqKind::VcRedist, "2022").with_architecture(Architecture::X64),
        ];

        let generator = BundleGenerator::new();
        let wxs = generator.generate(&prereqs);

        assert!(wxs.contains("PackageGroup"));
        assert!(wxs.contains(".NET 8.0"));
        assert!(wxs.contains("VC++"));
        assert!(wxs.contains("x64"));
    }

    #[test]
    fn test_generate_check_script() {
        let prereqs = vec![Prerequisite::new(PrereqKind::DotNetFramework, "4.8")];

        let generator = BundleGenerator::new();
        let script = generator.generate_check_script(&prereqs);

        assert!(script.contains("Prerequisites Check Script"));
        assert!(script.contains(".NET Framework"));
        assert!(script.contains("Test-Path"));
    }

    #[test]
    fn test_prereq_kind_as_str() {
        assert_eq!(PrereqKind::DotNetFramework.as_str(), ".NET Framework");
        assert_eq!(PrereqKind::VcRedist.as_str(), "Visual C++ Redistributable");
        assert_eq!(PrereqKind::NodeJs.as_str(), "Node.js");
    }

    #[test]
    fn test_architecture_as_str() {
        assert_eq!(Architecture::X86.as_str(), "x86");
        assert_eq!(Architecture::X64.as_str(), "x64");
        assert_eq!(Architecture::Arm64.as_str(), "arm64");
    }

    #[test]
    fn test_prerequisite_builder() {
        let prereq = Prerequisite::new(PrereqKind::VcRedist, "2022")
            .with_architecture(Architecture::X64)
            .with_min_version("14.30")
            .optional();

        assert_eq!(prereq.architecture, Some(Architecture::X64));
        assert_eq!(prereq.min_version, Some("14.30".to_string()));
        assert!(!prereq.required);
    }

    #[test]
    fn test_detect_multiple_files() {
        let detector = PrereqDetector::new();
        let files = vec![
            (
                "app.csproj".to_string(),
                "<Project><PropertyGroup><TargetFramework>net8.0</TargetFramework></PropertyGroup></Project>".to_string(),
            ),
            (
                "package.json".to_string(),
                r#"{"engines": {"node": ">=20.0"}}"#.to_string(),
            ),
        ];

        let prereqs = detector.detect_from_files(&files);
        assert_eq!(prereqs.len(), 2);

        let kinds: Vec<_> = prereqs.iter().map(|p| p.kind).collect();
        assert!(kinds.contains(&PrereqKind::DotNet));
        assert!(kinds.contains(&PrereqKind::NodeJs));
    }

    #[test]
    fn test_escape_xml() {
        assert_eq!(escape_xml("a < b"), "a &lt; b");
        assert_eq!(escape_xml("a & b"), "a &amp; b");
        assert_eq!(escape_xml("\"quote\""), "&quot;quote&quot;");
    }

    #[test]
    fn test_download_urls() {
        let prereq = Prerequisite::new(PrereqKind::DotNetFramework, "4.8");
        assert!(prereq.download_url.is_some());

        let prereq = Prerequisite::new(PrereqKind::VcRedist, "2022");
        assert!(prereq.download_url.is_some());
    }

    #[test]
    fn test_registry_value_types() {
        let prereq = Prerequisite::new(PrereqKind::DotNetFramework, "4.8");
        let reg = prereq.get_registry_detection().unwrap();
        assert!(matches!(reg.value_type, RegistryValueType::DWord));

        let prereq = Prerequisite::new(PrereqKind::Java, "17");
        let reg = prereq.get_registry_detection().unwrap();
        assert!(matches!(reg.value_type, RegistryValueType::String));
    }

    #[test]
    fn test_gradle_detection() {
        let detector = PrereqDetector::new();
        let content = r#"
            plugins {
                id 'java'
            }
            sourceCompatibility = '17'
        "#;

        let prereqs = detector.detect_from_content(content, "build.gradle");
        assert_eq!(prereqs.len(), 1);
        assert_eq!(prereqs[0].kind, PrereqKind::Java);
        assert_eq!(prereqs[0].version, "17");
    }

    #[test]
    fn test_prereq_source_tracking() {
        let detector = PrereqDetector::new();
        let content = "<Project><PropertyGroup><TargetFramework>net8.0</TargetFramework></PropertyGroup></Project>";

        let prereqs = detector.detect_from_content(content, "MyApp/app.csproj");
        assert_eq!(prereqs[0].source, Some("MyApp/app.csproj".to_string()));
    }

    #[test]
    fn test_vcredist_detect_condition() {
        let prereq = Prerequisite::new(PrereqKind::VcRedist, "2022")
            .with_architecture(Architecture::X64);
        let cond = prereq.get_detect_condition();
        assert_eq!(cond, "VCRUNTIME_X64");

        let prereq = Prerequisite::new(PrereqKind::VcRedist, "2022")
            .with_architecture(Architecture::X86);
        let cond = prereq.get_detect_condition();
        assert_eq!(cond, "VCRUNTIME_X86");
    }
}

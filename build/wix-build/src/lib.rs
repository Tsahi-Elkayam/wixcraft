//! wix-build - Unified build CLI for WiX installers
//!
//! Abstracts over WiX v3 (candle/light) and WiX v4+ (wix.exe) with a consistent interface.
//! Also provides preview capabilities to see what would be installed without building.
//!
//! # Example
//!
//! ```no_run
//! use wix_build::{BuildConfig, WixToolset};
//!
//! let config = BuildConfig::new("Product.wxs")
//!     .output("Product.msi")
//!     .extension("WixUIExtension")
//!     .define("Version", "1.0.0");
//!
//! let toolset = WixToolset::detect().unwrap();
//! let command = toolset.build_command(&config);
//! println!("Build command: {}", command);
//! ```

pub mod preview;

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum BuildError {
    #[error("WiX toolset not found. Install from https://wixtoolset.org/")]
    ToolsetNotFound,

    #[error("Invalid source file: {0}")]
    InvalidSource(String),

    #[error("Output path required for multiple source files")]
    OutputRequired,

    #[error("Extension not found: {0}")]
    ExtensionNotFound(String),

    #[error("Build failed: {0}")]
    BuildFailed(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
}

/// WiX toolset version
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WixVersion {
    V3,
    V4,
    V5,
}

impl WixVersion {
    pub fn as_str(&self) -> &'static str {
        match self {
            WixVersion::V3 => "3.x",
            WixVersion::V4 => "4.x",
            WixVersion::V5 => "5.x",
        }
    }
}

/// Detected WiX toolset installation
#[derive(Debug, Clone)]
pub struct WixToolset {
    pub version: WixVersion,
    pub path: PathBuf,
    pub candle_path: Option<PathBuf>,
    pub light_path: Option<PathBuf>,
    pub wix_path: Option<PathBuf>,
}

impl WixToolset {
    /// Detect installed WiX toolset
    pub fn detect() -> Result<Self, BuildError> {
        // Try WiX v4/v5 first (wix.exe or dotnet tool)
        if let Ok(wix_path) = which::which("wix") {
            return Ok(Self {
                version: WixVersion::V4, // Could be v5, but API is same
                path: wix_path.parent().unwrap_or(Path::new(".")).to_path_buf(),
                candle_path: None,
                light_path: None,
                wix_path: Some(wix_path),
            });
        }

        // Try WiX v3 (candle.exe + light.exe)
        if let (Ok(candle), Ok(light)) = (which::which("candle"), which::which("light")) {
            return Ok(Self {
                version: WixVersion::V3,
                path: candle.parent().unwrap_or(Path::new(".")).to_path_buf(),
                candle_path: Some(candle),
                light_path: Some(light),
                wix_path: None,
            });
        }

        Err(BuildError::ToolsetNotFound)
    }

    /// Create a toolset with a specific version (for testing)
    pub fn with_version(version: WixVersion) -> Self {
        Self {
            version,
            path: PathBuf::from("."),
            candle_path: if version == WixVersion::V3 {
                Some(PathBuf::from("candle.exe"))
            } else {
                None
            },
            light_path: if version == WixVersion::V3 {
                Some(PathBuf::from("light.exe"))
            } else {
                None
            },
            wix_path: if version != WixVersion::V3 {
                Some(PathBuf::from("wix"))
            } else {
                None
            },
        }
    }

    /// Build a command string for the given configuration
    pub fn build_command(&self, config: &BuildConfig) -> String {
        match self.version {
            WixVersion::V3 => self.build_v3_command(config),
            WixVersion::V4 | WixVersion::V5 => self.build_v4_command(config),
        }
    }

    /// Build separate candle and light commands for v3
    pub fn build_v3_commands(&self, config: &BuildConfig) -> (String, String) {
        let candle = self.build_candle_command(config);
        let light = self.build_light_command(config);
        (candle, light)
    }

    fn build_v3_command(&self, config: &BuildConfig) -> String {
        let (candle, light) = self.build_v3_commands(config);
        format!("{} && {}", candle, light)
    }

    fn build_candle_command(&self, config: &BuildConfig) -> String {
        let candle = self
            .candle_path
            .as_ref()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| "candle.exe".to_string());

        let mut cmd = vec![candle];

        // Architecture
        if let Some(arch) = &config.architecture {
            cmd.push("-arch".to_string());
            cmd.push(arch.as_wix_str().to_string());
        }

        // Defines
        for (key, value) in &config.defines {
            if let Some(v) = value {
                cmd.push(format!("-d{}={}", key, v));
            } else {
                cmd.push(format!("-d{}", key));
            }
        }

        // Extensions
        for ext in &config.extensions {
            cmd.push("-ext".to_string());
            cmd.push(ext.clone());
        }

        // Include paths
        for path in &config.include_paths {
            cmd.push("-I".to_string());
            cmd.push(path.to_string_lossy().to_string());
        }

        // Suppress warnings
        for code in &config.suppress_warnings {
            cmd.push(format!("-sw{}", code));
        }

        // Warnings as errors
        if config.warnings_as_errors {
            cmd.push("-wx".to_string());
        }

        // Verbose
        if config.verbose {
            cmd.push("-v".to_string());
        }

        // Output directory for .wixobj files
        if let Some(ref out) = config.output {
            let obj_dir = Path::new(out)
                .parent()
                .unwrap_or(Path::new("."))
                .to_string_lossy();
            if !obj_dir.is_empty() && obj_dir != "." {
                cmd.push("-out".to_string());
                cmd.push(format!("{}\\", obj_dir));
            }
        }

        // Source files
        for source in &config.sources {
            cmd.push(source.to_string_lossy().to_string());
        }

        cmd.join(" ")
    }

    fn build_light_command(&self, config: &BuildConfig) -> String {
        let light = self
            .light_path
            .as_ref()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| "light.exe".to_string());

        let mut cmd = vec![light];

        // Extensions
        for ext in &config.extensions {
            cmd.push("-ext".to_string());
            cmd.push(ext.clone());
        }

        // Suppress ICE validation
        if config.skip_validation {
            cmd.push("-sice:*".to_string());
        }

        // Cultures
        if !config.cultures.is_empty() {
            cmd.push("-cultures:".to_string() + &config.cultures.join(";"));
        }

        // Localization files
        for loc in &config.localization_files {
            cmd.push("-loc".to_string());
            cmd.push(loc.to_string_lossy().to_string());
        }

        // Bind paths
        for path in &config.bind_paths {
            cmd.push("-b".to_string());
            cmd.push(path.to_string_lossy().to_string());
        }

        // Suppress warnings
        for code in &config.suppress_warnings {
            cmd.push(format!("-sw{}", code));
        }

        // Warnings as errors
        if config.warnings_as_errors {
            cmd.push("-wx".to_string());
        }

        // Verbose
        if config.verbose {
            cmd.push("-v".to_string());
        }

        // Output
        if let Some(ref out) = config.output {
            cmd.push("-out".to_string());
            cmd.push(out.to_string_lossy().to_string());
        }

        // Object files (convert .wxs to .wixobj)
        for source in &config.sources {
            let obj = source.with_extension("wixobj");
            if let Some(ref out) = config.output {
                let obj_dir = Path::new(out).parent().unwrap_or(Path::new("."));
                let obj_name = obj.file_name().unwrap_or_default();
                cmd.push(obj_dir.join(obj_name).to_string_lossy().to_string());
            } else {
                cmd.push(obj.to_string_lossy().to_string());
            }
        }

        cmd.join(" ")
    }

    fn build_v4_command(&self, config: &BuildConfig) -> String {
        let wix = self
            .wix_path
            .as_ref()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| "wix".to_string());

        let mut cmd = vec![wix, "build".to_string()];

        // Architecture
        if let Some(arch) = &config.architecture {
            cmd.push("-arch".to_string());
            cmd.push(arch.as_wix_str().to_string());
        }

        // Defines
        for (key, value) in &config.defines {
            if let Some(v) = value {
                cmd.push("-d".to_string());
                cmd.push(format!("{}={}", key, v));
            } else {
                cmd.push("-d".to_string());
                cmd.push(key.clone());
            }
        }

        // Extensions
        for ext in &config.extensions {
            cmd.push("-ext".to_string());
            cmd.push(ext.clone());
        }

        // Include paths
        for path in &config.include_paths {
            cmd.push("-i".to_string());
            cmd.push(path.to_string_lossy().to_string());
        }

        // Bind paths
        for path in &config.bind_paths {
            cmd.push("-bindpath".to_string());
            cmd.push(path.to_string_lossy().to_string());
        }

        // Cultures
        if !config.cultures.is_empty() {
            cmd.push("-culture".to_string());
            cmd.push(config.cultures.join(";"));
        }

        // Localization files
        for loc in &config.localization_files {
            cmd.push("-loc".to_string());
            cmd.push(loc.to_string_lossy().to_string());
        }

        // Output
        if let Some(ref out) = config.output {
            cmd.push("-o".to_string());
            cmd.push(out.to_string_lossy().to_string());
        }

        // Intermediate directory
        if let Some(ref dir) = config.intermediate_dir {
            cmd.push("-intermediateFolder".to_string());
            cmd.push(dir.to_string_lossy().to_string());
        }

        // Output type
        if let Some(ref out_type) = config.output_type {
            cmd.push("-outputType".to_string());
            cmd.push(out_type.as_str().to_string());
        }

        // Verbose
        if config.verbose {
            cmd.push("-v".to_string());
        }

        // Pedantic
        if config.pedantic {
            cmd.push("-pedantic".to_string());
        }

        // Source files
        for source in &config.sources {
            cmd.push(source.to_string_lossy().to_string());
        }

        cmd.join(" ")
    }
}

/// Target architecture
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Architecture {
    #[default]
    X86,
    X64,
    Arm64,
}

impl Architecture {
    pub fn as_wix_str(&self) -> &'static str {
        match self {
            Architecture::X86 => "x86",
            Architecture::X64 => "x64",
            Architecture::Arm64 => "arm64",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "x86" | "win32" | "i386" | "i686" => Some(Architecture::X86),
            "x64" | "amd64" | "x86_64" => Some(Architecture::X64),
            "arm64" | "aarch64" => Some(Architecture::Arm64),
            _ => None,
        }
    }
}

/// Output type for WiX v4+
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputType {
    Msi,
    Msm,
    Bundle,
    Wixlib,
}

impl OutputType {
    pub fn as_str(&self) -> &'static str {
        match self {
            OutputType::Msi => "msi",
            OutputType::Msm => "msm",
            OutputType::Bundle => "bundle",
            OutputType::Wixlib => "wixlib",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "msi" => Some(OutputType::Msi),
            "msm" | "merge" => Some(OutputType::Msm),
            "bundle" | "exe" => Some(OutputType::Bundle),
            "wixlib" | "lib" => Some(OutputType::Wixlib),
            _ => None,
        }
    }
}

/// Build configuration
#[derive(Debug, Clone, Default)]
pub struct BuildConfig {
    /// Source WXS files
    pub sources: Vec<PathBuf>,
    /// Output file path
    pub output: Option<PathBuf>,
    /// Target architecture
    pub architecture: Option<Architecture>,
    /// Preprocessor defines
    pub defines: HashMap<String, Option<String>>,
    /// WiX extensions to use
    pub extensions: Vec<String>,
    /// Include search paths
    pub include_paths: Vec<PathBuf>,
    /// Bind paths for file resolution
    pub bind_paths: Vec<PathBuf>,
    /// Localization files (.wxl)
    pub localization_files: Vec<PathBuf>,
    /// Cultures for localization
    pub cultures: Vec<String>,
    /// Intermediate output directory
    pub intermediate_dir: Option<PathBuf>,
    /// Output type (v4+)
    pub output_type: Option<OutputType>,
    /// Warning codes to suppress
    pub suppress_warnings: Vec<String>,
    /// Treat warnings as errors
    pub warnings_as_errors: bool,
    /// Skip ICE validation
    pub skip_validation: bool,
    /// Verbose output
    pub verbose: bool,
    /// Pedantic mode (v4+)
    pub pedantic: bool,
}

impl BuildConfig {
    /// Create a new build configuration with source file(s)
    pub fn new<P: AsRef<Path>>(source: P) -> Self {
        Self {
            sources: vec![source.as_ref().to_path_buf()],
            ..Default::default()
        }
    }

    /// Create from multiple source files
    pub fn from_sources<I, P>(sources: I) -> Self
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        Self {
            sources: sources.into_iter().map(|p| p.as_ref().to_path_buf()).collect(),
            ..Default::default()
        }
    }

    /// Add a source file
    pub fn source<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.sources.push(path.as_ref().to_path_buf());
        self
    }

    /// Set output file
    pub fn output<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.output = Some(path.as_ref().to_path_buf());
        self
    }

    /// Set target architecture
    pub fn architecture(mut self, arch: Architecture) -> Self {
        self.architecture = Some(arch);
        self
    }

    /// Add a preprocessor define
    pub fn define<K: Into<String>, V: Into<String>>(mut self, key: K, value: V) -> Self {
        self.defines.insert(key.into(), Some(value.into()));
        self
    }

    /// Add a preprocessor define without value
    pub fn define_flag<K: Into<String>>(mut self, key: K) -> Self {
        self.defines.insert(key.into(), None);
        self
    }

    /// Add multiple defines from a map
    pub fn defines<I, K, V>(mut self, defines: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<String>,
    {
        for (k, v) in defines {
            self.defines.insert(k.into(), Some(v.into()));
        }
        self
    }

    /// Add a WiX extension
    pub fn extension<S: Into<String>>(mut self, ext: S) -> Self {
        self.extensions.push(ext.into());
        self
    }

    /// Add multiple extensions
    pub fn extensions<I, S>(mut self, extensions: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        for ext in extensions {
            self.extensions.push(ext.into());
        }
        self
    }

    /// Add an include path
    pub fn include_path<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.include_paths.push(path.as_ref().to_path_buf());
        self
    }

    /// Add a bind path
    pub fn bind_path<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.bind_paths.push(path.as_ref().to_path_buf());
        self
    }

    /// Add a localization file
    pub fn localization_file<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.localization_files.push(path.as_ref().to_path_buf());
        self
    }

    /// Set cultures
    pub fn cultures<I, S>(mut self, cultures: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.cultures = cultures.into_iter().map(Into::into).collect();
        self
    }

    /// Set intermediate directory
    pub fn intermediate_dir<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.intermediate_dir = Some(path.as_ref().to_path_buf());
        self
    }

    /// Set output type
    pub fn output_type(mut self, out_type: OutputType) -> Self {
        self.output_type = Some(out_type);
        self
    }

    /// Suppress a warning
    pub fn suppress_warning<S: Into<String>>(mut self, code: S) -> Self {
        self.suppress_warnings.push(code.into());
        self
    }

    /// Treat warnings as errors
    pub fn warnings_as_errors(mut self, enable: bool) -> Self {
        self.warnings_as_errors = enable;
        self
    }

    /// Skip ICE validation
    pub fn skip_validation(mut self, skip: bool) -> Self {
        self.skip_validation = skip;
        self
    }

    /// Enable verbose output
    pub fn verbose(mut self, enable: bool) -> Self {
        self.verbose = enable;
        self
    }

    /// Enable pedantic mode (v4+ only)
    pub fn pedantic(mut self, enable: bool) -> Self {
        self.pedantic = enable;
        self
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<(), BuildError> {
        if self.sources.is_empty() {
            return Err(BuildError::InvalidConfig("No source files specified".into()));
        }

        for source in &self.sources {
            if let Some(ext) = source.extension() {
                let ext = ext.to_string_lossy().to_lowercase();
                if ext != "wxs" && ext != "wixproj" {
                    return Err(BuildError::InvalidSource(format!(
                        "Expected .wxs or .wixproj file: {}",
                        source.display()
                    )));
                }
            } else {
                return Err(BuildError::InvalidSource(format!(
                    "File has no extension: {}",
                    source.display()
                )));
            }
        }

        if self.sources.len() > 1 && self.output.is_none() {
            return Err(BuildError::OutputRequired);
        }

        Ok(())
    }
}

/// Build profile with preset configurations
#[derive(Debug, Clone)]
pub struct BuildProfile {
    pub name: String,
    pub config: BuildConfig,
}

impl BuildProfile {
    /// Create a debug build profile
    pub fn debug<P: AsRef<Path>>(source: P) -> Self {
        Self {
            name: "debug".to_string(),
            config: BuildConfig::new(source)
                .define("Configuration", "Debug")
                .verbose(true),
        }
    }

    /// Create a release build profile
    pub fn release<P: AsRef<Path>>(source: P) -> Self {
        Self {
            name: "release".to_string(),
            config: BuildConfig::new(source)
                .define("Configuration", "Release")
                .warnings_as_errors(true),
        }
    }

    /// Create a CI/CD build profile
    pub fn ci<P: AsRef<Path>>(source: P) -> Self {
        Self {
            name: "ci".to_string(),
            config: BuildConfig::new(source)
                .define("Configuration", "Release")
                .warnings_as_errors(true)
                .pedantic(true),
        }
    }
}

/// Common WiX extensions
pub mod extensions {
    pub const UI: &str = "WixToolset.UI.wixext";
    pub const UTIL: &str = "WixToolset.Util.wixext";
    pub const FIREWALL: &str = "WixToolset.Firewall.wixext";
    pub const NETFX: &str = "WixToolset.Netfx.wixext";
    pub const DEPENDENCY: &str = "WixToolset.DependencyExtension.wixext";
    pub const BAL: &str = "WixToolset.Bal.wixext";
    pub const COMPLUS: &str = "WixToolset.ComPlus.wixext";
    pub const DIFXAPP: &str = "WixToolset.DifxApp.wixext";
    pub const DIRECTX: &str = "WixToolset.DirectX.wixext";
    pub const HTTP: &str = "WixToolset.Http.wixext";
    pub const IIS: &str = "WixToolset.Iis.wixext";
    pub const MSMQ: &str = "WixToolset.Msmq.wixext";
    pub const SQL: &str = "WixToolset.Sql.wixext";
    pub const VSX: &str = "WixToolset.VisualStudio.wixext";

    // Legacy WiX v3 extension names
    pub mod v3 {
        pub const UI: &str = "WixUIExtension";
        pub const UTIL: &str = "WixUtilExtension";
        pub const FIREWALL: &str = "WixFirewallExtension";
        pub const NETFX: &str = "WixNetFxExtension";
        pub const DEPENDENCY: &str = "WixDependencyExtension";
        pub const BAL: &str = "WixBalExtension";
        pub const COMPLUS: &str = "WixComPlusExtension";
        pub const DIFXAPP: &str = "WixDifxAppExtension";
        pub const DIRECTX: &str = "WixDirectXExtension";
        pub const HTTP: &str = "WixHttpExtension";
        pub const IIS: &str = "WixIIsExtension";
        pub const MSMQ: &str = "WixMsmqExtension";
        pub const SQL: &str = "WixSqlExtension";
        pub const VSX: &str = "WixVSExtension";
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_config_basic() {
        let config = BuildConfig::new("Product.wxs");
        assert_eq!(config.sources.len(), 1);
        assert_eq!(config.sources[0], PathBuf::from("Product.wxs"));
    }

    #[test]
    fn test_build_config_builder() {
        let config = BuildConfig::new("Product.wxs")
            .output("out/Product.msi")
            .architecture(Architecture::X64)
            .extension("WixUIExtension")
            .define("Version", "1.0.0")
            .define("ProductCode", "{12345}");

        assert_eq!(config.output, Some(PathBuf::from("out/Product.msi")));
        assert_eq!(config.architecture, Some(Architecture::X64));
        assert_eq!(config.extensions, vec!["WixUIExtension"]);
        assert_eq!(config.defines.len(), 2);
    }

    #[test]
    fn test_build_config_multiple_sources() {
        let config = BuildConfig::from_sources(["Product.wxs", "Features.wxs", "UI.wxs"])
            .output("Product.msi");

        assert_eq!(config.sources.len(), 3);
    }

    #[test]
    fn test_toolset_v3_command() {
        let toolset = WixToolset::with_version(WixVersion::V3);
        let config = BuildConfig::new("Product.wxs")
            .output("Product.msi")
            .architecture(Architecture::X64)
            .extension("WixUIExtension")
            .define("Version", "1.0.0");

        let cmd = toolset.build_command(&config);
        assert!(cmd.contains("candle.exe"));
        assert!(cmd.contains("light.exe"));
        assert!(cmd.contains("-arch x64"));
        assert!(cmd.contains("-ext WixUIExtension"));
        assert!(cmd.contains("-dVersion=1.0.0"));
    }

    #[test]
    fn test_toolset_v4_command() {
        let toolset = WixToolset::with_version(WixVersion::V4);
        let config = BuildConfig::new("Product.wxs")
            .output("Product.msi")
            .architecture(Architecture::X64)
            .extension("WixToolset.UI.wixext")
            .define("Version", "1.0.0");

        let cmd = toolset.build_command(&config);
        assert!(cmd.contains("wix build"));
        assert!(cmd.contains("-arch x64"));
        assert!(cmd.contains("-ext WixToolset.UI.wixext"));
        assert!(cmd.contains("-d Version=1.0.0"));
        assert!(cmd.contains("-o Product.msi"));
    }

    #[test]
    fn test_candle_command_separate() {
        let toolset = WixToolset::with_version(WixVersion::V3);
        let config = BuildConfig::new("Product.wxs")
            .architecture(Architecture::X86)
            .define("Debug", "true")
            .verbose(true);

        let (candle, _) = toolset.build_v3_commands(&config);
        assert!(candle.contains("candle.exe"));
        assert!(candle.contains("-arch x86"));
        assert!(candle.contains("-dDebug=true"));
        assert!(candle.contains("-v"));
    }

    #[test]
    fn test_light_command_separate() {
        let toolset = WixToolset::with_version(WixVersion::V3);
        let config = BuildConfig::new("Product.wxs")
            .output("Product.msi")
            .extension("WixUIExtension")
            .skip_validation(true)
            .cultures(["en-US", "fr-FR"]);

        let (_, light) = toolset.build_v3_commands(&config);
        assert!(light.contains("light.exe"));
        assert!(light.contains("-ext WixUIExtension"));
        assert!(light.contains("-sice:*"));
        assert!(light.contains("-cultures:en-US;fr-FR"));
        assert!(light.contains("-out Product.msi"));
    }

    #[test]
    fn test_v4_output_type() {
        let toolset = WixToolset::with_version(WixVersion::V4);
        let config = BuildConfig::new("Bundle.wxs")
            .output("Setup.exe")
            .output_type(OutputType::Bundle);

        let cmd = toolset.build_command(&config);
        assert!(cmd.contains("-outputType bundle"));
    }

    #[test]
    fn test_v4_pedantic() {
        let toolset = WixToolset::with_version(WixVersion::V4);
        let config = BuildConfig::new("Product.wxs").pedantic(true);

        let cmd = toolset.build_command(&config);
        assert!(cmd.contains("-pedantic"));
    }

    #[test]
    fn test_validate_no_sources() {
        let config = BuildConfig::default();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_invalid_extension() {
        let config = BuildConfig::new("Product.txt");
        let result = config.validate();
        assert!(result.is_err());
        if let Err(BuildError::InvalidSource(msg)) = result {
            assert!(msg.contains("Product.txt"));
        }
    }

    #[test]
    fn test_validate_multiple_sources_no_output() {
        let config = BuildConfig::from_sources(["A.wxs", "B.wxs"]);
        let result = config.validate();
        assert!(matches!(result, Err(BuildError::OutputRequired)));
    }

    #[test]
    fn test_validate_success() {
        let config = BuildConfig::new("Product.wxs").output("Product.msi");
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_architecture_from_str() {
        assert_eq!(Architecture::from_str("x86"), Some(Architecture::X86));
        assert_eq!(Architecture::from_str("X64"), Some(Architecture::X64));
        assert_eq!(Architecture::from_str("arm64"), Some(Architecture::Arm64));
        assert_eq!(Architecture::from_str("win32"), Some(Architecture::X86));
        assert_eq!(Architecture::from_str("amd64"), Some(Architecture::X64));
        assert_eq!(Architecture::from_str("unknown"), None);
    }

    #[test]
    fn test_output_type_from_str() {
        assert_eq!(OutputType::from_str("msi"), Some(OutputType::Msi));
        assert_eq!(OutputType::from_str("bundle"), Some(OutputType::Bundle));
        assert_eq!(OutputType::from_str("wixlib"), Some(OutputType::Wixlib));
        assert_eq!(OutputType::from_str("unknown"), None);
    }

    #[test]
    fn test_debug_profile() {
        let profile = BuildProfile::debug("Product.wxs");
        assert_eq!(profile.name, "debug");
        assert!(profile.config.verbose);
    }

    #[test]
    fn test_release_profile() {
        let profile = BuildProfile::release("Product.wxs");
        assert_eq!(profile.name, "release");
        assert!(profile.config.warnings_as_errors);
    }

    #[test]
    fn test_ci_profile() {
        let profile = BuildProfile::ci("Product.wxs");
        assert_eq!(profile.name, "ci");
        assert!(profile.config.pedantic);
        assert!(profile.config.warnings_as_errors);
    }

    #[test]
    fn test_wix_version_as_str() {
        assert_eq!(WixVersion::V3.as_str(), "3.x");
        assert_eq!(WixVersion::V4.as_str(), "4.x");
        assert_eq!(WixVersion::V5.as_str(), "5.x");
    }

    #[test]
    fn test_bind_paths() {
        let toolset = WixToolset::with_version(WixVersion::V3);
        let config = BuildConfig::new("Product.wxs")
            .output("Product.msi")
            .bind_path("files/")
            .bind_path("resources/");

        let (_, light) = toolset.build_v3_commands(&config);
        assert!(light.contains("-b files/"));
        assert!(light.contains("-b resources/"));
    }

    #[test]
    fn test_localization_files() {
        let toolset = WixToolset::with_version(WixVersion::V4);
        let config = BuildConfig::new("Product.wxs")
            .localization_file("en-US.wxl")
            .localization_file("fr-FR.wxl");

        let cmd = toolset.build_command(&config);
        assert!(cmd.contains("-loc en-US.wxl"));
        assert!(cmd.contains("-loc fr-FR.wxl"));
    }

    #[test]
    fn test_suppress_warnings() {
        let toolset = WixToolset::with_version(WixVersion::V3);
        let config = BuildConfig::new("Product.wxs")
            .output("Product.msi")
            .suppress_warning("1076")
            .suppress_warning("1079");

        let (candle, _) = toolset.build_v3_commands(&config);
        assert!(candle.contains("-sw1076"));
        assert!(candle.contains("-sw1079"));
    }

    #[test]
    fn test_include_paths() {
        let toolset = WixToolset::with_version(WixVersion::V3);
        let config = BuildConfig::new("Product.wxs")
            .include_path("inc/")
            .include_path("common/");

        let (candle, _) = toolset.build_v3_commands(&config);
        assert!(candle.contains("-I inc/"));
        assert!(candle.contains("-I common/"));
    }

    #[test]
    fn test_define_flag() {
        let toolset = WixToolset::with_version(WixVersion::V3);
        let config = BuildConfig::new("Product.wxs")
            .define_flag("EnableFeature");

        let (candle, _) = toolset.build_v3_commands(&config);
        assert!(candle.contains("-dEnableFeature"));
    }

    #[test]
    fn test_intermediate_dir() {
        let toolset = WixToolset::with_version(WixVersion::V4);
        let config = BuildConfig::new("Product.wxs")
            .intermediate_dir("obj/");

        let cmd = toolset.build_command(&config);
        assert!(cmd.contains("-intermediateFolder obj/"));
    }
}

//! wixcraft - Universal installer tooling framework
//!
//! The main orchestrator for WixCraft tools, providing project management,
//! build coordination, and tool discovery.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Project type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProjectType {
    /// Standard WiX installer
    Wix,
    /// Bundle (bootstrapper)
    Bundle,
    /// Merge module
    MergeModule,
    /// Patch
    Patch,
    /// Transform
    Transform,
}

impl ProjectType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ProjectType::Wix => "wix",
            ProjectType::Bundle => "bundle",
            ProjectType::MergeModule => "merge-module",
            ProjectType::Patch => "patch",
            ProjectType::Transform => "transform",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            ProjectType::Wix => "WiX installer project",
            ProjectType::Bundle => "Bootstrapper bundle project",
            ProjectType::MergeModule => "Merge module project",
            ProjectType::Patch => "Patch project",
            ProjectType::Transform => "Transform project",
        }
    }
}

/// Project configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub name: String,
    pub version: String,
    pub project_type: ProjectType,
    pub output_dir: PathBuf,
    pub source_files: Vec<PathBuf>,
    pub extensions: Vec<String>,
    pub variables: HashMap<String, String>,
    pub include_paths: Vec<PathBuf>,
    pub platform: Platform,
}

impl Default for ProjectConfig {
    fn default() -> Self {
        Self {
            name: "MyProject".to_string(),
            version: "1.0.0".to_string(),
            project_type: ProjectType::Wix,
            output_dir: PathBuf::from("bin"),
            source_files: Vec::new(),
            extensions: Vec::new(),
            variables: HashMap::new(),
            include_paths: Vec::new(),
            platform: Platform::X64,
        }
    }
}

impl ProjectConfig {
    pub fn new(name: &str, project_type: ProjectType) -> Self {
        Self {
            name: name.to_string(),
            project_type,
            ..Default::default()
        }
    }

    pub fn with_version(mut self, version: &str) -> Self {
        self.version = version.to_string();
        self
    }

    pub fn with_source(mut self, path: PathBuf) -> Self {
        self.source_files.push(path);
        self
    }

    pub fn with_extension(mut self, extension: &str) -> Self {
        self.extensions.push(extension.to_string());
        self
    }

    pub fn with_variable(mut self, key: &str, value: &str) -> Self {
        self.variables.insert(key.to_string(), value.to_string());
        self
    }

    pub fn with_output_dir(mut self, path: PathBuf) -> Self {
        self.output_dir = path;
        self
    }

    pub fn with_platform(mut self, platform: Platform) -> Self {
        self.platform = platform;
        self
    }
}

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
}

/// Build configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildConfig {
    pub debug: bool,
    pub verbose: bool,
    pub parallel: bool,
    pub suppress_ices: Vec<String>,
    pub suppress_warnings: Vec<String>,
    pub treat_warnings_as_errors: bool,
}

impl Default for BuildConfig {
    fn default() -> Self {
        Self {
            debug: false,
            verbose: false,
            parallel: true,
            suppress_ices: Vec::new(),
            suppress_warnings: Vec::new(),
            treat_warnings_as_errors: false,
        }
    }
}

impl BuildConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn debug(mut self) -> Self {
        self.debug = true;
        self
    }

    pub fn verbose(mut self) -> Self {
        self.verbose = true;
        self
    }

    pub fn suppress_ice(mut self, ice: &str) -> Self {
        self.suppress_ices.push(ice.to_string());
        self
    }
}

/// Tool registration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    pub name: String,
    pub description: String,
    pub version: String,
    pub path: Option<PathBuf>,
    pub category: ToolCategory,
}

impl Tool {
    pub fn new(name: &str, description: &str, category: ToolCategory) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            version: "0.1.0".to_string(),
            path: None,
            category,
        }
    }

    pub fn with_version(mut self, version: &str) -> Self {
        self.version = version.to_string();
        self
    }

    pub fn with_path(mut self, path: PathBuf) -> Self {
        self.path = Some(path);
        self
    }
}

/// Tool category
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ToolCategory {
    Authoring,
    Build,
    Debug,
    Runtime,
    Analytics,
    Ide,
    Core,
}

impl ToolCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            ToolCategory::Authoring => "authoring",
            ToolCategory::Build => "build",
            ToolCategory::Debug => "debug",
            ToolCategory::Runtime => "runtime",
            ToolCategory::Analytics => "analytics",
            ToolCategory::Ide => "ide",
            ToolCategory::Core => "core",
        }
    }
}

/// Tool registry
#[derive(Debug, Clone, Default)]
pub struct ToolRegistry {
    tools: HashMap<String, Tool>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, tool: Tool) {
        self.tools.insert(tool.name.clone(), tool);
    }

    pub fn get(&self, name: &str) -> Option<&Tool> {
        self.tools.get(name)
    }

    pub fn get_by_category(&self, category: ToolCategory) -> Vec<&Tool> {
        self.tools
            .values()
            .filter(|t| t.category == category)
            .collect()
    }

    pub fn all(&self) -> impl Iterator<Item = &Tool> {
        self.tools.values()
    }

    /// Register all built-in WixCraft tools
    pub fn register_builtin_tools(&mut self) {
        // Authoring tools
        self.register(Tool::new("wix-init", "Initialize new WiX project", ToolCategory::Authoring));
        self.register(Tool::new("wix-harvest", "Harvest files for WiX", ToolCategory::Authoring));
        self.register(Tool::new("wix-guid", "GUID generation utilities", ToolCategory::Authoring));
        self.register(Tool::new("wix-ui", "UI sequence generator", ToolCategory::Authoring));
        self.register(Tool::new("wix-wizard", "Interactive project wizard", ToolCategory::Authoring));
        self.register(Tool::new("wix-license", "License file generator", ToolCategory::Authoring));
        self.register(Tool::new("wix-env", "Environment variable helper", ToolCategory::Authoring));
        self.register(Tool::new("wix-prereq", "Prerequisite manager", ToolCategory::Authoring));
        self.register(Tool::new("wix-patch", "Patch/update generator", ToolCategory::Authoring));

        // Build tools
        self.register(Tool::new("wix-build", "Build WiX projects", ToolCategory::Build));
        self.register(Tool::new("wix-bundle", "Bundle builder", ToolCategory::Build));
        self.register(Tool::new("wix-ci", "CI/CD integration", ToolCategory::Build));

        // Debug tools
        self.register(Tool::new("wix-doctor", "Project diagnostics", ToolCategory::Debug));
        self.register(Tool::new("wix-diff", "Compare MSI files", ToolCategory::Debug));
        self.register(Tool::new("wix-test", "Testing framework", ToolCategory::Debug));
        self.register(Tool::new("wix-preview", "Preview installer", ToolCategory::Debug));
        self.register(Tool::new("wix-lux", "Custom action tester", ToolCategory::Debug));

        // Runtime tools
        self.register(Tool::new("wix-install", "MSI installer runner", ToolCategory::Runtime));
        self.register(Tool::new("wix-update", "Update manager", ToolCategory::Runtime));
        self.register(Tool::new("wix-uninstall", "Clean uninstaller", ToolCategory::Runtime));
        self.register(Tool::new("wix-repl", "Interactive REPL", ToolCategory::Runtime));

        // Analytics tools
        self.register(Tool::new("wix-analytics", "Installation analytics", ToolCategory::Analytics));
        self.register(Tool::new("wix-repair", "Repair tool", ToolCategory::Analytics));
        self.register(Tool::new("wix-silent", "Silent config generator", ToolCategory::Analytics));
        self.register(Tool::new("wix-msix", "MSIX converter", ToolCategory::Analytics));
        self.register(Tool::new("wix-deps", "Dependency analyzer", ToolCategory::Analytics));
        self.register(Tool::new("license-detector", "License detector", ToolCategory::Analytics));

        // IDE tools
        self.register(Tool::new("wix-vscode", "VS Code extension", ToolCategory::Ide));
        self.register(Tool::new("wix-sublime", "Sublime Text package", ToolCategory::Ide));
        self.register(Tool::new("wix-syntax", "Syntax highlighter", ToolCategory::Ide));
        self.register(Tool::new("wix-snippets", "Code snippets", ToolCategory::Ide));

        // Core tools
        self.register(Tool::new("wixcraft", "Main orchestrator", ToolCategory::Core));
        self.register(Tool::new("wix-msi", "MSI builder", ToolCategory::Core));
    }
}

/// Build result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildResult {
    pub success: bool,
    pub output_file: Option<PathBuf>,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
    pub duration_secs: f64,
}

impl BuildResult {
    pub fn success(output: PathBuf, duration: f64) -> Self {
        Self {
            success: true,
            output_file: Some(output),
            warnings: Vec::new(),
            errors: Vec::new(),
            duration_secs: duration,
        }
    }

    pub fn failure(errors: Vec<String>) -> Self {
        Self {
            success: false,
            output_file: None,
            warnings: Vec::new(),
            errors,
            duration_secs: 0.0,
        }
    }

    pub fn with_warning(mut self, warning: &str) -> Self {
        self.warnings.push(warning.to_string());
        self
    }
}

/// Project manifest (wixcraft.json)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectManifest {
    pub project: ProjectConfig,
    pub build: BuildConfig,
    pub tools: HashMap<String, serde_json::Value>,
}

impl Default for ProjectManifest {
    fn default() -> Self {
        Self {
            project: ProjectConfig::default(),
            build: BuildConfig::default(),
            tools: HashMap::new(),
        }
    }
}

impl ProjectManifest {
    pub fn new(project: ProjectConfig) -> Self {
        Self {
            project,
            ..Default::default()
        }
    }

    pub fn with_build_config(mut self, config: BuildConfig) -> Self {
        self.build = config;
        self
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_default()
    }

    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

/// Command executor
pub struct CommandExecutor;

impl CommandExecutor {
    /// Execute a tool command
    pub fn execute(_tool: &str, _args: &[String]) -> Result<String, String> {
        // Would execute the tool and return output
        Ok("Command executed successfully".to_string())
    }

    /// Check if a tool is available
    pub fn is_available(tool: &str) -> bool {
        // Would check if tool exists in PATH or registry
        !tool.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_type_as_str() {
        assert_eq!(ProjectType::Wix.as_str(), "wix");
        assert_eq!(ProjectType::Bundle.as_str(), "bundle");
    }

    #[test]
    fn test_project_type_description() {
        let desc = ProjectType::Wix.description();
        assert!(!desc.is_empty());
    }

    #[test]
    fn test_project_config_new() {
        let config = ProjectConfig::new("MyApp", ProjectType::Wix);
        assert_eq!(config.name, "MyApp");
        assert_eq!(config.project_type, ProjectType::Wix);
    }

    #[test]
    fn test_project_config_with_version() {
        let config = ProjectConfig::new("MyApp", ProjectType::Wix).with_version("2.0.0");
        assert_eq!(config.version, "2.0.0");
    }

    #[test]
    fn test_project_config_with_source() {
        let config = ProjectConfig::new("MyApp", ProjectType::Wix)
            .with_source(PathBuf::from("Product.wxs"));
        assert_eq!(config.source_files.len(), 1);
    }

    #[test]
    fn test_project_config_with_extension() {
        let config = ProjectConfig::new("MyApp", ProjectType::Wix)
            .with_extension("WixUIExtension");
        assert!(config.extensions.contains(&"WixUIExtension".to_string()));
    }

    #[test]
    fn test_project_config_with_variable() {
        let config = ProjectConfig::new("MyApp", ProjectType::Wix)
            .with_variable("Version", "1.0.0");
        assert_eq!(config.variables.get("Version"), Some(&"1.0.0".to_string()));
    }

    #[test]
    fn test_platform_as_str() {
        assert_eq!(Platform::X64.as_str(), "x64");
        assert_eq!(Platform::Arm64.as_str(), "arm64");
    }

    #[test]
    fn test_build_config_new() {
        let config = BuildConfig::new();
        assert!(!config.debug);
        assert!(config.parallel);
    }

    #[test]
    fn test_build_config_debug() {
        let config = BuildConfig::new().debug();
        assert!(config.debug);
    }

    #[test]
    fn test_build_config_verbose() {
        let config = BuildConfig::new().verbose();
        assert!(config.verbose);
    }

    #[test]
    fn test_build_config_suppress_ice() {
        let config = BuildConfig::new().suppress_ice("ICE03");
        assert!(config.suppress_ices.contains(&"ICE03".to_string()));
    }

    #[test]
    fn test_tool_new() {
        let tool = Tool::new("wix-build", "Build projects", ToolCategory::Build);
        assert_eq!(tool.name, "wix-build");
        assert_eq!(tool.category, ToolCategory::Build);
    }

    #[test]
    fn test_tool_with_version() {
        let tool = Tool::new("wix-build", "Build", ToolCategory::Build).with_version("1.0.0");
        assert_eq!(tool.version, "1.0.0");
    }

    #[test]
    fn test_tool_category_as_str() {
        assert_eq!(ToolCategory::Build.as_str(), "build");
        assert_eq!(ToolCategory::Authoring.as_str(), "authoring");
    }

    #[test]
    fn test_tool_registry_new() {
        let registry = ToolRegistry::new();
        assert!(registry.tools.is_empty());
    }

    #[test]
    fn test_tool_registry_register() {
        let mut registry = ToolRegistry::new();
        registry.register(Tool::new("test", "Test", ToolCategory::Debug));
        assert!(registry.get("test").is_some());
    }

    #[test]
    fn test_tool_registry_get_by_category() {
        let mut registry = ToolRegistry::new();
        registry.register(Tool::new("tool1", "Tool 1", ToolCategory::Build));
        registry.register(Tool::new("tool2", "Tool 2", ToolCategory::Build));
        registry.register(Tool::new("tool3", "Tool 3", ToolCategory::Debug));

        let build_tools = registry.get_by_category(ToolCategory::Build);
        assert_eq!(build_tools.len(), 2);
    }

    #[test]
    fn test_tool_registry_builtin() {
        let mut registry = ToolRegistry::new();
        registry.register_builtin_tools();
        assert!(registry.get("wix-build").is_some());
        assert!(registry.get("wix-install").is_some());
    }

    #[test]
    fn test_build_result_success() {
        let result = BuildResult::success(PathBuf::from("output.msi"), 10.5);
        assert!(result.success);
        assert!(result.output_file.is_some());
    }

    #[test]
    fn test_build_result_failure() {
        let result = BuildResult::failure(vec!["Error".to_string()]);
        assert!(!result.success);
        assert_eq!(result.errors.len(), 1);
    }

    #[test]
    fn test_build_result_with_warning() {
        let result = BuildResult::success(PathBuf::from("out.msi"), 5.0)
            .with_warning("Warning message");
        assert_eq!(result.warnings.len(), 1);
    }

    #[test]
    fn test_project_manifest_new() {
        let config = ProjectConfig::new("MyApp", ProjectType::Wix);
        let manifest = ProjectManifest::new(config);
        assert_eq!(manifest.project.name, "MyApp");
    }

    #[test]
    fn test_project_manifest_to_json() {
        let config = ProjectConfig::new("MyApp", ProjectType::Wix);
        let manifest = ProjectManifest::new(config);
        let json = manifest.to_json();
        assert!(json.contains("MyApp"));
    }

    #[test]
    fn test_project_manifest_from_json() {
        let json = r#"{"project":{"name":"Test","version":"1.0.0","project_type":"Wix","output_dir":"bin","source_files":[],"extensions":[],"variables":{},"include_paths":[],"platform":"X64"},"build":{"debug":false,"verbose":false,"parallel":true,"suppress_ices":[],"suppress_warnings":[],"treat_warnings_as_errors":false},"tools":{}}"#;
        let manifest = ProjectManifest::from_json(json).unwrap();
        assert_eq!(manifest.project.name, "Test");
    }

    #[test]
    fn test_command_executor_is_available() {
        assert!(CommandExecutor::is_available("wix-build"));
    }
}

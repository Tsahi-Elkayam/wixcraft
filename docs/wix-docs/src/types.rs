//! Core types for wix-docs

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// A documented WiX project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectDocs {
    /// Project name
    pub name: String,
    /// Project description
    pub description: Option<String>,
    /// Project version
    pub version: Option<String>,
    /// Source files
    pub files: Vec<FileDocs>,
    /// All components
    pub components: Vec<ComponentDocs>,
    /// All features
    pub features: Vec<FeatureDocs>,
    /// All directories
    pub directories: Vec<DirectoryDocs>,
    /// All custom actions
    pub custom_actions: Vec<CustomActionDocs>,
    /// All properties
    pub properties: Vec<PropertyDocs>,
    /// Cross-references
    pub references: Vec<Reference>,
}

impl ProjectDocs {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
            version: None,
            files: Vec::new(),
            components: Vec::new(),
            features: Vec::new(),
            directories: Vec::new(),
            custom_actions: Vec::new(),
            properties: Vec::new(),
            references: Vec::new(),
        }
    }

    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = Some(version.into());
        self
    }
}

/// Documentation for a single file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileDocs {
    /// File path
    pub path: PathBuf,
    /// File-level documentation
    pub description: Option<String>,
    /// Elements defined in this file
    pub elements: Vec<String>,
}

impl FileDocs {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            description: None,
            elements: Vec::new(),
        }
    }
}

/// Documentation for a component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentDocs {
    /// Component Id
    pub id: String,
    /// Component GUID
    pub guid: Option<String>,
    /// Description from doc comment
    pub description: Option<String>,
    /// Source file
    pub file: PathBuf,
    /// Line number
    pub line: usize,
    /// Files installed by this component
    pub files: Vec<FileEntry>,
    /// Registry entries
    pub registry: Vec<RegistryEntry>,
    /// Services installed
    pub services: Vec<ServiceEntry>,
    /// Features that include this component
    pub included_in_features: Vec<String>,
}

impl ComponentDocs {
    pub fn new(id: impl Into<String>, file: PathBuf, line: usize) -> Self {
        Self {
            id: id.into(),
            guid: None,
            description: None,
            file,
            line,
            files: Vec::new(),
            registry: Vec::new(),
            services: Vec::new(),
            included_in_features: Vec::new(),
        }
    }
}

/// A file installed by a component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    /// File Id
    pub id: Option<String>,
    /// Source path
    pub source: Option<String>,
    /// Target name
    pub name: Option<String>,
    /// Description
    pub description: Option<String>,
}

/// A registry entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryEntry {
    /// Registry root (HKLM, HKCU, etc.)
    pub root: String,
    /// Registry key path
    pub key: String,
    /// Value name
    pub name: Option<String>,
    /// Value type
    pub value_type: Option<String>,
    /// Description
    pub description: Option<String>,
}

/// A service installation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceEntry {
    /// Service name
    pub name: String,
    /// Display name
    pub display_name: Option<String>,
    /// Service type
    pub service_type: Option<String>,
    /// Start type
    pub start_type: Option<String>,
    /// Description
    pub description: Option<String>,
}

/// Documentation for a feature
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureDocs {
    /// Feature Id
    pub id: String,
    /// Feature title
    pub title: Option<String>,
    /// Feature level (install level)
    pub level: Option<String>,
    /// Description from doc comment
    pub description: Option<String>,
    /// Source file
    pub file: PathBuf,
    /// Line number
    pub line: usize,
    /// Components included
    pub components: Vec<String>,
    /// Child features
    pub children: Vec<String>,
    /// Parent feature
    pub parent: Option<String>,
}

impl FeatureDocs {
    pub fn new(id: impl Into<String>, file: PathBuf, line: usize) -> Self {
        Self {
            id: id.into(),
            title: None,
            level: None,
            description: None,
            file,
            line,
            components: Vec::new(),
            children: Vec::new(),
            parent: None,
        }
    }
}

/// Documentation for a directory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectoryDocs {
    /// Directory Id
    pub id: String,
    /// Directory name
    pub name: Option<String>,
    /// Description from doc comment
    pub description: Option<String>,
    /// Source file
    pub file: PathBuf,
    /// Line number
    pub line: usize,
    /// Parent directory
    pub parent: Option<String>,
    /// Child directories
    pub children: Vec<String>,
    /// Components in this directory
    pub components: Vec<String>,
}

impl DirectoryDocs {
    pub fn new(id: impl Into<String>, file: PathBuf, line: usize) -> Self {
        Self {
            id: id.into(),
            name: None,
            description: None,
            file,
            line,
            parent: None,
            children: Vec::new(),
            components: Vec::new(),
        }
    }
}

/// Documentation for a custom action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomActionDocs {
    /// CustomAction Id
    pub id: String,
    /// Description from doc comment
    pub description: Option<String>,
    /// Source file
    pub file: PathBuf,
    /// Line number
    pub line: usize,
    /// Binary reference
    pub binary: Option<String>,
    /// DLL entry point
    pub dll_entry: Option<String>,
    /// Script content (for inline scripts)
    pub script: Option<String>,
    /// Execute attribute
    pub execute: Option<String>,
    /// Return attribute
    pub return_attr: Option<String>,
    /// Impersonate attribute
    pub impersonate: Option<String>,
    /// When scheduled (InstallExecuteSequence, etc.)
    pub scheduled_in: Vec<ScheduleEntry>,
}

impl CustomActionDocs {
    pub fn new(id: impl Into<String>, file: PathBuf, line: usize) -> Self {
        Self {
            id: id.into(),
            description: None,
            file,
            line,
            binary: None,
            dll_entry: None,
            script: None,
            execute: None,
            return_attr: None,
            impersonate: None,
            scheduled_in: Vec::new(),
        }
    }
}

/// A schedule entry for a custom action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleEntry {
    /// Sequence name (InstallExecuteSequence, etc.)
    pub sequence: String,
    /// Condition
    pub condition: Option<String>,
    /// Before action
    pub before: Option<String>,
    /// After action
    pub after: Option<String>,
}

/// Documentation for a property
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropertyDocs {
    /// Property Id
    pub id: String,
    /// Default value
    pub value: Option<String>,
    /// Description from doc comment
    pub description: Option<String>,
    /// Source file
    pub file: PathBuf,
    /// Line number
    pub line: usize,
    /// Is secure property
    pub secure: bool,
    /// Is admin property
    pub admin: bool,
    /// Is hidden property
    pub hidden: bool,
    /// Where this property is used
    pub used_in: Vec<PropertyUsage>,
}

impl PropertyDocs {
    pub fn new(id: impl Into<String>, file: PathBuf, line: usize) -> Self {
        Self {
            id: id.into(),
            value: None,
            description: None,
            file,
            line,
            secure: false,
            admin: false,
            hidden: false,
            used_in: Vec::new(),
        }
    }
}

/// Where a property is used
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropertyUsage {
    /// Usage context (Condition, Value, etc.)
    pub context: String,
    /// Element where used
    pub element: String,
    /// File
    pub file: PathBuf,
    /// Line number
    pub line: usize,
}

/// A cross-reference between elements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reference {
    /// Source element type
    pub from_type: String,
    /// Source element Id
    pub from_id: String,
    /// Target element type
    pub to_type: String,
    /// Target element Id
    pub to_id: String,
    /// Reference type (includes, references, parent, child)
    pub ref_type: ReferenceType,
}

/// Type of reference
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReferenceType {
    /// Feature includes Component
    Includes,
    /// Element references another (ComponentRef, DirectoryRef)
    References,
    /// Parent-child relationship
    Parent,
    /// Child-parent relationship
    Child,
    /// Uses (property usage)
    Uses,
}

/// Output format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Html,
    Markdown,
    Json,
}

impl std::str::FromStr for OutputFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "html" => Ok(Self::Html),
            "markdown" | "md" => Ok(Self::Markdown),
            "json" => Ok(Self::Json),
            _ => Err(format!("Unknown format: {}", s)),
        }
    }
}

/// Documentation configuration
#[derive(Debug, Clone)]
pub struct DocsConfig {
    /// Output format
    pub format: OutputFormat,
    /// Output directory
    pub output_dir: PathBuf,
    /// Include private elements (starting with _)
    pub include_private: bool,
    /// Generate table of contents
    pub generate_toc: bool,
    /// Generate cross-reference graph
    pub generate_graph: bool,
    /// Project name override
    pub project_name: Option<String>,
}

impl Default for DocsConfig {
    fn default() -> Self {
        Self {
            format: OutputFormat::Html,
            output_dir: PathBuf::from("docs"),
            include_private: false,
            generate_toc: true,
            generate_graph: true,
            project_name: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_docs_new() {
        let project = ProjectDocs::new("TestProject");
        assert_eq!(project.name, "TestProject");
        assert!(project.description.is_none());
        assert!(project.files.is_empty());
    }

    #[test]
    fn test_project_docs_with_description() {
        let project = ProjectDocs::new("Test")
            .with_description("A test project")
            .with_version("1.0.0");
        assert_eq!(project.description, Some("A test project".to_string()));
        assert_eq!(project.version, Some("1.0.0".to_string()));
    }

    #[test]
    fn test_file_docs_new() {
        let file = FileDocs::new(PathBuf::from("test.wxs"));
        assert_eq!(file.path, PathBuf::from("test.wxs"));
        assert!(file.description.is_none());
    }

    #[test]
    fn test_component_docs_new() {
        let comp = ComponentDocs::new("C1", PathBuf::from("test.wxs"), 10);
        assert_eq!(comp.id, "C1");
        assert_eq!(comp.line, 10);
        assert!(comp.files.is_empty());
    }

    #[test]
    fn test_feature_docs_new() {
        let feature = FeatureDocs::new("F1", PathBuf::from("test.wxs"), 5);
        assert_eq!(feature.id, "F1");
        assert!(feature.components.is_empty());
    }

    #[test]
    fn test_directory_docs_new() {
        let dir = DirectoryDocs::new("INSTALLDIR", PathBuf::from("test.wxs"), 3);
        assert_eq!(dir.id, "INSTALLDIR");
        assert!(dir.parent.is_none());
    }

    #[test]
    fn test_custom_action_docs_new() {
        let ca = CustomActionDocs::new("CA_Install", PathBuf::from("test.wxs"), 20);
        assert_eq!(ca.id, "CA_Install");
        assert!(ca.scheduled_in.is_empty());
    }

    #[test]
    fn test_property_docs_new() {
        let prop = PropertyDocs::new("INSTALLDIR", PathBuf::from("test.wxs"), 15);
        assert_eq!(prop.id, "INSTALLDIR");
        assert!(!prop.secure);
    }

    #[test]
    fn test_output_format_from_str() {
        assert_eq!("html".parse::<OutputFormat>().unwrap(), OutputFormat::Html);
        assert_eq!("markdown".parse::<OutputFormat>().unwrap(), OutputFormat::Markdown);
        assert_eq!("md".parse::<OutputFormat>().unwrap(), OutputFormat::Markdown);
        assert_eq!("json".parse::<OutputFormat>().unwrap(), OutputFormat::Json);
        assert!("unknown".parse::<OutputFormat>().is_err());
    }

    #[test]
    fn test_docs_config_default() {
        let config = DocsConfig::default();
        assert_eq!(config.format, OutputFormat::Html);
        assert!(!config.include_private);
        assert!(config.generate_toc);
    }

    #[test]
    fn test_reference_type() {
        let ref1 = Reference {
            from_type: "Feature".to_string(),
            from_id: "F1".to_string(),
            to_type: "Component".to_string(),
            to_id: "C1".to_string(),
            ref_type: ReferenceType::Includes,
        };
        assert_eq!(ref1.ref_type, ReferenceType::Includes);
    }

    #[test]
    fn test_file_entry() {
        let entry = FileEntry {
            id: Some("F1".to_string()),
            source: Some("app.exe".to_string()),
            name: None,
            description: None,
        };
        assert_eq!(entry.id, Some("F1".to_string()));
    }

    #[test]
    fn test_registry_entry() {
        let entry = RegistryEntry {
            root: "HKLM".to_string(),
            key: "SOFTWARE\\MyApp".to_string(),
            name: Some("Version".to_string()),
            value_type: Some("string".to_string()),
            description: None,
        };
        assert_eq!(entry.root, "HKLM");
    }

    #[test]
    fn test_service_entry() {
        let entry = ServiceEntry {
            name: "MyService".to_string(),
            display_name: Some("My Service".to_string()),
            service_type: Some("ownProcess".to_string()),
            start_type: Some("auto".to_string()),
            description: None,
        };
        assert_eq!(entry.name, "MyService");
    }

    #[test]
    fn test_schedule_entry() {
        let entry = ScheduleEntry {
            sequence: "InstallExecuteSequence".to_string(),
            condition: Some("NOT Installed".to_string()),
            before: None,
            after: Some("InstallFiles".to_string()),
        };
        assert_eq!(entry.sequence, "InstallExecuteSequence");
    }

    #[test]
    fn test_property_usage() {
        let usage = PropertyUsage {
            context: "Condition".to_string(),
            element: "Component".to_string(),
            file: PathBuf::from("test.wxs"),
            line: 10,
        };
        assert_eq!(usage.context, "Condition");
    }
}

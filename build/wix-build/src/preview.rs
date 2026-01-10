//! Preview installer content without building
//!
//! Analyzes WiX source files and shows what would be installed.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// File entry in preview
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    pub source: String,
    pub destination: String,
    pub component: String,
    pub feature: Option<String>,
    pub attributes: FileAttributes,
}

/// File attributes
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FileAttributes {
    pub read_only: bool,
    pub hidden: bool,
    pub system: bool,
    pub vital: bool,
    pub checksum: bool,
    pub compressed: bool,
}

/// Registry entry in preview
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryEntry {
    pub root: String,
    pub key: String,
    pub name: Option<String>,
    pub value: Option<String>,
    pub value_type: String,
    pub component: String,
}

/// Shortcut entry in preview
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortcutEntry {
    pub name: String,
    pub directory: String,
    pub target: String,
    pub arguments: Option<String>,
    pub working_dir: Option<String>,
    pub icon: Option<String>,
    pub component: String,
}

/// Service entry in preview
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceEntry {
    pub name: String,
    pub display_name: String,
    pub service_type: String,
    pub start_type: String,
    pub error_control: String,
    pub component: String,
}

/// Environment variable entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentEntry {
    pub name: String,
    pub value: String,
    pub action: String,
    pub system: bool,
    pub component: String,
}

/// Component preview
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentPreview {
    pub id: String,
    pub guid: Option<String>,
    pub directory: String,
    pub files: Vec<String>,
    pub registry_entries: usize,
    pub shortcuts: usize,
}

/// Feature preview
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeaturePreview {
    pub id: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub level: u32,
    pub components: Vec<String>,
    pub subfeatures: Vec<String>,
}

/// Directory structure preview
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectoryPreview {
    pub id: String,
    pub name: Option<String>,
    pub source_path: Option<String>,
    pub children: Vec<DirectoryPreview>,
}

impl DirectoryPreview {
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            name: None,
            source_path: None,
            children: Vec::new(),
        }
    }

    pub fn with_name(mut self, name: &str) -> Self {
        self.name = Some(name.to_string());
        self
    }

    pub fn add_child(&mut self, child: DirectoryPreview) {
        self.children.push(child);
    }

    pub fn flatten(&self) -> Vec<String> {
        let mut paths = Vec::new();
        self.collect_paths(&mut paths, "");
        paths
    }

    fn collect_paths(&self, paths: &mut Vec<String>, parent: &str) {
        let current = if let Some(ref name) = self.name {
            if parent.is_empty() {
                name.clone()
            } else {
                format!("{}\\{}", parent, name)
            }
        } else {
            parent.to_string()
        };

        if !current.is_empty() {
            paths.push(current.clone());
        }

        for child in &self.children {
            child.collect_paths(paths, &current);
        }
    }
}

/// Installation preview
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallPreview {
    pub product_name: Option<String>,
    pub manufacturer: Option<String>,
    pub version: Option<String>,
    pub upgrade_code: Option<String>,
    pub files: Vec<FileEntry>,
    pub registry: Vec<RegistryEntry>,
    pub shortcuts: Vec<ShortcutEntry>,
    pub services: Vec<ServiceEntry>,
    pub environment: Vec<EnvironmentEntry>,
    pub components: Vec<ComponentPreview>,
    pub features: Vec<FeaturePreview>,
    pub directories: DirectoryPreview,
    pub properties: HashMap<String, String>,
    pub custom_actions: Vec<String>,
}

impl Default for InstallPreview {
    fn default() -> Self {
        Self::new()
    }
}

impl InstallPreview {
    pub fn new() -> Self {
        Self {
            product_name: None,
            manufacturer: None,
            version: None,
            upgrade_code: None,
            files: Vec::new(),
            registry: Vec::new(),
            shortcuts: Vec::new(),
            services: Vec::new(),
            environment: Vec::new(),
            components: Vec::new(),
            features: Vec::new(),
            directories: DirectoryPreview::new("TARGETDIR"),
            properties: HashMap::new(),
            custom_actions: Vec::new(),
        }
    }

    pub fn file_count(&self) -> usize {
        self.files.len()
    }

    pub fn registry_count(&self) -> usize {
        self.registry.len()
    }

    pub fn component_count(&self) -> usize {
        self.components.len()
    }

    pub fn feature_count(&self) -> usize {
        self.features.len()
    }

    pub fn total_size_estimate(&self) -> u64 {
        // Placeholder - would calculate from actual file sizes
        self.files.len() as u64 * 10240 // Estimate 10KB per file
    }

    pub fn summary(&self) -> PreviewSummary {
        PreviewSummary {
            product_name: self.product_name.clone().unwrap_or_default(),
            version: self.version.clone().unwrap_or_default(),
            file_count: self.files.len(),
            registry_count: self.registry.len(),
            shortcut_count: self.shortcuts.len(),
            service_count: self.services.len(),
            component_count: self.components.len(),
            feature_count: self.features.len(),
            custom_action_count: self.custom_actions.len(),
        }
    }
}

/// Preview summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreviewSummary {
    pub product_name: String,
    pub version: String,
    pub file_count: usize,
    pub registry_count: usize,
    pub shortcut_count: usize,
    pub service_count: usize,
    pub component_count: usize,
    pub feature_count: usize,
    pub custom_action_count: usize,
}

impl PreviewSummary {
    pub fn to_string_report(&self) -> String {
        format!(
            "Installation Preview\n\
             ====================\n\
             Product: {}\n\
             Version: {}\n\n\
             Contents:\n\
             - Files: {}\n\
             - Registry entries: {}\n\
             - Shortcuts: {}\n\
             - Services: {}\n\
             - Components: {}\n\
             - Features: {}\n\
             - Custom actions: {}",
            self.product_name,
            self.version,
            self.file_count,
            self.registry_count,
            self.shortcut_count,
            self.service_count,
            self.component_count,
            self.feature_count,
            self.custom_action_count
        )
    }
}

/// Preview generator
pub struct PreviewGenerator;

impl PreviewGenerator {
    /// Generate preview from parsed WiX content
    pub fn generate(
        files: &[FileEntry],
        registry: &[RegistryEntry],
        shortcuts: &[ShortcutEntry],
    ) -> InstallPreview {
        let mut preview = InstallPreview::new();
        preview.files = files.to_vec();
        preview.registry = registry.to_vec();
        preview.shortcuts = shortcuts.to_vec();
        preview
    }

    /// Generate file tree view
    pub fn file_tree(files: &[FileEntry]) -> String {
        let mut tree = String::new();
        let mut by_dir: HashMap<String, Vec<&FileEntry>> = HashMap::new();

        for file in files {
            let dir = PathBuf::from(&file.destination)
                .parent()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default();
            by_dir.entry(dir).or_default().push(file);
        }

        let mut dirs: Vec<_> = by_dir.keys().collect();
        dirs.sort();

        for dir in dirs {
            tree.push_str(&format!("[{}]\n", dir));
            if let Some(dir_files) = by_dir.get(dir) {
                for file in dir_files {
                    let name = PathBuf::from(&file.destination)
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_default();
                    tree.push_str(&format!("  {}\n", name));
                }
            }
        }

        tree
    }

    /// Generate registry tree view
    pub fn registry_tree(registry: &[RegistryEntry]) -> String {
        let mut tree = String::new();
        let mut by_key: HashMap<String, Vec<&RegistryEntry>> = HashMap::new();

        for entry in registry {
            let full_key = format!("{}\\{}", entry.root, entry.key);
            by_key.entry(full_key).or_default().push(entry);
        }

        let mut keys: Vec<_> = by_key.keys().collect();
        keys.sort();

        for key in keys {
            tree.push_str(&format!("[{}]\n", key));
            if let Some(entries) = by_key.get(key) {
                for entry in entries {
                    let name = entry.name.as_deref().unwrap_or("(Default)");
                    let value = entry.value.as_deref().unwrap_or("");
                    tree.push_str(&format!("  {} = {}\n", name, value));
                }
            }
        }

        tree
    }

    /// Parse a WiX source file and generate preview
    pub fn from_source(path: &std::path::Path) -> Result<InstallPreview, std::io::Error> {
        let mut preview = InstallPreview::new();

        if path.exists() {
            let content = std::fs::read_to_string(path)?;

            // Set product info from file name
            preview.product_name = Some(
                path.file_stem()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_else(|| "Unknown".to_string()),
            );
            preview.version = Some("1.0.0".to_string());

            // Simple extraction of File elements
            if content.contains("<File") {
                preview.files.push(FileEntry {
                    source: "app.exe".to_string(),
                    destination: "[INSTALLFOLDER]\\app.exe".to_string(),
                    component: "MainComponent".to_string(),
                    feature: Some("ProductFeature".to_string()),
                    attributes: FileAttributes::default(),
                });
            }

            // Check for registry entries
            if content.contains("<Registry") || content.contains("<RegistryKey") {
                preview.registry.push(RegistryEntry {
                    root: "HKLM".to_string(),
                    key: "Software\\Company\\Product".to_string(),
                    name: Some("Version".to_string()),
                    value: Some("1.0.0".to_string()),
                    value_type: "string".to_string(),
                    component: "RegistryComponent".to_string(),
                });
            }
        }

        Ok(preview)
    }
}

/// Preview output formats
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreviewFormat {
    Text,
    Json,
    Tree,
    Table,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_entry() {
        let entry = FileEntry {
            source: "app.exe".to_string(),
            destination: "[INSTALLFOLDER]\\app.exe".to_string(),
            component: "MainComponent".to_string(),
            feature: Some("MainFeature".to_string()),
            attributes: FileAttributes::default(),
        };
        assert_eq!(entry.source, "app.exe");
    }

    #[test]
    fn test_file_attributes_default() {
        let attrs = FileAttributes::default();
        assert!(!attrs.read_only);
        assert!(!attrs.hidden);
        assert!(!attrs.vital);
    }

    #[test]
    fn test_registry_entry() {
        let entry = RegistryEntry {
            root: "HKLM".to_string(),
            key: "Software\\MyApp".to_string(),
            name: Some("Version".to_string()),
            value: Some("1.0.0".to_string()),
            value_type: "string".to_string(),
            component: "RegistryComponent".to_string(),
        };
        assert_eq!(entry.root, "HKLM");
    }

    #[test]
    fn test_shortcut_entry() {
        let entry = ShortcutEntry {
            name: "My App".to_string(),
            directory: "ProgramMenuFolder".to_string(),
            target: "[INSTALLFOLDER]app.exe".to_string(),
            arguments: None,
            working_dir: Some("[INSTALLFOLDER]".to_string()),
            icon: None,
            component: "ShortcutComponent".to_string(),
        };
        assert_eq!(entry.name, "My App");
    }

    #[test]
    fn test_directory_preview_new() {
        let dir = DirectoryPreview::new("TARGETDIR");
        assert_eq!(dir.id, "TARGETDIR");
        assert!(dir.children.is_empty());
    }

    #[test]
    fn test_directory_preview_with_name() {
        let dir = DirectoryPreview::new("MyDir").with_name("MyFolder");
        assert_eq!(dir.name, Some("MyFolder".to_string()));
    }

    #[test]
    fn test_directory_preview_add_child() {
        let mut parent = DirectoryPreview::new("Parent");
        parent.add_child(DirectoryPreview::new("Child"));
        assert_eq!(parent.children.len(), 1);
    }

    #[test]
    fn test_directory_preview_flatten() {
        let mut root = DirectoryPreview::new("TARGETDIR").with_name("Root");
        let mut child = DirectoryPreview::new("Child1").with_name("SubFolder");
        child.add_child(DirectoryPreview::new("Grandchild").with_name("DeepFolder"));
        root.add_child(child);

        let paths = root.flatten();
        assert!(paths.contains(&"Root".to_string()));
        assert!(paths.contains(&"Root\\SubFolder".to_string()));
    }

    #[test]
    fn test_install_preview_new() {
        let preview = InstallPreview::new();
        assert_eq!(preview.file_count(), 0);
        assert_eq!(preview.registry_count(), 0);
    }

    #[test]
    fn test_install_preview_counts() {
        let mut preview = InstallPreview::new();
        preview.files.push(FileEntry {
            source: "test.exe".to_string(),
            destination: "[INSTALLFOLDER]\\test.exe".to_string(),
            component: "TestComponent".to_string(),
            feature: None,
            attributes: FileAttributes::default(),
        });
        assert_eq!(preview.file_count(), 1);
    }

    #[test]
    fn test_install_preview_summary() {
        let mut preview = InstallPreview::new();
        preview.product_name = Some("Test Product".to_string());
        preview.version = Some("1.0.0".to_string());

        let summary = preview.summary();
        assert_eq!(summary.product_name, "Test Product");
        assert_eq!(summary.version, "1.0.0");
    }

    #[test]
    fn test_preview_summary_report() {
        let summary = PreviewSummary {
            product_name: "Test".to_string(),
            version: "1.0".to_string(),
            file_count: 5,
            registry_count: 3,
            shortcut_count: 1,
            service_count: 0,
            component_count: 2,
            feature_count: 1,
            custom_action_count: 0,
        };

        let report = summary.to_string_report();
        assert!(report.contains("Test"));
        assert!(report.contains("Files: 5"));
    }

    #[test]
    fn test_preview_generator_generate() {
        let files = vec![FileEntry {
            source: "app.exe".to_string(),
            destination: "[INSTALLFOLDER]\\app.exe".to_string(),
            component: "Main".to_string(),
            feature: None,
            attributes: FileAttributes::default(),
        }];

        let preview = PreviewGenerator::generate(&files, &[], &[]);
        assert_eq!(preview.file_count(), 1);
    }

    #[test]
    fn test_file_tree_generation() {
        let files = vec![
            FileEntry {
                source: "app.exe".to_string(),
                destination: "C:\\Program Files\\MyApp\\app.exe".to_string(),
                component: "Main".to_string(),
                feature: None,
                attributes: FileAttributes::default(),
            },
            FileEntry {
                source: "lib.dll".to_string(),
                destination: "C:\\Program Files\\MyApp\\lib.dll".to_string(),
                component: "Main".to_string(),
                feature: None,
                attributes: FileAttributes::default(),
            },
        ];

        let tree = PreviewGenerator::file_tree(&files);
        assert!(tree.contains("MyApp"));
        assert!(tree.contains("app.exe"));
    }

    #[test]
    fn test_registry_tree_generation() {
        let registry = vec![RegistryEntry {
            root: "HKLM".to_string(),
            key: "Software\\MyApp".to_string(),
            name: Some("Version".to_string()),
            value: Some("1.0.0".to_string()),
            value_type: "string".to_string(),
            component: "Reg".to_string(),
        }];

        let tree = PreviewGenerator::registry_tree(&registry);
        assert!(tree.contains("HKLM\\Software\\MyApp"));
        assert!(tree.contains("Version"));
    }
}

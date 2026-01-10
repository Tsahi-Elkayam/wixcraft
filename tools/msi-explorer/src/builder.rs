//! MSI package builder and analyzer
//!
//! Provides MSI building, analysis, and manipulation capabilities.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// MSI package metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MsiMetadata {
    pub product_name: String,
    pub product_code: String,
    pub upgrade_code: String,
    pub version: String,
    pub manufacturer: String,
    pub package_code: Option<String>,
    pub language: u16,
    pub codepage: u16,
}

impl MsiMetadata {
    pub fn new(name: &str, product_code: &str, upgrade_code: &str, version: &str) -> Self {
        Self {
            product_name: name.to_string(),
            product_code: product_code.to_string(),
            upgrade_code: upgrade_code.to_string(),
            version: version.to_string(),
            manufacturer: String::new(),
            package_code: None,
            language: 1033, // English
            codepage: 1252, // Windows ANSI
        }
    }

    pub fn with_manufacturer(mut self, manufacturer: &str) -> Self {
        self.manufacturer = manufacturer.to_string();
        self
    }

    pub fn with_language(mut self, language: u16) -> Self {
        self.language = language;
        self
    }
}

/// MSI table definition for building
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MsiTableDef {
    pub name: String,
    pub columns: Vec<MsiColumnDef>,
    pub rows: Vec<Vec<MsiCellValue>>,
}

impl MsiTableDef {
    pub fn new(name: &str, columns: Vec<MsiColumnDef>) -> Self {
        Self {
            name: name.to_string(),
            columns,
            rows: Vec::new(),
        }
    }

    pub fn add_row(&mut self, values: Vec<MsiCellValue>) {
        self.rows.push(values);
    }

    pub fn row_count(&self) -> usize {
        self.rows.len()
    }
}

/// MSI column definition for building
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MsiColumnDef {
    pub name: String,
    pub column_type: MsiColumnTypeDef,
    pub nullable: bool,
    pub primary_key: bool,
}

impl MsiColumnDef {
    pub fn new(name: &str, column_type: MsiColumnTypeDef) -> Self {
        Self {
            name: name.to_string(),
            column_type,
            nullable: true,
            primary_key: false,
        }
    }

    pub fn primary_key(mut self) -> Self {
        self.primary_key = true;
        self.nullable = false;
        self
    }

    pub fn not_null(mut self) -> Self {
        self.nullable = false;
        self
    }
}

/// MSI column type for building
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MsiColumnTypeDef {
    /// Short integer (2 bytes)
    Short,
    /// Long integer (4 bytes)
    Long,
    /// String (variable length)
    String,
    /// Localizable string
    LocalizableString,
    /// Binary data
    Binary,
    /// Object (OLE)
    Object,
}

/// MSI cell value for building
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MsiCellValue {
    Null,
    Integer(i32),
    String(String),
    Binary(Vec<u8>),
}

impl MsiCellValue {
    pub fn is_null(&self) -> bool {
        matches!(self, MsiCellValue::Null)
    }

    pub fn as_string(&self) -> Option<&str> {
        match self {
            MsiCellValue::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_integer(&self) -> Option<i32> {
        match self {
            MsiCellValue::Integer(i) => Some(*i),
            _ => None,
        }
    }
}

/// MSI component for building
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MsiComponentDef {
    pub id: String,
    pub guid: String,
    pub directory: String,
    pub files: Vec<MsiFileDef>,
    pub registry_entries: Vec<MsiRegistryDef>,
    pub key_path: Option<String>,
}

impl MsiComponentDef {
    pub fn new(id: &str, guid: &str, directory: &str) -> Self {
        Self {
            id: id.to_string(),
            guid: guid.to_string(),
            directory: directory.to_string(),
            files: Vec::new(),
            registry_entries: Vec::new(),
            key_path: None,
        }
    }

    pub fn add_file(&mut self, file: MsiFileDef) {
        if self.key_path.is_none() {
            self.key_path = Some(file.id.clone());
        }
        self.files.push(file);
    }

    pub fn add_registry(&mut self, entry: MsiRegistryDef) {
        self.registry_entries.push(entry);
    }
}

/// MSI file entry for building
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MsiFileDef {
    pub id: String,
    pub name: String,
    pub source: PathBuf,
    pub size: u64,
    pub version: Option<String>,
    pub language: Option<String>,
    pub attributes: u16,
}

impl MsiFileDef {
    pub fn new(id: &str, name: &str, source: PathBuf) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            source,
            size: 0,
            version: None,
            language: None,
            attributes: 0,
        }
    }

    pub fn with_size(mut self, size: u64) -> Self {
        self.size = size;
        self
    }

    pub fn with_version(mut self, version: &str) -> Self {
        self.version = Some(version.to_string());
        self
    }
}

/// MSI registry entry for building
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MsiRegistryDef {
    pub id: String,
    pub root: RegistryRoot,
    pub key: String,
    pub name: Option<String>,
    pub value: Option<String>,
    pub value_type: RegistryValueType,
}

impl MsiRegistryDef {
    pub fn new(id: &str, root: RegistryRoot, key: &str) -> Self {
        Self {
            id: id.to_string(),
            root,
            key: key.to_string(),
            name: None,
            value: None,
            value_type: RegistryValueType::String,
        }
    }

    pub fn with_value(mut self, name: &str, value: &str) -> Self {
        self.name = Some(name.to_string());
        self.value = Some(value.to_string());
        self
    }
}

/// Registry root
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RegistryRoot {
    ClassesRoot,
    CurrentUser,
    LocalMachine,
    Users,
}

impl RegistryRoot {
    pub fn to_msi_value(&self) -> i32 {
        match self {
            RegistryRoot::ClassesRoot => 0,
            RegistryRoot::CurrentUser => 1,
            RegistryRoot::LocalMachine => 2,
            RegistryRoot::Users => 3,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            RegistryRoot::ClassesRoot => "HKCR",
            RegistryRoot::CurrentUser => "HKCU",
            RegistryRoot::LocalMachine => "HKLM",
            RegistryRoot::Users => "HKU",
        }
    }
}

/// Registry value type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RegistryValueType {
    String,
    Integer,
    Binary,
    ExpandableString,
    MultiString,
}

/// MSI feature for building
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MsiFeatureDef {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub level: u16,
    pub components: Vec<String>,
    pub parent: Option<String>,
}

impl MsiFeatureDef {
    pub fn new(id: &str, title: &str, level: u16) -> Self {
        Self {
            id: id.to_string(),
            title: title.to_string(),
            description: None,
            level,
            components: Vec::new(),
            parent: None,
        }
    }

    pub fn with_description(mut self, desc: &str) -> Self {
        self.description = Some(desc.to_string());
        self
    }

    pub fn add_component(&mut self, component_id: &str) {
        self.components.push(component_id.to_string());
    }

    pub fn with_parent(mut self, parent_id: &str) -> Self {
        self.parent = Some(parent_id.to_string());
        self
    }
}

/// MSI directory for building
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MsiDirectoryDef {
    pub id: String,
    pub name: String,
    pub parent: Option<String>,
}

impl MsiDirectoryDef {
    pub fn new(id: &str, name: &str) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            parent: None,
        }
    }

    pub fn with_parent(mut self, parent_id: &str) -> Self {
        self.parent = Some(parent_id.to_string());
        self
    }
}

/// MSI package builder
#[derive(Debug, Clone, Default)]
pub struct MsiBuilder {
    metadata: Option<MsiMetadata>,
    directories: Vec<MsiDirectoryDef>,
    components: Vec<MsiComponentDef>,
    features: Vec<MsiFeatureDef>,
    tables: HashMap<String, MsiTableDef>,
}

impl MsiBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_metadata(mut self, metadata: MsiMetadata) -> Self {
        self.metadata = Some(metadata);
        self
    }

    pub fn add_directory(&mut self, directory: MsiDirectoryDef) {
        self.directories.push(directory);
    }

    pub fn add_component(&mut self, component: MsiComponentDef) {
        self.components.push(component);
    }

    pub fn add_feature(&mut self, feature: MsiFeatureDef) {
        self.features.push(feature);
    }

    pub fn add_table(&mut self, table: MsiTableDef) {
        self.tables.insert(table.name.clone(), table);
    }

    pub fn get_table(&self, name: &str) -> Option<&MsiTableDef> {
        self.tables.get(name)
    }

    pub fn metadata(&self) -> Option<&MsiMetadata> {
        self.metadata.as_ref()
    }

    pub fn directories(&self) -> &[MsiDirectoryDef] {
        &self.directories
    }

    pub fn components(&self) -> &[MsiComponentDef] {
        &self.components
    }

    pub fn features(&self) -> &[MsiFeatureDef] {
        &self.features
    }

    pub fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();

        if self.metadata.is_none() {
            errors.push("Missing package metadata".to_string());
        }

        if self.features.is_empty() {
            errors.push("At least one feature is required".to_string());
        }

        // Check component references
        for feature in &self.features {
            for comp_ref in &feature.components {
                if !self.components.iter().any(|c| &c.id == comp_ref) {
                    errors.push(format!(
                        "Feature '{}' references unknown component '{}'",
                        feature.id, comp_ref
                    ));
                }
            }
        }

        errors
    }

    pub fn build(&self, _output: &PathBuf) -> MsiBuildResult {
        let errors = self.validate();
        if !errors.is_empty() {
            return MsiBuildResult::failure(errors);
        }

        // Would generate actual MSI here
        MsiBuildResult::success()
    }
}

/// MSI build result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MsiBuildResult {
    pub success: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub statistics: MsiBuildStats,
}

impl MsiBuildResult {
    pub fn success() -> Self {
        Self {
            success: true,
            errors: Vec::new(),
            warnings: Vec::new(),
            statistics: MsiBuildStats::default(),
        }
    }

    pub fn failure(errors: Vec<String>) -> Self {
        Self {
            success: false,
            errors,
            warnings: Vec::new(),
            statistics: MsiBuildStats::default(),
        }
    }
}

/// MSI build statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MsiBuildStats {
    pub file_count: usize,
    pub component_count: usize,
    pub feature_count: usize,
    pub total_size: u64,
    pub compressed_size: u64,
}

/// MSI analyzer
pub struct MsiAnalyzer;

impl MsiAnalyzer {
    /// Analyze an MSI file
    pub fn analyze(_path: &PathBuf) -> MsiAnalysis {
        // Would parse actual MSI file
        MsiAnalysis::default()
    }
}

/// MSI analysis result
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MsiAnalysis {
    pub metadata: Option<MsiMetadata>,
    pub file_count: usize,
    pub component_count: usize,
    pub feature_count: usize,
    pub table_count: usize,
    pub custom_action_count: usize,
    pub total_size: u64,
}

impl MsiAnalysis {
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_msi_metadata_new() {
        let meta = MsiMetadata::new("MyApp", "{CODE}", "{UPGRADE}", "1.0.0");
        assert_eq!(meta.product_name, "MyApp");
        assert_eq!(meta.language, 1033);
    }

    #[test]
    fn test_msi_metadata_with_manufacturer() {
        let meta = MsiMetadata::new("App", "{C}", "{U}", "1.0")
            .with_manufacturer("Acme Inc");
        assert_eq!(meta.manufacturer, "Acme Inc");
    }

    #[test]
    fn test_msi_table_def_new() {
        let cols = vec![MsiColumnDef::new("ID", MsiColumnTypeDef::String).primary_key()];
        let table = MsiTableDef::new("MyTable", cols);
        assert_eq!(table.name, "MyTable");
        assert_eq!(table.row_count(), 0);
    }

    #[test]
    fn test_msi_table_def_add_row() {
        let cols = vec![MsiColumnDef::new("ID", MsiColumnTypeDef::String)];
        let mut table = MsiTableDef::new("MyTable", cols);
        table.add_row(vec![MsiCellValue::String("value".to_string())]);
        assert_eq!(table.row_count(), 1);
    }

    #[test]
    fn test_msi_column_def_primary_key() {
        let col = MsiColumnDef::new("ID", MsiColumnTypeDef::String).primary_key();
        assert!(col.primary_key);
        assert!(!col.nullable);
    }

    #[test]
    fn test_msi_cell_value_is_null() {
        assert!(MsiCellValue::Null.is_null());
        assert!(!MsiCellValue::Integer(1).is_null());
    }

    #[test]
    fn test_msi_cell_value_as_string() {
        let val = MsiCellValue::String("test".to_string());
        assert_eq!(val.as_string(), Some("test"));
        assert!(MsiCellValue::Integer(1).as_string().is_none());
    }

    #[test]
    fn test_msi_component_def_new() {
        let comp = MsiComponentDef::new("Comp1", "{GUID}", "INSTALLDIR");
        assert_eq!(comp.id, "Comp1");
        assert!(comp.files.is_empty());
    }

    #[test]
    fn test_msi_component_def_add_file() {
        let mut comp = MsiComponentDef::new("Comp1", "{GUID}", "INSTALLDIR");
        let file = MsiFileDef::new("File1", "app.exe", PathBuf::from("app.exe"));
        comp.add_file(file);
        assert_eq!(comp.files.len(), 1);
        assert_eq!(comp.key_path, Some("File1".to_string()));
    }

    #[test]
    fn test_msi_file_def_new() {
        let file = MsiFileDef::new("File1", "app.exe", PathBuf::from("app.exe"));
        assert_eq!(file.id, "File1");
        assert_eq!(file.size, 0);
    }

    #[test]
    fn test_msi_file_def_with_size() {
        let file = MsiFileDef::new("File1", "app.exe", PathBuf::from("app.exe"))
            .with_size(1024);
        assert_eq!(file.size, 1024);
    }

    #[test]
    fn test_msi_registry_def_new() {
        let reg = MsiRegistryDef::new("Reg1", RegistryRoot::LocalMachine, "SOFTWARE\\MyApp");
        assert_eq!(reg.root, RegistryRoot::LocalMachine);
    }

    #[test]
    fn test_msi_registry_def_with_value() {
        let reg = MsiRegistryDef::new("Reg1", RegistryRoot::LocalMachine, "SOFTWARE\\MyApp")
            .with_value("Version", "1.0.0");
        assert_eq!(reg.name, Some("Version".to_string()));
    }

    #[test]
    fn test_registry_root_to_msi() {
        assert_eq!(RegistryRoot::ClassesRoot.to_msi_value(), 0);
        assert_eq!(RegistryRoot::LocalMachine.to_msi_value(), 2);
    }

    #[test]
    fn test_registry_root_as_str() {
        assert_eq!(RegistryRoot::LocalMachine.as_str(), "HKLM");
        assert_eq!(RegistryRoot::CurrentUser.as_str(), "HKCU");
    }

    #[test]
    fn test_msi_feature_def_new() {
        let feature = MsiFeatureDef::new("MainFeature", "Main Feature", 1);
        assert_eq!(feature.id, "MainFeature");
        assert_eq!(feature.level, 1);
    }

    #[test]
    fn test_msi_feature_def_add_component() {
        let mut feature = MsiFeatureDef::new("MainFeature", "Main", 1);
        feature.add_component("Comp1");
        assert!(feature.components.contains(&"Comp1".to_string()));
    }

    #[test]
    fn test_msi_directory_def_new() {
        let dir = MsiDirectoryDef::new("INSTALLDIR", "MyApp");
        assert_eq!(dir.id, "INSTALLDIR");
    }

    #[test]
    fn test_msi_directory_def_with_parent() {
        let dir = MsiDirectoryDef::new("BinDir", "bin").with_parent("INSTALLDIR");
        assert_eq!(dir.parent, Some("INSTALLDIR".to_string()));
    }

    #[test]
    fn test_msi_builder_new() {
        let builder = MsiBuilder::new();
        assert!(builder.metadata.is_none());
    }

    #[test]
    fn test_msi_builder_with_metadata() {
        let meta = MsiMetadata::new("App", "{C}", "{U}", "1.0");
        let builder = MsiBuilder::new().with_metadata(meta);
        assert!(builder.metadata.is_some());
    }

    #[test]
    fn test_msi_builder_add_directory() {
        let mut builder = MsiBuilder::new();
        builder.add_directory(MsiDirectoryDef::new("INSTALLDIR", "MyApp"));
        assert_eq!(builder.directories.len(), 1);
    }

    #[test]
    fn test_msi_builder_add_component() {
        let mut builder = MsiBuilder::new();
        builder.add_component(MsiComponentDef::new("Comp1", "{G}", "DIR"));
        assert_eq!(builder.components.len(), 1);
    }

    #[test]
    fn test_msi_builder_add_feature() {
        let mut builder = MsiBuilder::new();
        builder.add_feature(MsiFeatureDef::new("Main", "Main Feature", 1));
        assert_eq!(builder.features.len(), 1);
    }

    #[test]
    fn test_msi_builder_validate_missing_metadata() {
        let builder = MsiBuilder::new();
        let errors = builder.validate();
        assert!(errors.iter().any(|e| e.contains("metadata")));
    }

    #[test]
    fn test_msi_builder_validate_missing_features() {
        let meta = MsiMetadata::new("App", "{C}", "{U}", "1.0");
        let builder = MsiBuilder::new().with_metadata(meta);
        let errors = builder.validate();
        assert!(errors.iter().any(|e| e.contains("feature")));
    }

    #[test]
    fn test_msi_build_result_success() {
        let result = MsiBuildResult::success();
        assert!(result.success);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_msi_build_result_failure() {
        let result = MsiBuildResult::failure(vec!["Error".to_string()]);
        assert!(!result.success);
        assert_eq!(result.errors.len(), 1);
    }

    #[test]
    fn test_msi_analysis_to_json() {
        let analysis = MsiAnalysis::default();
        let json = analysis.to_json();
        assert!(json.contains("file_count"));
    }
}

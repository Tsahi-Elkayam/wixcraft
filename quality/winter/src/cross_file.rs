//! Cross-file validation for WiX projects
//!
//! Performs two-pass validation:
//! 1. First pass: Collect all definitions (Component, Feature, Directory, Property, etc.)
//! 2. Second pass: Validate references against collected definitions

use crate::diagnostic::{Diagnostic, Location, Severity};
use crate::plugin::Document;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

/// Types of symbols that can be defined/referenced
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SymbolKind {
    Component,
    ComponentGroup,
    Feature,
    FeatureGroup,
    Directory,
    Property,
    CustomAction,
    Binary,
    Icon,
    Media,
    Variable,
    Payload,
    PayloadGroup,
    PackageGroup,
    Bundle,
    UI,
    Dialog,
    Fragment,
}

impl SymbolKind {
    /// Get the element name that defines this symbol
    pub fn definition_element(&self) -> &'static str {
        match self {
            SymbolKind::Component => "Component",
            SymbolKind::ComponentGroup => "ComponentGroup",
            SymbolKind::Feature => "Feature",
            SymbolKind::FeatureGroup => "FeatureGroup",
            SymbolKind::Directory => "Directory",
            SymbolKind::Property => "Property",
            SymbolKind::CustomAction => "CustomAction",
            SymbolKind::Binary => "Binary",
            SymbolKind::Icon => "Icon",
            SymbolKind::Media => "Media",
            SymbolKind::Variable => "Variable",
            SymbolKind::Payload => "Payload",
            SymbolKind::PayloadGroup => "PayloadGroup",
            SymbolKind::PackageGroup => "PackageGroup",
            SymbolKind::Bundle => "Bundle",
            SymbolKind::UI => "UI",
            SymbolKind::Dialog => "Dialog",
            SymbolKind::Fragment => "Fragment",
        }
    }

    /// Get the reference element name for this symbol
    pub fn reference_element(&self) -> Option<&'static str> {
        match self {
            SymbolKind::Component => Some("ComponentRef"),
            SymbolKind::ComponentGroup => Some("ComponentGroupRef"),
            SymbolKind::Feature => Some("FeatureRef"),
            SymbolKind::FeatureGroup => Some("FeatureGroupRef"),
            SymbolKind::Directory => Some("DirectoryRef"),
            SymbolKind::Property => Some("PropertyRef"),
            SymbolKind::CustomAction => Some("CustomActionRef"),
            SymbolKind::Binary => None,
            SymbolKind::Icon => None,
            SymbolKind::Media => None,
            SymbolKind::Variable => Some("VariableRef"),
            SymbolKind::Payload => Some("PayloadRef"),
            SymbolKind::PayloadGroup => Some("PayloadGroupRef"),
            SymbolKind::PackageGroup => Some("PackageGroupRef"),
            SymbolKind::Bundle => None,
            SymbolKind::UI => Some("UIRef"),
            SymbolKind::Dialog => Some("DialogRef"),
            SymbolKind::Fragment => None,
        }
    }
}

/// A symbol definition
#[derive(Debug, Clone)]
pub struct SymbolDefinition {
    /// The symbol ID
    pub id: String,

    /// The kind of symbol
    pub kind: SymbolKind,

    /// Source file
    pub file: PathBuf,

    /// Line number
    pub line: usize,

    /// Column number
    pub column: usize,
}

/// A symbol reference
#[derive(Debug, Clone)]
pub struct SymbolReference {
    /// The symbol ID being referenced
    pub id: String,

    /// The kind of symbol
    pub kind: SymbolKind,

    /// Source file
    pub file: PathBuf,

    /// Line number
    pub line: usize,

    /// Column number
    pub column: usize,

    /// The element that contains this reference
    pub element: String,
}

/// Index of all symbols across files
#[derive(Debug, Default)]
pub struct SymbolIndex {
    /// All definitions keyed by (kind, id)
    pub definitions: HashMap<(SymbolKind, String), Vec<SymbolDefinition>>,

    /// All references
    pub references: Vec<SymbolReference>,

    /// Standard directories (built-in, always valid)
    pub standard_directories: HashSet<String>,
}

impl SymbolIndex {
    /// Create a new symbol index
    pub fn new() -> Self {
        let mut index = Self::default();

        // Add standard WiX directories that are always valid
        let standard_dirs = [
            "TARGETDIR",
            "ProgramFilesFolder",
            "ProgramFiles64Folder",
            "ProgramFiles6432Folder",
            "CommonFilesFolder",
            "CommonFiles64Folder",
            "CommonFiles6432Folder",
            "ProgramMenuFolder",
            "StartMenuFolder",
            "StartupFolder",
            "DesktopFolder",
            "AppDataFolder",
            "LocalAppDataFolder",
            "TempFolder",
            "WindowsFolder",
            "SystemFolder",
            "System64Folder",
            "System16Folder",
            "FontsFolder",
            "FavoritesFolder",
            "SendToFolder",
            "NetHoodFolder",
            "PrintHoodFolder",
            "TemplateFolder",
            "AdminToolsFolder",
            "PersonalFolder",
            "MyPicturesFolder",
            "CommonAppDataFolder",
            "WindowsVolume",
        ];

        for dir in standard_dirs {
            index.standard_directories.insert(dir.to_string());
        }

        index
    }

    /// Add a definition
    pub fn add_definition(&mut self, def: SymbolDefinition) {
        self.definitions
            .entry((def.kind, def.id.clone()))
            .or_default()
            .push(def);
    }

    /// Add a reference
    pub fn add_reference(&mut self, reference: SymbolReference) {
        self.references.push(reference);
    }

    /// Check if a symbol is defined
    pub fn is_defined(&self, kind: SymbolKind, id: &str) -> bool {
        // Special case for standard directories
        if kind == SymbolKind::Directory && self.standard_directories.contains(id) {
            return true;
        }

        self.definitions.contains_key(&(kind, id.to_string()))
    }

    /// Get definitions for a symbol
    pub fn get_definitions(&self, kind: SymbolKind, id: &str) -> Option<&Vec<SymbolDefinition>> {
        self.definitions.get(&(kind, id.to_string()))
    }

    /// Get duplicate definitions
    pub fn get_duplicates(&self) -> Vec<(&(SymbolKind, String), &Vec<SymbolDefinition>)> {
        self.definitions
            .iter()
            .filter(|(_, defs)| defs.len() > 1)
            .collect()
    }
}

/// Cross-file validator
pub struct CrossFileValidator {
    /// Symbol index
    index: SymbolIndex,
}

impl CrossFileValidator {
    /// Create a new validator
    pub fn new() -> Self {
        Self {
            index: SymbolIndex::new(),
        }
    }

    /// Get the symbol index (for testing)
    pub fn index(&self) -> &SymbolIndex {
        &self.index
    }

    /// First pass: collect definitions from a document
    pub fn collect_definitions(&mut self, document: &dyn Document, file: &Path) {
        for node in document.iter() {
            if node.kind() != "element" {
                continue;
            }

            let name = node.name();
            let location = node.location();

            // Try to get the Id attribute
            let id = match node.get("Id") {
                Some(id) => id.to_string(),
                None => continue,
            };

            // Match element name to symbol kind
            let kind = match name {
                "Component" => SymbolKind::Component,
                "ComponentGroup" => SymbolKind::ComponentGroup,
                "Feature" => SymbolKind::Feature,
                "FeatureGroup" => SymbolKind::FeatureGroup,
                "Directory" => SymbolKind::Directory,
                "StandardDirectory" => SymbolKind::Directory,
                "Property" => SymbolKind::Property,
                "CustomAction" => SymbolKind::CustomAction,
                "Binary" => SymbolKind::Binary,
                "Icon" => SymbolKind::Icon,
                "Media" => SymbolKind::Media,
                "Variable" => SymbolKind::Variable,
                "Payload" => SymbolKind::Payload,
                "PayloadGroup" => SymbolKind::PayloadGroup,
                "PackageGroup" => SymbolKind::PackageGroup,
                "Bundle" => SymbolKind::Bundle,
                "UI" => SymbolKind::UI,
                "Dialog" => SymbolKind::Dialog,
                "Fragment" => SymbolKind::Fragment,
                _ => continue,
            };

            self.index.add_definition(SymbolDefinition {
                id,
                kind,
                file: file.to_path_buf(),
                line: location.line,
                column: location.column,
            });
        }
    }

    /// First pass: collect references from a document
    pub fn collect_references(&mut self, document: &dyn Document, file: &Path) {
        for node in document.iter() {
            if node.kind() != "element" {
                continue;
            }

            let name = node.name();
            let location = node.location();

            // Match reference elements
            let (kind, id_attr) = match name {
                "ComponentRef" => (SymbolKind::Component, "Id"),
                "ComponentGroupRef" => (SymbolKind::ComponentGroup, "Id"),
                "FeatureRef" => (SymbolKind::Feature, "Id"),
                "FeatureGroupRef" => (SymbolKind::FeatureGroup, "Id"),
                "DirectoryRef" => (SymbolKind::Directory, "Id"),
                "PropertyRef" => (SymbolKind::Property, "Id"),
                "CustomActionRef" => (SymbolKind::CustomAction, "Id"),
                "VariableRef" => (SymbolKind::Variable, "Id"),
                "PayloadRef" => (SymbolKind::Payload, "Id"),
                "PayloadGroupRef" => (SymbolKind::PayloadGroup, "Id"),
                "PackageGroupRef" => (SymbolKind::PackageGroup, "Id"),
                "UIRef" => (SymbolKind::UI, "Id"),
                "DialogRef" => (SymbolKind::Dialog, "Id"),
                // Also check for Ref attributes on other elements
                "File" => {
                    // File can reference a Component via Component attribute
                    if let Some(comp_id) = node.get("Component") {
                        self.index.add_reference(SymbolReference {
                            id: comp_id.to_string(),
                            kind: SymbolKind::Component,
                            file: file.to_path_buf(),
                            line: location.line,
                            column: location.column,
                            element: name.to_string(),
                        });
                    }
                    continue;
                }
                "Shortcut" => {
                    // Shortcut references Directory via Directory attribute
                    if let Some(dir_id) = node.get("Directory") {
                        self.index.add_reference(SymbolReference {
                            id: dir_id.to_string(),
                            kind: SymbolKind::Directory,
                            file: file.to_path_buf(),
                            line: location.line,
                            column: location.column,
                            element: name.to_string(),
                        });
                    }
                    // And Icon via Icon attribute
                    if let Some(icon_id) = node.get("Icon") {
                        self.index.add_reference(SymbolReference {
                            id: icon_id.to_string(),
                            kind: SymbolKind::Icon,
                            file: file.to_path_buf(),
                            line: location.line,
                            column: location.column,
                            element: name.to_string(),
                        });
                    }
                    continue;
                }
                "Custom" => {
                    // Custom action schedule references CustomAction
                    if let Some(action_id) = node.get("Action") {
                        self.index.add_reference(SymbolReference {
                            id: action_id.to_string(),
                            kind: SymbolKind::CustomAction,
                            file: file.to_path_buf(),
                            line: location.line,
                            column: location.column,
                            element: name.to_string(),
                        });
                    }
                    continue;
                }
                "SetProperty" => {
                    // SetProperty references Property via Id
                    if let Some(prop_id) = node.get("Id") {
                        self.index.add_reference(SymbolReference {
                            id: prop_id.to_string(),
                            kind: SymbolKind::Property,
                            file: file.to_path_buf(),
                            line: location.line,
                            column: location.column,
                            element: name.to_string(),
                        });
                    }
                    continue;
                }
                _ => continue,
            };

            // Get the Id attribute
            let id = match node.get(id_attr) {
                Some(id) => id.to_string(),
                None => continue,
            };

            self.index.add_reference(SymbolReference {
                id,
                kind,
                file: file.to_path_buf(),
                line: location.line,
                column: location.column,
                element: name.to_string(),
            });
        }
    }

    /// Validate all references against definitions
    pub fn validate(&self) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        // Check for undefined references
        for reference in &self.index.references {
            if !self.index.is_defined(reference.kind, &reference.id) {
                let rule_id = format!(
                    "xref-undefined-{}",
                    reference.kind.definition_element().to_lowercase()
                );
                let message = format!(
                    "{} references undefined {} '{}'",
                    reference.element,
                    reference.kind.definition_element(),
                    reference.id
                );

                diagnostics.push(Diagnostic::new(
                    &rule_id,
                    Severity::Error,
                    &message,
                    Location::new(reference.file.clone(), reference.line, reference.column),
                ));
            }
        }

        // Check for duplicate definitions
        for ((kind, id), defs) in self.index.get_duplicates() {
            if defs.len() > 1 {
                let rule_id = format!(
                    "xref-duplicate-{}",
                    kind.definition_element().to_lowercase()
                );

                // Report on each duplicate
                for def in defs.iter().skip(1) {
                    let first = &defs[0];
                    let message = format!(
                        "Duplicate {} '{}' (first defined at {}:{})",
                        kind.definition_element(),
                        id,
                        first.file.display(),
                        first.line
                    );

                    diagnostics.push(Diagnostic::new(
                        &rule_id,
                        Severity::Error,
                        &message,
                        Location::new(def.file.clone(), def.line, def.column),
                    ));
                }
            }
        }

        diagnostics
    }

    /// Reset the validator for reuse
    pub fn reset(&mut self) {
        self.index = SymbolIndex::new();
    }
}

impl Default for CrossFileValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symbol_kind_definition_element() {
        assert_eq!(SymbolKind::Component.definition_element(), "Component");
        assert_eq!(SymbolKind::Feature.definition_element(), "Feature");
        assert_eq!(SymbolKind::Directory.definition_element(), "Directory");
    }

    #[test]
    fn test_symbol_kind_reference_element() {
        assert_eq!(
            SymbolKind::Component.reference_element(),
            Some("ComponentRef")
        );
        assert_eq!(SymbolKind::Feature.reference_element(), Some("FeatureRef"));
        assert_eq!(SymbolKind::Binary.reference_element(), None);
    }

    #[test]
    fn test_symbol_index_standard_directories() {
        let index = SymbolIndex::new();
        assert!(index.is_defined(SymbolKind::Directory, "TARGETDIR"));
        assert!(index.is_defined(SymbolKind::Directory, "ProgramFilesFolder"));
        assert!(!index.is_defined(SymbolKind::Directory, "CustomDir"));
    }

    #[test]
    fn test_symbol_index_add_definition() {
        let mut index = SymbolIndex::new();
        index.add_definition(SymbolDefinition {
            id: "MyComponent".to_string(),
            kind: SymbolKind::Component,
            file: PathBuf::from("test.wxs"),
            line: 10,
            column: 5,
        });

        assert!(index.is_defined(SymbolKind::Component, "MyComponent"));
        assert!(!index.is_defined(SymbolKind::Component, "OtherComponent"));
    }

    #[test]
    fn test_symbol_index_duplicates() {
        let mut index = SymbolIndex::new();
        index.add_definition(SymbolDefinition {
            id: "MyComponent".to_string(),
            kind: SymbolKind::Component,
            file: PathBuf::from("file1.wxs"),
            line: 10,
            column: 5,
        });
        index.add_definition(SymbolDefinition {
            id: "MyComponent".to_string(),
            kind: SymbolKind::Component,
            file: PathBuf::from("file2.wxs"),
            line: 20,
            column: 5,
        });

        let duplicates = index.get_duplicates();
        assert_eq!(duplicates.len(), 1);
    }

    #[test]
    fn test_validator_undefined_reference() {
        let mut validator = CrossFileValidator::new();

        // Add a reference without definition
        validator.index.add_reference(SymbolReference {
            id: "MissingComponent".to_string(),
            kind: SymbolKind::Component,
            file: PathBuf::from("test.wxs"),
            line: 15,
            column: 10,
            element: "ComponentRef".to_string(),
        });

        let diagnostics = validator.validate();
        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0].rule_id.contains("undefined"));
    }

    #[test]
    fn test_validator_defined_reference() {
        let mut validator = CrossFileValidator::new();

        // Add definition
        validator.index.add_definition(SymbolDefinition {
            id: "MyComponent".to_string(),
            kind: SymbolKind::Component,
            file: PathBuf::from("test.wxs"),
            line: 10,
            column: 5,
        });

        // Add reference to defined component
        validator.index.add_reference(SymbolReference {
            id: "MyComponent".to_string(),
            kind: SymbolKind::Component,
            file: PathBuf::from("test.wxs"),
            line: 20,
            column: 10,
            element: "ComponentRef".to_string(),
        });

        let diagnostics = validator.validate();
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn test_validator_standard_directory_reference() {
        let mut validator = CrossFileValidator::new();

        // Add reference to standard directory (no definition needed)
        validator.index.add_reference(SymbolReference {
            id: "ProgramFilesFolder".to_string(),
            kind: SymbolKind::Directory,
            file: PathBuf::from("test.wxs"),
            line: 10,
            column: 5,
            element: "DirectoryRef".to_string(),
        });

        let diagnostics = validator.validate();
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn test_validator_duplicate_definition() {
        let mut validator = CrossFileValidator::new();

        // Add duplicate definitions
        validator.index.add_definition(SymbolDefinition {
            id: "DupeFeature".to_string(),
            kind: SymbolKind::Feature,
            file: PathBuf::from("file1.wxs"),
            line: 10,
            column: 5,
        });
        validator.index.add_definition(SymbolDefinition {
            id: "DupeFeature".to_string(),
            kind: SymbolKind::Feature,
            file: PathBuf::from("file2.wxs"),
            line: 20,
            column: 5,
        });

        let diagnostics = validator.validate();
        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0].rule_id.contains("duplicate"));
    }
}

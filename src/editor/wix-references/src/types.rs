//! Types for symbol references and definitions

use serde::Serialize;
use std::path::PathBuf;

/// Position in source (1-based line and column)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Position {
    pub line: u32,
    pub character: u32,
}

impl Position {
    pub fn new(line: u32, character: u32) -> Self {
        Self { line, character }
    }
}

/// Range in source document
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Range {
    pub start: Position,
    pub end: Position,
}

impl Range {
    pub fn new(start_line: u32, start_col: u32, end_line: u32, end_col: u32) -> Self {
        Self {
            start: Position::new(start_line, start_col),
            end: Position::new(end_line, end_col),
        }
    }

    /// Create from byte offsets in source
    pub fn from_offsets(source: &str, start_offset: usize, end_offset: usize) -> Self {
        let start = offset_to_position(source, start_offset);
        let end = offset_to_position(source, end_offset);
        Self { start, end }
    }
}

/// Convert byte offset to line/column position (1-based)
fn offset_to_position(source: &str, offset: usize) -> Position {
    let mut line = 1u32;
    let mut col = 1u32;

    for (i, ch) in source.char_indices() {
        if i >= offset {
            break;
        }
        if ch == '\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
    }

    Position::new(line, col)
}

/// Location in a file
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Location {
    pub file: PathBuf,
    pub range: Range,
}

impl Location {
    pub fn new(file: PathBuf, range: Range) -> Self {
        Self { file, range }
    }
}

/// Reference element types in WiX
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "PascalCase")]
pub enum ReferenceKind {
    ComponentRef,
    ComponentGroupRef,
    DirectoryRef,
    FeatureRef,
    FeatureGroupRef,
    PropertyRef,
    CustomActionRef,
    BinaryRef,
}

impl ReferenceKind {
    /// Get the definition element name for this reference type
    pub fn definition_element(&self) -> &'static str {
        match self {
            ReferenceKind::ComponentRef => "Component",
            ReferenceKind::ComponentGroupRef => "ComponentGroup",
            ReferenceKind::DirectoryRef => "Directory",
            ReferenceKind::FeatureRef => "Feature",
            ReferenceKind::FeatureGroupRef => "FeatureGroup",
            ReferenceKind::PropertyRef => "Property",
            ReferenceKind::CustomActionRef => "CustomAction",
            ReferenceKind::BinaryRef => "Binary",
        }
    }

    /// Get the reference element name
    pub fn element_name(&self) -> &'static str {
        match self {
            ReferenceKind::ComponentRef => "ComponentRef",
            ReferenceKind::ComponentGroupRef => "ComponentGroupRef",
            ReferenceKind::DirectoryRef => "DirectoryRef",
            ReferenceKind::FeatureRef => "FeatureRef",
            ReferenceKind::FeatureGroupRef => "FeatureGroupRef",
            ReferenceKind::PropertyRef => "PropertyRef",
            ReferenceKind::CustomActionRef => "CustomActionRef",
            ReferenceKind::BinaryRef => "BinaryRef",
        }
    }

    /// Try to create from element name
    pub fn from_element_name(name: &str) -> Option<Self> {
        match name {
            "ComponentRef" => Some(ReferenceKind::ComponentRef),
            "ComponentGroupRef" => Some(ReferenceKind::ComponentGroupRef),
            "DirectoryRef" => Some(ReferenceKind::DirectoryRef),
            "FeatureRef" => Some(ReferenceKind::FeatureRef),
            "FeatureGroupRef" => Some(ReferenceKind::FeatureGroupRef),
            "PropertyRef" => Some(ReferenceKind::PropertyRef),
            "CustomActionRef" => Some(ReferenceKind::CustomActionRef),
            "BinaryRef" => Some(ReferenceKind::BinaryRef),
            _ => None,
        }
    }

    /// All reference kinds
    pub fn all() -> &'static [ReferenceKind] {
        &[
            ReferenceKind::ComponentRef,
            ReferenceKind::ComponentGroupRef,
            ReferenceKind::DirectoryRef,
            ReferenceKind::FeatureRef,
            ReferenceKind::FeatureGroupRef,
            ReferenceKind::PropertyRef,
            ReferenceKind::CustomActionRef,
            ReferenceKind::BinaryRef,
        ]
    }
}

/// Definition element types in WiX
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "PascalCase")]
pub enum DefinitionKind {
    Component,
    ComponentGroup,
    Directory,
    StandardDirectory,
    Feature,
    FeatureGroup,
    Property,
    CustomAction,
    Binary,
    Fragment,
    Package,
    Module,
    Bundle,
}

impl DefinitionKind {
    /// Try to create from element name
    pub fn from_element_name(name: &str) -> Option<Self> {
        match name {
            "Component" => Some(DefinitionKind::Component),
            "ComponentGroup" => Some(DefinitionKind::ComponentGroup),
            "Directory" => Some(DefinitionKind::Directory),
            "StandardDirectory" => Some(DefinitionKind::StandardDirectory),
            "Feature" => Some(DefinitionKind::Feature),
            "FeatureGroup" => Some(DefinitionKind::FeatureGroup),
            "Property" => Some(DefinitionKind::Property),
            "CustomAction" => Some(DefinitionKind::CustomAction),
            "Binary" => Some(DefinitionKind::Binary),
            "Fragment" => Some(DefinitionKind::Fragment),
            "Package" => Some(DefinitionKind::Package),
            "Module" => Some(DefinitionKind::Module),
            "Bundle" => Some(DefinitionKind::Bundle),
            _ => None,
        }
    }

    /// Get the element name
    pub fn element_name(&self) -> &'static str {
        match self {
            DefinitionKind::Component => "Component",
            DefinitionKind::ComponentGroup => "ComponentGroup",
            DefinitionKind::Directory => "Directory",
            DefinitionKind::StandardDirectory => "StandardDirectory",
            DefinitionKind::Feature => "Feature",
            DefinitionKind::FeatureGroup => "FeatureGroup",
            DefinitionKind::Property => "Property",
            DefinitionKind::CustomAction => "CustomAction",
            DefinitionKind::Binary => "Binary",
            DefinitionKind::Fragment => "Fragment",
            DefinitionKind::Package => "Package",
            DefinitionKind::Module => "Module",
            DefinitionKind::Bundle => "Bundle",
        }
    }

    /// Get the canonical type name for indexing (groups similar types)
    pub fn canonical_type(&self) -> &'static str {
        match self {
            DefinitionKind::Component | DefinitionKind::ComponentGroup => "Component",
            DefinitionKind::Directory | DefinitionKind::StandardDirectory => "Directory",
            DefinitionKind::Feature | DefinitionKind::FeatureGroup => "Feature",
            _ => self.element_name(),
        }
    }
}

/// A symbol definition (Component, Directory, Feature, etc.)
#[derive(Debug, Clone, Serialize)]
pub struct SymbolDefinition {
    /// Symbol identifier (the Id attribute value)
    pub id: String,
    /// Definition kind
    pub kind: DefinitionKind,
    /// Location where defined
    pub location: Location,
    /// Additional metadata (Name, Title, etc.)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

impl SymbolDefinition {
    pub fn new(id: String, kind: DefinitionKind, location: Location) -> Self {
        Self {
            id,
            kind,
            location,
            detail: None,
        }
    }

    pub fn with_detail(mut self, detail: String) -> Self {
        self.detail = Some(detail);
        self
    }
}

/// A symbol reference (ComponentRef, DirectoryRef, etc.)
#[derive(Debug, Clone, Serialize)]
pub struct SymbolReference {
    /// Referenced identifier
    pub id: String,
    /// Reference kind
    pub kind: ReferenceKind,
    /// Location of the reference
    pub location: Location,
}

impl SymbolReference {
    pub fn new(id: String, kind: ReferenceKind, location: Location) -> Self {
        Self { id, kind, location }
    }
}

/// Result of a "Go to Definition" request
#[derive(Debug, Clone, Serialize)]
pub struct DefinitionResult {
    /// The definition location (if found)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub definition: Option<SymbolDefinition>,
    /// Error message (if definition not found)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl DefinitionResult {
    pub fn found(definition: SymbolDefinition) -> Self {
        Self {
            definition: Some(definition),
            error: None,
        }
    }

    pub fn not_found(id: &str, kind: &str) -> Self {
        Self {
            definition: None,
            error: Some(format!("No {} found with Id '{}'", kind, id)),
        }
    }

    pub fn no_symbol() -> Self {
        Self {
            definition: None,
            error: Some("No symbol at this position".to_string()),
        }
    }
}

/// Result of a "Find References" request
#[derive(Debug, Clone, Serialize)]
pub struct ReferencesResult {
    /// The definition (if requested)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub definition: Option<SymbolDefinition>,
    /// All references to the symbol
    pub references: Vec<SymbolReference>,
    /// Total count
    pub count: usize,
}

impl ReferencesResult {
    pub fn new(definition: Option<SymbolDefinition>, references: Vec<SymbolReference>) -> Self {
        let count = references.len();
        Self {
            definition,
            references,
            count,
        }
    }
}

/// What the cursor is on
#[derive(Debug, Clone)]
pub enum SymbolTarget {
    /// On a reference element (ComponentRef Id="X")
    Reference {
        kind: ReferenceKind,
        id: String,
        range: Range,
    },
    /// On a definition element (Component Id="X")
    Definition {
        kind: DefinitionKind,
        id: String,
        range: Range,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reference_kind_mapping() {
        assert_eq!(
            ReferenceKind::ComponentRef.definition_element(),
            "Component"
        );
        assert_eq!(ReferenceKind::DirectoryRef.definition_element(), "Directory");
        assert_eq!(ReferenceKind::FeatureRef.definition_element(), "Feature");
    }

    #[test]
    fn test_reference_kind_from_element() {
        assert_eq!(
            ReferenceKind::from_element_name("ComponentRef"),
            Some(ReferenceKind::ComponentRef)
        );
        assert_eq!(ReferenceKind::from_element_name("Unknown"), None);
    }

    #[test]
    fn test_definition_kind_from_element() {
        assert_eq!(
            DefinitionKind::from_element_name("Component"),
            Some(DefinitionKind::Component)
        );
        assert_eq!(DefinitionKind::from_element_name("Unknown"), None);
    }

    #[test]
    fn test_position_creation() {
        let pos = Position::new(5, 10);
        assert_eq!(pos.line, 5);
        assert_eq!(pos.character, 10);
    }

    #[test]
    fn test_range_from_offsets() {
        let source = "abc\ndef\nghi";
        let range = Range::from_offsets(source, 0, 3);
        assert_eq!(range.start.line, 1);
        assert_eq!(range.start.character, 1);
        assert_eq!(range.end.line, 1);
        assert_eq!(range.end.character, 4);
    }

    #[test]
    fn test_location_creation() {
        let loc = Location::new(
            PathBuf::from("test.wxs"),
            Range::new(1, 1, 1, 10),
        );
        assert_eq!(loc.file, PathBuf::from("test.wxs"));
    }

    #[test]
    fn test_symbol_definition() {
        let def = SymbolDefinition::new(
            "MainComp".to_string(),
            DefinitionKind::Component,
            Location::new(PathBuf::from("test.wxs"), Range::new(1, 1, 1, 30)),
        )
        .with_detail("Main Component".to_string());

        assert_eq!(def.id, "MainComp");
        assert_eq!(def.detail, Some("Main Component".to_string()));
    }

    #[test]
    fn test_definition_result_found() {
        let def = SymbolDefinition::new(
            "Test".to_string(),
            DefinitionKind::Component,
            Location::new(PathBuf::from("test.wxs"), Range::new(1, 1, 1, 10)),
        );
        let result = DefinitionResult::found(def);
        assert!(result.definition.is_some());
        assert!(result.error.is_none());
    }

    #[test]
    fn test_definition_result_not_found() {
        let result = DefinitionResult::not_found("Missing", "Component");
        assert!(result.definition.is_none());
        assert!(result.error.is_some());
    }

    #[test]
    fn test_canonical_type() {
        assert_eq!(DefinitionKind::Component.canonical_type(), "Component");
        assert_eq!(DefinitionKind::ComponentGroup.canonical_type(), "Component");
        assert_eq!(DefinitionKind::Directory.canonical_type(), "Directory");
        assert_eq!(DefinitionKind::StandardDirectory.canonical_type(), "Directory");
    }
}

//! Cross-file symbol index

use std::collections::HashMap;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

use super::extractor::{extract_from_source, ExtractionResult};
use super::types::{DefinitionKind, ReferenceKind, SymbolDefinition, SymbolReference};

/// Cross-file symbol index for WiX projects
#[derive(Debug, Default)]
pub struct SymbolIndex {
    /// Definitions grouped by canonical type, then by id
    definitions: HashMap<String, HashMap<String, SymbolDefinition>>,
    /// References grouped by canonical type, then by id
    references: HashMap<String, HashMap<String, Vec<SymbolReference>>>,
}

impl SymbolIndex {
    pub fn new() -> Self {
        Self::default()
    }

    /// Index a single source file
    pub fn index_source(&mut self, source: &str, file: &Path) -> Result<(), String> {
        let result = extract_from_source(source, file)?;
        self.add_extraction_result(result);
        Ok(())
    }

    /// Index a file from disk
    pub fn index_file(&mut self, path: &Path) -> Result<(), String> {
        let source = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;
        self.index_source(&source, path)
    }

    /// Index all .wxs files in a directory recursively
    pub fn index_directory(&mut self, dir: &Path) -> Result<usize, String> {
        let mut count = 0;
        for entry in WalkDir::new(dir)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.extension().map(|e| e == "wxs").unwrap_or(false) {
                if let Ok(source) = fs::read_to_string(path) {
                    let _ = self.index_source(&source, path);
                    count += 1;
                }
            }
        }
        Ok(count)
    }

    /// Add extraction result to the index
    pub fn add_extraction_result(&mut self, result: ExtractionResult) {
        for def in result.definitions {
            let canonical = def.kind.canonical_type().to_string();
            self.definitions
                .entry(canonical)
                .or_default()
                .insert(def.id.clone(), def);
        }

        for reference in result.references {
            let canonical = canonical_type_for_reference(&reference.kind);
            self.references
                .entry(canonical)
                .or_default()
                .entry(reference.id.clone())
                .or_default()
                .push(reference);
        }
    }

    /// Add a definition manually (for external/builtin definitions)
    pub fn add_definition(&mut self, id: &str, kind: DefinitionKind, detail: Option<String>) {
        use super::types::{Location, Position, Range};
        use std::path::PathBuf;

        let location = Location::new(
            PathBuf::from("<builtin>"),
            Range::new(Position::new(0, 0), Position::new(0, 0)),
        );

        let mut def = SymbolDefinition::new(id, kind, location);
        if let Some(d) = detail {
            def = def.with_detail(d);
        }

        let canonical = kind.canonical_type().to_string();
        self.definitions
            .entry(canonical)
            .or_default()
            .insert(id.to_string(), def);
    }

    /// Get definition by type and id
    pub fn get_definition(&self, element_type: &str, id: &str) -> Option<&SymbolDefinition> {
        let canonical = canonical_type_for_lookup(element_type);
        self.definitions.get(canonical)?.get(id)
    }

    /// Get definition for a reference
    pub fn get_definition_for_ref(&self, reference: &SymbolReference) -> Option<&SymbolDefinition> {
        let canonical = canonical_type_for_reference(&reference.kind);
        self.definitions.get(&canonical)?.get(&reference.id)
    }

    /// Find all references to a definition
    pub fn find_references(&self, def: &SymbolDefinition) -> Vec<&SymbolReference> {
        let canonical = def.kind.canonical_type();
        self.references
            .get(canonical)
            .and_then(|by_id| by_id.get(&def.id))
            .map(|refs| refs.iter().collect())
            .unwrap_or_default()
    }

    /// Check if a definition exists
    pub fn has_definition(&self, element_type: &str, id: &str) -> bool {
        self.get_definition(element_type, id).is_some()
    }

    /// Get all definitions of a type
    pub fn definitions_of_type(&self, element_type: &str) -> Vec<&SymbolDefinition> {
        let canonical = canonical_type_for_lookup(element_type);
        self.definitions
            .get(canonical)
            .map(|by_id| by_id.values().collect())
            .unwrap_or_default()
    }

    /// Get all definitions
    pub fn all_definitions(&self) -> Vec<&SymbolDefinition> {
        self.definitions
            .values()
            .flat_map(|by_id| by_id.values())
            .collect()
    }

    /// Get all references
    pub fn all_references(&self) -> Vec<&SymbolReference> {
        self.references
            .values()
            .flat_map(|by_id| by_id.values().flatten())
            .collect()
    }

    /// Clear the index
    pub fn clear(&mut self) {
        self.definitions.clear();
        self.references.clear();
    }
}

/// Get canonical type for a reference kind
fn canonical_type_for_reference(kind: &ReferenceKind) -> String {
    match kind {
        ReferenceKind::ComponentRef | ReferenceKind::ComponentGroupRef => "Component",
        ReferenceKind::DirectoryRef => "Directory",
        ReferenceKind::FeatureRef | ReferenceKind::FeatureGroupRef => "Feature",
        ReferenceKind::PropertyRef => "Property",
        ReferenceKind::CustomActionRef => "CustomAction",
        ReferenceKind::BinaryRef => "Binary",
    }
    .to_string()
}

/// Get canonical type for lookup (handles both reference and definition element names)
fn canonical_type_for_lookup(element_type: &str) -> &str {
    match element_type {
        "ComponentRef" | "ComponentGroupRef" | "Component" | "ComponentGroup" => "Component",
        "DirectoryRef" | "Directory" | "StandardDirectory" => "Directory",
        "FeatureRef" | "FeatureGroupRef" | "Feature" | "FeatureGroup" => "Feature",
        "PropertyRef" | "Property" => "Property",
        "CustomActionRef" | "CustomAction" => "CustomAction",
        "BinaryRef" | "Binary" => "Binary",
        "Package" | "Module" | "Bundle" => "Package",
        "Fragment" => "Fragment",
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_index_source() {
        let mut index = SymbolIndex::new();

        let source = r#"<Wix>
            <Component Id="C1" />
            <Directory Id="D1" />
        </Wix>"#;

        index.index_source(source, Path::new("test.wxs")).unwrap();

        assert!(index.has_definition("Component", "C1"));
        assert!(index.has_definition("Directory", "D1"));
    }

    #[test]
    fn test_get_definition() {
        let mut index = SymbolIndex::new();

        let source = r#"<Wix><Component Id="C1" /></Wix>"#;
        index.index_source(source, Path::new("test.wxs")).unwrap();

        let def = index.get_definition("Component", "C1").unwrap();
        assert_eq!(def.id, "C1");
        assert_eq!(def.kind, DefinitionKind::Component);
    }

    #[test]
    fn test_find_references() {
        let mut index = SymbolIndex::new();

        let source = r#"<Wix>
            <Component Id="C1" />
            <Feature Id="F1">
                <ComponentRef Id="C1" />
            </Feature>
        </Wix>"#;

        index.index_source(source, Path::new("test.wxs")).unwrap();

        let def = index.get_definition("Component", "C1").unwrap();
        let refs = index.find_references(def);

        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].id, "C1");
    }

    #[test]
    fn test_component_group_ref() {
        let mut index = SymbolIndex::new();

        let source = r#"<Wix>
            <ComponentGroup Id="CG1" />
            <Feature Id="F1">
                <ComponentGroupRef Id="CG1" />
            </Feature>
        </Wix>"#;

        index.index_source(source, Path::new("test.wxs")).unwrap();

        // ComponentGroup should be found via Component canonical type
        assert!(index.has_definition("ComponentGroup", "CG1"));
        assert!(index.has_definition("Component", "CG1")); // Same canonical type
    }

    #[test]
    fn test_add_definition_manually() {
        let mut index = SymbolIndex::new();
        index.add_definition(
            "TARGETDIR",
            DefinitionKind::Directory,
            Some("Standard directory".to_string()),
        );

        assert!(index.has_definition("Directory", "TARGETDIR"));
    }

    #[test]
    fn test_add_definition_without_detail() {
        let mut index = SymbolIndex::new();
        index.add_definition("CustomDir", DefinitionKind::Directory, None);

        assert!(index.has_definition("Directory", "CustomDir"));
        let def = index.get_definition("Directory", "CustomDir").unwrap();
        assert!(def.detail.is_none());
    }

    #[test]
    fn test_definitions_of_type() {
        let mut index = SymbolIndex::new();

        let source = r#"<Wix>
            <Component Id="C1" />
            <Component Id="C2" />
            <Directory Id="D1" />
        </Wix>"#;

        index.index_source(source, Path::new("test.wxs")).unwrap();

        let components = index.definitions_of_type("Component");
        assert_eq!(components.len(), 2);

        let directories = index.definitions_of_type("Directory");
        assert_eq!(directories.len(), 1);
    }

    #[test]
    fn test_definitions_of_type_unknown() {
        let index = SymbolIndex::new();
        let unknown = index.definitions_of_type("UnknownType");
        assert!(unknown.is_empty());
    }

    #[test]
    fn test_cross_file_indexing() {
        let mut index = SymbolIndex::new();

        // File 1: definitions
        let source1 = r#"<Wix><Component Id="C1" /></Wix>"#;
        index.index_source(source1, Path::new("defs.wxs")).unwrap();

        // File 2: references
        let source2 = r#"<Wix><ComponentRef Id="C1" /></Wix>"#;
        index.index_source(source2, Path::new("refs.wxs")).unwrap();

        let def = index.get_definition("Component", "C1").unwrap();
        let refs = index.find_references(def);

        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].location.file.to_str().unwrap(), "refs.wxs");
    }

    #[test]
    fn test_index_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.wxs");
        {
            let mut file = File::create(&file_path).unwrap();
            writeln!(file, r#"<Wix><Component Id="FileComp" /></Wix>"#).unwrap();
        }

        let mut index = SymbolIndex::new();
        index.index_file(&file_path).unwrap();

        assert!(index.has_definition("Component", "FileComp"));
    }

    #[test]
    fn test_index_file_not_found() {
        let mut index = SymbolIndex::new();
        let result = index.index_file(Path::new("/nonexistent/path.wxs"));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Failed to read"));
    }

    #[test]
    fn test_index_directory() {
        let temp_dir = TempDir::new().unwrap();

        // Create two .wxs files
        let file1 = temp_dir.path().join("file1.wxs");
        {
            let mut f = File::create(&file1).unwrap();
            writeln!(f, r#"<Wix><Component Id="C1" /></Wix>"#).unwrap();
        }

        let file2 = temp_dir.path().join("file2.wxs");
        {
            let mut f = File::create(&file2).unwrap();
            writeln!(f, r#"<Wix><Component Id="C2" /></Wix>"#).unwrap();
        }

        // Create a non-.wxs file (should be ignored)
        let file3 = temp_dir.path().join("readme.txt");
        {
            let mut f = File::create(&file3).unwrap();
            writeln!(f, "readme").unwrap();
        }

        let mut index = SymbolIndex::new();
        let count = index.index_directory(temp_dir.path()).unwrap();

        assert_eq!(count, 2);
        assert!(index.has_definition("Component", "C1"));
        assert!(index.has_definition("Component", "C2"));
    }

    #[test]
    fn test_get_definition_for_ref() {
        let mut index = SymbolIndex::new();

        let source = r#"<Wix>
            <Component Id="C1" />
            <Feature Id="F1">
                <ComponentRef Id="C1" />
            </Feature>
        </Wix>"#;

        index.index_source(source, Path::new("test.wxs")).unwrap();

        let refs = index.all_references();
        let comp_ref = refs.iter().find(|r| r.id == "C1").unwrap();

        let def = index.get_definition_for_ref(comp_ref).unwrap();
        assert_eq!(def.id, "C1");
    }

    #[test]
    fn test_all_definitions() {
        let mut index = SymbolIndex::new();

        let source = r#"<Wix>
            <Component Id="C1" />
            <Directory Id="D1" />
            <Feature Id="F1" />
        </Wix>"#;

        index.index_source(source, Path::new("test.wxs")).unwrap();

        let all = index.all_definitions();
        assert_eq!(all.len(), 3);
    }

    #[test]
    fn test_all_references() {
        let mut index = SymbolIndex::new();

        let source = r#"<Wix>
            <ComponentRef Id="C1" />
            <DirectoryRef Id="D1" />
        </Wix>"#;

        index.index_source(source, Path::new("test.wxs")).unwrap();

        let all = index.all_references();
        assert_eq!(all.len(), 2);
    }

    #[test]
    fn test_clear() {
        let mut index = SymbolIndex::new();

        let source = r#"<Wix><Component Id="C1" /></Wix>"#;
        index.index_source(source, Path::new("test.wxs")).unwrap();

        assert!(index.has_definition("Component", "C1"));

        index.clear();

        assert!(!index.has_definition("Component", "C1"));
        assert!(index.all_definitions().is_empty());
        assert!(index.all_references().is_empty());
    }

    #[test]
    fn test_canonical_type_for_lookup_variants() {
        // These are covered via has_definition calls
        let mut index = SymbolIndex::new();

        let source = r#"<Wix>
            <Feature Id="F1" />
            <FeatureGroup Id="FG1" />
            <Property Id="P1" Value="test" />
            <CustomAction Id="CA1" />
            <Binary Id="B1" SourceFile="test.dll" />
            <Fragment Id="Frag1" />
            <Package Name="TestPkg" />
        </Wix>"#;

        index.index_source(source, Path::new("test.wxs")).unwrap();

        // Test canonical type lookup for definitions
        assert!(index.has_definition("Feature", "F1"));
        assert!(index.has_definition("FeatureGroup", "FG1"));
        assert!(index.has_definition("Feature", "FG1")); // Same canonical
        assert!(index.has_definition("Property", "P1"));
        assert!(index.has_definition("CustomAction", "CA1"));
        assert!(index.has_definition("Binary", "B1"));
        assert!(index.has_definition("Fragment", "Frag1"));
        assert!(index.has_definition("Package", "TestPkg"));
    }

    #[test]
    fn test_canonical_type_for_reference_variants() {
        let mut index = SymbolIndex::new();

        let source = r#"<Wix>
            <DirectoryRef Id="D1" />
            <FeatureRef Id="F1" />
            <FeatureGroupRef Id="FG1" />
            <PropertyRef Id="P1" />
            <CustomActionRef Id="CA1" />
            <BinaryRef Id="B1" />
        </Wix>"#;

        index.index_source(source, Path::new("test.wxs")).unwrap();

        let all_refs = index.all_references();
        assert_eq!(all_refs.len(), 6);

        // Verify reference kinds
        assert!(all_refs
            .iter()
            .any(|r| r.kind == ReferenceKind::DirectoryRef));
        assert!(all_refs.iter().any(|r| r.kind == ReferenceKind::FeatureRef));
        assert!(all_refs
            .iter()
            .any(|r| r.kind == ReferenceKind::FeatureGroupRef));
        assert!(all_refs
            .iter()
            .any(|r| r.kind == ReferenceKind::PropertyRef));
        assert!(all_refs
            .iter()
            .any(|r| r.kind == ReferenceKind::CustomActionRef));
        assert!(all_refs.iter().any(|r| r.kind == ReferenceKind::BinaryRef));
    }

    #[test]
    fn test_find_references_no_refs() {
        let mut index = SymbolIndex::new();

        let source = r#"<Wix><Component Id="C1" /></Wix>"#;
        index.index_source(source, Path::new("test.wxs")).unwrap();

        let def = index.get_definition("Component", "C1").unwrap();
        let refs = index.find_references(def);

        assert!(refs.is_empty());
    }
}

//! Symbol index for cross-file reference resolution

use crate::extractor::extract_from_source;
use crate::types::{DefinitionKind, ReferenceKind, SymbolDefinition, SymbolReference};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Project-wide symbol index
#[derive(Debug, Default)]
pub struct SymbolIndex {
    /// Definitions: canonical_type -> id -> SymbolDefinition
    definitions: HashMap<String, HashMap<String, SymbolDefinition>>,
    /// References: canonical_type -> id -> Vec<SymbolReference>
    references: HashMap<String, HashMap<String, Vec<SymbolReference>>>,
    /// Indexed files
    indexed_files: Vec<PathBuf>,
}

impl SymbolIndex {
    /// Create a new empty index
    pub fn new() -> Self {
        Self::default()
    }

    /// Index a single file
    pub fn index_file(&mut self, path: &Path, source: &str) -> Result<(), String> {
        let result = extract_from_source(source, path)?;

        // Add definitions
        for def in result.definitions {
            let type_key = def.kind.canonical_type().to_string();
            self.definitions
                .entry(type_key)
                .or_default()
                .insert(def.id.clone(), def);
        }

        // Add references
        for reference in result.references {
            let type_key = reference.kind.definition_element().to_string();
            self.references
                .entry(type_key)
                .or_default()
                .entry(reference.id.clone())
                .or_default()
                .push(reference);
        }

        self.indexed_files.push(path.to_path_buf());

        Ok(())
    }

    /// Index a file from disk
    pub fn index_file_from_disk(&mut self, path: &Path) -> Result<(), String> {
        let source =
            fs::read_to_string(path).map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;
        self.index_file(path, &source)
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
                if let Err(e) = self.index_file_from_disk(path) {
                    eprintln!("Warning: {}", e);
                } else {
                    count += 1;
                }
            }
        }

        Ok(count)
    }

    /// Index multiple files
    pub fn index_files(&mut self, files: &[PathBuf]) -> Result<usize, String> {
        let mut count = 0;

        for path in files {
            if let Err(e) = self.index_file_from_disk(path) {
                eprintln!("Warning: {}", e);
            } else {
                count += 1;
            }
        }

        Ok(count)
    }

    /// Get definition for a symbol by type and id
    pub fn get_definition(&self, element_type: &str, id: &str) -> Option<&SymbolDefinition> {
        // Map reference and definition types to canonical types
        let canonical = match element_type {
            "ComponentRef" | "ComponentGroupRef" | "Component" | "ComponentGroup" => "Component",
            "DirectoryRef" | "Directory" | "StandardDirectory" => "Directory",
            "FeatureRef" | "FeatureGroupRef" | "Feature" | "FeatureGroup" => "Feature",
            "PropertyRef" | "Property" => "Property",
            "CustomActionRef" | "CustomAction" => "CustomAction",
            "BinaryRef" | "Binary" => "Binary",
            other => other,
        };

        self.definitions.get(canonical)?.get(id)
    }

    /// Get definition for a reference kind
    pub fn get_definition_for_ref(
        &self,
        kind: ReferenceKind,
        id: &str,
    ) -> Option<&SymbolDefinition> {
        self.get_definition(kind.definition_element(), id)
    }

    /// Get all references to a symbol
    pub fn get_references(&self, element_type: &str, id: &str) -> Vec<&SymbolReference> {
        // Canonical type for lookups
        let canonical = match element_type {
            "Component" | "ComponentGroup" => "Component",
            "Directory" | "StandardDirectory" => "Directory",
            "Feature" | "FeatureGroup" => "Feature",
            other => other,
        };

        self.references
            .get(canonical)
            .and_then(|m| m.get(id))
            .map(|refs| refs.iter().collect())
            .unwrap_or_default()
    }

    /// Get all definitions
    pub fn all_definitions(&self) -> Vec<&SymbolDefinition> {
        self.definitions
            .values()
            .flat_map(|m| m.values())
            .collect()
    }

    /// Get all references
    pub fn all_references(&self) -> Vec<&SymbolReference> {
        self.references
            .values()
            .flat_map(|m| m.values().flatten())
            .collect()
    }

    /// Get count of definitions
    pub fn definition_count(&self) -> usize {
        self.definitions.values().map(|m| m.len()).sum()
    }

    /// Get count of references
    pub fn reference_count(&self) -> usize {
        self.references
            .values()
            .map(|m| m.values().map(|v| v.len()).sum::<usize>())
            .sum()
    }

    /// Get indexed file count
    pub fn file_count(&self) -> usize {
        self.indexed_files.len()
    }

    /// Check if a definition exists
    pub fn has_definition(&self, element_type: &str, id: &str) -> bool {
        self.get_definition(element_type, id).is_some()
    }

    /// Find all unreferenced definitions
    pub fn find_unreferenced(&self, kind: DefinitionKind) -> Vec<&SymbolDefinition> {
        let canonical = kind.canonical_type();

        let Some(defs) = self.definitions.get(canonical) else {
            return vec![];
        };

        let refs = self.references.get(canonical);

        defs.values()
            .filter(|def| {
                def.kind == kind
                    && refs
                        .map(|r| !r.contains_key(&def.id))
                        .unwrap_or(true)
            })
            .collect()
    }

    /// Find all missing definitions (references without definitions)
    pub fn find_missing_definitions(&self) -> Vec<(&SymbolReference, &str)> {
        let mut missing = Vec::new();

        for (type_key, id_map) in &self.references {
            for (id, refs) in id_map {
                if !self.has_definition(type_key, id) {
                    for reference in refs {
                        missing.push((reference, type_key.as_str()));
                    }
                }
            }
        }

        missing
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_index_component() {
        let mut index = SymbolIndex::new();
        let source = r#"<Wix><Component Id="MainComp" Guid="*" /></Wix>"#;

        index.index_file(Path::new("test.wxs"), source).unwrap();

        assert!(index.has_definition("Component", "MainComp"));
        assert_eq!(index.definition_count(), 1);
    }

    #[test]
    fn test_index_reference() {
        let mut index = SymbolIndex::new();
        let source = r#"<Wix><ComponentRef Id="MainComp" /></Wix>"#;

        index.index_file(Path::new("test.wxs"), source).unwrap();

        let refs = index.get_references("Component", "MainComp");
        assert_eq!(refs.len(), 1);
    }

    #[test]
    fn test_get_definition_for_ref() {
        let mut index = SymbolIndex::new();
        let source = r#"<Wix><Component Id="MainComp" /><ComponentRef Id="MainComp" /></Wix>"#;

        index.index_file(Path::new("test.wxs"), source).unwrap();

        let def = index.get_definition_for_ref(ReferenceKind::ComponentRef, "MainComp");
        assert!(def.is_some());
        assert_eq!(def.unwrap().id, "MainComp");
    }

    #[test]
    fn test_cross_file_references() {
        let mut index = SymbolIndex::new();

        // File 1: Definition
        let source1 = r#"<Wix><Component Id="SharedComp" /></Wix>"#;
        index.index_file(Path::new("defs.wxs"), source1).unwrap();

        // File 2: Reference
        let source2 = r#"<Wix><ComponentRef Id="SharedComp" /></Wix>"#;
        index.index_file(Path::new("refs.wxs"), source2).unwrap();

        // Should find definition from different file
        let def = index.get_definition("Component", "SharedComp");
        assert!(def.is_some());
        assert_eq!(def.unwrap().location.file, Path::new("defs.wxs"));
    }

    #[test]
    fn test_multiple_references() {
        let mut index = SymbolIndex::new();
        let source = r#"
<Wix>
    <Component Id="Comp1" />
    <Feature Id="F1"><ComponentRef Id="Comp1" /></Feature>
    <Feature Id="F2"><ComponentRef Id="Comp1" /></Feature>
</Wix>"#;

        index.index_file(Path::new("test.wxs"), source).unwrap();

        let refs = index.get_references("Component", "Comp1");
        assert_eq!(refs.len(), 2);
    }

    #[test]
    fn test_find_missing_definitions() {
        let mut index = SymbolIndex::new();
        let source = r#"<Wix><ComponentRef Id="MissingComp" /></Wix>"#;

        index.index_file(Path::new("test.wxs"), source).unwrap();

        let missing = index.find_missing_definitions();
        assert_eq!(missing.len(), 1);
        assert_eq!(missing[0].0.id, "MissingComp");
    }

    #[test]
    fn test_find_unreferenced() {
        let mut index = SymbolIndex::new();
        let source = r#"
<Wix>
    <Component Id="UsedComp" />
    <Component Id="UnusedComp" />
    <ComponentRef Id="UsedComp" />
</Wix>"#;

        index.index_file(Path::new("test.wxs"), source).unwrap();

        let unreferenced = index.find_unreferenced(DefinitionKind::Component);
        assert_eq!(unreferenced.len(), 1);
        assert_eq!(unreferenced[0].id, "UnusedComp");
    }

    #[test]
    fn test_directory_ref_to_directory() {
        let mut index = SymbolIndex::new();
        let source = r#"
<Wix>
    <Directory Id="TARGETDIR" />
    <DirectoryRef Id="TARGETDIR" />
</Wix>"#;

        index.index_file(Path::new("test.wxs"), source).unwrap();

        let def = index.get_definition("DirectoryRef", "TARGETDIR");
        assert!(def.is_some());
    }

    #[test]
    fn test_feature_ref_to_feature() {
        let mut index = SymbolIndex::new();
        let source = r#"
<Wix>
    <Feature Id="MainFeature" />
    <FeatureRef Id="MainFeature" />
</Wix>"#;

        index.index_file(Path::new("test.wxs"), source).unwrap();

        let def = index.get_definition_for_ref(ReferenceKind::FeatureRef, "MainFeature");
        assert!(def.is_some());
    }

    #[test]
    fn test_component_group_ref() {
        let mut index = SymbolIndex::new();
        let source = r#"
<Wix>
    <ComponentGroup Id="MyGroup" />
    <ComponentGroupRef Id="MyGroup" />
</Wix>"#;

        index.index_file(Path::new("test.wxs"), source).unwrap();

        let def = index.get_definition_for_ref(ReferenceKind::ComponentGroupRef, "MyGroup");
        assert!(def.is_some());
    }

    #[test]
    fn test_all_definitions() {
        let mut index = SymbolIndex::new();
        let source = r#"
<Wix>
    <Component Id="C1" />
    <Component Id="C2" />
    <Directory Id="D1" />
</Wix>"#;

        index.index_file(Path::new("test.wxs"), source).unwrap();

        let all = index.all_definitions();
        assert_eq!(all.len(), 3);
    }

    #[test]
    fn test_file_count() {
        let mut index = SymbolIndex::new();

        index.index_file(Path::new("f1.wxs"), "<Wix />").unwrap();
        index.index_file(Path::new("f2.wxs"), "<Wix />").unwrap();

        assert_eq!(index.file_count(), 2);
    }
}

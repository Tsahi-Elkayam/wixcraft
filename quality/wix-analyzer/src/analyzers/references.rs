//! Reference navigation - Go to Definition, Find References

use crate::core::{
    AnalysisResult, Location, SymbolIndex,
    WixDocument, symbol_at_position, SymbolAtPosition,
};
use super::Analyzer;

/// Analyzer for reference navigation (Go to Definition, Find References)
pub struct ReferencesAnalyzer;

impl ReferencesAnalyzer {
    pub fn new() -> Self {
        Self
    }

    /// Go to definition from a position
    pub fn go_to_definition(
        &self,
        doc: &WixDocument,
        index: &SymbolIndex,
        line: usize,
        column: usize,
    ) -> Option<Location> {
        let symbol = symbol_at_position(doc, line, column)?;

        match symbol {
            SymbolAtPosition::Reference { id, kind, .. } => {
                let element_type = kind.definition_element();
                let def = index.get_definition(element_type, &id)?;
                Some(def.location.clone())
            }
            SymbolAtPosition::Definition { .. } => {
                // Already at definition
                None
            }
        }
    }

    /// Find all references to symbol at position
    pub fn find_references(
        &self,
        doc: &WixDocument,
        index: &SymbolIndex,
        line: usize,
        column: usize,
    ) -> Vec<Location> {
        let symbol = match symbol_at_position(doc, line, column) {
            Some(s) => s,
            None => return Vec::new(),
        };

        match symbol {
            SymbolAtPosition::Definition { id, kind, .. } => {
                // Find the definition in the index
                if let Some(def) = index.get_definition(kind.element_name(), &id) {
                    index
                        .find_references(def)
                        .into_iter()
                        .map(|r| r.location.clone())
                        .collect()
                } else {
                    Vec::new()
                }
            }
            SymbolAtPosition::Reference { id, kind, .. } => {
                // Find the definition first, then all its references
                let element_type = kind.definition_element();
                if let Some(def) = index.get_definition(element_type, &id) {
                    index
                        .find_references(def)
                        .into_iter()
                        .map(|r| r.location.clone())
                        .collect()
                } else {
                    Vec::new()
                }
            }
        }
    }
}

impl Default for ReferencesAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl Analyzer for ReferencesAnalyzer {
    fn analyze(&self, _doc: &WixDocument, _index: &SymbolIndex) -> AnalysisResult {
        // Reference analyzer doesn't produce diagnostics, it's for navigation
        AnalysisResult::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn setup_index(source: &str) -> SymbolIndex {
        let mut index = SymbolIndex::new();
        index.index_source(source, Path::new("test.wxs")).unwrap();
        index
    }

    #[test]
    fn test_default_impl() {
        let analyzer = ReferencesAnalyzer::default();
        let doc = WixDocument::parse("<Wix />", Path::new("test.wxs")).unwrap();
        let index = SymbolIndex::new();
        let result = analyzer.analyze(&doc, &index);
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_go_to_definition() {
        let source = r#"<Wix>
    <Component Id="C1" />
    <Feature Id="F1">
        <ComponentRef Id="C1" />
    </Feature>
</Wix>"#;

        let index = setup_index(source);
        let doc = WixDocument::parse(source, Path::new("test.wxs")).unwrap();
        let analyzer = ReferencesAnalyzer::new();

        // Position on ComponentRef
        let location = analyzer.go_to_definition(&doc, &index, 4, 10);
        assert!(location.is_some());

        let loc = location.unwrap();
        assert_eq!(loc.range.start.line, 2); // Component is on line 2
    }

    #[test]
    fn test_go_to_definition_already_at_definition() {
        let source = r#"<Wix>
    <Component Id="C1" />
</Wix>"#;

        let index = setup_index(source);
        let doc = WixDocument::parse(source, Path::new("test.wxs")).unwrap();
        let analyzer = ReferencesAnalyzer::new();

        // Position on Component definition (not a ref)
        let location = analyzer.go_to_definition(&doc, &index, 2, 10);
        assert!(location.is_none()); // Already at definition
    }

    #[test]
    fn test_go_to_definition_no_symbol() {
        let source = r#"<Wix>
    <Component Id="C1" />
</Wix>"#;

        let index = setup_index(source);
        let doc = WixDocument::parse(source, Path::new("test.wxs")).unwrap();
        let analyzer = ReferencesAnalyzer::new();

        // Position on whitespace/no symbol
        let location = analyzer.go_to_definition(&doc, &index, 1, 1);
        assert!(location.is_none());
    }

    #[test]
    fn test_go_to_definition_ref_not_found() {
        let source = r#"<Wix>
    <ComponentRef Id="NonExistent" />
</Wix>"#;

        let index = setup_index(source);
        let doc = WixDocument::parse(source, Path::new("test.wxs")).unwrap();
        let analyzer = ReferencesAnalyzer::new();

        // Reference to non-existent component
        let location = analyzer.go_to_definition(&doc, &index, 2, 10);
        assert!(location.is_none());
    }

    #[test]
    fn test_find_references() {
        let source = r#"<Wix>
    <Component Id="C1" />
    <Feature Id="F1">
        <ComponentRef Id="C1" />
    </Feature>
    <Feature Id="F2">
        <ComponentRef Id="C1" />
    </Feature>
</Wix>"#;

        let index = setup_index(source);
        let doc = WixDocument::parse(source, Path::new("test.wxs")).unwrap();
        let analyzer = ReferencesAnalyzer::new();

        // Position on Component definition
        let refs = analyzer.find_references(&doc, &index, 2, 10);
        assert_eq!(refs.len(), 2);
    }

    #[test]
    fn test_find_references_no_symbol() {
        let source = r#"<Wix>
    <Component Id="C1" />
</Wix>"#;

        let index = setup_index(source);
        let doc = WixDocument::parse(source, Path::new("test.wxs")).unwrap();
        let analyzer = ReferencesAnalyzer::new();

        // Position on whitespace/no symbol
        let refs = analyzer.find_references(&doc, &index, 1, 1);
        assert!(refs.is_empty());
    }

    #[test]
    fn test_find_references_from_ref() {
        let source = r#"<Wix>
    <Component Id="C1" />
    <Feature Id="F1">
        <ComponentRef Id="C1" />
    </Feature>
</Wix>"#;

        let index = setup_index(source);
        let doc = WixDocument::parse(source, Path::new("test.wxs")).unwrap();
        let analyzer = ReferencesAnalyzer::new();

        // Position on ComponentRef
        let refs = analyzer.find_references(&doc, &index, 4, 10);
        assert_eq!(refs.len(), 1);
    }

    #[test]
    fn test_find_references_def_not_in_index() {
        let source = r#"<Wix>
    <Component Id="C1" />
</Wix>"#;

        // Don't index the source - empty index
        let index = SymbolIndex::new();
        let doc = WixDocument::parse(source, Path::new("test.wxs")).unwrap();
        let analyzer = ReferencesAnalyzer::new();

        // Definition not in index
        let refs = analyzer.find_references(&doc, &index, 2, 10);
        assert!(refs.is_empty());
    }

    #[test]
    fn test_find_references_from_orphan_ref() {
        let source = r#"<Wix>
    <ComponentRef Id="NonExistent" />
</Wix>"#;

        let index = setup_index(source);
        let doc = WixDocument::parse(source, Path::new("test.wxs")).unwrap();
        let analyzer = ReferencesAnalyzer::new();

        // Reference to non-existent definition
        let refs = analyzer.find_references(&doc, &index, 2, 10);
        assert!(refs.is_empty());
    }
}

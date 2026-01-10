//! Project index for cross-file symbol references

use crate::types::Location;
use anyhow::Result;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// A symbol in the project
#[derive(Debug, Clone)]
pub struct Symbol {
    /// Symbol name (e.g., "MyComponent")
    pub name: String,

    /// Symbol kind (e.g., "Component", "Directory", "Feature")
    pub kind: String,

    /// Location of definition
    pub location: Location,

    /// Parent element ID (if any)
    pub parent_id: Option<String>,

    /// Preview text (line content)
    pub preview: Option<String>,
}

/// Index of all symbols across project files
#[derive(Debug, Default)]
pub struct ProjectIndex {
    /// Symbols by kind -> name -> locations
    symbols: HashMap<String, HashMap<String, Vec<Symbol>>>,

    /// All words from all files (for All Autocomplete)
    words: HashMap<String, Vec<PathBuf>>,

    /// Files that have been indexed
    indexed_files: Vec<PathBuf>,
}

impl ProjectIndex {
    pub fn new() -> Self {
        Self::default()
    }

    /// Index all WiX files in a directory
    pub fn index_directory(&mut self, root: &Path) -> Result<usize> {
        use walkdir::WalkDir;

        let mut count = 0;

        for entry in WalkDir::new(root)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path()
                    .extension()
                    .map(|ext| ext == "wxs" || ext == "wxi")
                    .unwrap_or(false)
            })
        {
            if self.index_file(entry.path()).is_ok() {
                count += 1;
            }
        }

        Ok(count)
    }

    /// Index a single file
    pub fn index_file(&mut self, path: &Path) -> Result<()> {
        let content = std::fs::read_to_string(path)?;
        let path_buf = path.to_path_buf();

        // Remove old entries for this file
        self.remove_file(&path_buf);

        // Parse and index symbols
        self.parse_symbols(&content, &path_buf);

        // Index words
        self.index_words(&content, &path_buf);

        self.indexed_files.push(path_buf);

        Ok(())
    }

    /// Remove a file from the index
    pub fn remove_file(&mut self, path: &PathBuf) {
        // Remove from symbols
        for kind_map in self.symbols.values_mut() {
            for symbols in kind_map.values_mut() {
                symbols.retain(|s| s.location.path != *path);
            }
        }

        // Remove from words
        for paths in self.words.values_mut() {
            paths.retain(|p| p != path);
        }

        self.indexed_files.retain(|p| p != path);
    }

    /// Parse symbols from XML content
    fn parse_symbols(&mut self, content: &str, path: &PathBuf) {
        // Simple regex-free parser for Id attributes on key elements
        let symbol_elements = [
            "Component",
            "ComponentGroup",
            "Directory",
            "DirectoryRef",
            "StandardDirectory",
            "Feature",
            "FeatureGroup",
            "Property",
            "CustomAction",
            "Binary",
            "Fragment",
        ];

        for (line_num, line) in content.lines().enumerate() {
            let line_num = line_num + 1; // 1-based

            for elem in &symbol_elements {
                // Look for <Element ... Id="..."
                if let Some(start) = line.find(&format!("<{}", elem)) {
                    let after_tag = &line[start..];

                    // Extract Id attribute
                    if let Some(id) = extract_attribute(after_tag, "Id") {
                        let col = start + 1;
                        let symbol = Symbol {
                            name: id.clone(),
                            kind: elem.to_string(),
                            location: Location::point(
                                path.clone(),
                                line_num as u32,
                                col as u32,
                            ),
                            parent_id: None,
                            preview: Some(line.trim().to_string()),
                        };

                        self.add_symbol(symbol);
                    }
                }
            }
        }
    }

    /// Index words from content
    fn index_words(&mut self, content: &str, path: &PathBuf) {
        let word_re = |c: char| c.is_alphanumeric() || c == '_';

        for word in content.split(|c: char| !word_re(c)) {
            if word.len() >= 3 {
                // Skip very short words
                self.words
                    .entry(word.to_string())
                    .or_default()
                    .push(path.clone());
            }
        }
    }

    /// Add a symbol to the index
    fn add_symbol(&mut self, symbol: Symbol) {
        self.symbols
            .entry(symbol.kind.clone())
            .or_default()
            .entry(symbol.name.clone())
            .or_default()
            .push(symbol);
    }

    /// Find symbols by name (across all kinds)
    pub fn find_symbol(&self, name: &str) -> Vec<&Symbol> {
        let mut results = Vec::new();

        for kind_map in self.symbols.values() {
            if let Some(symbols) = kind_map.get(name) {
                results.extend(symbols.iter());
            }
        }

        results
    }

    /// Find symbols by kind and name
    pub fn find_symbol_by_kind(&self, kind: &str, name: &str) -> Vec<&Symbol> {
        self.symbols
            .get(kind)
            .and_then(|m| m.get(name))
            .map(|v| v.iter().collect())
            .unwrap_or_default()
    }

    /// Find definition for a reference element
    /// e.g., ComponentRef -> Component, DirectoryRef -> Directory
    pub fn find_definition_for_ref(&self, ref_kind: &str, id: &str) -> Option<&Symbol> {
        // Map ref elements to their targets
        let target_kind = match ref_kind {
            "ComponentRef" => "Component",
            "ComponentGroupRef" => "ComponentGroup",
            "DirectoryRef" => "Directory",
            "FeatureRef" => "Feature",
            "FeatureGroupRef" => "FeatureGroup",
            "PropertyRef" => "Property",
            "CustomActionRef" => "CustomAction",
            "UIRef" => "UI",
            _ => return None,
        };

        self.find_symbol_by_kind(target_kind, id).first().copied()
    }

    /// Get words matching prefix (for All Autocomplete)
    pub fn get_words_matching(&self, prefix: &str, limit: usize) -> Vec<&str> {
        let prefix_lower = prefix.to_lowercase();
        self.words
            .keys()
            .filter(|w| w.to_lowercase().starts_with(&prefix_lower))
            .take(limit)
            .map(|s| s.as_str())
            .collect()
    }

    /// Get all symbols of a kind
    pub fn get_symbols_by_kind(&self, kind: &str) -> Vec<&Symbol> {
        self.symbols
            .get(kind)
            .map(|m| m.values().flatten().collect())
            .unwrap_or_default()
    }

    /// Get file count
    pub fn file_count(&self) -> usize {
        self.indexed_files.len()
    }

    /// Get symbol count
    pub fn symbol_count(&self) -> usize {
        self.symbols.values().map(|m| m.values().map(|v| v.len()).sum::<usize>()).sum()
    }

    /// Get word count
    pub fn word_count(&self) -> usize {
        self.words.len()
    }
}

/// Extract attribute value from tag string
fn extract_attribute(tag: &str, attr_name: &str) -> Option<String> {
    // Look for attr="value" or attr='value'
    let patterns = [
        format!("{}=\"", attr_name),
        format!("{}='", attr_name),
    ];

    for pattern in &patterns {
        if let Some(start) = tag.find(pattern) {
            let value_start = start + pattern.len();
            let quote = pattern.chars().last().unwrap();
            if let Some(end) = tag[value_start..].find(quote) {
                return Some(tag[value_start..value_start + end].to_string());
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_attribute() {
        assert_eq!(
            extract_attribute(r#"<Component Id="MyComp" Guid="*">"#, "Id"),
            Some("MyComp".to_string())
        );
        assert_eq!(
            extract_attribute(r#"<Component Id='MyComp'>"#, "Id"),
            Some("MyComp".to_string())
        );
        assert_eq!(extract_attribute(r#"<Component>"#, "Id"), None);
    }

    #[test]
    fn test_index_symbols() {
        let mut index = ProjectIndex::new();
        let content = r#"
<Wix>
  <Package>
    <Component Id="C1" Guid="*">
    </Component>
    <Component Id="C2" Guid="*">
    </Component>
  </Package>
</Wix>
"#;

        index.parse_symbols(content, &PathBuf::from("test.wxs"));

        assert_eq!(index.symbol_count(), 2);

        let c1 = index.find_symbol("C1");
        assert_eq!(c1.len(), 1);
        assert_eq!(c1[0].kind, "Component");

        let all_components = index.get_symbols_by_kind("Component");
        assert_eq!(all_components.len(), 2);
    }

    #[test]
    fn test_index_words() {
        let mut index = ProjectIndex::new();
        let content = "Hello World TestWord AnotherTest";

        index.index_words(content, &PathBuf::from("test.wxs"));

        let matches = index.get_words_matching("Test", 10);
        assert!(matches.contains(&"TestWord"));
    }

    #[test]
    fn test_find_definition_for_ref() {
        let mut index = ProjectIndex::new();
        let content = r#"
<Component Id="MyComp">
</Component>
"#;

        index.parse_symbols(content, &PathBuf::from("test.wxs"));

        let def = index.find_definition_for_ref("ComponentRef", "MyComp");
        assert!(def.is_some());
        assert_eq!(def.unwrap().name, "MyComp");
    }

    #[test]
    fn test_remove_file() {
        let mut index = ProjectIndex::new();
        let path1 = PathBuf::from("file1.wxs");
        let path2 = PathBuf::from("file2.wxs");

        index.parse_symbols(r#"<Component Id="C1">"#, &path1);
        index.parse_symbols(r#"<Component Id="C2">"#, &path2);
        index.indexed_files.push(path1.clone());
        index.indexed_files.push(path2.clone());

        assert_eq!(index.symbol_count(), 2);
        assert_eq!(index.file_count(), 2);

        index.remove_file(&path1);

        assert_eq!(index.symbol_count(), 1);
        assert_eq!(index.file_count(), 1);
    }
}

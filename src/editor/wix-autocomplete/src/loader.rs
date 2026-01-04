//! Load wix-data (elements, keywords, snippets)

use crate::types::{AttributeDef, ElementDef, Keywords, Snippet};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LoadError {
    #[error("Failed to read directory: {0}")]
    ReadDir(#[from] std::io::Error),
    #[error("Failed to parse {file}: {source}")]
    Parse {
        file: PathBuf,
        source: serde_json::Error,
    },
    #[error("wix-data directory not found: {0}")]
    NotFound(PathBuf),
}

/// Loaded wix-data for autocomplete
#[derive(Debug, Default)]
pub struct WixData {
    /// Element definitions indexed by name
    pub elements: HashMap<String, ElementDef>,
    /// Parent → valid children mapping
    pub parent_children: HashMap<String, Vec<String>>,
    /// Keywords (directories, properties, etc.)
    pub keywords: Keywords,
    /// Code snippets
    pub snippets: Vec<Snippet>,
}

impl WixData {
    /// Load wix-data from directory
    pub fn load(wix_data_path: &Path) -> Result<Self, LoadError> {
        if !wix_data_path.exists() {
            return Err(LoadError::NotFound(wix_data_path.to_path_buf()));
        }

        let mut data = WixData::default();

        // Load elements
        let elements_dir = wix_data_path.join("elements");
        if elements_dir.exists() {
            data.load_elements(&elements_dir)?;
        }

        // Load keywords
        let keywords_path = wix_data_path.join("keywords/keywords.json");
        if keywords_path.exists() {
            data.load_keywords(&keywords_path)?;
        }

        // Load snippets
        let snippets_path = wix_data_path.join("snippets/snippets.json");
        if snippets_path.exists() {
            data.load_snippets(&snippets_path)?;
        }

        // Build parent→children map
        data.build_parent_children_map();

        Ok(data)
    }

    /// Load all element definitions
    fn load_elements(&mut self, dir: &Path) -> Result<(), LoadError> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().is_some_and(|ext| ext == "json") {
                let content = fs::read_to_string(&path)?;
                let element: ElementDef =
                    serde_json::from_str(&content).map_err(|e| LoadError::Parse {
                        file: path.clone(),
                        source: e,
                    })?;
                self.elements.insert(element.name.clone(), element);
            }
        }
        Ok(())
    }

    /// Load keywords
    fn load_keywords(&mut self, path: &Path) -> Result<(), LoadError> {
        let content = fs::read_to_string(path)?;
        self.keywords = serde_json::from_str(&content).map_err(|e| LoadError::Parse {
            file: path.to_path_buf(),
            source: e,
        })?;
        Ok(())
    }

    /// Load snippets
    fn load_snippets(&mut self, path: &Path) -> Result<(), LoadError> {
        let content = fs::read_to_string(path)?;

        #[derive(serde::Deserialize)]
        struct SnippetsFile {
            snippets: Vec<Snippet>,
        }

        let file: SnippetsFile =
            serde_json::from_str(&content).map_err(|e| LoadError::Parse {
                file: path.to_path_buf(),
                source: e,
            })?;
        self.snippets = file.snippets;
        Ok(())
    }

    /// Build parent→children mapping from element definitions
    fn build_parent_children_map(&mut self) {
        for element in self.elements.values() {
            for parent in &element.parents {
                self.parent_children
                    .entry(parent.clone())
                    .or_default()
                    .push(element.name.clone());
            }
        }

        // Sort children for consistent ordering
        for children in self.parent_children.values_mut() {
            children.sort();
            children.dedup();
        }
    }

    /// Get valid children for a parent element
    pub fn get_children(&self, parent: &str) -> Vec<&ElementDef> {
        self.parent_children
            .get(parent)
            .map(|names| {
                names
                    .iter()
                    .filter_map(|n| self.elements.get(n))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get element definition by name
    pub fn get_element(&self, name: &str) -> Option<&ElementDef> {
        self.elements.get(name)
    }

    /// Get attribute definition for an element
    pub fn get_attribute(&self, element: &str, attribute: &str) -> Option<&AttributeDef> {
        self.elements
            .get(element)
            .and_then(|e| e.attributes.get(attribute))
    }

    /// Get all snippets matching a prefix
    pub fn get_snippets_by_prefix(&self, prefix: &str) -> Vec<&Snippet> {
        self.snippets
            .iter()
            .filter(|s| s.prefix.starts_with(prefix) || prefix.starts_with(&s.prefix))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_wix_data() -> TempDir {
        let temp = TempDir::new().unwrap();

        // Create elements directory
        let elements_dir = temp.path().join("elements");
        fs::create_dir(&elements_dir).unwrap();

        // Create Package element
        let package = r#"{
            "name": "Package",
            "description": "Root package element",
            "parents": ["Wix"],
            "children": ["Component", "Directory", "Feature"],
            "attributes": {
                "Name": {"type": "string", "required": true, "description": "Package name"},
                "Version": {"type": "version", "required": true, "description": "Package version"}
            }
        }"#;
        fs::write(elements_dir.join("package.json"), package).unwrap();

        // Create Component element
        let component = r#"{
            "name": "Component",
            "description": "Component element",
            "parents": ["Package", "Directory"],
            "children": ["File", "RegistryKey"],
            "attributes": {
                "Guid": {"type": "guid", "required": true, "description": "Component GUID"},
                "Id": {"type": "identifier", "required": false, "description": "Component ID"}
            }
        }"#;
        fs::write(elements_dir.join("component.json"), component).unwrap();

        // Create keywords
        let keywords_dir = temp.path().join("keywords");
        fs::create_dir(&keywords_dir).unwrap();
        let keywords = r#"{
            "elements": ["Package", "Component"],
            "standardDirectories": ["ProgramFilesFolder", "SystemFolder"],
            "builtinProperties": ["ProductName", "ProductVersion"],
            "preprocessorDirectives": ["if", "endif"]
        }"#;
        fs::write(keywords_dir.join("keywords.json"), keywords).unwrap();

        // Create snippets
        let snippets_dir = temp.path().join("snippets");
        fs::create_dir(&snippets_dir).unwrap();
        let snippets = r#"{
            "snippets": [
                {
                    "name": "Component with File",
                    "prefix": "comp",
                    "description": "Creates a component with a file",
                    "body": ["<Component Guid=\"*\">", "  <File Source=\"${1:file}\" />", "</Component>"]
                }
            ]
        }"#;
        fs::write(snippets_dir.join("snippets.json"), snippets).unwrap();

        temp
    }

    #[test]
    fn test_load_wix_data() {
        let temp = create_test_wix_data();
        let data = WixData::load(temp.path()).unwrap();

        assert_eq!(data.elements.len(), 2);
        assert!(data.elements.contains_key("Package"));
        assert!(data.elements.contains_key("Component"));
    }

    #[test]
    fn test_parent_children_map() {
        let temp = create_test_wix_data();
        let data = WixData::load(temp.path()).unwrap();

        let package_children = data.parent_children.get("Package").unwrap();
        assert!(package_children.contains(&"Component".to_string()));
    }

    #[test]
    fn test_get_children() {
        let temp = create_test_wix_data();
        let data = WixData::load(temp.path()).unwrap();

        let children = data.get_children("Package");
        assert!(!children.is_empty());
        assert!(children.iter().any(|c| c.name == "Component"));
    }

    #[test]
    fn test_get_attribute() {
        let temp = create_test_wix_data();
        let data = WixData::load(temp.path()).unwrap();

        let attr = data.get_attribute("Component", "Guid").unwrap();
        assert_eq!(attr.attr_type, "guid");
        assert!(attr.required);
    }

    #[test]
    fn test_load_keywords() {
        let temp = create_test_wix_data();
        let data = WixData::load(temp.path()).unwrap();

        assert!(data.keywords.standard_directories.contains(&"ProgramFilesFolder".to_string()));
        assert!(data.keywords.builtin_properties.contains(&"ProductName".to_string()));
    }

    #[test]
    fn test_load_snippets() {
        let temp = create_test_wix_data();
        let data = WixData::load(temp.path()).unwrap();

        assert_eq!(data.snippets.len(), 1);
        assert_eq!(data.snippets[0].prefix, "comp");
    }

    #[test]
    fn test_get_snippets_by_prefix() {
        let temp = create_test_wix_data();
        let data = WixData::load(temp.path()).unwrap();

        let snippets = data.get_snippets_by_prefix("co");
        assert_eq!(snippets.len(), 1);
        assert_eq!(snippets[0].name, "Component with File");
    }

    #[test]
    fn test_not_found() {
        let result = WixData::load(Path::new("/nonexistent/path"));
        assert!(matches!(result, Err(LoadError::NotFound(_))));
    }

    #[test]
    fn test_invalid_element_json() {
        let temp = TempDir::new().unwrap();

        let elements_dir = temp.path().join("elements");
        fs::create_dir(&elements_dir).unwrap();
        fs::write(elements_dir.join("bad.json"), "not valid json").unwrap();

        let keywords_dir = temp.path().join("keywords");
        fs::create_dir(&keywords_dir).unwrap();
        fs::write(keywords_dir.join("keywords.json"), "{}").unwrap();

        let snippets_dir = temp.path().join("snippets");
        fs::create_dir(&snippets_dir).unwrap();
        fs::write(snippets_dir.join("snippets.json"), r#"{"snippets":[]}"#).unwrap();

        let result = WixData::load(temp.path());
        assert!(matches!(result, Err(LoadError::Parse { .. })));
    }

    #[test]
    fn test_invalid_keywords_json() {
        let temp = TempDir::new().unwrap();

        let elements_dir = temp.path().join("elements");
        fs::create_dir(&elements_dir).unwrap();

        let keywords_dir = temp.path().join("keywords");
        fs::create_dir(&keywords_dir).unwrap();
        fs::write(keywords_dir.join("keywords.json"), "not valid json").unwrap();

        let snippets_dir = temp.path().join("snippets");
        fs::create_dir(&snippets_dir).unwrap();
        fs::write(snippets_dir.join("snippets.json"), r#"{"snippets":[]}"#).unwrap();

        let result = WixData::load(temp.path());
        assert!(matches!(result, Err(LoadError::Parse { .. })));
    }

    #[test]
    fn test_invalid_snippets_json() {
        let temp = TempDir::new().unwrap();

        let elements_dir = temp.path().join("elements");
        fs::create_dir(&elements_dir).unwrap();

        let keywords_dir = temp.path().join("keywords");
        fs::create_dir(&keywords_dir).unwrap();
        fs::write(keywords_dir.join("keywords.json"), "{}").unwrap();

        let snippets_dir = temp.path().join("snippets");
        fs::create_dir(&snippets_dir).unwrap();
        fs::write(snippets_dir.join("snippets.json"), "not valid json").unwrap();

        let result = WixData::load(temp.path());
        assert!(matches!(result, Err(LoadError::Parse { .. })));
    }

    #[test]
    fn test_get_element() {
        let temp = create_test_wix_data();
        let data = WixData::load(temp.path()).unwrap();

        let elem = data.get_element("Package");
        assert!(elem.is_some());
        assert_eq!(elem.unwrap().name, "Package");

        let none = data.get_element("NonExistent");
        assert!(none.is_none());
    }

    #[test]
    fn test_get_attribute_unknown_element() {
        let temp = create_test_wix_data();
        let data = WixData::load(temp.path()).unwrap();

        let attr = data.get_attribute("UnknownElement", "Attr");
        assert!(attr.is_none());
    }

    #[test]
    fn test_get_attribute_unknown_attribute() {
        let temp = create_test_wix_data();
        let data = WixData::load(temp.path()).unwrap();

        let attr = data.get_attribute("Component", "UnknownAttr");
        assert!(attr.is_none());
    }

    #[test]
    fn test_get_children_unknown_parent() {
        let temp = create_test_wix_data();
        let data = WixData::load(temp.path()).unwrap();

        let children = data.get_children("UnknownParent");
        assert!(children.is_empty());
    }

    #[test]
    fn test_get_snippets_no_match() {
        let temp = create_test_wix_data();
        let data = WixData::load(temp.path()).unwrap();

        let snippets = data.get_snippets_by_prefix("xyz");
        assert!(snippets.is_empty());
    }

    #[test]
    fn test_get_snippets_empty_prefix() {
        let temp = create_test_wix_data();
        let data = WixData::load(temp.path()).unwrap();

        let snippets = data.get_snippets_by_prefix("");
        // Empty prefix should match all
        assert!(!snippets.is_empty());
    }

    #[test]
    fn test_empty_elements_dir() {
        let temp = TempDir::new().unwrap();

        let elements_dir = temp.path().join("elements");
        fs::create_dir(&elements_dir).unwrap();
        // No element files

        let keywords_dir = temp.path().join("keywords");
        fs::create_dir(&keywords_dir).unwrap();
        fs::write(keywords_dir.join("keywords.json"), "{}").unwrap();

        let snippets_dir = temp.path().join("snippets");
        fs::create_dir(&snippets_dir).unwrap();
        fs::write(snippets_dir.join("snippets.json"), r#"{"snippets":[]}"#).unwrap();

        let result = WixData::load(temp.path());
        assert!(result.is_ok());
        assert!(result.unwrap().elements.is_empty());
    }

    #[test]
    fn test_load_error_display() {
        let err = LoadError::NotFound(PathBuf::from("/test/path"));
        assert!(err.to_string().contains("/test/path"));
    }
}

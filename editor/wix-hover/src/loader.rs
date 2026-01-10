//! Load wix-data for hover information.
//!
//! This module handles loading element definitions and keywords from
//! a wix-data directory structure.

use crate::types::{AttributeDef, ElementDef, Keywords};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Errors that can occur when loading wix-data.
#[derive(Error, Debug)]
pub enum LoadError {
    /// Failed to read a directory or file.
    #[error("Failed to read: {0}")]
    ReadDir(#[from] std::io::Error),

    /// Failed to parse a JSON file.
    #[error("Failed to parse {file}: {source}")]
    Parse {
        file: PathBuf,
        source: serde_json::Error,
    },

    /// The wix-data directory was not found.
    #[error("wix-data directory not found: {0}")]
    NotFound(PathBuf),
}

/// Loaded wix-data for hover.
///
/// Contains element definitions and keywords loaded from the wix-data
/// directory structure.
#[derive(Debug, Default)]
pub struct WixData {
    /// Element definitions indexed by name.
    pub elements: HashMap<String, ElementDef>,

    /// Keywords (directories, properties, etc.)
    pub keywords: Keywords,
}

impl WixData {
    /// Load wix-data from a directory.
    ///
    /// # Arguments
    /// * `wix_data_path` - Path to the wix-data directory
    ///
    /// # Returns
    /// Loaded `WixData` or an error.
    ///
    /// # Directory Structure
    /// ```text
    /// wix-data/
    /// ├── elements/
    /// │   ├── component.json
    /// │   ├── file.json
    /// │   └── ...
    /// └── keywords/
    ///     └── keywords.json
    /// ```
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

        Ok(data)
    }

    /// Load all element definitions from a directory.
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

    /// Load keywords from a file.
    fn load_keywords(&mut self, path: &Path) -> Result<(), LoadError> {
        let content = fs::read_to_string(path)?;
        self.keywords = serde_json::from_str(&content).map_err(|e| LoadError::Parse {
            file: path.to_path_buf(),
            source: e,
        })?;
        Ok(())
    }

    /// Get element definition by name.
    pub fn get_element(&self, name: &str) -> Option<&ElementDef> {
        self.elements.get(name)
    }

    /// Get attribute definition for an element.
    pub fn get_attribute(&self, element: &str, attribute: &str) -> Option<&AttributeDef> {
        self.elements
            .get(element)
            .and_then(|e| e.attributes.get(attribute))
    }

    /// Check if a value is a standard directory.
    pub fn is_standard_directory(&self, value: &str) -> bool {
        self.keywords
            .standard_directories
            .contains(&value.to_string())
    }

    /// Check if a value is a builtin property.
    pub fn is_builtin_property(&self, value: &str) -> bool {
        self.keywords
            .builtin_properties
            .contains(&value.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_wix_data() -> TempDir {
        let temp = TempDir::new().unwrap();

        let elements_dir = temp.path().join("elements");
        fs::create_dir(&elements_dir).unwrap();

        let component = r#"{
            "name": "Component",
            "description": "A component is a grouping of resources.",
            "documentation": "https://wixtoolset.org/docs/schema/wxs/component/",
            "since": "v3",
            "parents": ["Directory", "DirectoryRef"],
            "children": ["File", "RegistryKey"],
            "attributes": {
                "Id": {"type": "identifier", "required": false, "description": "Component identifier"},
                "Guid": {"type": "guid", "required": true, "description": "Component GUID"}
            }
        }"#;
        fs::write(elements_dir.join("component.json"), component).unwrap();

        let keywords_dir = temp.path().join("keywords");
        fs::create_dir(&keywords_dir).unwrap();
        let keywords = r#"{
            "elements": ["Component", "File"],
            "standardDirectories": ["ProgramFilesFolder", "SystemFolder"],
            "builtinProperties": ["ProductName", "ProductVersion"],
            "preprocessorDirectives": ["if", "endif"]
        }"#;
        fs::write(keywords_dir.join("keywords.json"), keywords).unwrap();

        temp
    }

    #[test]
    fn test_load_wix_data() {
        let temp = create_test_wix_data();
        let data = WixData::load(temp.path()).unwrap();

        assert!(data.elements.contains_key("Component"));
    }

    #[test]
    fn test_get_element() {
        let temp = create_test_wix_data();
        let data = WixData::load(temp.path()).unwrap();

        let elem = data.get_element("Component").unwrap();
        assert_eq!(elem.name, "Component");
        assert_eq!(elem.since, Some("v3".to_string()));
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
    fn test_is_standard_directory() {
        let temp = create_test_wix_data();
        let data = WixData::load(temp.path()).unwrap();

        assert!(data.is_standard_directory("ProgramFilesFolder"));
        assert!(!data.is_standard_directory("CustomFolder"));
    }

    #[test]
    fn test_is_builtin_property() {
        let temp = create_test_wix_data();
        let data = WixData::load(temp.path()).unwrap();

        assert!(data.is_builtin_property("ProductName"));
        assert!(!data.is_builtin_property("CustomProperty"));
    }

    #[test]
    fn test_not_found() {
        let result = WixData::load(Path::new("/nonexistent/path"));
        assert!(matches!(result, Err(LoadError::NotFound(_))));
    }

    #[test]
    fn test_invalid_json() {
        let temp = TempDir::new().unwrap();
        let elements_dir = temp.path().join("elements");
        fs::create_dir(&elements_dir).unwrap();
        fs::write(elements_dir.join("bad.json"), "not valid json").unwrap();

        let result = WixData::load(temp.path());
        assert!(matches!(result, Err(LoadError::Parse { .. })));
    }

    #[test]
    fn test_empty_wix_data() {
        let temp = TempDir::new().unwrap();
        let elements_dir = temp.path().join("elements");
        fs::create_dir(&elements_dir).unwrap();

        let data = WixData::load(temp.path()).unwrap();
        assert!(data.elements.is_empty());
    }

    #[test]
    fn test_load_error_display() {
        let err = LoadError::NotFound(PathBuf::from("/test/path"));
        assert!(err.to_string().contains("/test/path"));
    }
}

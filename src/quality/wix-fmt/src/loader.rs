//! Load wix-data for element ordering

use serde::Deserialize;
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

/// Element definition from wix-data
#[derive(Debug, Clone, Deserialize)]
pub struct ElementDef {
    pub name: String,
    #[serde(default)]
    pub children: Vec<String>,
    #[serde(default)]
    pub attributes: HashMap<String, AttributeDef>,
}

/// Attribute definition
#[derive(Debug, Clone, Deserialize)]
pub struct AttributeDef {
    #[serde(rename = "type", default)]
    pub attr_type: String,
    #[serde(default)]
    pub required: bool,
}

/// Loaded wix-data for formatting
#[derive(Debug, Default)]
pub struct WixData {
    /// Element definitions indexed by name
    pub elements: HashMap<String, ElementDef>,
    /// Canonical child order for each parent
    pub child_order: HashMap<String, Vec<String>>,
    /// Attribute priority for each element (required attrs first)
    pub attr_priority: HashMap<String, Vec<String>>,
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

        // Build ordering maps
        data.build_child_order();
        data.build_attr_priority();

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

    /// Build canonical child ordering from element definitions
    fn build_child_order(&mut self) {
        for element in self.elements.values() {
            if !element.children.is_empty() {
                self.child_order
                    .insert(element.name.clone(), element.children.clone());
            }
        }
    }

    /// Build attribute priority ordering (required first, then alphabetical)
    fn build_attr_priority(&mut self) {
        for element in self.elements.values() {
            if element.attributes.is_empty() {
                continue;
            }

            let mut required: Vec<_> = element
                .attributes
                .iter()
                .filter(|(_, def)| def.required)
                .map(|(name, _)| name.clone())
                .collect();

            let mut optional: Vec<_> = element
                .attributes
                .iter()
                .filter(|(_, def)| !def.required)
                .map(|(name, _)| name.clone())
                .collect();

            // Sort each group alphabetically, but Id comes first
            required.sort_by(|a, b| {
                if a == "Id" {
                    std::cmp::Ordering::Less
                } else if b == "Id" {
                    std::cmp::Ordering::Greater
                } else {
                    a.cmp(b)
                }
            });

            optional.sort_by(|a, b| {
                if a == "Id" {
                    std::cmp::Ordering::Less
                } else if b == "Id" {
                    std::cmp::Ordering::Greater
                } else {
                    a.cmp(b)
                }
            });

            let mut priority = required;
            priority.extend(optional);

            self.attr_priority.insert(element.name.clone(), priority);
        }
    }

    /// Get canonical order of children for a parent element
    pub fn get_child_order(&self, parent: &str) -> Option<&[String]> {
        self.child_order.get(parent).map(|v| v.as_slice())
    }

    /// Get attribute priority for an element
    pub fn get_attr_priority(&self, element: &str) -> Option<&[String]> {
        self.attr_priority.get(element).map(|v| v.as_slice())
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

        let package = r#"{
            "name": "Package",
            "children": ["Directory", "Component", "Feature"],
            "attributes": {
                "Name": {"type": "string", "required": true},
                "Version": {"type": "version", "required": true},
                "Id": {"type": "identifier", "required": false}
            }
        }"#;
        fs::write(elements_dir.join("package.json"), package).unwrap();

        let component = r#"{
            "name": "Component",
            "children": ["File", "RegistryKey"],
            "attributes": {
                "Guid": {"type": "guid", "required": true},
                "Id": {"type": "identifier", "required": false}
            }
        }"#;
        fs::write(elements_dir.join("component.json"), component).unwrap();

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
    fn test_child_order() {
        let temp = create_test_wix_data();
        let data = WixData::load(temp.path()).unwrap();

        let order = data.get_child_order("Package").unwrap();
        assert_eq!(order, &["Directory", "Component", "Feature"]);
    }

    #[test]
    fn test_attr_priority() {
        let temp = create_test_wix_data();
        let data = WixData::load(temp.path()).unwrap();

        let priority = data.get_attr_priority("Package").unwrap();
        // Required first: Name, Version; then optional: Id
        // But Id comes first within its group
        assert!(priority.contains(&"Name".to_string()));
        assert!(priority.contains(&"Version".to_string()));
        assert!(priority.contains(&"Id".to_string()));
    }

    #[test]
    fn test_attr_priority_id_first() {
        let temp = create_test_wix_data();
        let data = WixData::load(temp.path()).unwrap();

        let priority = data.get_attr_priority("Component").unwrap();
        // Guid is required, Id is optional
        // Required come first, so Guid before Id
        assert_eq!(priority[0], "Guid");
        assert_eq!(priority[1], "Id");
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
    fn test_empty_elements_dir() {
        let temp = TempDir::new().unwrap();
        let elements_dir = temp.path().join("elements");
        fs::create_dir(&elements_dir).unwrap();

        let data = WixData::load(temp.path()).unwrap();
        assert!(data.elements.is_empty());
    }

    #[test]
    fn test_get_child_order_unknown() {
        let temp = create_test_wix_data();
        let data = WixData::load(temp.path()).unwrap();

        assert!(data.get_child_order("Unknown").is_none());
    }

    #[test]
    fn test_get_attr_priority_unknown() {
        let temp = create_test_wix_data();
        let data = WixData::load(temp.path()).unwrap();

        assert!(data.get_attr_priority("Unknown").is_none());
    }

    #[test]
    fn test_load_error_display() {
        let err = LoadError::NotFound(PathBuf::from("/test/path"));
        assert!(err.to_string().contains("/test/path"));
    }
}

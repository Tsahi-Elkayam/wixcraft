//! Schema data loader from wixkb database

use crate::types::{AttributeDef, ElementDef, Keywords, Snippet};
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::Path;

/// Schema data loaded from wixkb
#[derive(Debug, Clone, Default)]
pub struct SchemaData {
    pub elements: HashMap<String, ElementDef>,
    pub snippets: Vec<Snippet>,
    pub keywords: Keywords,
}

impl SchemaData {
    /// Load from wixkb directory (contains database/wixkb.db and config/templates/)
    pub fn load(wixkb_path: &Path) -> Result<Self> {
        let mut data = SchemaData::default();

        // Try SQLite database first
        let db_path = wixkb_path.join("database/wixkb.db");
        if db_path.exists() {
            data.load_from_database(&db_path)?;
        }

        // Load JSON templates
        let templates_path = wixkb_path.join("config/templates");
        if templates_path.exists() {
            data.load_snippets(&templates_path.join("snippets.json"))?;
            data.load_keywords(&templates_path)?;
        }

        Ok(data)
    }

    fn load_from_database(&mut self, db_path: &Path) -> Result<()> {
        use rusqlite::Connection;

        let conn = Connection::open(db_path)
            .with_context(|| format!("Failed to open wixkb database: {}", db_path.display()))?;

        // Load elements
        let mut stmt = conn.prepare(
            "SELECT id, name, description, documentation_url FROM elements",
        )?;

        let element_rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, Option<String>>(2)?,
                row.get::<_, Option<String>>(3)?,
            ))
        })?;

        let mut element_ids: HashMap<i64, String> = HashMap::new();

        for row in element_rows {
            let (id, name, description, doc_url) = row?;
            element_ids.insert(id, name.clone());
            self.elements.insert(
                name.clone(),
                ElementDef {
                    name,
                    description: description.unwrap_or_default(),
                    documentation: doc_url,
                    children: Vec::new(),
                    parents: Vec::new(),
                    attributes: HashMap::new(),
                },
            );
        }

        // Load parent-child relationships
        let mut stmt = conn.prepare(
            "SELECT element_id, child_id FROM element_children",
        )?;

        let child_rows = stmt.query_map([], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?))
        })?;

        for row in child_rows {
            let (element_id, child_id) = row?;
            if let (Some(element_name), Some(child_name)) =
                (element_ids.get(&element_id), element_ids.get(&child_id))
            {
                if let Some(element) = self.elements.get_mut(element_name) {
                    element.children.push(child_name.clone());
                }
                if let Some(child) = self.elements.get_mut(child_name) {
                    child.parents.push(element_name.clone());
                }
            }
        }

        // Load attributes
        let mut stmt = conn.prepare(
            "SELECT element_id, name, attr_type, required, default_value, description
             FROM attributes",
        )?;

        let attr_rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, bool>(3)?,
                row.get::<_, Option<String>>(4)?,
                row.get::<_, Option<String>>(5)?,
            ))
        })?;

        for row in attr_rows {
            let (element_id, name, attr_type, required, default_value, description) = row?;
            if let Some(element_name) = element_ids.get(&element_id) {
                if let Some(element) = self.elements.get_mut(element_name) {
                    element.attributes.insert(
                        name.clone(),
                        AttributeDef {
                            attr_type,
                            required,
                            description: description.unwrap_or_default(),
                            default: default_value,
                            values: None,
                        },
                    );
                }
            }
        }

        Ok(())
    }

    fn load_snippets(&mut self, path: &Path) -> Result<()> {
        if !path.exists() {
            return Ok(());
        }

        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read snippets: {}", path.display()))?;

        #[derive(serde::Deserialize)]
        struct SnippetsFile {
            snippets: Vec<Snippet>,
        }

        let file: SnippetsFile = serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse snippets: {}", path.display()))?;

        self.snippets = file.snippets;
        Ok(())
    }

    fn load_keywords(&mut self, templates_path: &Path) -> Result<()> {
        // Load standard directories
        let std_dirs_path = templates_path.join("standard-directories.json");
        if std_dirs_path.exists() {
            let content = std::fs::read_to_string(&std_dirs_path)?;

            #[derive(serde::Deserialize)]
            struct StdDirsFile {
                #[serde(rename = "standardDirectories")]
                dirs: Vec<StdDir>,
            }

            #[derive(serde::Deserialize)]
            struct StdDir {
                id: String,
            }

            if let Ok(file) = serde_json::from_str::<StdDirsFile>(&content) {
                self.keywords.standard_directories = file.dirs.into_iter().map(|d| d.id).collect();
            }
        }

        // Load MSI properties
        let props_path = templates_path.join("msi-properties.json");
        if props_path.exists() {
            let content = std::fs::read_to_string(&props_path)?;

            #[derive(serde::Deserialize)]
            struct PropsFile {
                properties: Vec<PropDef>,
            }

            #[derive(serde::Deserialize)]
            struct PropDef {
                name: String,
            }

            if let Ok(file) = serde_json::from_str::<PropsFile>(&content) {
                self.keywords.builtin_properties = file.properties.into_iter().map(|p| p.name).collect();
            }
        }

        Ok(())
    }

    /// Get element definition by name
    pub fn get_element(&self, name: &str) -> Option<&ElementDef> {
        self.elements.get(name)
    }

    /// Get valid children for an element
    pub fn get_children(&self, parent: &str) -> Vec<&ElementDef> {
        self.elements
            .get(parent)
            .map(|e| {
                e.children
                    .iter()
                    .filter_map(|name| self.elements.get(name))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get snippets matching prefix
    pub fn get_snippets_by_prefix(&self, prefix: &str) -> Vec<&Snippet> {
        let prefix_lower = prefix.to_lowercase();
        self.snippets
            .iter()
            .filter(|s| {
                s.prefix.to_lowercase().starts_with(&prefix_lower)
                    || s.name.to_lowercase().starts_with(&prefix_lower)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_default() {
        let schema = SchemaData::default();
        assert!(schema.elements.is_empty());
        assert!(schema.snippets.is_empty());
    }

    #[test]
    fn test_get_element() {
        let mut schema = SchemaData::default();
        schema.elements.insert(
            "Package".to_string(),
            ElementDef {
                name: "Package".to_string(),
                description: "Package element".to_string(),
                ..Default::default()
            },
        );

        assert!(schema.get_element("Package").is_some());
        assert!(schema.get_element("Unknown").is_none());
    }

    #[test]
    fn test_get_children() {
        let mut schema = SchemaData::default();

        schema.elements.insert(
            "Package".to_string(),
            ElementDef {
                name: "Package".to_string(),
                children: vec!["Component".to_string()],
                ..Default::default()
            },
        );

        schema.elements.insert(
            "Component".to_string(),
            ElementDef {
                name: "Component".to_string(),
                ..Default::default()
            },
        );

        let children = schema.get_children("Package");
        assert_eq!(children.len(), 1);
        assert_eq!(children[0].name, "Component");
    }

    #[test]
    fn test_get_snippets_by_prefix() {
        let mut schema = SchemaData::default();
        schema.snippets = vec![
            Snippet {
                name: "component".to_string(),
                prefix: "comp".to_string(),
                description: "Component".to_string(),
                body: vec![],
            },
            Snippet {
                name: "directory".to_string(),
                prefix: "dir".to_string(),
                description: "Directory".to_string(),
                body: vec![],
            },
        ];

        let matches = schema.get_snippets_by_prefix("co");
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].prefix, "comp");

        let all = schema.get_snippets_by_prefix("");
        assert_eq!(all.len(), 2);
    }
}

//! wix-msi - Cross-platform MSI compiler
//!
//! Build MSI packages on any operating system without Windows.
//!
//! This library provides:
//! - MSI database structure generation
//! - Table definitions and population
//! - WiX source parsing
//! - GUID generation

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// MSI database representation
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MsiDatabase {
    pub summary_info: SummaryInfo,
    pub tables: HashMap<String, MsiTable>,
}

/// MSI Summary Information stream
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SummaryInfo {
    pub title: Option<String>,
    pub subject: Option<String>,
    pub author: Option<String>,
    pub keywords: Option<String>,
    pub comments: Option<String>,
    pub template: Option<String>,
    pub revision_number: Option<String>,
    pub creating_app: String,
    pub security: u32,
    pub page_count: u32,
    pub word_count: u32,
}

/// MSI table
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MsiTable {
    pub name: String,
    pub columns: Vec<MsiColumn>,
    pub rows: Vec<Vec<MsiValue>>,
}

/// MSI column definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MsiColumn {
    pub name: String,
    pub column_type: ColumnType,
    pub nullable: bool,
    pub primary_key: bool,
    pub size: Option<u32>,
}

/// MSI column types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ColumnType {
    Integer,
    String,
    Binary,
    Stream,
}

/// MSI cell value
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MsiValue {
    Null,
    Integer(i32),
    String(String),
    Binary(Vec<u8>),
}

impl MsiDatabase {
    pub fn new() -> Self {
        let mut db = Self::default();
        db.summary_info.creating_app = "wix-msi 0.1.0".to_string();
        db.summary_info.security = 2; // Read-only
        db
    }

    /// Initialize standard MSI tables
    pub fn init_standard_tables(&mut self) {
        // Property table
        self.tables.insert(
            "Property".to_string(),
            MsiTable {
                name: "Property".to_string(),
                columns: vec![
                    MsiColumn {
                        name: "Property".to_string(),
                        column_type: ColumnType::String,
                        nullable: false,
                        primary_key: true,
                        size: Some(72),
                    },
                    MsiColumn {
                        name: "Value".to_string(),
                        column_type: ColumnType::String,
                        nullable: false,
                        primary_key: false,
                        size: Some(0), // Unlimited
                    },
                ],
                rows: Vec::new(),
            },
        );

        // Directory table
        self.tables.insert(
            "Directory".to_string(),
            MsiTable {
                name: "Directory".to_string(),
                columns: vec![
                    MsiColumn {
                        name: "Directory".to_string(),
                        column_type: ColumnType::String,
                        nullable: false,
                        primary_key: true,
                        size: Some(72),
                    },
                    MsiColumn {
                        name: "Directory_Parent".to_string(),
                        column_type: ColumnType::String,
                        nullable: true,
                        primary_key: false,
                        size: Some(72),
                    },
                    MsiColumn {
                        name: "DefaultDir".to_string(),
                        column_type: ColumnType::String,
                        nullable: false,
                        primary_key: false,
                        size: Some(255),
                    },
                ],
                rows: Vec::new(),
            },
        );

        // Component table
        self.tables.insert(
            "Component".to_string(),
            MsiTable {
                name: "Component".to_string(),
                columns: vec![
                    MsiColumn {
                        name: "Component".to_string(),
                        column_type: ColumnType::String,
                        nullable: false,
                        primary_key: true,
                        size: Some(72),
                    },
                    MsiColumn {
                        name: "ComponentId".to_string(),
                        column_type: ColumnType::String,
                        nullable: true,
                        primary_key: false,
                        size: Some(38),
                    },
                    MsiColumn {
                        name: "Directory_".to_string(),
                        column_type: ColumnType::String,
                        nullable: false,
                        primary_key: false,
                        size: Some(72),
                    },
                    MsiColumn {
                        name: "Attributes".to_string(),
                        column_type: ColumnType::Integer,
                        nullable: false,
                        primary_key: false,
                        size: None,
                    },
                    MsiColumn {
                        name: "Condition".to_string(),
                        column_type: ColumnType::String,
                        nullable: true,
                        primary_key: false,
                        size: Some(255),
                    },
                    MsiColumn {
                        name: "KeyPath".to_string(),
                        column_type: ColumnType::String,
                        nullable: true,
                        primary_key: false,
                        size: Some(72),
                    },
                ],
                rows: Vec::new(),
            },
        );

        // Feature table
        self.tables.insert(
            "Feature".to_string(),
            MsiTable {
                name: "Feature".to_string(),
                columns: vec![
                    MsiColumn {
                        name: "Feature".to_string(),
                        column_type: ColumnType::String,
                        nullable: false,
                        primary_key: true,
                        size: Some(38),
                    },
                    MsiColumn {
                        name: "Feature_Parent".to_string(),
                        column_type: ColumnType::String,
                        nullable: true,
                        primary_key: false,
                        size: Some(38),
                    },
                    MsiColumn {
                        name: "Title".to_string(),
                        column_type: ColumnType::String,
                        nullable: true,
                        primary_key: false,
                        size: Some(64),
                    },
                    MsiColumn {
                        name: "Description".to_string(),
                        column_type: ColumnType::String,
                        nullable: true,
                        primary_key: false,
                        size: Some(255),
                    },
                    MsiColumn {
                        name: "Display".to_string(),
                        column_type: ColumnType::Integer,
                        nullable: true,
                        primary_key: false,
                        size: None,
                    },
                    MsiColumn {
                        name: "Level".to_string(),
                        column_type: ColumnType::Integer,
                        nullable: false,
                        primary_key: false,
                        size: None,
                    },
                    MsiColumn {
                        name: "Directory_".to_string(),
                        column_type: ColumnType::String,
                        nullable: true,
                        primary_key: false,
                        size: Some(72),
                    },
                    MsiColumn {
                        name: "Attributes".to_string(),
                        column_type: ColumnType::Integer,
                        nullable: false,
                        primary_key: false,
                        size: None,
                    },
                ],
                rows: Vec::new(),
            },
        );

        // File table
        self.tables.insert(
            "File".to_string(),
            MsiTable {
                name: "File".to_string(),
                columns: vec![
                    MsiColumn {
                        name: "File".to_string(),
                        column_type: ColumnType::String,
                        nullable: false,
                        primary_key: true,
                        size: Some(72),
                    },
                    MsiColumn {
                        name: "Component_".to_string(),
                        column_type: ColumnType::String,
                        nullable: false,
                        primary_key: false,
                        size: Some(72),
                    },
                    MsiColumn {
                        name: "FileName".to_string(),
                        column_type: ColumnType::String,
                        nullable: false,
                        primary_key: false,
                        size: Some(255),
                    },
                    MsiColumn {
                        name: "FileSize".to_string(),
                        column_type: ColumnType::Integer,
                        nullable: false,
                        primary_key: false,
                        size: None,
                    },
                    MsiColumn {
                        name: "Version".to_string(),
                        column_type: ColumnType::String,
                        nullable: true,
                        primary_key: false,
                        size: Some(72),
                    },
                    MsiColumn {
                        name: "Language".to_string(),
                        column_type: ColumnType::String,
                        nullable: true,
                        primary_key: false,
                        size: Some(20),
                    },
                    MsiColumn {
                        name: "Attributes".to_string(),
                        column_type: ColumnType::Integer,
                        nullable: true,
                        primary_key: false,
                        size: None,
                    },
                    MsiColumn {
                        name: "Sequence".to_string(),
                        column_type: ColumnType::Integer,
                        nullable: false,
                        primary_key: false,
                        size: None,
                    },
                ],
                rows: Vec::new(),
            },
        );
    }

    /// Add a property to the database
    pub fn add_property(&mut self, name: &str, value: &str) {
        if let Some(table) = self.tables.get_mut("Property") {
            table.rows.push(vec![
                MsiValue::String(name.to_string()),
                MsiValue::String(value.to_string()),
            ]);
        }
    }

    /// Add a directory to the database
    pub fn add_directory(&mut self, id: &str, parent: Option<&str>, name: &str) {
        if let Some(table) = self.tables.get_mut("Directory") {
            table.rows.push(vec![
                MsiValue::String(id.to_string()),
                parent.map(|p| MsiValue::String(p.to_string())).unwrap_or(MsiValue::Null),
                MsiValue::String(name.to_string()),
            ]);
        }
    }

    /// Add a component to the database
    pub fn add_component(&mut self, id: &str, guid: Option<&str>, directory: &str, key_path: Option<&str>) {
        if let Some(table) = self.tables.get_mut("Component") {
            let component_guid = guid
                .map(String::from)
                .unwrap_or_else(|| format!("{{{}}}", Uuid::new_v4().to_string().to_uppercase()));

            table.rows.push(vec![
                MsiValue::String(id.to_string()),
                MsiValue::String(component_guid),
                MsiValue::String(directory.to_string()),
                MsiValue::Integer(0), // Attributes
                MsiValue::Null,       // Condition
                key_path.map(|k| MsiValue::String(k.to_string())).unwrap_or(MsiValue::Null),
            ]);
        }
    }

    /// Add a feature to the database
    pub fn add_feature(&mut self, id: &str, parent: Option<&str>, title: Option<&str>, level: i32) {
        if let Some(table) = self.tables.get_mut("Feature") {
            table.rows.push(vec![
                MsiValue::String(id.to_string()),
                parent.map(|p| MsiValue::String(p.to_string())).unwrap_or(MsiValue::Null),
                title.map(|t| MsiValue::String(t.to_string())).unwrap_or(MsiValue::Null),
                MsiValue::Null, // Description
                MsiValue::Null, // Display
                MsiValue::Integer(level),
                MsiValue::Null,     // Directory_
                MsiValue::Integer(0), // Attributes
            ]);
        }
    }
}

/// WiX source to MSI compiler
pub struct MsiCompiler;

impl MsiCompiler {
    /// Compile WiX source to MSI database
    pub fn compile(content: &str) -> anyhow::Result<MsiDatabase> {
        let mut db = MsiDatabase::new();
        db.init_standard_tables();

        if let Ok(doc) = roxmltree::Document::parse(content) {
            // Extract product info
            for node in doc.descendants() {
                match node.tag_name().name() {
                    "Package" | "Product" => {
                        if let Some(name) = node.attribute("Name") {
                            db.add_property("ProductName", name);
                            db.summary_info.subject = Some(name.to_string());
                        }
                        if let Some(version) = node.attribute("Version") {
                            db.add_property("ProductVersion", version);
                        }
                        if let Some(manufacturer) = node.attribute("Manufacturer") {
                            db.add_property("Manufacturer", manufacturer);
                            db.summary_info.author = Some(manufacturer.to_string());
                        }
                        if let Some(id) = node.attribute("Id") {
                            let product_code = if id == "*" {
                                format!("{{{}}}", Uuid::new_v4().to_string().to_uppercase())
                            } else {
                                id.to_string()
                            };
                            db.add_property("ProductCode", &product_code);
                        }
                        if let Some(upgrade_code) = node.attribute("UpgradeCode") {
                            db.add_property("UpgradeCode", upgrade_code);
                            db.summary_info.revision_number = Some(upgrade_code.to_string());
                        }
                    }
                    "Directory" | "StandardDirectory" => {
                        let id = node.attribute("Id").unwrap_or("");
                        let name = node.attribute("Name").unwrap_or(id);
                        let parent = node.parent()
                            .and_then(|p| p.attribute("Id"))
                            .filter(|_| node.parent().map(|p| p.tag_name().name()) == Some("Directory"));

                        db.add_directory(id, parent, name);
                    }
                    "Component" => {
                        let id = node.attribute("Id").unwrap_or("");
                        let guid = node.attribute("Guid");
                        let directory = find_parent_directory(&node).unwrap_or("INSTALLDIR");

                        db.add_component(id, guid.filter(|g| *g != "*"), directory, None);
                    }
                    "Feature" => {
                        let id = node.attribute("Id").unwrap_or("");
                        let parent = node.parent()
                            .filter(|p| p.tag_name().name() == "Feature")
                            .and_then(|p| p.attribute("Id"));
                        let title = node.attribute("Title");
                        let level: i32 = node.attribute("Level")
                            .and_then(|l| l.parse().ok())
                            .unwrap_or(1);

                        db.add_feature(id, parent, title, level);
                    }
                    _ => {}
                }
            }
        }

        Ok(db)
    }
}

fn find_parent_directory<'a>(node: &roxmltree::Node<'a, 'a>) -> Option<&'a str> {
    let mut current = node.parent();
    while let Some(parent) = current {
        if parent.tag_name().name() == "Directory" || parent.tag_name().name() == "DirectoryRef" {
            return parent.attribute("Id");
        }
        current = parent.parent();
    }
    None
}

/// Generate GUID
pub fn generate_guid() -> String {
    format!("{{{}}}", Uuid::new_v4().to_string().to_uppercase())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_database() {
        let db = MsiDatabase::new();
        assert!(db.summary_info.creating_app.contains("wix-msi"));
    }

    #[test]
    fn test_init_tables() {
        let mut db = MsiDatabase::new();
        db.init_standard_tables();

        assert!(db.tables.contains_key("Property"));
        assert!(db.tables.contains_key("Directory"));
        assert!(db.tables.contains_key("Component"));
        assert!(db.tables.contains_key("Feature"));
        assert!(db.tables.contains_key("File"));
    }

    #[test]
    fn test_add_property() {
        let mut db = MsiDatabase::new();
        db.init_standard_tables();
        db.add_property("ProductName", "TestApp");

        let table = db.tables.get("Property").unwrap();
        assert_eq!(table.rows.len(), 1);
    }

    #[test]
    fn test_compile_basic() {
        let content = r#"
        <Wix xmlns="http://wixtoolset.org/schemas/v4/wxs">
            <Package Name="TestApp" Version="1.0.0" Manufacturer="Test"
                     UpgradeCode="{12345678-1234-1234-1234-123456789012}">
                <Feature Id="MainFeature" Title="Main" Level="1" />
            </Package>
        </Wix>
        "#;

        let db = MsiCompiler::compile(content).unwrap();
        let props = db.tables.get("Property").unwrap();
        assert!(props.rows.iter().any(|r| {
            matches!(&r[0], MsiValue::String(s) if s == "ProductName")
        }));
    }

    #[test]
    fn test_generate_guid() {
        let guid = generate_guid();
        assert!(guid.starts_with('{'));
        assert!(guid.ends_with('}'));
        assert_eq!(guid.len(), 38);
    }
}

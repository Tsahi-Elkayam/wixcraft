//! Database operation tests

use tempfile::tempdir;
use wixkb::db::Database;
use wixkb::models::*;

#[test]
fn test_database_create_and_open() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");

    // Create
    let db = Database::create(&db_path).unwrap();
    assert!(db_path.exists());
    drop(db);

    // Reopen
    let db = Database::open(&db_path).unwrap();
    let stats = db.get_stats().unwrap();
    assert_eq!(stats.schema_version, "1.0.0");
}

#[test]
fn test_database_stats_empty() {
    let db = Database::open_memory().unwrap();
    let stats = db.get_stats().unwrap();

    assert_eq!(stats.elements, 0);
    assert_eq!(stats.attributes, 0);
    assert_eq!(stats.rules, 0);
    assert_eq!(stats.errors, 0);
    assert_eq!(stats.ice_rules, 0);
    assert_eq!(stats.msi_tables, 0);
    assert_eq!(stats.snippets, 0);
    assert_eq!(stats.keywords, 0);
}

#[test]
fn test_insert_element() {
    let db = Database::open_memory().unwrap();

    let elem = Element {
        id: 0,
        name: "Package".to_string(),
        namespace: "wix".to_string(),
        since_version: Some("v4".to_string()),
        deprecated_version: None,
        description: Some("Root package element".to_string()),
        documentation_url: Some("https://wixtoolset.org/docs/".to_string()),
        remarks: Some("This is the root element".to_string()),
        example: Some("<Package Name=\"Test\" />".to_string()),
    };

    let id = db.insert_element(&elem).unwrap();
    assert!(id > 0);

    let stats = db.get_stats().unwrap();
    assert_eq!(stats.elements, 1);
}

#[test]
fn test_get_element_by_name() {
    let db = Database::open_memory().unwrap();

    let elem = Element {
        id: 0,
        name: "Component".to_string(),
        namespace: "wix".to_string(),
        since_version: Some("v4".to_string()),
        deprecated_version: None,
        description: Some("Groups resources".to_string()),
        documentation_url: None,
        remarks: None,
        example: None,
    };
    db.insert_element(&elem).unwrap();

    let retrieved = db.get_element("Component").unwrap().unwrap();
    assert_eq!(retrieved.name, "Component");
    assert_eq!(retrieved.namespace, "wix");
    assert_eq!(retrieved.description, Some("Groups resources".to_string()));
}

#[test]
fn test_get_element_not_found() {
    let db = Database::open_memory().unwrap();
    let result = db.get_element("NonExistent").unwrap();
    assert!(result.is_none());
}

#[test]
fn test_get_element_case_insensitive() {
    let db = Database::open_memory().unwrap();

    let elem = Element {
        id: 0,
        name: "Package".to_string(),
        namespace: "wix".to_string(),
        since_version: None,
        deprecated_version: None,
        description: None,
        documentation_url: None,
        remarks: None,
        example: None,
    };
    db.insert_element(&elem).unwrap();

    assert!(db.get_element("Package").unwrap().is_some());
    assert!(db.get_element("package").unwrap().is_some());
    assert!(db.get_element("PACKAGE").unwrap().is_some());
    assert!(db.get_element("PaCkAgE").unwrap().is_some());
}

#[test]
fn test_search_elements_by_prefix() {
    let db = Database::open_memory().unwrap();

    for name in ["Package", "Property", "PropertyRef", "Component"] {
        let elem = Element {
            id: 0,
            name: name.to_string(),
            namespace: "wix".to_string(),
            since_version: None,
            deprecated_version: None,
            description: None,
            documentation_url: None,
            remarks: None,
            example: None,
        };
        db.insert_element(&elem).unwrap();
    }

    let results = db.search_elements("Pro", 10).unwrap();
    assert_eq!(results.len(), 2);
    assert!(results.iter().any(|e| e.name == "Property"));
    assert!(results.iter().any(|e| e.name == "PropertyRef"));
}

#[test]
fn test_search_elements_with_limit() {
    let db = Database::open_memory().unwrap();

    for i in 0..10 {
        let elem = Element {
            id: 0,
            name: format!("Element{}", i),
            namespace: "wix".to_string(),
            since_version: None,
            deprecated_version: None,
            description: None,
            documentation_url: None,
            remarks: None,
            example: None,
        };
        db.insert_element(&elem).unwrap();
    }

    let results = db.search_elements("Element", 5).unwrap();
    assert_eq!(results.len(), 5);
}

#[test]
fn test_search_elements_fts() {
    let db = Database::open_memory().unwrap();

    let elements = vec![
        ("Package", "Root element for installer"),
        ("Component", "Groups files together"),
        ("File", "Represents a file"),
    ];

    for (name, desc) in elements {
        let elem = Element {
            id: 0,
            name: name.to_string(),
            namespace: "wix".to_string(),
            since_version: None,
            deprecated_version: None,
            description: Some(desc.to_string()),
            documentation_url: None,
            remarks: None,
            example: None,
        };
        db.insert_element(&elem).unwrap();
    }

    let results = db.search_elements_fts("installer", 10).unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].name, "Package");

    let results = db.search_elements_fts("file", 10).unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].name, "File");
}

#[test]
fn test_insert_and_get_attribute() {
    let db = Database::open_memory().unwrap();

    let elem = Element {
        id: 0,
        name: "Component".to_string(),
        namespace: "wix".to_string(),
        since_version: None,
        deprecated_version: None,
        description: None,
        documentation_url: None,
        remarks: None,
        example: None,
    };
    let elem_id = db.insert_element(&elem).unwrap();

    let attr = Attribute {
        id: 0,
        element_id: elem_id,
        name: "Guid".to_string(),
        attr_type: AttributeType::Guid,
        required: true,
        default_value: None,
        description: Some("Component GUID".to_string()),
        since_version: None,
        deprecated_version: None,
        enum_values: Vec::new(),
    };
    db.insert_attribute(&attr).unwrap();

    let attrs = db.get_attributes("Component").unwrap();
    assert_eq!(attrs.len(), 1);
    assert_eq!(attrs[0].name, "Guid");
    assert_eq!(attrs[0].attr_type, AttributeType::Guid);
    assert!(attrs[0].required);
}

#[test]
fn test_attributes_required_first() {
    let db = Database::open_memory().unwrap();

    let elem = Element {
        id: 0,
        name: "Test".to_string(),
        namespace: "wix".to_string(),
        since_version: None,
        deprecated_version: None,
        description: None,
        documentation_url: None,
        remarks: None,
        example: None,
    };
    let elem_id = db.insert_element(&elem).unwrap();

    // Insert optional first
    let attr1 = Attribute {
        id: 0,
        element_id: elem_id,
        name: "Optional".to_string(),
        attr_type: AttributeType::String,
        required: false,
        default_value: None,
        description: None,
        since_version: None,
        deprecated_version: None,
        enum_values: Vec::new(),
    };
    db.insert_attribute(&attr1).unwrap();

    // Then required
    let attr2 = Attribute {
        id: 0,
        element_id: elem_id,
        name: "Required".to_string(),
        attr_type: AttributeType::String,
        required: true,
        default_value: None,
        description: None,
        since_version: None,
        deprecated_version: None,
        enum_values: Vec::new(),
    };
    db.insert_attribute(&attr2).unwrap();

    let attrs = db.get_attributes("Test").unwrap();
    assert_eq!(attrs.len(), 2);
    // Required should be first
    assert!(attrs[0].required);
    assert_eq!(attrs[0].name, "Required");
}

#[test]
fn test_attribute_enum_values() {
    let db = Database::open_memory().unwrap();

    let elem = Element {
        id: 0,
        name: "Test".to_string(),
        namespace: "wix".to_string(),
        since_version: None,
        deprecated_version: None,
        description: None,
        documentation_url: None,
        remarks: None,
        example: None,
    };
    let elem_id = db.insert_element(&elem).unwrap();

    let attr = Attribute {
        id: 0,
        element_id: elem_id,
        name: "Type".to_string(),
        attr_type: AttributeType::Enum,
        required: false,
        default_value: None,
        description: None,
        since_version: None,
        deprecated_version: None,
        enum_values: vec!["file".to_string(), "directory".to_string(), "registry".to_string()],
    };
    db.insert_attribute(&attr).unwrap();

    let attrs = db.get_attributes("Test").unwrap();
    assert_eq!(attrs[0].attr_type, AttributeType::Enum);
    assert_eq!(attrs[0].enum_values.len(), 3);
    assert!(attrs[0].enum_values.contains(&"file".to_string()));
}

#[test]
fn test_last_updated() {
    let db = Database::open_memory().unwrap();

    let stats_before = db.get_stats().unwrap();
    assert!(stats_before.last_updated.is_none());

    db.set_last_updated().unwrap();

    let stats_after = db.get_stats().unwrap();
    assert!(stats_after.last_updated.is_some());
}

//! Tests for the main WixKb library API

use tempfile::tempdir;
use wixkb::db::Database;
use wixkb::models::*;
use wixkb::WixKb;

fn setup_db_with_data() -> (tempfile::TempDir, std::path::PathBuf) {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");

    let db = Database::create(&db_path).unwrap();

    // Insert elements
    let elements = vec![
        ("Package", "Root package element"),
        ("Component", "Groups resources"),
        ("File", "Represents a file"),
        ("Directory", "Directory structure"),
        ("Feature", "Installation feature"),
    ];

    for (name, desc) in &elements {
        let elem = Element {
            id: 0,
            name: name.to_string(),
            namespace: "wix".to_string(),
            since_version: Some("v4".to_string()),
            deprecated_version: None,
            description: Some(desc.to_string()),
            documentation_url: Some(format!("https://docs/{}", name.to_lowercase())),
            remarks: None,
            example: None,
        };
        let elem_id = db.insert_element(&elem).unwrap();

        // Add attributes for Package
        if *name == "Package" {
            for (attr_name, attr_type, required) in [
                ("Name", AttributeType::String, true),
                ("Version", AttributeType::Version, true),
                ("Manufacturer", AttributeType::String, true),
            ] {
                let attr = Attribute {
                    id: 0,
                    element_id: elem_id,
                    name: attr_name.to_string(),
                    attr_type,
                    required,
                    default_value: None,
                    description: None,
                    since_version: None,
                    deprecated_version: None,
                    enum_values: Vec::new(),
                };
                db.insert_attribute(&attr).unwrap();
            }
        }
    }

    // Insert rules
    let rule = Rule {
        id: 0,
        rule_id: "COMP001".to_string(),
        category: "component".to_string(),
        severity: Severity::Error,
        name: "Component Guid".to_string(),
        description: Some("Component must have Guid".to_string()),
        rationale: None,
        fix_suggestion: None,
        enabled: true,
        auto_fixable: false,
        conditions: Vec::new(),
    };
    db.insert_rule(&rule).unwrap();

    // Insert other data
    db.conn().execute(
        "INSERT INTO ice_rules (code, severity, description) VALUES ('ICE03', 'error', 'Schema validation')",
        [],
    ).unwrap();

    db.conn().execute(
        "INSERT INTO standard_directories (name, description, windows_path) VALUES ('ProgramFilesFolder', 'Program Files', 'C:\\Program Files')",
        [],
    ).unwrap();

    db.conn().execute(
        "INSERT INTO builtin_properties (name, description) VALUES ('ProductName', 'Product name')",
        [],
    ).unwrap();

    db.conn().execute(
        "INSERT INTO snippets (prefix, name, body, scope) VALUES ('wix-pkg', 'Package', '<Package />', 'wxs')",
        [],
    ).unwrap();

    db.conn().execute(
        "INSERT INTO keywords (word, category) VALUES ('Package', 'element')",
        [],
    ).unwrap();

    db.conn().execute(
        "INSERT INTO migrations (from_version, to_version, change_type, old_value, new_value) VALUES ('v3', 'v4', 'renamed', 'Product', 'Package')",
        [],
    ).unwrap();

    db.set_last_updated().unwrap();
    drop(db);

    (dir, db_path)
}

#[test]
fn test_wixkb_open() {
    let (_dir, db_path) = setup_db_with_data();
    let kb = WixKb::open(&db_path).unwrap();
    let stats = kb.get_stats().unwrap();
    assert_eq!(stats.elements, 5);
}

#[test]
fn test_wixkb_create() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("new.db");

    let kb = WixKb::create(&db_path).unwrap();
    let stats = kb.get_stats().unwrap();
    assert_eq!(stats.elements, 0);
    assert!(db_path.exists());
}

#[test]
fn test_wixkb_get_element() {
    let (_dir, db_path) = setup_db_with_data();
    let kb = WixKb::open(&db_path).unwrap();

    let elem = kb.get_element("Package").unwrap().unwrap();
    assert_eq!(elem.name, "Package");
    assert_eq!(elem.description, Some("Root package element".to_string()));
}

#[test]
fn test_wixkb_get_element_not_found() {
    let (_dir, db_path) = setup_db_with_data();
    let kb = WixKb::open(&db_path).unwrap();

    let result = kb.get_element("NonExistent").unwrap();
    assert!(result.is_none());
}

#[test]
fn test_wixkb_search_elements() {
    let (_dir, db_path) = setup_db_with_data();
    let kb = WixKb::open(&db_path).unwrap();

    let results = kb.search_elements("Comp", 10).unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].name, "Component");
}

#[test]
fn test_wixkb_search_elements_fts() {
    let (_dir, db_path) = setup_db_with_data();
    let kb = WixKb::open(&db_path).unwrap();

    let results = kb.search_elements_fts("package", 10).unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].name, "Package");
}

#[test]
fn test_wixkb_get_attributes() {
    let (_dir, db_path) = setup_db_with_data();
    let kb = WixKb::open(&db_path).unwrap();

    let attrs = kb.get_attributes("Package").unwrap();
    assert_eq!(attrs.len(), 3);
    assert!(attrs.iter().all(|a| a.required));
}

#[test]
fn test_wixkb_get_children() {
    let (_dir, db_path) = setup_db_with_data();
    let kb = WixKb::open(&db_path).unwrap();

    // No children set up, should return empty
    let children = kb.get_children("Package").unwrap();
    assert!(children.is_empty());
}

#[test]
fn test_wixkb_get_parents() {
    let (_dir, db_path) = setup_db_with_data();
    let kb = WixKb::open(&db_path).unwrap();

    // No parents set up, should return empty
    let parents = kb.get_parents("Component").unwrap();
    assert!(parents.is_empty());
}

#[test]
fn test_wixkb_get_rule() {
    let (_dir, db_path) = setup_db_with_data();
    let kb = WixKb::open(&db_path).unwrap();

    let rule = kb.get_rule("COMP001").unwrap().unwrap();
    assert_eq!(rule.rule_id, "COMP001");
    assert_eq!(rule.severity, Severity::Error);
}

#[test]
fn test_wixkb_get_rules_by_category() {
    let (_dir, db_path) = setup_db_with_data();
    let kb = WixKb::open(&db_path).unwrap();

    let rules = kb.get_rules_by_category("component").unwrap();
    assert_eq!(rules.len(), 1);
}

#[test]
fn test_wixkb_get_enabled_rules() {
    let (_dir, db_path) = setup_db_with_data();
    let kb = WixKb::open(&db_path).unwrap();

    let rules = kb.get_enabled_rules().unwrap();
    assert_eq!(rules.len(), 1);
}

#[test]
fn test_wixkb_search_rules() {
    let (_dir, db_path) = setup_db_with_data();
    let kb = WixKb::open(&db_path).unwrap();

    let results = kb.search_rules("Guid", 10).unwrap();
    assert_eq!(results.len(), 1);
}

#[test]
fn test_wixkb_get_error() {
    let (_dir, db_path) = setup_db_with_data();
    let kb = WixKb::open(&db_path).unwrap();

    // No errors inserted, should return None
    let result = kb.get_error("WIX0001").unwrap();
    assert!(result.is_none());
}

#[test]
fn test_wixkb_get_ice_rule() {
    let (_dir, db_path) = setup_db_with_data();
    let kb = WixKb::open(&db_path).unwrap();

    let ice = kb.get_ice_rule("ICE03").unwrap().unwrap();
    assert_eq!(ice.code, "ICE03");
}

#[test]
fn test_wixkb_get_all_ice_rules() {
    let (_dir, db_path) = setup_db_with_data();
    let kb = WixKb::open(&db_path).unwrap();

    let rules = kb.get_all_ice_rules().unwrap();
    assert_eq!(rules.len(), 1);
}

#[test]
fn test_wixkb_get_standard_directory() {
    let (_dir, db_path) = setup_db_with_data();
    let kb = WixKb::open(&db_path).unwrap();

    let dir = kb.get_standard_directory("ProgramFilesFolder").unwrap().unwrap();
    assert_eq!(dir.name, "ProgramFilesFolder");
}

#[test]
fn test_wixkb_get_all_standard_directories() {
    let (_dir, db_path) = setup_db_with_data();
    let kb = WixKb::open(&db_path).unwrap();

    let dirs = kb.get_all_standard_directories().unwrap();
    assert_eq!(dirs.len(), 1);
}

#[test]
fn test_wixkb_get_builtin_property() {
    let (_dir, db_path) = setup_db_with_data();
    let kb = WixKb::open(&db_path).unwrap();

    let prop = kb.get_builtin_property("ProductName").unwrap().unwrap();
    assert_eq!(prop.name, "ProductName");
}

#[test]
fn test_wixkb_get_snippets() {
    let (_dir, db_path) = setup_db_with_data();
    let kb = WixKb::open(&db_path).unwrap();

    let snippets = kb.get_snippets("wix").unwrap();
    assert_eq!(snippets.len(), 1);
}

#[test]
fn test_wixkb_get_all_snippets() {
    let (_dir, db_path) = setup_db_with_data();
    let kb = WixKb::open(&db_path).unwrap();

    let snippets = kb.get_all_snippets().unwrap();
    assert_eq!(snippets.len(), 1);
}

#[test]
fn test_wixkb_get_keywords() {
    let (_dir, db_path) = setup_db_with_data();
    let kb = WixKb::open(&db_path).unwrap();

    let keywords = kb.get_keywords("element").unwrap();
    assert_eq!(keywords.len(), 1);
    assert!(keywords.contains(&"Package".to_string()));
}

#[test]
fn test_wixkb_get_migrations() {
    let (_dir, db_path) = setup_db_with_data();
    let kb = WixKb::open(&db_path).unwrap();

    let migrations = kb.get_migrations("v3", "v4").unwrap();
    assert_eq!(migrations.len(), 1);
    assert_eq!(migrations[0].change_type, "renamed");
}

#[test]
fn test_wixkb_get_stats() {
    let (_dir, db_path) = setup_db_with_data();
    let kb = WixKb::open(&db_path).unwrap();

    let stats = kb.get_stats().unwrap();
    assert_eq!(stats.elements, 5);
    assert_eq!(stats.attributes, 3);
    assert_eq!(stats.rules, 1);
    assert_eq!(stats.ice_rules, 1);
    assert_eq!(stats.snippets, 1);
    assert_eq!(stats.keywords, 1);
    assert!(stats.last_updated.is_some());
}

#[test]
fn test_wixkb_cache_preload() {
    let (_dir, db_path) = setup_db_with_data();
    let mut kb = WixKb::open(&db_path).unwrap();

    kb.preload_cache().unwrap();

    // Second access should come from cache
    let elem = kb.get_element("Package").unwrap().unwrap();
    assert_eq!(elem.name, "Package");
}

#[test]
fn test_wixkb_cache_clear() {
    let (_dir, db_path) = setup_db_with_data();
    let mut kb = WixKb::open(&db_path).unwrap();

    kb.preload_cache().unwrap();
    kb.clear_cache();

    // Should still work after cache clear
    let elem = kb.get_element("Package").unwrap().unwrap();
    assert_eq!(elem.name, "Package");
}

#[test]
fn test_wixkb_db_access() {
    let (_dir, db_path) = setup_db_with_data();
    let kb = WixKb::open(&db_path).unwrap();

    // Access underlying db
    let db = kb.db();
    let stats = db.get_stats().unwrap();
    assert_eq!(stats.elements, 5);
}

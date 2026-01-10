//! Harvest module tests

use tempfile::tempdir;
use wixkb::db::Database;
use wixkb::harvest::Harvester;

fn create_test_config(dir: &std::path::Path) -> std::path::PathBuf {
    let config_path = dir.join("sources.yaml");
    let content = r#"
version: "1.0"
sources:
  elements:
    local-elements:
      path: "data/elements.json"
      parser: "json"
      targets:
        - keywords
  rules:
    local-rules:
      path: "data/rules.json"
      parser: "rules"
      targets: []
  migration:
    v3-v4:
      path: "data/migration.json"
      parser: "migration"
      targets: []
parsers:
  xml:
    type: xml
  json:
    type: json
  rules:
    type: rules
  migration:
    type: migration
harvest:
  cache_dir: ".cache"
  timeout_seconds: 30
  retry_count: 3
  user_agent: "wixkb/test"
  rate_limit:
    requests_per_second: 2
    burst: 5
"#;
    std::fs::write(&config_path, content).unwrap();

    // Create data directory
    let data_dir = dir.join("data");
    std::fs::create_dir_all(&data_dir).unwrap();

    config_path
}

#[test]
fn test_harvester_new() {
    let dir = tempdir().unwrap();
    let config_path = create_test_config(dir.path());
    let base_path = dir.path().to_path_buf();

    let harvester = Harvester::new(&config_path, &base_path);
    assert!(harvester.is_ok());
}

#[test]
fn test_harvester_new_missing_config() {
    let dir = tempdir().unwrap();
    let missing_path = dir.path().join("nonexistent.yaml");
    let base_path = dir.path().to_path_buf();

    let result = Harvester::new(&missing_path, &base_path);
    assert!(result.is_err());
}

#[test]
fn test_import_keywords() {
    let dir = tempdir().unwrap();
    let config_path = create_test_config(dir.path());
    let base_path = dir.path().to_path_buf();

    // Create keywords JSON
    let keywords_json = r#"{
        "elements": ["Package", "Component", "File"],
        "preprocessor": ["define", "if", "endif"]
    }"#;
    std::fs::write(dir.path().join("data/elements.json"), keywords_json).unwrap();

    let db = Database::open_memory().unwrap();
    let harvester = Harvester::new(&config_path, &base_path).unwrap();

    let value: serde_json::Value = serde_json::from_str(keywords_json).unwrap();
    let count = harvester.import_keywords(&db, &value).unwrap();

    assert_eq!(count, 6);

    let elements = db.get_keywords("element").unwrap();
    assert_eq!(elements.len(), 3);
    assert!(elements.contains(&"Package".to_string()));

    let preprocessor = db.get_keywords("preprocessor").unwrap();
    assert_eq!(preprocessor.len(), 3);
    assert!(preprocessor.contains(&"define".to_string()));
}

#[test]
fn test_import_standard_directories() {
    let dir = tempdir().unwrap();
    let config_path = create_test_config(dir.path());
    let base_path = dir.path().to_path_buf();

    let json = r#"{
        "standardDirectories": ["ProgramFilesFolder", "SystemFolder", "WindowsFolder"]
    }"#;
    std::fs::write(dir.path().join("data/elements.json"), json).unwrap();

    let db = Database::open_memory().unwrap();
    let harvester = Harvester::new(&config_path, &base_path).unwrap();

    let value: serde_json::Value = serde_json::from_str(json).unwrap();
    let count = harvester.import_standard_directories(&db, &value).unwrap();

    assert_eq!(count, 3);

    let dirs = db.get_all_standard_directories().unwrap();
    assert_eq!(dirs.len(), 3);
}

#[test]
fn test_import_builtin_properties() {
    let dir = tempdir().unwrap();
    let config_path = create_test_config(dir.path());
    let base_path = dir.path().to_path_buf();

    let json = r#"{
        "builtInProperties": ["ProductName", "ProductVersion", "Manufacturer"]
    }"#;

    let db = Database::open_memory().unwrap();
    let harvester = Harvester::new(&config_path, &base_path).unwrap();

    let value: serde_json::Value = serde_json::from_str(json).unwrap();
    let count = harvester.import_builtin_properties(&db, &value).unwrap();

    assert_eq!(count, 3);
}

#[test]
fn test_import_snippets() {
    let dir = tempdir().unwrap();
    let config_path = create_test_config(dir.path());
    let base_path = dir.path().to_path_buf();

    let json = r#"{
        "snippets": [
            {"prefix": "wix-pkg", "name": "Package", "body": "<Package />", "description": "Insert package"},
            {"prefix": "wix-comp", "name": "Component", "body": "<Component />", "description": "Insert component"}
        ]
    }"#;

    let db = Database::open_memory().unwrap();
    let harvester = Harvester::new(&config_path, &base_path).unwrap();

    let value: serde_json::Value = serde_json::from_str(json).unwrap();
    let count = harvester.import_snippets(&db, &value).unwrap();

    assert_eq!(count, 2);

    let snippets = db.get_all_snippets().unwrap();
    assert_eq!(snippets.len(), 2);
}

#[test]
fn test_import_errors() {
    let dir = tempdir().unwrap();
    let config_path = create_test_config(dir.path());
    let base_path = dir.path().to_path_buf();

    let json = r#"{
        "errors": [
            {"code": "WIX0001", "severity": "error", "message": "Error 1", "description": "Desc 1"},
            {"code": "WIX0002", "severity": "warning", "message": "Error 2", "description": "Desc 2"}
        ]
    }"#;

    let db = Database::open_memory().unwrap();
    let harvester = Harvester::new(&config_path, &base_path).unwrap();

    let value: serde_json::Value = serde_json::from_str(json).unwrap();
    let count = harvester.import_errors(&db, &value).unwrap();

    assert_eq!(count, 2);

    let error = db.get_error("WIX0001").unwrap();
    assert!(error.is_some());
}

#[test]
fn test_import_ice_rules() {
    let dir = tempdir().unwrap();
    let config_path = create_test_config(dir.path());
    let base_path = dir.path().to_path_buf();

    let json = r#"{
        "iceErrors": [
            {"code": "ICE01", "severity": "error", "description": "ICE 01 desc", "tables": ["Component", "File"]},
            {"code": "ICE02", "severity": "warning", "description": "ICE 02 desc"}
        ]
    }"#;

    let db = Database::open_memory().unwrap();
    let harvester = Harvester::new(&config_path, &base_path).unwrap();

    let value: serde_json::Value = serde_json::from_str(json).unwrap();
    let count = harvester.import_ice_rules(&db, &value).unwrap();

    assert_eq!(count, 2);

    let ice = db.get_ice_rule("ICE01").unwrap();
    assert!(ice.is_some());
    assert_eq!(ice.unwrap().tables_affected.len(), 2);
}

#[test]
fn test_parse_rules() {
    let dir = tempdir().unwrap();
    let config_path = create_test_config(dir.path());
    let base_path = dir.path().to_path_buf();

    let json = r#"{
        "rules": [
            {"id": "COMP001", "category": "component", "severity": "error", "name": "Component GUID", "description": "Require GUID"},
            {"id": "FILE001", "category": "file", "severity": "warning", "name": "File KeyPath", "description": "Recommend KeyPath"}
        ]
    }"#;

    std::fs::write(dir.path().join("data/rules.json"), json).unwrap();

    let db = Database::open_memory().unwrap();
    let harvester = Harvester::new(&config_path, &base_path).unwrap();

    let count = harvester.parse_rules(&db, json).unwrap();

    assert_eq!(count, 2);

    let rule = db.get_rule("COMP001").unwrap();
    assert!(rule.is_some());
    assert_eq!(rule.unwrap().category, "component");
}

#[test]
fn test_parse_migration() {
    let dir = tempdir().unwrap();
    let config_path = create_test_config(dir.path());
    let base_path = dir.path().to_path_buf();

    let json = r#"{
        "from": "v3",
        "to": "v4",
        "changes": [
            {"type": "renamed", "old": "Product", "new": "Package", "notes": "Root element renamed"},
            {"type": "removed", "element": "TARGETDIR", "migration": "Use StandardDirectory"}
        ]
    }"#;

    std::fs::write(dir.path().join("data/migration.json"), json).unwrap();

    let db = Database::open_memory().unwrap();
    let harvester = Harvester::new(&config_path, &base_path).unwrap();

    let count = harvester.parse_migration(&db, json).unwrap();

    assert_eq!(count, 2);

    let migrations = db.get_migrations("v3", "v4").unwrap();
    assert_eq!(migrations.len(), 2);
}

#[test]
fn test_parse_xsd() {
    let dir = tempdir().unwrap();
    let config_path = create_test_config(dir.path());
    let base_path = dir.path().to_path_buf();

    let xsd = r#"<?xml version="1.0" encoding="utf-8"?>
<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema">
  <xs:element name="Package">
    <xs:annotation>
      <xs:documentation>Root package element</xs:documentation>
    </xs:annotation>
  </xs:element>
  <xs:element name="Component">
    <xs:annotation>
      <xs:documentation>Groups files and registry entries</xs:documentation>
    </xs:annotation>
  </xs:element>
</xs:schema>"#;

    let db = Database::open_memory().unwrap();
    let harvester = Harvester::new(&config_path, &base_path).unwrap();

    let source = wixkb::config::SourceDef {
        url: None,
        path: None,
        parser: "xml".to_string(),
        targets: vec![],
        extension: Some("wix".to_string()),
    };

    let count = harvester.parse_xsd(&db, xsd, &source).unwrap();

    assert_eq!(count, 2);

    let pkg = db.get_element("Package").unwrap();
    assert!(pkg.is_some());
    assert_eq!(pkg.unwrap().description, Some("Root package element".to_string()));
}

#[test]
fn test_import_missing_array() {
    let dir = tempdir().unwrap();
    let config_path = create_test_config(dir.path());
    let base_path = dir.path().to_path_buf();

    let json = r#"{"other": "data"}"#;

    let db = Database::open_memory().unwrap();
    let harvester = Harvester::new(&config_path, &base_path).unwrap();

    let value: serde_json::Value = serde_json::from_str(json).unwrap();

    // Missing standardDirectories should return error
    let result = harvester.import_standard_directories(&db, &value);
    assert!(result.is_err());

    // Missing builtInProperties should return error
    let result = harvester.import_builtin_properties(&db, &value);
    assert!(result.is_err());

    // Missing snippets should return error
    let result = harvester.import_snippets(&db, &value);
    assert!(result.is_err());

    // Missing errors should return error
    let result = harvester.import_errors(&db, &value);
    assert!(result.is_err());

    // Missing iceErrors should return error
    let result = harvester.import_ice_rules(&db, &value);
    assert!(result.is_err());
}

#[test]
fn test_parse_rules_missing_rules() {
    let dir = tempdir().unwrap();
    let config_path = create_test_config(dir.path());
    let base_path = dir.path().to_path_buf();

    let json = r#"{"other": "data"}"#;

    let db = Database::open_memory().unwrap();
    let harvester = Harvester::new(&config_path, &base_path).unwrap();

    let result = harvester.parse_rules(&db, json);
    assert!(result.is_err());
}

#[test]
fn test_parse_migration_missing_changes() {
    let dir = tempdir().unwrap();
    let config_path = create_test_config(dir.path());
    let base_path = dir.path().to_path_buf();

    let json = r#"{"from": "v3", "to": "v4"}"#;

    let db = Database::open_memory().unwrap();
    let harvester = Harvester::new(&config_path, &base_path).unwrap();

    let result = harvester.parse_migration(&db, json);
    assert!(result.is_err());
}

// Test empty arrays
#[test]
fn test_import_empty_keywords() {
    let dir = tempdir().unwrap();
    let config_path = create_test_config(dir.path());
    let base_path = dir.path().to_path_buf();

    let json = r#"{"elements": [], "preprocessor": []}"#;

    let db = Database::open_memory().unwrap();
    let harvester = Harvester::new(&config_path, &base_path).unwrap();

    let value: serde_json::Value = serde_json::from_str(json).unwrap();
    let count = harvester.import_keywords(&db, &value).unwrap();

    assert_eq!(count, 0);
}

#[test]
fn test_import_partial_keywords() {
    let dir = tempdir().unwrap();
    let config_path = create_test_config(dir.path());
    let base_path = dir.path().to_path_buf();

    // Only elements, no preprocessor
    let json = r#"{"elements": ["Package"]}"#;

    let db = Database::open_memory().unwrap();
    let harvester = Harvester::new(&config_path, &base_path).unwrap();

    let value: serde_json::Value = serde_json::from_str(json).unwrap();
    let count = harvester.import_keywords(&db, &value).unwrap();

    assert_eq!(count, 1);
}

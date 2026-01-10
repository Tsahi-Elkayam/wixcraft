//! Rule-related tests

use tempfile::tempdir;
use wixkb::db::Database;
use wixkb::models::*;

#[test]
fn test_insert_rule() {
    let db = Database::open_memory().unwrap();

    let rule = Rule {
        id: 0,
        rule_id: "COMP001".to_string(),
        category: "component".to_string(),
        severity: Severity::Error,
        name: "Component must have Guid".to_string(),
        description: Some("Every component needs a GUID".to_string()),
        rationale: Some("For upgrade handling".to_string()),
        fix_suggestion: Some("Add Guid=\"*\"".to_string()),
        enabled: true,
        auto_fixable: true,
        conditions: Vec::new(),
    };

    let id = db.insert_rule(&rule).unwrap();
    assert!(id > 0);

    let stats = db.get_stats().unwrap();
    assert_eq!(stats.rules, 1);
}

#[test]
fn test_get_rule_by_id() {
    let db = Database::open_memory().unwrap();

    let rule = Rule {
        id: 0,
        rule_id: "FILE001".to_string(),
        category: "file".to_string(),
        severity: Severity::Warning,
        name: "File should have KeyPath".to_string(),
        description: Some("Mark files as KeyPath".to_string()),
        rationale: None,
        fix_suggestion: None,
        enabled: true,
        auto_fixable: false,
        conditions: Vec::new(),
    };
    db.insert_rule(&rule).unwrap();

    let retrieved = db.get_rule("FILE001").unwrap().unwrap();
    assert_eq!(retrieved.rule_id, "FILE001");
    assert_eq!(retrieved.category, "file");
    assert_eq!(retrieved.severity, Severity::Warning);
    assert_eq!(retrieved.name, "File should have KeyPath");
}

#[test]
fn test_get_rule_not_found() {
    let db = Database::open_memory().unwrap();
    let result = db.get_rule("NONEXISTENT").unwrap();
    assert!(result.is_none());
}

#[test]
fn test_get_rules_by_category() {
    let db = Database::open_memory().unwrap();

    let rules = vec![
        ("COMP001", "component", "Rule 1"),
        ("COMP002", "component", "Rule 2"),
        ("FILE001", "file", "Rule 3"),
    ];

    for (id, cat, name) in rules {
        let rule = Rule {
            id: 0,
            rule_id: id.to_string(),
            category: cat.to_string(),
            severity: Severity::Warning,
            name: name.to_string(),
            description: None,
            rationale: None,
            fix_suggestion: None,
            enabled: true,
            auto_fixable: false,
            conditions: Vec::new(),
        };
        db.insert_rule(&rule).unwrap();
    }

    let comp_rules = db.get_rules_by_category("component").unwrap();
    assert_eq!(comp_rules.len(), 2);

    let file_rules = db.get_rules_by_category("file").unwrap();
    assert_eq!(file_rules.len(), 1);

    let empty_rules = db.get_rules_by_category("nonexistent").unwrap();
    assert_eq!(empty_rules.len(), 0);
}

#[test]
fn test_get_enabled_rules() {
    let db = Database::open_memory().unwrap();

    let rules = vec![
        ("RULE001", true),
        ("RULE002", false),
        ("RULE003", true),
    ];

    for (id, enabled) in rules {
        let rule = Rule {
            id: 0,
            rule_id: id.to_string(),
            category: "test".to_string(),
            severity: Severity::Warning,
            name: id.to_string(),
            description: None,
            rationale: None,
            fix_suggestion: None,
            enabled,
            auto_fixable: false,
            conditions: Vec::new(),
        };
        db.insert_rule(&rule).unwrap();
    }

    let enabled = db.get_enabled_rules().unwrap();
    assert_eq!(enabled.len(), 2);
    assert!(enabled.iter().all(|r| r.enabled));
}

#[test]
fn test_search_rules() {
    let db = Database::open_memory().unwrap();

    let rules = vec![
        ("COMP001", "Component GUID required"),
        ("FILE001", "File KeyPath recommended"),
        ("DIR001", "Directory structure"),
    ];

    for (id, name) in rules {
        let rule = Rule {
            id: 0,
            rule_id: id.to_string(),
            category: "test".to_string(),
            severity: Severity::Warning,
            name: name.to_string(),
            description: None,
            rationale: None,
            fix_suggestion: None,
            enabled: true,
            auto_fixable: false,
            conditions: Vec::new(),
        };
        db.insert_rule(&rule).unwrap();
    }

    let results = db.search_rules("GUID", 10).unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].rule_id, "COMP001");
}

#[test]
fn test_rule_severity_levels() {
    let db = Database::open_memory().unwrap();

    let severities = vec![
        ("ERR001", Severity::Error),
        ("WARN001", Severity::Warning),
        ("INFO001", Severity::Info),
    ];

    for (id, severity) in severities.clone() {
        let rule = Rule {
            id: 0,
            rule_id: id.to_string(),
            category: "test".to_string(),
            severity,
            name: id.to_string(),
            description: None,
            rationale: None,
            fix_suggestion: None,
            enabled: true,
            auto_fixable: false,
            conditions: Vec::new(),
        };
        db.insert_rule(&rule).unwrap();
    }

    for (id, expected_severity) in severities {
        let rule = db.get_rule(id).unwrap().unwrap();
        assert_eq!(rule.severity, expected_severity);
    }
}

#[test]
fn test_ice_rule_insert_and_get() {
    let db = Database::open_memory().unwrap();

    db.conn().execute(
        "INSERT INTO ice_rules (code, severity, description, resolution, tables_affected)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params!["ICE03", "error", "Schema validation", "Fix schema", "Component,File"],
    ).unwrap();

    let ice = db.get_ice_rule("ICE03").unwrap().unwrap();
    assert_eq!(ice.code, "ICE03");
    assert_eq!(ice.severity, Severity::Error);
    assert_eq!(ice.description, Some("Schema validation".to_string()));
    assert_eq!(ice.resolution, Some("Fix schema".to_string()));
    assert_eq!(ice.tables_affected.len(), 2);
    assert!(ice.tables_affected.contains(&"Component".to_string()));
    assert!(ice.tables_affected.contains(&"File".to_string()));
}

#[test]
fn test_ice_rule_not_found() {
    let db = Database::open_memory().unwrap();
    let result = db.get_ice_rule("ICE99").unwrap();
    assert!(result.is_none());
}

#[test]
fn test_get_all_ice_rules() {
    let db = Database::open_memory().unwrap();

    for code in ["ICE01", "ICE02", "ICE03"] {
        db.conn().execute(
            "INSERT INTO ice_rules (code, severity, description) VALUES (?1, ?2, ?3)",
            rusqlite::params![code, "error", format!("{} description", code)],
        ).unwrap();
    }

    let rules = db.get_all_ice_rules().unwrap();
    assert_eq!(rules.len(), 3);
}

#[test]
fn test_error_code_insert_and_get() {
    let db = Database::open_memory().unwrap();

    db.conn().execute(
        "INSERT INTO errors (code, severity, message_template, description, resolution)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![
            "WIX0001",
            "error",
            "Element {0} cannot be child of {1}",
            "Invalid parent-child relationship",
            "Check documentation for valid parents"
        ],
    ).unwrap();

    let error = db.get_error("WIX0001").unwrap().unwrap();
    assert_eq!(error.code, "WIX0001");
    assert_eq!(error.severity, Severity::Error);
    assert!(error.message_template.contains("{0}"));
}

#[test]
fn test_error_not_found() {
    let db = Database::open_memory().unwrap();
    let result = db.get_error("WIX9999").unwrap();
    assert!(result.is_none());
}

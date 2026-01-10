//! CLI integration tests

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::tempdir;
use wixkb::db::Database;
use wixkb::models::*;

fn create_test_db() -> (tempfile::TempDir, std::path::PathBuf) {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("database").join("wixkb.db");
    std::fs::create_dir_all(db_path.parent().unwrap()).unwrap();

    let db = Database::create(&db_path).unwrap();

    // Insert test data
    let elem = Element {
        id: 0,
        name: "Package".to_string(),
        namespace: "wix".to_string(),
        since_version: Some("v4".to_string()),
        deprecated_version: None,
        description: Some("Root package element".to_string()),
        documentation_url: Some("https://wixtoolset.org/docs/".to_string()),
        remarks: None,
        example: None,
    };
    let elem_id = db.insert_element(&elem).unwrap();

    let attr = Attribute {
        id: 0,
        element_id: elem_id,
        name: "Name".to_string(),
        attr_type: AttributeType::String,
        required: true,
        default_value: None,
        description: None,
        since_version: None,
        deprecated_version: None,
        enum_values: Vec::new(),
    };
    db.insert_attribute(&attr).unwrap();

    let rule = Rule {
        id: 0,
        rule_id: "COMP001".to_string(),
        category: "component".to_string(),
        severity: Severity::Error,
        name: "Component Guid".to_string(),
        description: Some("Guid required".to_string()),
        rationale: Some("For upgrades".to_string()),
        fix_suggestion: Some("Add Guid".to_string()),
        enabled: true,
        auto_fixable: false,
        conditions: Vec::new(),
    };
    db.insert_rule(&rule).unwrap();

    db.conn().execute(
        "INSERT INTO ice_rules (code, severity, description, resolution) VALUES ('ICE03', 'error', 'Schema', 'Fix it')",
        [],
    ).unwrap();

    db.conn().execute(
        "INSERT INTO standard_directories (name, description, windows_path) VALUES ('ProgramFilesFolder', 'Program Files', 'C:\\Program Files')",
        [],
    ).unwrap();

    db.conn().execute(
        "INSERT INTO snippets (prefix, name, body, scope) VALUES ('wix-pkg', 'Package', '<Package />', 'wxs')",
        [],
    ).unwrap();

    db.set_last_updated().unwrap();
    drop(db);

    (dir, db_path)
}

// wixkb CLI tests

#[test]
fn test_wixkb_stats() {
    let (dir, db_path) = create_test_db();

    Command::cargo_bin("wixkb")
        .unwrap()
        .args(["--database", db_path.to_str().unwrap(), "stats"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Elements:"))
        .stdout(predicate::str::contains("1"));
}

#[test]
fn test_wixkb_element() {
    let (dir, db_path) = create_test_db();

    Command::cargo_bin("wixkb")
        .unwrap()
        .args(["--database", db_path.to_str().unwrap(), "element", "Package"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Element: Package"))
        .stdout(predicate::str::contains("Root package element"));
}

#[test]
fn test_wixkb_element_not_found() {
    let (dir, db_path) = create_test_db();

    Command::cargo_bin("wixkb")
        .unwrap()
        .args(["--database", db_path.to_str().unwrap(), "element", "NonExistent"])
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn test_wixkb_element_json() {
    let (dir, db_path) = create_test_db();

    Command::cargo_bin("wixkb")
        .unwrap()
        .args(["--database", db_path.to_str().unwrap(), "--format", "json", "element", "Package"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("\"name\": \"Package\""));
}

#[test]
fn test_wixkb_attribute() {
    let (dir, db_path) = create_test_db();

    Command::cargo_bin("wixkb")
        .unwrap()
        .args(["--database", db_path.to_str().unwrap(), "attribute", "Package", "Name"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Attribute: @Name"));
}

#[test]
fn test_wixkb_attribute_not_found() {
    let (dir, db_path) = create_test_db();

    Command::cargo_bin("wixkb")
        .unwrap()
        .args(["--database", db_path.to_str().unwrap(), "attribute", "Package", "NonExistent"])
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn test_wixkb_search() {
    let (dir, db_path) = create_test_db();

    Command::cargo_bin("wixkb")
        .unwrap()
        .args(["--database", db_path.to_str().unwrap(), "search", "package"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Package"));
}

#[test]
fn test_wixkb_search_no_results() {
    let (dir, db_path) = create_test_db();

    Command::cargo_bin("wixkb")
        .unwrap()
        .args(["--database", db_path.to_str().unwrap(), "search", "zzzznotfound"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("No results"));
}

#[test]
fn test_wixkb_children() {
    let (dir, db_path) = create_test_db();

    Command::cargo_bin("wixkb")
        .unwrap()
        .args(["--database", db_path.to_str().unwrap(), "children", "Package"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("No children"));
}

#[test]
fn test_wixkb_parents() {
    let (dir, db_path) = create_test_db();

    Command::cargo_bin("wixkb")
        .unwrap()
        .args(["--database", db_path.to_str().unwrap(), "parents", "Package"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("No parents"));
}

#[test]
fn test_wixkb_rule() {
    let (dir, db_path) = create_test_db();

    Command::cargo_bin("wixkb")
        .unwrap()
        .args(["--database", db_path.to_str().unwrap(), "rule", "COMP001"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("[COMP001]"))
        .stdout(predicate::str::contains("Component Guid"));
}

#[test]
fn test_wixkb_rule_not_found() {
    let (dir, db_path) = create_test_db();

    Command::cargo_bin("wixkb")
        .unwrap()
        .args(["--database", db_path.to_str().unwrap(), "rule", "NONEXISTENT"])
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn test_wixkb_rules() {
    let (dir, db_path) = create_test_db();

    Command::cargo_bin("wixkb")
        .unwrap()
        .args(["--database", db_path.to_str().unwrap(), "rules"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("COMP001"));
}

#[test]
fn test_wixkb_rules_by_category() {
    let (dir, db_path) = create_test_db();

    Command::cargo_bin("wixkb")
        .unwrap()
        .args(["--database", db_path.to_str().unwrap(), "rules", "--category", "component"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("COMP001"));
}

#[test]
fn test_wixkb_ice() {
    let (dir, db_path) = create_test_db();

    Command::cargo_bin("wixkb")
        .unwrap()
        .args(["--database", db_path.to_str().unwrap(), "ice", "ICE03"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("[ICE03]"));
}

#[test]
fn test_wixkb_ice_not_found() {
    let (dir, db_path) = create_test_db();

    Command::cargo_bin("wixkb")
        .unwrap()
        .args(["--database", db_path.to_str().unwrap(), "ice", "ICE99"])
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn test_wixkb_directory() {
    let (dir, db_path) = create_test_db();

    Command::cargo_bin("wixkb")
        .unwrap()
        .args(["--database", db_path.to_str().unwrap(), "directory", "ProgramFilesFolder"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("ProgramFilesFolder"));
}

#[test]
fn test_wixkb_directory_not_found() {
    let (dir, db_path) = create_test_db();

    Command::cargo_bin("wixkb")
        .unwrap()
        .args(["--database", db_path.to_str().unwrap(), "directory", "NonExistent"])
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn test_wixkb_snippets() {
    let (dir, db_path) = create_test_db();

    Command::cargo_bin("wixkb")
        .unwrap()
        .args(["--database", db_path.to_str().unwrap(), "snippets"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("wix-pkg"));
}

#[test]
fn test_wixkb_snippets_by_prefix() {
    let (dir, db_path) = create_test_db();

    Command::cargo_bin("wixkb")
        .unwrap()
        .args(["--database", db_path.to_str().unwrap(), "snippets", "--prefix", "wix"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("wix-pkg"));
}

// database CLI tests

#[test]
fn test_database_init() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("new.db");

    Command::cargo_bin("database")
        .unwrap()
        .args(["--database", db_path.to_str().unwrap(), "init"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Created database"));

    assert!(db_path.exists());
}

#[test]
fn test_database_init_force() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("existing.db");
    std::fs::write(&db_path, "dummy").unwrap();

    Command::cargo_bin("database")
        .unwrap()
        .args(["--database", db_path.to_str().unwrap(), "init", "--force"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Created database"));
}

#[test]
fn test_database_init_exists() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("existing.db");
    std::fs::write(&db_path, "dummy").unwrap();

    Command::cargo_bin("database")
        .unwrap()
        .args(["--database", db_path.to_str().unwrap(), "init"])
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("already exists"));
}

#[test]
fn test_database_stats() {
    let (dir, db_path) = create_test_db();

    Command::cargo_bin("database")
        .unwrap()
        .args(["--database", db_path.to_str().unwrap(), "stats"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Elements:"))
        .stdout(predicate::str::contains("File size:"));
}

#[test]
fn test_database_check() {
    let (dir, db_path) = create_test_db();

    Command::cargo_bin("database")
        .unwrap()
        .args(["--database", db_path.to_str().unwrap(), "check"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("OK"));
}

#[test]
fn test_database_path() {
    let (dir, db_path) = create_test_db();

    Command::cargo_bin("database")
        .unwrap()
        .args(["--database", db_path.to_str().unwrap(), "path"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("wixkb.db"));
}

#[test]
fn test_database_vacuum() {
    let (dir, db_path) = create_test_db();

    Command::cargo_bin("database")
        .unwrap()
        .args(["--database", db_path.to_str().unwrap(), "vacuum"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Vacuumed"));
}

#[test]
fn test_database_backup() {
    let (dir, db_path) = create_test_db();
    let backup_path = dir.path().join("backup.db");

    Command::cargo_bin("database")
        .unwrap()
        .args(["--database", db_path.to_str().unwrap(), "backup", backup_path.to_str().unwrap()])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Backed up"));

    assert!(backup_path.exists());
}

#[test]
fn test_database_export() {
    let (dir, db_path) = create_test_db();
    let export_path = dir.path().join("export.json");

    Command::cargo_bin("database")
        .unwrap()
        .args(["--database", db_path.to_str().unwrap(), "export", export_path.to_str().unwrap()])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Exported"));

    assert!(export_path.exists());
    let content = std::fs::read_to_string(&export_path).unwrap();
    assert!(content.contains("Package"));
}

#[test]
fn test_database_reset_requires_yes() {
    let (dir, db_path) = create_test_db();

    Command::cargo_bin("database")
        .unwrap()
        .args(["--database", db_path.to_str().unwrap(), "reset"])
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("--yes"));
}

#[test]
fn test_database_reset_with_yes() {
    let (dir, db_path) = create_test_db();

    Command::cargo_bin("database")
        .unwrap()
        .args(["--database", db_path.to_str().unwrap(), "reset", "--yes"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("reset"));
}

// harvest CLI tests

#[test]
fn test_harvest_status_no_db() {
    let dir = tempdir().unwrap();

    Command::cargo_bin("harvest")
        .unwrap()
        .args(["--database", dir.path().join("nonexistent.db").to_str().unwrap(), "status"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("not initialized"));
}

#[test]
fn test_harvest_status_with_db() {
    let (dir, db_path) = create_test_db();

    Command::cargo_bin("harvest")
        .unwrap()
        .args(["--database", db_path.to_str().unwrap(), "status"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("initialized"));
}

#[test]
fn test_harvest_validate_missing_config() {
    let dir = tempdir().unwrap();

    Command::cargo_bin("harvest")
        .unwrap()
        .args(["--config", dir.path().join("nonexistent.yaml").to_str().unwrap(), "validate"])
        .current_dir(dir.path())
        .assert()
        .failure();
}

fn create_harvest_config(dir: &std::path::Path) -> std::path::PathBuf {
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
parsers:
  json:
    type: json
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

    // Create data directory and file
    let data_dir = dir.join("data");
    std::fs::create_dir_all(&data_dir).unwrap();
    std::fs::write(data_dir.join("elements.json"), r#"{"elements": ["Package"]}"#).unwrap();

    config_path
}

#[test]
fn test_harvest_validate_valid() {
    let dir = tempdir().unwrap();
    let config_path = create_harvest_config(dir.path());

    Command::cargo_bin("harvest")
        .unwrap()
        .args(["--config", config_path.to_str().unwrap(), "validate"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("valid"))
        .stdout(predicate::str::contains("Categories"));
}

#[test]
fn test_harvest_list() {
    let dir = tempdir().unwrap();
    let config_path = create_harvest_config(dir.path());

    Command::cargo_bin("harvest")
        .unwrap()
        .args(["--config", config_path.to_str().unwrap(), "list"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("[elements]"))
        .stdout(predicate::str::contains("local-elements"));
}

#[test]
fn test_harvest_list_by_category() {
    let dir = tempdir().unwrap();
    let config_path = create_harvest_config(dir.path());

    Command::cargo_bin("harvest")
        .unwrap()
        .args(["--config", config_path.to_str().unwrap(), "list", "--category", "elements"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("local-elements"));
}

#[test]
fn test_harvest_list_category_not_found() {
    let dir = tempdir().unwrap();
    let config_path = create_harvest_config(dir.path());

    Command::cargo_bin("harvest")
        .unwrap()
        .args(["--config", config_path.to_str().unwrap(), "list", "--category", "nonexistent"])
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn test_harvest_clear_cache_empty() {
    let dir = tempdir().unwrap();
    let config_path = create_harvest_config(dir.path());

    Command::cargo_bin("harvest")
        .unwrap()
        .args(["--config", config_path.to_str().unwrap(), "clear-cache"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("empty"));
}

#[test]
fn test_harvest_clear_cache_with_files() {
    let dir = tempdir().unwrap();
    let config_path = create_harvest_config(dir.path());

    // Create cache directory with files
    let cache_dir = dir.path().join(".cache");
    std::fs::create_dir_all(&cache_dir).unwrap();
    std::fs::write(cache_dir.join("test.cache"), "cached").unwrap();

    Command::cargo_bin("harvest")
        .unwrap()
        .args(["--config", config_path.to_str().unwrap(), "clear-cache"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Cleared"));
}

#[test]
fn test_harvest_all_no_db() {
    let dir = tempdir().unwrap();
    let config_path = create_harvest_config(dir.path());

    Command::cargo_bin("harvest")
        .unwrap()
        .args(["--config", config_path.to_str().unwrap(), "--database", "nonexistent.db", "all"])
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn test_harvest_source_no_db() {
    let dir = tempdir().unwrap();
    let config_path = create_harvest_config(dir.path());

    Command::cargo_bin("harvest")
        .unwrap()
        .args(["--config", config_path.to_str().unwrap(), "--database", "nonexistent.db", "source", "elements", "local"])
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn test_harvest_all_with_db() {
    let (dir, db_path) = create_test_db();
    let config_path = create_harvest_config(dir.path());

    Command::cargo_bin("harvest")
        .unwrap()
        .args(["--config", config_path.to_str().unwrap(), "--database", db_path.to_str().unwrap(), "all"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Harvest complete"));
}

#[test]
fn test_harvest_all_verbose() {
    let (dir, db_path) = create_test_db();
    let config_path = create_harvest_config(dir.path());

    Command::cargo_bin("harvest")
        .unwrap()
        .args(["--config", config_path.to_str().unwrap(), "--database", db_path.to_str().unwrap(), "--verbose", "all"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Source details"));
}

#[test]
fn test_harvest_source_not_found() {
    let (dir, db_path) = create_test_db();
    let config_path = create_harvest_config(dir.path());

    Command::cargo_bin("harvest")
        .unwrap()
        .args(["--config", config_path.to_str().unwrap(), "--database", db_path.to_str().unwrap(), "source", "elements", "nonexistent"])
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn test_harvest_source_success() {
    let (dir, db_path) = create_test_db();
    let config_path = create_harvest_config(dir.path());

    Command::cargo_bin("harvest")
        .unwrap()
        .args(["--config", config_path.to_str().unwrap(), "--database", db_path.to_str().unwrap(), "source", "elements", "local-elements"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Harvested"));
}

// More wixkb CLI tests for JSON output

#[test]
fn test_wixkb_attribute_json() {
    let (dir, db_path) = create_test_db();

    Command::cargo_bin("wixkb")
        .unwrap()
        .args(["--database", db_path.to_str().unwrap(), "--format", "json", "attribute", "Package", "Name"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("\"name\": \"Name\""));
}

#[test]
fn test_wixkb_search_json() {
    let (dir, db_path) = create_test_db();

    Command::cargo_bin("wixkb")
        .unwrap()
        .args(["--database", db_path.to_str().unwrap(), "--format", "json", "search", "package"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("["));
}

#[test]
fn test_wixkb_children_json() {
    let (dir, db_path) = create_test_db();

    Command::cargo_bin("wixkb")
        .unwrap()
        .args(["--database", db_path.to_str().unwrap(), "--format", "json", "children", "Package"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("["));
}

#[test]
fn test_wixkb_parents_json() {
    let (dir, db_path) = create_test_db();

    Command::cargo_bin("wixkb")
        .unwrap()
        .args(["--database", db_path.to_str().unwrap(), "--format", "json", "parents", "Package"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("["));
}

#[test]
fn test_wixkb_rule_json() {
    let (dir, db_path) = create_test_db();

    Command::cargo_bin("wixkb")
        .unwrap()
        .args(["--database", db_path.to_str().unwrap(), "--format", "json", "rule", "COMP001"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("\"rule_id\": \"COMP001\""));
}

#[test]
fn test_wixkb_rules_json() {
    let (dir, db_path) = create_test_db();

    Command::cargo_bin("wixkb")
        .unwrap()
        .args(["--database", db_path.to_str().unwrap(), "--format", "json", "rules"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("["));
}

#[test]
fn test_wixkb_ice_json() {
    let (dir, db_path) = create_test_db();

    Command::cargo_bin("wixkb")
        .unwrap()
        .args(["--database", db_path.to_str().unwrap(), "--format", "json", "ice", "ICE03"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("\"code\": \"ICE03\""));
}

#[test]
fn test_wixkb_directory_json() {
    let (dir, db_path) = create_test_db();

    Command::cargo_bin("wixkb")
        .unwrap()
        .args(["--database", db_path.to_str().unwrap(), "--format", "json", "directory", "ProgramFilesFolder"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("\"name\": \"ProgramFilesFolder\""));
}

#[test]
fn test_wixkb_snippets_json() {
    let (dir, db_path) = create_test_db();

    Command::cargo_bin("wixkb")
        .unwrap()
        .args(["--database", db_path.to_str().unwrap(), "--format", "json", "snippets"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("["));
}

#[test]
fn test_wixkb_stats_json() {
    let (dir, db_path) = create_test_db();

    Command::cargo_bin("wixkb")
        .unwrap()
        .args(["--database", db_path.to_str().unwrap(), "--format", "json", "stats"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("\"elements\""));
}

#[test]
fn test_wixkb_error_command() {
    let (dir, db_path) = create_test_db();

    // Add an error to the test db
    let db = wixkb::db::Database::open(&db_path).unwrap();
    db.conn().execute(
        "INSERT INTO errors (code, severity, message_template, description, resolution) VALUES ('WIX0001', 'error', 'Test error', 'Description', 'Fix it')",
        [],
    ).unwrap();
    drop(db);

    Command::cargo_bin("wixkb")
        .unwrap()
        .args(["--database", db_path.to_str().unwrap(), "error", "WIX0001"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("[WIX0001]"));
}

#[test]
fn test_wixkb_error_json() {
    let (dir, db_path) = create_test_db();

    let db = wixkb::db::Database::open(&db_path).unwrap();
    db.conn().execute(
        "INSERT INTO errors (code, severity, message_template) VALUES ('WIX0002', 'warning', 'Another error')",
        [],
    ).unwrap();
    drop(db);

    Command::cargo_bin("wixkb")
        .unwrap()
        .args(["--database", db_path.to_str().unwrap(), "--format", "json", "error", "WIX0002"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("\"code\": \"WIX0002\""));
}

#[test]
fn test_wixkb_error_not_found() {
    let (dir, db_path) = create_test_db();

    Command::cargo_bin("wixkb")
        .unwrap()
        .args(["--database", db_path.to_str().unwrap(), "error", "NONEXISTENT"])
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

// config CLI tests

fn create_sources_config(dir: &std::path::Path) -> std::path::PathBuf {
    let config_path = dir.join("sources.yaml");
    let content = r#"
version: "1.0"
sources:
  elements:
    wix-core:
      url: "https://example.com/wix.xsd"
      parser: "xml"
      targets: []
    wix-local:
      path: "data/elements.json"
      parser: "json"
      targets:
        - keywords
  rules:
    lint-rules:
      path: "data/rules.json"
      parser: "rules"
      targets: []
parsers:
  xml:
    type: xml
  json:
    type: json
  rules:
    type: rules
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
    config_path
}

#[test]
fn test_config_show() {
    let dir = tempdir().unwrap();
    let config_path = create_sources_config(dir.path());

    Command::cargo_bin("config")
        .unwrap()
        .args(["--config", config_path.to_str().unwrap(), "show"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Configuration:"))
        .stdout(predicate::str::contains("Sources:"));
}

#[test]
fn test_config_show_not_found() {
    let dir = tempdir().unwrap();

    Command::cargo_bin("config")
        .unwrap()
        .args(["--config", "nonexistent.yaml", "show"])
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn test_config_sources_list() {
    let dir = tempdir().unwrap();
    let config_path = create_sources_config(dir.path());

    Command::cargo_bin("config")
        .unwrap()
        .args(["--config", config_path.to_str().unwrap(), "sources", "list"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("[elements]"))
        .stdout(predicate::str::contains("wix-core"));
}

#[test]
fn test_config_sources_list_by_category() {
    let dir = tempdir().unwrap();
    let config_path = create_sources_config(dir.path());

    Command::cargo_bin("config")
        .unwrap()
        .args(["--config", config_path.to_str().unwrap(), "sources", "list", "--category", "elements"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("[elements]"));
}

#[test]
fn test_config_sources_add() {
    let dir = tempdir().unwrap();
    let config_path = create_sources_config(dir.path());

    Command::cargo_bin("config")
        .unwrap()
        .args(["--config", config_path.to_str().unwrap(), "sources", "add", "test", "new-source", "--url", "https://example.com"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Adding source"));
}

#[test]
fn test_config_sources_remove() {
    let dir = tempdir().unwrap();
    let config_path = create_sources_config(dir.path());

    Command::cargo_bin("config")
        .unwrap()
        .args(["--config", config_path.to_str().unwrap(), "sources", "remove", "elements", "wix-core"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Removing source"));
}

#[test]
fn test_config_sources_enable() {
    let dir = tempdir().unwrap();
    let config_path = create_sources_config(dir.path());

    Command::cargo_bin("config")
        .unwrap()
        .args(["--config", config_path.to_str().unwrap(), "sources", "enable", "elements", "wix-core"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Enabling"));
}

#[test]
fn test_config_sources_disable() {
    let dir = tempdir().unwrap();
    let config_path = create_sources_config(dir.path());

    Command::cargo_bin("config")
        .unwrap()
        .args(["--config", config_path.to_str().unwrap(), "sources", "disable", "elements", "wix-core"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Disabling"));
}

#[test]
fn test_config_rules_enable() {
    let dir = tempdir().unwrap();

    Command::cargo_bin("config")
        .unwrap()
        .args(["rules", "enable", "COMP001"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Enabled rule"));

    assert!(dir.path().join(".wixlintrc.json").exists());
}

#[test]
fn test_config_rules_disable() {
    let dir = tempdir().unwrap();

    Command::cargo_bin("config")
        .unwrap()
        .args(["rules", "disable", "COMP001"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Disabled rule"));
}

#[test]
fn test_config_rules_severity() {
    let dir = tempdir().unwrap();

    Command::cargo_bin("config")
        .unwrap()
        .args(["rules", "severity", "COMP001", "warning"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Set COMP001 severity"));
}

#[test]
fn test_config_rules_severity_invalid() {
    let dir = tempdir().unwrap();

    Command::cargo_bin("config")
        .unwrap()
        .args(["rules", "severity", "COMP001", "invalid"])
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("Invalid severity"));
}

#[test]
fn test_config_rules_list_no_config() {
    let dir = tempdir().unwrap();

    Command::cargo_bin("config")
        .unwrap()
        .args(["rules", "list"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("No lint configuration"));
}

#[test]
fn test_config_rules_list_with_config() {
    let dir = tempdir().unwrap();
    let lint_path = dir.path().join(".wixlintrc.json");
    std::fs::write(&lint_path, r#"{"disabled_rules": ["COMP001"], "enabled_rules": ["FILE001"], "severity_overrides": {"DIR001": "error"}}"#).unwrap();

    Command::cargo_bin("config")
        .unwrap()
        .args(["rules", "list"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Disabled rules"))
        .stdout(predicate::str::contains("COMP001"))
        .stdout(predicate::str::contains("Enabled rules"))
        .stdout(predicate::str::contains("FILE001"))
        .stdout(predicate::str::contains("Severity overrides"));
}

#[test]
fn test_config_lint_show_no_file() {
    let dir = tempdir().unwrap();

    Command::cargo_bin("config")
        .unwrap()
        .args(["lint", "show"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("No .wixlintrc.json"));
}

#[test]
fn test_config_lint_show_with_file() {
    let dir = tempdir().unwrap();
    let lint_path = dir.path().join(".wixlintrc.json");
    std::fs::write(&lint_path, r#"{"disabled_rules": []}"#).unwrap();

    Command::cargo_bin("config")
        .unwrap()
        .args(["lint", "show"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("disabled_rules"));
}

#[test]
fn test_config_lint_init() {
    let dir = tempdir().unwrap();

    Command::cargo_bin("config")
        .unwrap()
        .args(["lint", "init"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Created"));

    assert!(dir.path().join(".wixlintrc.json").exists());
}

#[test]
fn test_config_lint_init_exists() {
    let dir = tempdir().unwrap();
    let lint_path = dir.path().join(".wixlintrc.json");
    std::fs::write(&lint_path, "{}").unwrap();

    Command::cargo_bin("config")
        .unwrap()
        .args(["lint", "init"])
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("already exists"));
}

#[test]
fn test_config_lint_init_force() {
    let dir = tempdir().unwrap();
    let lint_path = dir.path().join(".wixlintrc.json");
    std::fs::write(&lint_path, "{}").unwrap();

    Command::cargo_bin("config")
        .unwrap()
        .args(["lint", "init", "--force"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Created"));
}

#[test]
fn test_config_lint_validate_valid() {
    let dir = tempdir().unwrap();
    let lint_path = dir.path().join(".wixlintrc.json");
    std::fs::write(&lint_path, r#"{"disabled_rules": [], "enabled_rules": [], "severity_overrides": {}}"#).unwrap();

    Command::cargo_bin("config")
        .unwrap()
        .args(["lint", "validate"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("valid"));
}

#[test]
fn test_config_lint_validate_invalid() {
    let dir = tempdir().unwrap();
    let lint_path = dir.path().join(".wixlintrc.json");
    std::fs::write(&lint_path, "not valid json").unwrap();

    Command::cargo_bin("config")
        .unwrap()
        .args(["lint", "validate"])
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("Invalid"));
}

#[test]
fn test_config_lint_validate_no_file() {
    let dir = tempdir().unwrap();

    Command::cargo_bin("config")
        .unwrap()
        .args(["lint", "validate"])
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("No .wixlintrc.json"));
}

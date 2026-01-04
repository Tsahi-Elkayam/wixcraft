//! Integration tests for wix-lint

use std::path::PathBuf;
use wix_lint::{
    config::Config,
    diagnostics::Severity,
    engine::LintEngine,
    loader::RuleLoader,
    output::{format_json, format_sarif},
    parser::WixDocument,
    rules::Rule,
};

fn fixtures_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

fn wix_data_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("core/wix-data")
}

fn load_rules() -> Vec<Rule> {
    let wix_data = wix_data_path();
    if !wix_data.exists() {
        return Vec::new();
    }
    let loader = RuleLoader::new(&wix_data);
    loader.load_all().unwrap_or_default()
}

#[test]
fn test_parse_valid_file() {
    let path = fixtures_path().join("valid.wxs");
    let result = WixDocument::parse_file(&path);

    assert!(result.is_ok());
    let wix_file = result.unwrap();
    assert!(!wix_file.elements.is_empty());
}

#[test]
fn test_parse_invalid_file() {
    let path = fixtures_path().join("invalid.wxs");
    let result = WixDocument::parse_file(&path);

    assert!(result.is_ok());
    let wix_file = result.unwrap();
    assert!(!wix_file.elements.is_empty());
}

#[test]
fn test_parse_file_with_disable_comments() {
    let path = fixtures_path().join("with-disable.wxs");
    let result = WixDocument::parse_file(&path);

    assert!(result.is_ok());
    let wix_file = result.unwrap();

    // Should have parsed inline disables
    assert!(!wix_file.inline_disables.is_empty());
}

#[test]
fn test_lint_engine_basic() {
    let rules = load_rules();
    if rules.is_empty() {
        return; // Skip if rules not available
    }

    let config = Config::default();
    let engine = LintEngine::new(rules, config);

    let path = fixtures_path().join("valid.wxs");
    let diagnostics = engine.lint_file(&path).unwrap();

    // Valid file should have few or no errors
    let errors: Vec<_> = diagnostics
        .iter()
        .filter(|d| d.severity == Severity::Error)
        .collect();
    assert!(
        errors.len() < 5,
        "Valid file should have minimal errors, found {}",
        errors.len()
    );
}

#[test]
fn test_lint_engine_finds_issues() {
    let rules = load_rules();
    if rules.is_empty() {
        return; // Skip if rules not available
    }

    let config = Config::default();
    let engine = LintEngine::new(rules, config);

    let path = fixtures_path().join("invalid.wxs");
    let diagnostics = engine.lint_file(&path).unwrap();

    // Invalid file should have issues
    assert!(
        !diagnostics.is_empty(),
        "Invalid file should have diagnostics"
    );
}

#[test]
fn test_inline_disable_suppresses_warnings() {
    let rules = load_rules();
    if rules.is_empty() {
        return; // Skip if rules not available
    }

    let config = Config::default();
    let engine = LintEngine::new(rules, config);

    let path = fixtures_path().join("with-disable.wxs");
    let diagnostics = engine.lint_file(&path).unwrap();

    // Check that disabled rules are not reported
    let package_upgrade_errors: Vec<_> = diagnostics
        .iter()
        .filter(|d| d.rule_id == "package-requires-upgradecode")
        .filter(|d| d.location.line == 4) // The Package line
        .collect();

    assert!(
        package_upgrade_errors.is_empty(),
        "Disabled rule should not be reported"
    );
}

#[test]
fn test_config_disable_rule() {
    let rules = load_rules();
    if rules.is_empty() {
        return; // Skip if rules not available
    }

    let mut config = Config::default();
    config.disabled_rules.push("package-requires-upgradecode".to_string());

    let engine = LintEngine::new(rules, config);

    let path = fixtures_path().join("invalid.wxs");
    let diagnostics = engine.lint_file(&path).unwrap();

    // Disabled rule should not appear
    let upgrade_code_errors: Vec<_> = diagnostics
        .iter()
        .filter(|d| d.rule_id == "package-requires-upgradecode")
        .collect();

    assert!(
        upgrade_code_errors.is_empty(),
        "Disabled rule should not be reported"
    );
}

#[test]
fn test_config_severity_override() {
    let rules = load_rules();
    if rules.is_empty() {
        return; // Skip if rules not available
    }

    let mut config = Config::default();
    // Override any error to info
    config.severity_overrides.insert("package-requires-upgradecode".to_string(), Severity::Info);

    let engine = LintEngine::new(rules, config);

    let path = fixtures_path().join("invalid.wxs");
    let diagnostics = engine.lint_file(&path).unwrap();

    // Check that severity was overridden
    let override_diags: Vec<_> = diagnostics
        .iter()
        .filter(|d| d.rule_id == "package-requires-upgradecode")
        .collect();

    for diag in override_diags {
        assert_eq!(
            diag.severity,
            Severity::Info,
            "Severity should be overridden to Info"
        );
    }
}

#[test]
fn test_config_min_severity() {
    let rules = load_rules();
    if rules.is_empty() {
        return; // Skip if rules not available
    }

    let mut config = Config::default();
    config.min_severity = Severity::Error;

    let engine = LintEngine::new(rules, config);

    let path = fixtures_path().join("invalid.wxs");
    let diagnostics = engine.lint_file(&path).unwrap();

    // Should only have errors, no warnings or info
    for diag in &diagnostics {
        assert_eq!(
            diag.severity,
            Severity::Error,
            "Only errors should be reported with min_severity=Error"
        );
    }
}

#[test]
fn test_json_output_format() {
    let rules = load_rules();
    if rules.is_empty() {
        return; // Skip if rules not available
    }

    let config = Config::default();
    let engine = LintEngine::new(rules, config);

    let path = fixtures_path().join("invalid.wxs");
    let diagnostics = engine.lint_file(&path).unwrap();

    let json_output = format_json(&diagnostics);

    // Should be valid JSON
    let parsed: serde_json::Value = serde_json::from_str(&json_output).expect("Invalid JSON output");

    // Should have diagnostics array
    assert!(parsed["diagnostics"].is_array());

    // Should have summary
    assert!(parsed["summary"].is_object());
    assert!(parsed["summary"]["total"].is_number());
}

#[test]
fn test_sarif_output_format() {
    let rules = load_rules();
    if rules.is_empty() {
        return; // Skip if rules not available
    }

    let config = Config::default();
    let engine = LintEngine::new(rules, config);

    let path = fixtures_path().join("invalid.wxs");
    let diagnostics = engine.lint_file(&path).unwrap();

    let sarif_output = format_sarif(&diagnostics);

    // Should be valid JSON
    let parsed: serde_json::Value =
        serde_json::from_str(&sarif_output).expect("Invalid SARIF output");

    // Should be SARIF format
    assert_eq!(parsed["version"], "2.1.0");
    assert!(parsed["runs"].is_array());
    assert_eq!(parsed["runs"][0]["tool"]["driver"]["name"], "wix-lint");
}

#[test]
fn test_multiple_files() {
    let rules = load_rules();
    if rules.is_empty() {
        return; // Skip if rules not available
    }

    let config = Config::default();
    let engine = LintEngine::new(rules, config);

    let files = vec![
        fixtures_path().join("valid.wxs"),
        fixtures_path().join("invalid.wxs"),
    ];

    let mut all_diagnostics = Vec::new();

    for path in &files {
        if let Ok(diagnostics) = engine.lint_file(path) {
            all_diagnostics.extend(diagnostics);
        }
    }

    // Should have diagnostics from at least the invalid file
    assert!(!all_diagnostics.is_empty());
}

#[test]
fn test_element_hierarchy() {
    let path = fixtures_path().join("valid.wxs");
    let wix_file = WixDocument::parse_file(&path).unwrap();

    // Find Package element
    let package = wix_file
        .elements
        .iter()
        .find(|e| e.name == "Package");

    assert!(package.is_some(), "Should find Package element");

    let package = package.unwrap();
    assert!(
        package.attributes.contains_key("Name"),
        "Package should have Name attribute"
    );
}

#[test]
fn test_diagnostics_have_source_lines() {
    let rules = load_rules();
    if rules.is_empty() {
        return; // Skip if rules not available
    }

    let config = Config::default();
    let engine = LintEngine::new(rules, config);

    let path = fixtures_path().join("invalid.wxs");
    let diagnostics = engine.lint_file(&path).unwrap();

    // Most diagnostics should have source lines
    let with_source: Vec<_> = diagnostics
        .iter()
        .filter(|d| d.source_line.is_some())
        .collect();

    assert!(
        with_source.len() > diagnostics.len() / 2,
        "Most diagnostics should have source lines"
    );
}

#[test]
fn test_empty_wix_document() {
    let content = "<?xml version=\"1.0\"?><Wix xmlns=\"http://wixtoolset.org/schemas/v4/wxs\"></Wix>";

    let result = WixDocument::parse_str(content);
    assert!(result.is_ok());
}

#[test]
fn test_malformed_xml() {
    let content = "<not valid xml";

    let result = WixDocument::parse_str(content);
    assert!(result.is_err());
}

#[test]
fn test_rule_loader_from_directory() {
    let wix_data = wix_data_path();

    if !wix_data.exists() {
        return; // Skip if wix-data not available
    }

    let loader = RuleLoader::new(&wix_data);
    let rules = loader.load_all().unwrap();

    // Should have loaded rules
    assert!(!rules.is_empty(), "Should load rules from wix-data");

    // Each rule should have required fields
    for rule in &rules {
        assert!(!rule.id.is_empty(), "Rule should have id");
        assert!(!rule.name.is_empty(), "Rule should have name");
        assert!(!rule.element.is_empty(), "Rule should have element");
        assert!(!rule.condition.is_empty(), "Rule should have condition");
    }
}

#[test]
fn test_inline_disable_all() {
    let content = r#"<?xml version="1.0"?>
<Wix xmlns="http://wixtoolset.org/schemas/v4/wxs">
  <!-- wix-lint-disable -->
  <Package Name="Test" Version="1.0.0">
  </Package>
</Wix>"#;

    let doc = WixDocument::parse_str(content).unwrap();

    // Should have a disable-all directive
    assert!(!doc.inline_disables.is_empty());

    // Find a disable that covers all rules
    let has_disable_all = doc.inline_disables.values().any(|d| d.rules.is_empty());
    assert!(has_disable_all, "Should have disable-all directive");
}

#[test]
fn test_inline_disable_specific_rules() {
    let content = r#"<?xml version="1.0"?>
<Wix xmlns="http://wixtoolset.org/schemas/v4/wxs">
  <!-- wix-lint-disable-next-line package-requires-upgradecode -->
  <Package Name="Test" Version="1.0.0">
  </Package>
</Wix>"#;

    let doc = WixDocument::parse_str(content).unwrap();

    // Should have a disable directive
    assert!(!doc.inline_disables.is_empty());

    // Find a disable with specific rule
    let has_specific_disable = doc.inline_disables.values().any(|d| {
        d.rules.contains("package-requires-upgradecode")
    });
    assert!(has_specific_disable, "Should have specific rule disable");
}

#[test]
fn test_wix_element_attributes() {
    let content = r#"<?xml version="1.0"?>
<Wix xmlns="http://wixtoolset.org/schemas/v4/wxs">
  <Package Name="TestApp" Version="1.0.0" Manufacturer="Acme" UpgradeCode="12345678-1234-1234-1234-123456789012">
  </Package>
</Wix>"#;

    let doc = WixDocument::parse_str(content).unwrap();

    let package = doc.elements.iter().find(|e| e.name == "Package").unwrap();

    assert_eq!(package.attributes.get("Name"), Some(&"TestApp".to_string()));
    assert_eq!(package.attributes.get("Version"), Some(&"1.0.0".to_string()));
    assert_eq!(package.attributes.get("Manufacturer"), Some(&"Acme".to_string()));
}

#[test]
fn test_source_line_retrieval() {
    let path = fixtures_path().join("valid.wxs");
    let doc = WixDocument::parse_file(&path).unwrap();

    // Get source line for Package element
    let package = doc.elements.iter().find(|e| e.name == "Package").unwrap();

    // Source line should be retrievable
    let lines: Vec<&str> = doc.source.lines().collect();
    if package.line > 0 && package.line <= lines.len() {
        let source_line = lines[package.line - 1];
        assert!(source_line.contains("Package"), "Source line should contain Package");
    }
}

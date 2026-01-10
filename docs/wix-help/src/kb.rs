//! Knowledge base loader for wix-help
//!
//! Loads elements, snippets, errors, and rules from wix-data JSON files.

use crate::types::{
    ElementDef, ErrorsFile, IceError, LintRule, RulesFile, Snippet, SnippetsFile, WixError,
};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

/// The knowledge base containing all help content
#[derive(Debug, Default)]
pub struct KnowledgeBase {
    /// Element definitions by name (case-insensitive key)
    pub elements: HashMap<String, ElementDef>,
    /// Snippets by name and prefix
    pub snippets: Vec<Snippet>,
    /// WiX errors by code
    pub wix_errors: HashMap<String, WixError>,
    /// ICE errors by code
    pub ice_errors: HashMap<String, IceError>,
    /// Lint rules by ID
    pub lint_rules: HashMap<String, LintRule>,
}

impl KnowledgeBase {
    /// Create an empty knowledge base
    pub fn new() -> Self {
        Self::default()
    }

    /// Load knowledge base from a wix-data directory
    pub fn load(wix_data_path: &Path) -> Result<Self, KbError> {
        let mut kb = Self::new();

        // Load elements
        let elements_dir = wix_data_path.join("elements");
        if elements_dir.exists() {
            kb.load_elements(&elements_dir)?;
        }

        // Load snippets
        let snippets_file = wix_data_path.join("snippets").join("snippets.json");
        if snippets_file.exists() {
            kb.load_snippets(&snippets_file)?;
        }

        // Load errors
        let errors_file = wix_data_path.join("errors").join("wix-errors.json");
        if errors_file.exists() {
            kb.load_errors(&errors_file)?;
        }

        // Load rules
        let rules_dir = wix_data_path.join("rules");
        if rules_dir.exists() {
            kb.load_rules(&rules_dir)?;
        }

        Ok(kb)
    }

    /// Load elements from a directory
    fn load_elements(&mut self, dir: &Path) -> Result<(), KbError> {
        for entry in WalkDir::new(dir).max_depth(1) {
            let entry = entry.map_err(|e| KbError::Io(e.to_string()))?;
            let path = entry.path();

            if path.is_file() && path.extension().is_some_and(|e| e == "json") {
                let content = fs::read_to_string(path).map_err(|e| KbError::Io(e.to_string()))?;
                match serde_json::from_str::<ElementDef>(&content) {
                    Ok(elem) => {
                        self.elements.insert(elem.name.to_lowercase(), elem);
                    }
                    Err(e) => {
                        // Skip invalid files but log the error
                        eprintln!("Warning: Failed to parse {}: {}", path.display(), e);
                    }
                }
            }
        }
        Ok(())
    }

    /// Load snippets from a file
    fn load_snippets(&mut self, path: &Path) -> Result<(), KbError> {
        let content = fs::read_to_string(path).map_err(|e| KbError::Io(e.to_string()))?;
        let file: SnippetsFile =
            serde_json::from_str(&content).map_err(|e| KbError::Parse(e.to_string()))?;
        self.snippets = file.snippets;
        Ok(())
    }

    /// Load errors from a file
    fn load_errors(&mut self, path: &Path) -> Result<(), KbError> {
        let content = fs::read_to_string(path).map_err(|e| KbError::Io(e.to_string()))?;
        let file: ErrorsFile =
            serde_json::from_str(&content).map_err(|e| KbError::Parse(e.to_string()))?;

        for err in file.errors {
            self.wix_errors.insert(err.code.to_uppercase(), err);
        }
        for err in file.ice_errors {
            self.ice_errors.insert(err.code.to_uppercase(), err);
        }
        Ok(())
    }

    /// Load rules from a directory
    fn load_rules(&mut self, dir: &Path) -> Result<(), KbError> {
        for entry in WalkDir::new(dir).max_depth(1) {
            let entry = entry.map_err(|e| KbError::Io(e.to_string()))?;
            let path = entry.path();

            if path.is_file() && path.extension().is_some_and(|e| e == "json") {
                // Skip schema files
                if path.file_name().is_some_and(|n| n.to_string_lossy().contains("schema")) {
                    continue;
                }

                let content = fs::read_to_string(path).map_err(|e| KbError::Io(e.to_string()))?;
                match serde_json::from_str::<RulesFile>(&content) {
                    Ok(file) => {
                        for rule in file.rules {
                            self.lint_rules.insert(rule.id.clone(), rule);
                        }
                    }
                    Err(e) => {
                        eprintln!("Warning: Failed to parse {}: {}", path.display(), e);
                    }
                }
            }
        }
        Ok(())
    }

    /// Get an element by name (case-insensitive)
    pub fn get_element(&self, name: &str) -> Option<&ElementDef> {
        self.elements.get(&name.to_lowercase())
    }

    /// Get a snippet by name or prefix
    pub fn get_snippet(&self, name_or_prefix: &str) -> Option<&Snippet> {
        let lower = name_or_prefix.to_lowercase();
        self.snippets.iter().find(|s| {
            s.name.to_lowercase() == lower || s.prefix.to_lowercase() == lower
        })
    }

    /// Get a WiX error by code
    pub fn get_wix_error(&self, code: &str) -> Option<&WixError> {
        self.wix_errors.get(&code.to_uppercase())
    }

    /// Get an ICE error by code
    pub fn get_ice_error(&self, code: &str) -> Option<&IceError> {
        self.ice_errors.get(&code.to_uppercase())
    }

    /// Get a lint rule by ID
    pub fn get_rule(&self, id: &str) -> Option<&LintRule> {
        // Try exact match first
        if let Some(rule) = self.lint_rules.get(id) {
            return Some(rule);
        }
        // Try case-insensitive match
        let lower = id.to_lowercase();
        self.lint_rules
            .values()
            .find(|r| r.id.to_lowercase() == lower)
    }

    /// Search for elements by partial name
    pub fn search_elements(&self, query: &str) -> Vec<&ElementDef> {
        let lower = query.to_lowercase();
        self.elements
            .values()
            .filter(|e| e.name.to_lowercase().contains(&lower))
            .collect()
    }

    /// Search for snippets by partial name or description
    pub fn search_snippets(&self, query: &str) -> Vec<&Snippet> {
        let lower = query.to_lowercase();
        self.snippets
            .iter()
            .filter(|s| {
                s.name.to_lowercase().contains(&lower)
                    || s.description.to_lowercase().contains(&lower)
                    || s.prefix.to_lowercase().contains(&lower)
            })
            .collect()
    }

    /// Search for errors by partial code or description
    pub fn search_errors(&self, query: &str) -> Vec<ErrorSearchResult<'_>> {
        let lower = query.to_lowercase();
        let mut results = Vec::new();

        for err in self.wix_errors.values() {
            if err.code.to_lowercase().contains(&lower)
                || err.description.to_lowercase().contains(&lower)
            {
                results.push(ErrorSearchResult::Wix(err));
            }
        }

        for err in self.ice_errors.values() {
            if err.code.to_lowercase().contains(&lower)
                || err.description.to_lowercase().contains(&lower)
            {
                results.push(ErrorSearchResult::Ice(err));
            }
        }

        results
    }

    /// Search for rules by partial ID or description
    pub fn search_rules(&self, query: &str) -> Vec<&LintRule> {
        let lower = query.to_lowercase();
        self.lint_rules
            .values()
            .filter(|r| {
                r.id.to_lowercase().contains(&lower)
                    || r.name.to_lowercase().contains(&lower)
                    || r.description.to_lowercase().contains(&lower)
            })
            .collect()
    }

    /// Get all element names sorted
    pub fn list_elements(&self) -> Vec<&str> {
        let mut names: Vec<_> = self.elements.values().map(|e| e.name.as_str()).collect();
        names.sort();
        names
    }

    /// Get all snippet names sorted
    pub fn list_snippets(&self) -> Vec<&str> {
        let mut names: Vec<_> = self.snippets.iter().map(|s| s.name.as_str()).collect();
        names.sort();
        names
    }

    /// Get all error codes sorted
    pub fn list_errors(&self) -> Vec<&str> {
        let mut codes: Vec<_> = self
            .wix_errors
            .values()
            .map(|e| e.code.as_str())
            .chain(self.ice_errors.values().map(|e| e.code.as_str()))
            .collect();
        codes.sort();
        codes
    }

    /// Get all rule IDs sorted
    pub fn list_rules(&self) -> Vec<&str> {
        let mut ids: Vec<_> = self.lint_rules.values().map(|r| r.id.as_str()).collect();
        ids.sort();
        ids
    }
}

/// Error search result (can be either WiX or ICE error)
#[derive(Debug)]
pub enum ErrorSearchResult<'a> {
    Wix(&'a WixError),
    Ice(&'a IceError),
}

/// Knowledge base errors
#[derive(Debug)]
pub enum KbError {
    Io(String),
    Parse(String),
}

impl std::fmt::Display for KbError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KbError::Io(e) => write!(f, "IO error: {}", e),
            KbError::Parse(e) => write!(f, "Parse error: {}", e),
        }
    }
}

impl std::error::Error for KbError {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    fn create_test_kb() -> (TempDir, KnowledgeBase) {
        let dir = TempDir::new().unwrap();

        // Create elements directory
        let elements_dir = dir.path().join("elements");
        fs::create_dir(&elements_dir).unwrap();

        // Create a test element
        let element_json = r#"{
            "name": "Component",
            "namespace": "wix",
            "since": "v3",
            "description": "A component is the smallest unit of installation.",
            "documentation": "https://wixtoolset.org/docs/schema/wxs/component/",
            "parents": ["Directory"],
            "children": ["File"],
            "attributes": {
                "Id": {
                    "type": "identifier",
                    "required": false,
                    "description": "Unique identifier"
                },
                "Guid": {
                    "type": "guid",
                    "required": true,
                    "description": "Component GUID"
                }
            },
            "examples": [
                {
                    "description": "Basic component",
                    "code": "<Component Guid=\"*\" />"
                }
            ]
        }"#;
        let mut f = fs::File::create(elements_dir.join("component.json")).unwrap();
        f.write_all(element_json.as_bytes()).unwrap();

        // Create snippets directory and file
        let snippets_dir = dir.path().join("snippets");
        fs::create_dir(&snippets_dir).unwrap();
        let snippets_json = r#"{
            "snippets": [
                {
                    "name": "component",
                    "prefix": "comp",
                    "description": "Create a component",
                    "body": ["<Component Guid=\"*\">", "</Component>"]
                }
            ]
        }"#;
        let mut f = fs::File::create(snippets_dir.join("snippets.json")).unwrap();
        f.write_all(snippets_json.as_bytes()).unwrap();

        // Create errors directory and file
        let errors_dir = dir.path().join("errors");
        fs::create_dir(&errors_dir).unwrap();
        let errors_json = r#"{
            "errors": [
                {
                    "code": "WIX0001",
                    "severity": "error",
                    "message": "Invalid parent element",
                    "description": "An element was placed inside an invalid parent.",
                    "resolution": "Check the allowed parent elements."
                }
            ],
            "iceErrors": [
                {
                    "code": "ICE03",
                    "severity": "error",
                    "description": "Basic validation",
                    "tables": ["_Validation"],
                    "resolution": "Ensure database tables conform to schema."
                }
            ]
        }"#;
        let mut f = fs::File::create(errors_dir.join("wix-errors.json")).unwrap();
        f.write_all(errors_json.as_bytes()).unwrap();

        // Create rules directory and file
        let rules_dir = dir.path().join("rules");
        fs::create_dir(&rules_dir).unwrap();
        let rules_json = r#"{
            "rules": [
                {
                    "id": "component-requires-guid",
                    "name": "Component requires GUID",
                    "description": "Every component must have a GUID",
                    "severity": "error",
                    "element": "Component",
                    "message": "Missing GUID"
                }
            ]
        }"#;
        let mut f = fs::File::create(rules_dir.join("component-rules.json")).unwrap();
        f.write_all(rules_json.as_bytes()).unwrap();

        let kb = KnowledgeBase::load(dir.path()).unwrap();
        (dir, kb)
    }

    #[test]
    fn test_load_elements() {
        let (_dir, kb) = create_test_kb();
        assert_eq!(kb.elements.len(), 1);
        let elem = kb.get_element("Component").unwrap();
        assert_eq!(elem.name, "Component");
    }

    #[test]
    fn test_get_element_case_insensitive() {
        let (_dir, kb) = create_test_kb();
        assert!(kb.get_element("component").is_some());
        assert!(kb.get_element("COMPONENT").is_some());
        assert!(kb.get_element("Component").is_some());
    }

    #[test]
    fn test_load_snippets() {
        let (_dir, kb) = create_test_kb();
        assert_eq!(kb.snippets.len(), 1);
        let snippet = kb.get_snippet("component").unwrap();
        assert_eq!(snippet.prefix, "comp");
    }

    #[test]
    fn test_get_snippet_by_prefix() {
        let (_dir, kb) = create_test_kb();
        let snippet = kb.get_snippet("comp").unwrap();
        assert_eq!(snippet.name, "component");
    }

    #[test]
    fn test_load_errors() {
        let (_dir, kb) = create_test_kb();
        assert_eq!(kb.wix_errors.len(), 1);
        assert_eq!(kb.ice_errors.len(), 1);
    }

    #[test]
    fn test_get_wix_error() {
        let (_dir, kb) = create_test_kb();
        let err = kb.get_wix_error("WIX0001").unwrap();
        assert_eq!(err.severity, "error");
    }

    #[test]
    fn test_get_wix_error_case_insensitive() {
        let (_dir, kb) = create_test_kb();
        assert!(kb.get_wix_error("wix0001").is_some());
    }

    #[test]
    fn test_get_ice_error() {
        let (_dir, kb) = create_test_kb();
        let err = kb.get_ice_error("ICE03").unwrap();
        assert_eq!(err.tables.len(), 1);
    }

    #[test]
    fn test_load_rules() {
        let (_dir, kb) = create_test_kb();
        assert_eq!(kb.lint_rules.len(), 1);
    }

    #[test]
    fn test_get_rule() {
        let (_dir, kb) = create_test_kb();
        let rule = kb.get_rule("component-requires-guid").unwrap();
        assert_eq!(rule.severity, "error");
    }

    #[test]
    fn test_search_elements() {
        let (_dir, kb) = create_test_kb();
        let results = kb.search_elements("comp");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_search_snippets() {
        let (_dir, kb) = create_test_kb();
        let results = kb.search_snippets("component");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_search_errors() {
        let (_dir, kb) = create_test_kb();
        let results = kb.search_errors("WIX");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_search_rules() {
        let (_dir, kb) = create_test_kb();
        let results = kb.search_rules("guid");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_list_elements() {
        let (_dir, kb) = create_test_kb();
        let names = kb.list_elements();
        assert!(names.contains(&"Component"));
    }

    #[test]
    fn test_list_snippets() {
        let (_dir, kb) = create_test_kb();
        let names = kb.list_snippets();
        assert!(names.contains(&"component"));
    }

    #[test]
    fn test_list_errors() {
        let (_dir, kb) = create_test_kb();
        let codes = kb.list_errors();
        assert!(codes.contains(&"WIX0001"));
        assert!(codes.contains(&"ICE03"));
    }

    #[test]
    fn test_list_rules() {
        let (_dir, kb) = create_test_kb();
        let ids = kb.list_rules();
        assert!(ids.contains(&"component-requires-guid"));
    }

    #[test]
    fn test_empty_kb() {
        let kb = KnowledgeBase::new();
        assert!(kb.elements.is_empty());
        assert!(kb.snippets.is_empty());
    }

    #[test]
    fn test_kb_error_display() {
        let io_err = KbError::Io("file not found".to_string());
        assert!(io_err.to_string().contains("IO error"));

        let parse_err = KbError::Parse("invalid json".to_string());
        assert!(parse_err.to_string().contains("Parse error"));
    }

    #[test]
    fn test_get_element_not_found() {
        let (_dir, kb) = create_test_kb();
        assert!(kb.get_element("NotAnElement").is_none());
    }

    #[test]
    fn test_get_snippet_not_found() {
        let (_dir, kb) = create_test_kb();
        assert!(kb.get_snippet("notasnippet").is_none());
    }

    #[test]
    fn test_get_error_not_found() {
        let (_dir, kb) = create_test_kb();
        assert!(kb.get_wix_error("WIX9999").is_none());
        assert!(kb.get_ice_error("ICE99").is_none());
    }

    #[test]
    fn test_get_rule_not_found() {
        let (_dir, kb) = create_test_kb();
        assert!(kb.get_rule("not-a-rule").is_none());
    }

    #[test]
    fn test_element_required_attributes() {
        let (_dir, kb) = create_test_kb();
        let elem = kb.get_element("Component").unwrap();
        let required = elem.required_attributes();
        assert_eq!(required.len(), 1);
        assert_eq!(required[0].0, "Guid");
    }

    #[test]
    fn test_element_optional_attributes() {
        let (_dir, kb) = create_test_kb();
        let elem = kb.get_element("Component").unwrap();
        let optional = elem.optional_attributes();
        assert_eq!(optional.len(), 1);
        assert_eq!(optional[0].0, "Id");
    }

    #[test]
    fn test_load_missing_directory() {
        let dir = TempDir::new().unwrap();
        let kb = KnowledgeBase::load(dir.path()).unwrap();
        assert!(kb.elements.is_empty());
    }

    #[test]
    fn test_error_search_result_debug() {
        let err = WixError {
            code: "WIX0001".to_string(),
            severity: "error".to_string(),
            message: "test".to_string(),
            description: "test".to_string(),
            resolution: "test".to_string(),
        };
        let result = ErrorSearchResult::Wix(&err);
        let debug = format!("{:?}", result);
        assert!(debug.contains("Wix"));
    }

    #[test]
    fn test_search_snippets_by_description() {
        let (_dir, kb) = create_test_kb();
        // Search by description
        let results = kb.search_snippets("Create");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_search_snippets_by_prefix() {
        let (_dir, kb) = create_test_kb();
        // Search by prefix
        let results = kb.search_snippets("comp");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_search_wix_errors_by_description() {
        let (_dir, kb) = create_test_kb();
        // Search by description
        let results = kb.search_errors("placed inside");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_search_ice_errors() {
        let (_dir, kb) = create_test_kb();
        // Search should find ICE error
        let results = kb.search_errors("ICE03");
        assert_eq!(results.len(), 1);
        // Verify it's an ICE error
        if let ErrorSearchResult::Ice(ice) = results[0] {
            assert_eq!(ice.code, "ICE03");
        } else {
            panic!("Expected ICE error");
        }
    }

    #[test]
    fn test_search_ice_errors_by_description() {
        let (_dir, kb) = create_test_kb();
        // Search by description
        let results = kb.search_errors("validation");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_load_invalid_element_file() {
        let dir = TempDir::new().unwrap();
        let elements_dir = dir.path().join("elements");
        fs::create_dir(&elements_dir).unwrap();

        // Create an invalid JSON file
        let mut f = fs::File::create(elements_dir.join("invalid.json")).unwrap();
        f.write_all(b"{ invalid json }").unwrap();

        // Load should succeed but skip invalid file
        let kb = KnowledgeBase::load(dir.path()).unwrap();
        assert!(kb.elements.is_empty());
    }

    #[test]
    fn test_load_invalid_rules_file() {
        let dir = TempDir::new().unwrap();
        let rules_dir = dir.path().join("rules");
        fs::create_dir(&rules_dir).unwrap();

        // Create an invalid JSON file
        let mut f = fs::File::create(rules_dir.join("invalid.json")).unwrap();
        f.write_all(b"{ invalid json }").unwrap();

        // Load should succeed but skip invalid file
        let kb = KnowledgeBase::load(dir.path()).unwrap();
        assert!(kb.lint_rules.is_empty());
    }
}

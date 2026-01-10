//! wix-help - PowerShell-like help system for WiX
//!
//! Provides help for WiX elements, attributes, errors, snippets, and lint rules.
//!
//! # Usage
//!
//! ```no_run
//! use wix_help::{HelpSystem, OutputFormat};
//! use std::path::Path;
//!
//! let help = HelpSystem::load(Path::new("path/to/wix-data")).unwrap();
//!
//! // Get help for an element
//! if let Some(output) = help.get_element_help("Component", OutputFormat::Text, true) {
//!     println!("{}", output);
//! }
//!
//! // Get help for an error
//! if let Some(output) = help.get_error_help("WIX0001", OutputFormat::Text) {
//!     println!("{}", output);
//! }
//! ```

pub mod formatter;
pub mod kb;
pub mod types;

pub use formatter::*;
pub use kb::{ErrorSearchResult, KbError, KnowledgeBase};
pub use types::*;

use std::path::Path;

/// The help system
pub struct HelpSystem {
    kb: KnowledgeBase,
}

impl HelpSystem {
    /// Load the help system from a wix-data directory
    pub fn load(wix_data_path: &Path) -> Result<Self, KbError> {
        let kb = KnowledgeBase::load(wix_data_path)?;
        Ok(Self { kb })
    }

    /// Create a help system with an existing knowledge base
    pub fn with_kb(kb: KnowledgeBase) -> Self {
        Self { kb }
    }

    /// Get the underlying knowledge base
    pub fn kb(&self) -> &KnowledgeBase {
        &self.kb
    }

    /// Get help for an element
    pub fn get_element_help(
        &self,
        name: &str,
        format: OutputFormat,
        show_examples: bool,
    ) -> Option<String> {
        self.kb
            .get_element(name)
            .map(|elem| format_element(elem, format, show_examples))
    }

    /// Get help for a snippet
    pub fn get_snippet_help(&self, name_or_prefix: &str, format: OutputFormat) -> Option<String> {
        self.kb
            .get_snippet(name_or_prefix)
            .map(|snippet| format_snippet(snippet, format))
    }

    /// Get help for an error (WiX or ICE)
    pub fn get_error_help(&self, code: &str, format: OutputFormat) -> Option<String> {
        if let Some(err) = self.kb.get_wix_error(code) {
            return Some(format_wix_error(err, format));
        }
        if let Some(err) = self.kb.get_ice_error(code) {
            return Some(format_ice_error(err, format));
        }
        None
    }

    /// Get help for a lint rule
    pub fn get_rule_help(&self, id: &str, format: OutputFormat) -> Option<String> {
        self.kb.get_rule(id).map(|rule| format_rule(rule, format))
    }

    /// Search for elements matching a query
    pub fn search_elements(&self, query: &str, format: OutputFormat) -> String {
        let results = self.kb.search_elements(query);
        let names: Vec<_> = results.iter().map(|e| e.name.as_str()).collect();
        format_list(
            &format!("Elements matching '{}'", query),
            &names,
            format,
            "element",
        )
    }

    /// Search for snippets matching a query
    pub fn search_snippets(&self, query: &str, format: OutputFormat) -> String {
        let results = self.kb.search_snippets(query);
        let names: Vec<_> = results.iter().map(|s| s.name.as_str()).collect();
        format_list(
            &format!("Snippets matching '{}'", query),
            &names,
            format,
            "snippet",
        )
    }

    /// Search for errors matching a query
    pub fn search_errors(&self, query: &str, format: OutputFormat) -> String {
        let results = self.kb.search_errors(query);
        let codes: Vec<_> = results
            .iter()
            .map(|r| match r {
                ErrorSearchResult::Wix(e) => e.code.as_str(),
                ErrorSearchResult::Ice(e) => e.code.as_str(),
            })
            .collect();
        format_list(
            &format!("Errors matching '{}'", query),
            &codes,
            format,
            "error",
        )
    }

    /// Search for rules matching a query
    pub fn search_rules(&self, query: &str, format: OutputFormat) -> String {
        let results = self.kb.search_rules(query);
        let ids: Vec<_> = results.iter().map(|r| r.id.as_str()).collect();
        format_list(
            &format!("Rules matching '{}'", query),
            &ids,
            format,
            "rule",
        )
    }

    /// List all elements
    pub fn list_elements(&self, format: OutputFormat) -> String {
        let names = self.kb.list_elements();
        format_list("Available Elements", &names, format, "element")
    }

    /// List all snippets
    pub fn list_snippets(&self, format: OutputFormat) -> String {
        let names = self.kb.list_snippets();
        format_list("Available Snippets", &names, format, "snippet")
    }

    /// List all errors
    pub fn list_errors(&self, format: OutputFormat) -> String {
        let codes = self.kb.list_errors();
        format_list("Error Codes", &codes, format, "error")
    }

    /// List all rules
    pub fn list_rules(&self, format: OutputFormat) -> String {
        let ids = self.kb.list_rules();
        format_list("Lint Rules", &ids, format, "rule")
    }

    /// Try to get help for a topic (auto-detect type)
    pub fn get_help(
        &self,
        topic: &str,
        format: OutputFormat,
        show_examples: bool,
    ) -> Option<String> {
        // Try element first (most common)
        if let Some(output) = self.get_element_help(topic, format, show_examples) {
            return Some(output);
        }

        // Try error codes
        if let Some(output) = self.get_error_help(topic, format) {
            return Some(output);
        }

        // Try snippet
        if let Some(output) = self.get_snippet_help(topic, format) {
            return Some(output);
        }

        // Try rule
        if let Some(output) = self.get_rule_help(topic, format) {
            return Some(output);
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use tempfile::TempDir;

    fn create_test_wix_data() -> TempDir {
        let dir = TempDir::new().unwrap();

        // Create elements directory
        let elements_dir = dir.path().join("elements");
        fs::create_dir(&elements_dir).unwrap();

        let element_json = r#"{
            "name": "Component",
            "namespace": "wix",
            "since": "v3",
            "description": "A component is the smallest unit of installation.",
            "documentation": "https://wixtoolset.org/docs/",
            "parents": ["Directory"],
            "children": ["File"],
            "attributes": {
                "Id": {"type": "identifier", "required": false, "description": "Identifier"},
                "Guid": {"type": "guid", "required": true, "description": "GUID"}
            },
            "examples": [{"description": "Basic", "code": "<Component />"}]
        }"#;
        let mut f = fs::File::create(elements_dir.join("component.json")).unwrap();
        f.write_all(element_json.as_bytes()).unwrap();

        // Create snippets
        let snippets_dir = dir.path().join("snippets");
        fs::create_dir(&snippets_dir).unwrap();
        let snippets_json = r#"{"snippets": [{"name": "component", "prefix": "comp", "description": "Create component", "body": ["<Component />"]}]}"#;
        let mut f = fs::File::create(snippets_dir.join("snippets.json")).unwrap();
        f.write_all(snippets_json.as_bytes()).unwrap();

        // Create errors
        let errors_dir = dir.path().join("errors");
        fs::create_dir(&errors_dir).unwrap();
        let errors_json = r#"{"errors": [{"code": "WIX0001", "severity": "error", "message": "Error", "description": "Desc", "resolution": "Fix"}], "iceErrors": [{"code": "ICE03", "severity": "error", "description": "ICE", "tables": [], "resolution": "Fix"}]}"#;
        let mut f = fs::File::create(errors_dir.join("wix-errors.json")).unwrap();
        f.write_all(errors_json.as_bytes()).unwrap();

        // Create rules
        let rules_dir = dir.path().join("rules");
        fs::create_dir(&rules_dir).unwrap();
        let rules_json = r#"{"rules": [{"id": "component-requires-guid", "name": "GUID required", "description": "Desc", "severity": "error", "element": "Component", "message": "Msg"}]}"#;
        let mut f = fs::File::create(rules_dir.join("component-rules.json")).unwrap();
        f.write_all(rules_json.as_bytes()).unwrap();

        dir
    }

    #[test]
    fn test_help_system_load() {
        let dir = create_test_wix_data();
        let help = HelpSystem::load(dir.path()).unwrap();
        assert!(!help.kb().elements.is_empty());
    }

    #[test]
    fn test_get_element_help() {
        let dir = create_test_wix_data();
        let help = HelpSystem::load(dir.path()).unwrap();
        let output = help.get_element_help("Component", OutputFormat::Text, true);
        assert!(output.is_some());
        assert!(output.unwrap().contains("Component"));
    }

    #[test]
    fn test_get_element_help_not_found() {
        let dir = create_test_wix_data();
        let help = HelpSystem::load(dir.path()).unwrap();
        let output = help.get_element_help("NotAnElement", OutputFormat::Text, true);
        assert!(output.is_none());
    }

    #[test]
    fn test_get_snippet_help() {
        let dir = create_test_wix_data();
        let help = HelpSystem::load(dir.path()).unwrap();
        let output = help.get_snippet_help("comp", OutputFormat::Text);
        assert!(output.is_some());
    }

    #[test]
    fn test_get_error_help_wix() {
        let dir = create_test_wix_data();
        let help = HelpSystem::load(dir.path()).unwrap();
        let output = help.get_error_help("WIX0001", OutputFormat::Text);
        assert!(output.is_some());
    }

    #[test]
    fn test_get_error_help_ice() {
        let dir = create_test_wix_data();
        let help = HelpSystem::load(dir.path()).unwrap();
        let output = help.get_error_help("ICE03", OutputFormat::Text);
        assert!(output.is_some());
    }

    #[test]
    fn test_get_rule_help() {
        let dir = create_test_wix_data();
        let help = HelpSystem::load(dir.path()).unwrap();
        let output = help.get_rule_help("component-requires-guid", OutputFormat::Text);
        assert!(output.is_some());
    }

    #[test]
    fn test_search_elements() {
        let dir = create_test_wix_data();
        let help = HelpSystem::load(dir.path()).unwrap();
        let output = help.search_elements("Comp", OutputFormat::Text);
        assert!(output.contains("Component"));
    }

    #[test]
    fn test_search_snippets() {
        let dir = create_test_wix_data();
        let help = HelpSystem::load(dir.path()).unwrap();
        let output = help.search_snippets("comp", OutputFormat::Text);
        assert!(output.contains("component"));
    }

    #[test]
    fn test_search_errors() {
        let dir = create_test_wix_data();
        let help = HelpSystem::load(dir.path()).unwrap();
        let output = help.search_errors("WIX", OutputFormat::Text);
        assert!(output.contains("WIX0001"));
    }

    #[test]
    fn test_search_rules() {
        let dir = create_test_wix_data();
        let help = HelpSystem::load(dir.path()).unwrap();
        let output = help.search_rules("guid", OutputFormat::Text);
        assert!(output.contains("component-requires-guid"));
    }

    #[test]
    fn test_list_elements() {
        let dir = create_test_wix_data();
        let help = HelpSystem::load(dir.path()).unwrap();
        let output = help.list_elements(OutputFormat::Text);
        assert!(output.contains("Component"));
    }

    #[test]
    fn test_list_snippets() {
        let dir = create_test_wix_data();
        let help = HelpSystem::load(dir.path()).unwrap();
        let output = help.list_snippets(OutputFormat::Text);
        assert!(output.contains("component"));
    }

    #[test]
    fn test_list_errors() {
        let dir = create_test_wix_data();
        let help = HelpSystem::load(dir.path()).unwrap();
        let output = help.list_errors(OutputFormat::Text);
        assert!(output.contains("WIX0001"));
    }

    #[test]
    fn test_list_rules() {
        let dir = create_test_wix_data();
        let help = HelpSystem::load(dir.path()).unwrap();
        let output = help.list_rules(OutputFormat::Text);
        assert!(output.contains("component-requires-guid"));
    }

    #[test]
    fn test_get_help_auto_detect_element() {
        let dir = create_test_wix_data();
        let help = HelpSystem::load(dir.path()).unwrap();
        let output = help.get_help("Component", OutputFormat::Text, false);
        assert!(output.is_some());
    }

    #[test]
    fn test_get_help_auto_detect_error() {
        let dir = create_test_wix_data();
        let help = HelpSystem::load(dir.path()).unwrap();
        let output = help.get_help("WIX0001", OutputFormat::Text, false);
        assert!(output.is_some());
    }

    #[test]
    fn test_get_help_auto_detect_snippet() {
        let dir = create_test_wix_data();
        let help = HelpSystem::load(dir.path()).unwrap();
        let output = help.get_help("comp", OutputFormat::Text, false);
        assert!(output.is_some());
    }

    #[test]
    fn test_get_help_auto_detect_rule() {
        let dir = create_test_wix_data();
        let help = HelpSystem::load(dir.path()).unwrap();
        let output = help.get_help("component-requires-guid", OutputFormat::Text, false);
        assert!(output.is_some());
    }

    #[test]
    fn test_get_help_not_found() {
        let dir = create_test_wix_data();
        let help = HelpSystem::load(dir.path()).unwrap();
        let output = help.get_help("NotATopic", OutputFormat::Text, false);
        assert!(output.is_none());
    }

    #[test]
    fn test_with_kb() {
        let kb = KnowledgeBase::new();
        let help = HelpSystem::with_kb(kb);
        assert!(help.kb().elements.is_empty());
    }

    #[test]
    fn test_output_formats() {
        let dir = create_test_wix_data();
        let help = HelpSystem::load(dir.path()).unwrap();

        // Test all formats
        let text = help.get_element_help("Component", OutputFormat::Text, false);
        assert!(text.is_some());

        let json = help.get_element_help("Component", OutputFormat::Json, false);
        assert!(json.is_some());
        // Verify it's valid JSON
        let _: serde_json::Value = serde_json::from_str(&json.unwrap()).unwrap();

        let md = help.get_element_help("Component", OutputFormat::Markdown, false);
        assert!(md.is_some());
        assert!(md.unwrap().starts_with("# "));
    }

    #[test]
    fn test_search_json_format() {
        let dir = create_test_wix_data();
        let help = HelpSystem::load(dir.path()).unwrap();
        let output = help.search_elements("Comp", OutputFormat::Json);
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert!(parsed["items"].is_array());
    }

    #[test]
    fn test_list_markdown_format() {
        let dir = create_test_wix_data();
        let help = HelpSystem::load(dir.path()).unwrap();
        let output = help.list_elements(OutputFormat::Markdown);
        assert!(output.contains("# Available Elements"));
        assert!(output.contains("- `Component`"));
    }

    #[test]
    fn test_search_errors_includes_ice() {
        let dir = create_test_wix_data();
        let help = HelpSystem::load(dir.path()).unwrap();
        // Search for ICE errors
        let output = help.search_errors("ICE", OutputFormat::Text);
        assert!(output.contains("ICE03"));
    }
}

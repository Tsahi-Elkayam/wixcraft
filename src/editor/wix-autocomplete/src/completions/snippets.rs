//! Snippet completions

use crate::loader::WixData;
use crate::types::{CompletionItem, CompletionKind};

/// Complete snippets matching prefix
pub fn complete_snippets(data: &WixData, prefix: &str) -> Vec<CompletionItem> {
    data.get_snippets_by_prefix(prefix)
        .into_iter()
        .map(|snippet| {
            CompletionItem::new(&snippet.prefix, CompletionKind::Snippet)
                .with_detail(&snippet.name)
                .with_documentation(&snippet.description)
                .with_insert_text(snippet.body_text())
                .with_priority(5) // Snippets have high priority when prefix matches
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_data() -> (TempDir, WixData) {
        let temp = TempDir::new().unwrap();

        let elements_dir = temp.path().join("elements");
        fs::create_dir(&elements_dir).unwrap();

        let keywords_dir = temp.path().join("keywords");
        fs::create_dir(&keywords_dir).unwrap();
        fs::write(
            keywords_dir.join("keywords.json"),
            r#"{"standardDirectories":[],"builtinProperties":[],"elements":[],"preprocessorDirectives":[]}"#,
        )
        .unwrap();

        let snippets_dir = temp.path().join("snippets");
        fs::create_dir(&snippets_dir).unwrap();
        let snippets = r#"{
            "snippets": [
                {
                    "name": "Component with File",
                    "prefix": "comp",
                    "description": "Creates a component with file",
                    "body": ["<Component Guid=\"*\">", "  <File Source=\"${1:file}\" />", "</Component>"]
                },
                {
                    "name": "Directory",
                    "prefix": "dir",
                    "description": "Creates a directory",
                    "body": ["<Directory Id=\"${1:Id}\" Name=\"${2:Name}\">", "</Directory>"]
                },
                {
                    "name": "Feature",
                    "prefix": "feat",
                    "description": "Creates a feature",
                    "body": ["<Feature Id=\"${1:Id}\" Title=\"${2:Title}\">", "</Feature>"]
                }
            ]
        }"#;
        fs::write(snippets_dir.join("snippets.json"), snippets).unwrap();

        let data = WixData::load(temp.path()).unwrap();
        (temp, data)
    }

    #[test]
    fn test_complete_snippets_exact_prefix() {
        let (_temp, data) = create_test_data();
        let completions = complete_snippets(&data, "comp");

        assert_eq!(completions.len(), 1);
        assert_eq!(completions[0].label, "comp");
        assert_eq!(completions[0].detail, Some("Component with File".to_string()));
    }

    #[test]
    fn test_complete_snippets_partial_prefix() {
        let (_temp, data) = create_test_data();
        let completions = complete_snippets(&data, "co");

        assert_eq!(completions.len(), 1);
        assert_eq!(completions[0].label, "comp");
    }

    #[test]
    fn test_complete_snippets_no_match() {
        let (_temp, data) = create_test_data();
        let completions = complete_snippets(&data, "xyz");

        assert!(completions.is_empty());
    }

    #[test]
    fn test_snippet_body_text() {
        let (_temp, data) = create_test_data();
        let completions = complete_snippets(&data, "comp");

        let body = &completions[0].insert_text;
        assert!(body.contains("<Component"));
        assert!(body.contains("<File"));
        assert!(body.contains("</Component>"));
    }

    #[test]
    fn test_snippet_priority() {
        let (_temp, data) = create_test_data();
        let completions = complete_snippets(&data, "dir");

        assert_eq!(completions[0].sort_priority, 5);
    }
}

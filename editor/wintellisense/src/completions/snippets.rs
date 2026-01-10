//! Snippet completions

use crate::loader::SchemaData;
use crate::types::{CompletionItem, CompletionKind};

/// Complete snippets matching prefix
pub fn complete_snippets(schema: &SchemaData, prefix: &str) -> Vec<CompletionItem> {
    schema
        .get_snippets_by_prefix(prefix)
        .into_iter()
        .map(|snippet| {
            CompletionItem::new(&snippet.prefix, CompletionKind::Snippet)
                .with_insert_text(snippet.body_text())
                .with_detail(&snippet.name)
                .with_documentation(&snippet.description)
                .with_priority(5)
                .as_snippet()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Snippet;

    #[test]
    fn test_complete_snippets() {
        let mut schema = SchemaData::default();
        schema.snippets = vec![
            Snippet {
                name: "component".to_string(),
                prefix: "comp".to_string(),
                description: "Create component".to_string(),
                body: vec!["<Component>".to_string(), "</Component>".to_string()],
            },
            Snippet {
                name: "directory".to_string(),
                prefix: "dir".to_string(),
                description: "Create directory".to_string(),
                body: vec!["<Directory>".to_string()],
            },
        ];

        let items = complete_snippets(&schema, "co");
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].label, "comp");
    }

    #[test]
    fn test_snippet_body() {
        let mut schema = SchemaData::default();
        schema.snippets = vec![Snippet {
            name: "test".to_string(),
            prefix: "test".to_string(),
            description: "Test".to_string(),
            body: vec!["line1".to_string(), "line2".to_string()],
        }];

        let items = complete_snippets(&schema, "test");
        assert_eq!(items.len(), 1);
        assert!(items[0].insert_text.contains("line1"));
        assert!(items[0].insert_text.contains("line2"));
    }
}

//! Word completions from project index (All Autocomplete style)

use crate::index::ProjectIndex;
use crate::types::{CompletionItem, CompletionKind};

/// Complete words from the project index
pub fn complete_words(index: &ProjectIndex, prefix: &str, limit: usize) -> Vec<CompletionItem> {
    index
        .get_words_matching(prefix, limit)
        .into_iter()
        .map(|word| {
            CompletionItem::new(word, CompletionKind::Word)
                .with_detail("Project word")
                .with_priority(100) // Lower priority than schema items
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_complete_words_empty() {
        let index = ProjectIndex::new();
        let items = complete_words(&index, "Test", 10);
        assert!(items.is_empty());
    }

    #[test]
    fn test_word_completion_item() {
        let item = CompletionItem::new("TestWord", CompletionKind::Word)
            .with_detail("Project word")
            .with_priority(100);

        assert_eq!(item.label, "TestWord");
        assert_eq!(item.kind, CompletionKind::Word);
        assert_eq!(item.sort_priority, 100);
    }
}

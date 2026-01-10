//! Completion providers

mod attributes;
mod elements;
mod snippets;
mod values;
mod words;

use crate::index::ProjectIndex;
use crate::loader::SchemaData;
use crate::types::{CompletionResult, CursorContext};

/// Get completions for the given context
pub fn get_completions(
    schema: &SchemaData,
    index: &ProjectIndex,
    ctx: &CursorContext,
    _source: &str,
    max: usize,
) -> CompletionResult {
    let mut items = Vec::new();

    if ctx.should_suggest_values() {
        // Attribute value completions
        items.extend(values::complete_values(schema, index, ctx));
    } else if ctx.should_suggest_attributes() {
        // Attribute name completions
        items.extend(attributes::complete_attributes(schema, ctx));
    } else if ctx.should_suggest_elements() {
        // Element completions
        items.extend(elements::complete_elements(schema, ctx));

        // Snippet completions
        items.extend(snippets::complete_snippets(schema, &ctx.prefix));

        // Word completions (from project)
        if !ctx.prefix.is_empty() {
            items.extend(words::complete_words(index, &ctx.prefix, 10));
        }
    }

    // Sort by priority then label
    items.sort_by(|a, b| {
        a.sort_priority
            .cmp(&b.sort_priority)
            .then(a.label.cmp(&b.label))
    });

    // Limit results
    items.truncate(max);

    CompletionResult::new(items)
}

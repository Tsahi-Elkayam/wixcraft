//! Output formatters for wix-help

use crate::types::{AttributeDef, ElementDef, IceError, LintRule, OutputFormat, Snippet, WixError};
use colored::*;

/// Format element help
pub fn format_element(elem: &ElementDef, format: OutputFormat, show_examples: bool) -> String {
    match format {
        OutputFormat::Text => format_element_text(elem, show_examples),
        OutputFormat::Json => format_element_json(elem),
        OutputFormat::Markdown => format_element_markdown(elem, show_examples),
    }
}

fn format_element_text(elem: &ElementDef, show_examples: bool) -> String {
    let mut output = String::new();

    // Header
    output.push_str(&format!("{}\n", elem.name.bold().cyan()));
    output.push_str(&"─".repeat(60));
    output.push('\n');

    // Description
    if !elem.description.is_empty() {
        output.push_str(&format!("\n{}\n", elem.description));
    }

    // Since / Namespace
    if !elem.since.is_empty() || !elem.namespace.is_empty() {
        output.push('\n');
        if !elem.since.is_empty() {
            output.push_str(&format!("{}  {}\n", "Since:".yellow(), elem.since));
        }
        if !elem.namespace.is_empty() {
            output.push_str(&format!("{} {}\n", "Namespace:".yellow(), elem.namespace));
        }
    }

    // Documentation link
    if !elem.documentation.is_empty() {
        output.push_str(&format!("\n{} {}\n", "Documentation:".yellow(), elem.documentation));
    }

    // Parents
    if !elem.parents.is_empty() {
        output.push_str(&format!("\n{}\n", "PARENTS".bold()));
        for parent in &elem.parents {
            output.push_str(&format!("  {}\n", parent));
        }
    }

    // Children
    if !elem.children.is_empty() {
        output.push_str(&format!("\n{}\n", "CHILDREN".bold()));
        for child in &elem.children {
            output.push_str(&format!("  {}\n", child));
        }
    }

    // Attributes
    if !elem.attributes.is_empty() {
        output.push_str(&format!("\n{}\n", "ATTRIBUTES".bold()));

        // Required attributes first
        let required = elem.required_attributes();
        if !required.is_empty() {
            output.push_str(&format!("\n  {}\n", "Required:".green()));
            for (name, attr) in required {
                output.push_str(&format_attribute_text(name, attr, 4));
            }
        }

        // Optional attributes
        let optional = elem.optional_attributes();
        if !optional.is_empty() {
            output.push_str(&format!("\n  {}\n", "Optional:".blue()));
            for (name, attr) in optional {
                output.push_str(&format_attribute_text(name, attr, 4));
            }
        }
    }

    // MSI Tables
    if !elem.msi_tables.is_empty() {
        output.push_str(&format!("\n{}\n", "MSI TABLES".bold()));
        output.push_str(&format!("  {}\n", elem.msi_tables.join(", ")));
    }

    // Related rules
    if !elem.rules.is_empty() {
        output.push_str(&format!("\n{}\n", "RELATED RULES".bold()));
        for rule in &elem.rules {
            output.push_str(&format!("  {}\n", rule));
        }
    }

    // Examples
    if show_examples && !elem.examples.is_empty() {
        output.push_str(&format!("\n{}\n", "EXAMPLES".bold()));
        for (i, example) in elem.examples.iter().enumerate() {
            if !example.description.is_empty() {
                output.push_str(&format!("\n  {}. {}\n", i + 1, example.description));
            } else {
                output.push_str(&format!("\n  {}.\n", i + 1));
            }
            // Format code with indentation
            for line in example.code.lines() {
                output.push_str(&format!("     {}\n", line.dimmed()));
            }
        }
    }

    output
}

fn format_attribute_text(name: &str, attr: &AttributeDef, indent: usize) -> String {
    let mut output = String::new();
    let spaces = " ".repeat(indent);

    output.push_str(&format!("{}{}\n", spaces, name.cyan()));

    // Type and default
    let mut type_info = format!("{}  Type: {}", spaces, attr.attr_type);
    if let Some(ref default) = attr.default {
        type_info.push_str(&format!(" (default: {})", default));
    }
    output.push_str(&format!("{}\n", type_info.dimmed()));

    // Description
    if !attr.description.is_empty() {
        output.push_str(&format!("{}  {}\n", spaces, attr.description));
    }

    // Enum values
    if !attr.values.is_empty() {
        output.push_str(&format!(
            "{}  Values: {}\n",
            spaces,
            attr.values.join(", ").dimmed()
        ));
    }

    output
}

fn format_element_json(elem: &ElementDef) -> String {
    serde_json::to_string_pretty(elem).unwrap_or_else(|_| "{}".to_string())
}

fn format_element_markdown(elem: &ElementDef, show_examples: bool) -> String {
    let mut output = String::new();

    // Header
    output.push_str(&format!("# {}\n\n", elem.name));

    // Description
    if !elem.description.is_empty() {
        output.push_str(&format!("{}\n\n", elem.description));
    }

    // Metadata
    if !elem.since.is_empty() || !elem.namespace.is_empty() {
        if !elem.since.is_empty() {
            output.push_str(&format!("**Since:** {}\n", elem.since));
        }
        if !elem.namespace.is_empty() {
            output.push_str(&format!("**Namespace:** {}\n", elem.namespace));
        }
        output.push('\n');
    }

    // Documentation link
    if !elem.documentation.is_empty() {
        output.push_str(&format!(
            "[Official Documentation]({})\n\n",
            elem.documentation
        ));
    }

    // Parents
    if !elem.parents.is_empty() {
        output.push_str("## Parents\n\n");
        for parent in &elem.parents {
            output.push_str(&format!("- `{}`\n", parent));
        }
        output.push('\n');
    }

    // Children
    if !elem.children.is_empty() {
        output.push_str("## Children\n\n");
        for child in &elem.children {
            output.push_str(&format!("- `{}`\n", child));
        }
        output.push('\n');
    }

    // Attributes
    if !elem.attributes.is_empty() {
        output.push_str("## Attributes\n\n");
        output.push_str("| Name | Type | Required | Default | Description |\n");
        output.push_str("|------|------|----------|---------|-------------|\n");

        // Sort attributes: required first, then by name
        let mut attrs: Vec<_> = elem.attributes.iter().collect();
        attrs.sort_by(|a, b| {
            let a_req = a.1.required.unwrap_or(false);
            let b_req = b.1.required.unwrap_or(false);
            b_req.cmp(&a_req).then_with(|| a.0.cmp(b.0))
        });

        for (name, attr) in attrs {
            let required = if attr.required.unwrap_or(false) {
                "Yes"
            } else {
                "No"
            };
            let default = attr.default.as_deref().unwrap_or("-");
            let desc = attr.description.replace('|', "\\|");
            output.push_str(&format!(
                "| `{}` | {} | {} | {} | {} |\n",
                name, attr.attr_type, required, default, desc
            ));
        }
        output.push('\n');
    }

    // Examples
    if show_examples && !elem.examples.is_empty() {
        output.push_str("## Examples\n\n");
        for (i, example) in elem.examples.iter().enumerate() {
            if !example.description.is_empty() {
                output.push_str(&format!("### {}. {}\n\n", i + 1, example.description));
            } else {
                output.push_str(&format!("### Example {}\n\n", i + 1));
            }
            output.push_str("```xml\n");
            output.push_str(&example.code);
            output.push_str("\n```\n\n");
        }
    }

    output
}

/// Format snippet help
pub fn format_snippet(snippet: &Snippet, format: OutputFormat) -> String {
    match format {
        OutputFormat::Text => format_snippet_text(snippet),
        OutputFormat::Json => format_snippet_json(snippet),
        OutputFormat::Markdown => format_snippet_markdown(snippet),
    }
}

fn format_snippet_text(snippet: &Snippet) -> String {
    let mut output = String::new();

    output.push_str(&format!("{}\n", snippet.name.bold().cyan()));
    output.push_str(&"─".repeat(60));
    output.push('\n');

    output.push_str(&format!("\n{} {}\n", "Prefix:".yellow(), snippet.prefix));

    if !snippet.description.is_empty() {
        output.push_str(&format!("\n{}\n", snippet.description));
    }

    output.push_str(&format!("\n{}\n", "CODE".bold()));
    for line in &snippet.body {
        output.push_str(&format!("  {}\n", line.dimmed()));
    }

    output
}

fn format_snippet_json(snippet: &Snippet) -> String {
    serde_json::to_string_pretty(snippet).unwrap_or_else(|_| "{}".to_string())
}

fn format_snippet_markdown(snippet: &Snippet) -> String {
    let mut output = String::new();

    output.push_str(&format!("# {}\n\n", snippet.name));

    output.push_str(&format!("**Prefix:** `{}`\n\n", snippet.prefix));

    if !snippet.description.is_empty() {
        output.push_str(&format!("{}\n\n", snippet.description));
    }

    output.push_str("## Code\n\n```xml\n");
    output.push_str(&snippet.body.join("\n"));
    output.push_str("\n```\n");

    output
}

/// Format WiX error help
pub fn format_wix_error(err: &WixError, format: OutputFormat) -> String {
    match format {
        OutputFormat::Text => format_wix_error_text(err),
        OutputFormat::Json => format_wix_error_json(err),
        OutputFormat::Markdown => format_wix_error_markdown(err),
    }
}

fn format_wix_error_text(err: &WixError) -> String {
    let mut output = String::new();

    let severity_color = match err.severity.as_str() {
        "error" => err.code.red().bold(),
        "warning" => err.code.yellow().bold(),
        _ => err.code.blue().bold(),
    };

    output.push_str(&format!("{}\n", severity_color));
    output.push_str(&"─".repeat(60));
    output.push('\n');

    output.push_str(&format!("\n{} {}\n", "Severity:".yellow(), err.severity));

    if !err.message.is_empty() {
        output.push_str(&format!("\n{}\n{}\n", "MESSAGE".bold(), err.message));
    }

    if !err.description.is_empty() {
        output.push_str(&format!("\n{}\n{}\n", "DESCRIPTION".bold(), err.description));
    }

    if !err.resolution.is_empty() {
        output.push_str(&format!("\n{}\n{}\n", "RESOLUTION".bold().green(), err.resolution));
    }

    output
}

fn format_wix_error_json(err: &WixError) -> String {
    serde_json::to_string_pretty(err).unwrap_or_else(|_| "{}".to_string())
}

fn format_wix_error_markdown(err: &WixError) -> String {
    let mut output = String::new();

    output.push_str(&format!("# {}\n\n", err.code));
    output.push_str(&format!("**Severity:** {}\n\n", err.severity));

    if !err.message.is_empty() {
        output.push_str(&format!("## Message\n\n{}\n\n", err.message));
    }

    if !err.description.is_empty() {
        output.push_str(&format!("## Description\n\n{}\n\n", err.description));
    }

    if !err.resolution.is_empty() {
        output.push_str(&format!("## Resolution\n\n{}\n\n", err.resolution));
    }

    output
}

/// Format ICE error help
pub fn format_ice_error(err: &IceError, format: OutputFormat) -> String {
    match format {
        OutputFormat::Text => format_ice_error_text(err),
        OutputFormat::Json => format_ice_error_json(err),
        OutputFormat::Markdown => format_ice_error_markdown(err),
    }
}

fn format_ice_error_text(err: &IceError) -> String {
    let mut output = String::new();

    let severity_color = match err.severity.as_str() {
        "error" => err.code.red().bold(),
        "warning" => err.code.yellow().bold(),
        _ => err.code.blue().bold(),
    };

    output.push_str(&format!("{}\n", severity_color));
    output.push_str(&"─".repeat(60));
    output.push('\n');

    output.push_str(&format!("\n{} {}\n", "Severity:".yellow(), err.severity));

    if !err.description.is_empty() {
        output.push_str(&format!("\n{}\n{}\n", "DESCRIPTION".bold(), err.description));
    }

    if !err.tables.is_empty() {
        output.push_str(&format!(
            "\n{}\n{}\n",
            "RELATED TABLES".bold(),
            err.tables.join(", ")
        ));
    }

    if !err.resolution.is_empty() {
        output.push_str(&format!("\n{}\n{}\n", "RESOLUTION".bold().green(), err.resolution));
    }

    output
}

fn format_ice_error_json(err: &IceError) -> String {
    serde_json::to_string_pretty(err).unwrap_or_else(|_| "{}".to_string())
}

fn format_ice_error_markdown(err: &IceError) -> String {
    let mut output = String::new();

    output.push_str(&format!("# {}\n\n", err.code));
    output.push_str(&format!("**Severity:** {}\n\n", err.severity));

    if !err.description.is_empty() {
        output.push_str(&format!("## Description\n\n{}\n\n", err.description));
    }

    if !err.tables.is_empty() {
        output.push_str("## Related Tables\n\n");
        for table in &err.tables {
            output.push_str(&format!("- `{}`\n", table));
        }
        output.push('\n');
    }

    if !err.resolution.is_empty() {
        output.push_str(&format!("## Resolution\n\n{}\n\n", err.resolution));
    }

    output
}

/// Format lint rule help
pub fn format_rule(rule: &LintRule, format: OutputFormat) -> String {
    match format {
        OutputFormat::Text => format_rule_text(rule),
        OutputFormat::Json => format_rule_json(rule),
        OutputFormat::Markdown => format_rule_markdown(rule),
    }
}

fn format_rule_text(rule: &LintRule) -> String {
    let mut output = String::new();

    let severity_color = match rule.severity.as_str() {
        "error" => rule.id.red().bold(),
        "warning" => rule.id.yellow().bold(),
        _ => rule.id.blue().bold(),
    };

    output.push_str(&format!("{}\n", severity_color));
    output.push_str(&"─".repeat(60));
    output.push('\n');

    output.push_str(&format!("\n{} {}\n", "Name:".yellow(), rule.name));
    output.push_str(&format!("{} {}\n", "Severity:".yellow(), rule.severity));

    if !rule.element.is_empty() {
        output.push_str(&format!("{} {}\n", "Element:".yellow(), rule.element));
    }

    if !rule.description.is_empty() {
        output.push_str(&format!("\n{}\n{}\n", "DESCRIPTION".bold(), rule.description));
    }

    if !rule.message.is_empty() {
        output.push_str(&format!("\n{}\n{}\n", "MESSAGE".bold(), rule.message));
    }

    output
}

fn format_rule_json(rule: &LintRule) -> String {
    serde_json::to_string_pretty(rule).unwrap_or_else(|_| "{}".to_string())
}

fn format_rule_markdown(rule: &LintRule) -> String {
    let mut output = String::new();

    output.push_str(&format!("# {}\n\n", rule.id));
    output.push_str(&format!("**Name:** {}\n", rule.name));
    output.push_str(&format!("**Severity:** {}\n", rule.severity));

    if !rule.element.is_empty() {
        output.push_str(&format!("**Element:** `{}`\n", rule.element));
    }
    output.push('\n');

    if !rule.description.is_empty() {
        output.push_str(&format!("## Description\n\n{}\n\n", rule.description));
    }

    if !rule.message.is_empty() {
        output.push_str(&format!("## Message\n\n{}\n\n", rule.message));
    }

    output
}

/// Format a list of items
pub fn format_list(
    title: &str,
    items: &[&str],
    format: OutputFormat,
    item_type: &str,
) -> String {
    match format {
        OutputFormat::Text => format_list_text(title, items),
        OutputFormat::Json => format_list_json(items, item_type),
        OutputFormat::Markdown => format_list_markdown(title, items),
    }
}

fn format_list_text(title: &str, items: &[&str]) -> String {
    let mut output = String::new();

    output.push_str(&format!("{} ({} items)\n", title.bold().cyan(), items.len()));
    output.push_str(&"─".repeat(60));
    output.push('\n');

    // Print in columns if terminal is wide enough
    let max_len = items.iter().map(|s| s.len()).max().unwrap_or(20);
    let cols = std::cmp::max(1, 60 / (max_len + 2));

    for chunk in items.chunks(cols) {
        for item in chunk {
            output.push_str(&format!("  {:<width$}", item, width = max_len + 2));
        }
        output.push('\n');
    }

    output
}

fn format_list_json(items: &[&str], item_type: &str) -> String {
    let obj = serde_json::json!({
        "type": item_type,
        "count": items.len(),
        "items": items
    });
    serde_json::to_string_pretty(&obj).unwrap_or_else(|_| "{}".to_string())
}

fn format_list_markdown(title: &str, items: &[&str]) -> String {
    let mut output = String::new();

    output.push_str(&format!("# {}\n\n", title));
    output.push_str(&format!("**Total:** {} items\n\n", items.len()));

    for item in items {
        output.push_str(&format!("- `{}`\n", item));
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn sample_element() -> ElementDef {
        let mut attrs = HashMap::new();
        attrs.insert(
            "Id".to_string(),
            AttributeDef {
                attr_type: "identifier".to_string(),
                required: Some(false),
                default: None,
                description: "Unique identifier".to_string(),
                values: vec![],
            },
        );
        attrs.insert(
            "Guid".to_string(),
            AttributeDef {
                attr_type: "guid".to_string(),
                required: Some(true),
                default: None,
                description: "Component GUID".to_string(),
                values: vec![],
            },
        );

        ElementDef {
            name: "Component".to_string(),
            namespace: "wix".to_string(),
            since: "v3".to_string(),
            description: "A component is the smallest unit of installation.".to_string(),
            documentation: "https://wixtoolset.org/docs/schema/wxs/component/".to_string(),
            parents: vec!["Directory".to_string()],
            children: vec!["File".to_string()],
            attributes: attrs,
            msi_tables: vec!["Component".to_string()],
            rules: vec!["component-requires-guid".to_string()],
            examples: vec![crate::types::Example {
                description: "Basic component".to_string(),
                code: "<Component Guid=\"*\" />".to_string(),
            }],
        }
    }

    fn sample_snippet() -> Snippet {
        Snippet {
            name: "component".to_string(),
            prefix: "comp".to_string(),
            description: "Create a component".to_string(),
            body: vec!["<Component Guid=\"*\">".to_string(), "</Component>".to_string()],
        }
    }

    fn sample_wix_error() -> WixError {
        WixError {
            code: "WIX0001".to_string(),
            severity: "error".to_string(),
            message: "Invalid parent".to_string(),
            description: "An element was placed incorrectly.".to_string(),
            resolution: "Check parent elements.".to_string(),
        }
    }

    fn sample_ice_error() -> IceError {
        IceError {
            code: "ICE03".to_string(),
            severity: "error".to_string(),
            description: "Schema validation".to_string(),
            tables: vec!["_Validation".to_string()],
            resolution: "Fix schema".to_string(),
        }
    }

    fn sample_rule() -> LintRule {
        LintRule {
            id: "component-requires-guid".to_string(),
            name: "Component requires GUID".to_string(),
            description: "Every component must have a GUID".to_string(),
            severity: "error".to_string(),
            element: "Component".to_string(),
            message: "Missing GUID".to_string(),
        }
    }

    #[test]
    fn test_format_element_text() {
        let elem = sample_element();
        let output = format_element(&elem, OutputFormat::Text, true);
        assert!(output.contains("Component"));
        assert!(output.contains("smallest unit"));
        assert!(output.contains("ATTRIBUTES"));
        assert!(output.contains("EXAMPLES"));
    }

    #[test]
    fn test_format_element_text_no_examples() {
        let elem = sample_element();
        let output = format_element(&elem, OutputFormat::Text, false);
        assert!(output.contains("Component"));
        assert!(!output.contains("EXAMPLES"));
    }

    #[test]
    fn test_format_element_json() {
        let elem = sample_element();
        let output = format_element(&elem, OutputFormat::Json, true);
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(parsed["name"], "Component");
    }

    #[test]
    fn test_format_element_markdown() {
        let elem = sample_element();
        let output = format_element(&elem, OutputFormat::Markdown, true);
        assert!(output.contains("# Component"));
        assert!(output.contains("## Attributes"));
        assert!(output.contains("## Examples"));
        assert!(output.contains("```xml"));
    }

    #[test]
    fn test_format_snippet_text() {
        let snippet = sample_snippet();
        let output = format_snippet(&snippet, OutputFormat::Text);
        assert!(output.contains("component"));
        assert!(output.contains("Prefix:"));
        assert!(output.contains("comp"));
    }

    #[test]
    fn test_format_snippet_json() {
        let snippet = sample_snippet();
        let output = format_snippet(&snippet, OutputFormat::Json);
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(parsed["prefix"], "comp");
    }

    #[test]
    fn test_format_snippet_markdown() {
        let snippet = sample_snippet();
        let output = format_snippet(&snippet, OutputFormat::Markdown);
        assert!(output.contains("# component"));
        assert!(output.contains("**Prefix:** `comp`"));
    }

    #[test]
    fn test_format_wix_error_text() {
        let err = sample_wix_error();
        let output = format_wix_error(&err, OutputFormat::Text);
        assert!(output.contains("WIX0001"));
        assert!(output.contains("RESOLUTION"));
    }

    #[test]
    fn test_format_wix_error_json() {
        let err = sample_wix_error();
        let output = format_wix_error(&err, OutputFormat::Json);
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(parsed["code"], "WIX0001");
    }

    #[test]
    fn test_format_wix_error_markdown() {
        let err = sample_wix_error();
        let output = format_wix_error(&err, OutputFormat::Markdown);
        assert!(output.contains("# WIX0001"));
        assert!(output.contains("## Resolution"));
    }

    #[test]
    fn test_format_ice_error_text() {
        let err = sample_ice_error();
        let output = format_ice_error(&err, OutputFormat::Text);
        assert!(output.contains("ICE03"));
        assert!(output.contains("RELATED TABLES"));
    }

    #[test]
    fn test_format_ice_error_json() {
        let err = sample_ice_error();
        let output = format_ice_error(&err, OutputFormat::Json);
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(parsed["code"], "ICE03");
    }

    #[test]
    fn test_format_ice_error_markdown() {
        let err = sample_ice_error();
        let output = format_ice_error(&err, OutputFormat::Markdown);
        assert!(output.contains("# ICE03"));
        assert!(output.contains("## Related Tables"));
    }

    #[test]
    fn test_format_rule_text() {
        let rule = sample_rule();
        let output = format_rule(&rule, OutputFormat::Text);
        assert!(output.contains("component-requires-guid"));
        assert!(output.contains("Name:"));
    }

    #[test]
    fn test_format_rule_json() {
        let rule = sample_rule();
        let output = format_rule(&rule, OutputFormat::Json);
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(parsed["id"], "component-requires-guid");
    }

    #[test]
    fn test_format_rule_markdown() {
        let rule = sample_rule();
        let output = format_rule(&rule, OutputFormat::Markdown);
        assert!(output.contains("# component-requires-guid"));
        assert!(output.contains("**Severity:**"));
    }

    #[test]
    fn test_format_list_text() {
        let items = vec!["Component", "Feature", "Directory"];
        let output = format_list("Elements", &items, OutputFormat::Text, "element");
        assert!(output.contains("Elements"));
        assert!(output.contains("3 items"));
    }

    #[test]
    fn test_format_list_json() {
        let items = vec!["Component", "Feature"];
        let output = format_list("Elements", &items, OutputFormat::Json, "element");
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(parsed["count"], 2);
        assert_eq!(parsed["type"], "element");
    }

    #[test]
    fn test_format_list_markdown() {
        let items = vec!["Component", "Feature"];
        let output = format_list("Elements", &items, OutputFormat::Markdown, "element");
        assert!(output.contains("# Elements"));
        assert!(output.contains("- `Component`"));
    }

    #[test]
    fn test_format_element_with_enum_attribute() {
        let mut attrs = HashMap::new();
        attrs.insert(
            "Start".to_string(),
            AttributeDef {
                attr_type: "enum".to_string(),
                required: Some(false),
                default: Some("auto".to_string()),
                description: "Start type".to_string(),
                values: vec!["auto".to_string(), "demand".to_string()],
            },
        );

        let elem = ElementDef {
            name: "ServiceInstall".to_string(),
            namespace: "wix".to_string(),
            since: "v3".to_string(),
            description: String::new(),
            documentation: String::new(),
            parents: vec![],
            children: vec![],
            attributes: attrs,
            msi_tables: vec![],
            rules: vec![],
            examples: vec![],
        };

        let output = format_element(&elem, OutputFormat::Text, false);
        assert!(output.contains("Values:"));
        assert!(output.contains("auto, demand"));
    }

    #[test]
    fn test_format_wix_error_warning_severity() {
        let err = WixError {
            code: "WIX0301".to_string(),
            severity: "warning".to_string(),
            message: String::new(),
            description: String::new(),
            resolution: String::new(),
        };
        let output = format_wix_error(&err, OutputFormat::Text);
        assert!(output.contains("WIX0301"));
    }

    #[test]
    fn test_format_rule_warning_severity() {
        let rule = LintRule {
            id: "test-rule".to_string(),
            name: "Test".to_string(),
            description: String::new(),
            severity: "warning".to_string(),
            element: String::new(),
            message: String::new(),
        };
        let output = format_rule(&rule, OutputFormat::Text);
        assert!(output.contains("test-rule"));
    }

    #[test]
    fn test_format_rule_info_severity() {
        let rule = LintRule {
            id: "test-rule".to_string(),
            name: "Test".to_string(),
            description: String::new(),
            severity: "info".to_string(),
            element: String::new(),
            message: String::new(),
        };
        let output = format_rule(&rule, OutputFormat::Text);
        assert!(output.contains("test-rule"));
    }

    #[test]
    fn test_format_empty_element() {
        let elem = ElementDef {
            name: "Empty".to_string(),
            namespace: String::new(),
            since: String::new(),
            description: String::new(),
            documentation: String::new(),
            parents: vec![],
            children: vec![],
            attributes: HashMap::new(),
            msi_tables: vec![],
            rules: vec![],
            examples: vec![],
        };

        let output = format_element(&elem, OutputFormat::Text, true);
        assert!(output.contains("Empty"));
        assert!(!output.contains("ATTRIBUTES"));
    }

    #[test]
    fn test_format_list_empty() {
        let items: Vec<&str> = vec![];
        let output = format_list("Empty", &items, OutputFormat::Text, "test");
        assert!(output.contains("0 items"));
    }

    #[test]
    fn test_format_element_with_empty_example_description() {
        let elem = ElementDef {
            name: "TestElem".to_string(),
            namespace: "wix".to_string(),
            since: "v3".to_string(),
            description: "Test element".to_string(),
            documentation: String::new(),
            parents: vec![],
            children: vec![],
            attributes: HashMap::new(),
            msi_tables: vec![],
            rules: vec![],
            examples: vec![crate::types::Example {
                description: String::new(), // Empty description
                code: "<Test />".to_string(),
            }],
        };

        // Text format
        let output = format_element(&elem, OutputFormat::Text, true);
        assert!(output.contains("EXAMPLES"));
        assert!(output.contains("<Test />"));

        // Markdown format
        let output_md = format_element(&elem, OutputFormat::Markdown, true);
        assert!(output_md.contains("Example 1"));
        assert!(output_md.contains("```xml"));
    }

    #[test]
    fn test_format_wix_error_info_severity() {
        let err = WixError {
            code: "WIX0100".to_string(),
            severity: "info".to_string(), // Not "error" or "warning"
            message: "Informational".to_string(),
            description: "Info description".to_string(),
            resolution: String::new(),
        };
        let output = format_wix_error(&err, OutputFormat::Text);
        assert!(output.contains("WIX0100"));
    }

    #[test]
    fn test_format_ice_error_info_severity() {
        let err = IceError {
            code: "ICE99".to_string(),
            severity: "info".to_string(), // Not "error" or "warning"
            description: "Info ice".to_string(),
            tables: vec![],
            resolution: String::new(),
        };
        let output = format_ice_error(&err, OutputFormat::Text);
        assert!(output.contains("ICE99"));
    }

    #[test]
    fn test_format_ice_error_warning_severity() {
        let err = IceError {
            code: "ICE50".to_string(),
            severity: "warning".to_string(),
            description: "Warning ice".to_string(),
            tables: vec![],
            resolution: String::new(),
        };
        let output = format_ice_error(&err, OutputFormat::Text);
        assert!(output.contains("ICE50"));
    }
}

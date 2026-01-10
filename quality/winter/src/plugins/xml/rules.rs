//! Generated XML lint rules
//!
//! Generated from wixkb database on 2026-01-06 10:16:29
//! Total rules: 7
//!
//! DO NOT EDIT MANUALLY - regenerate with gen-rules.py

use crate::diagnostic::Severity;
use crate::rule::Rule;

/// Get all built-in XML rules
pub fn builtin_rules() -> Vec<Rule> {
    vec![
        Rule::new(
            "xml-declaration-missing",
            "name == \"#document\" && !hasChild('?xml')",
            "XML file should have an XML declaration (<?xml version=\"1.0\"?>)",
        )
        .with_severity(Severity::Info)
        .with_target(Some("element"), None)
        .with_tag("best-practice")
        ,

        Rule::new(
            "xml-default-namespace",
            "!attributes.xmlns && name != \"#text\" && name != \"#comment\"",
            "Root element should declare a default namespace",
        )
        .with_severity(Severity::Info)
        .with_target(Some("element"), None)
        .with_tag("best-practice")
        ,

        Rule::new(
            "xml-empty-attribute",
            "attributes.Id == \"\" || attributes.Name == \"\" || attributes.Value == \"\"",
            "Element has an empty attribute value",
        )
        .with_severity(Severity::Warning)
        .with_target(Some("element"), None)
        .with_tag("validation")
        ,

        Rule::new(
            "xml-encoding-missing",
            "name == \"?xml\" && !attributes.encoding",
            "XML declaration should specify encoding (e.g., encoding=\"utf-8\")",
        )
        .with_severity(Severity::Info)
        .with_target(Some("element"), None)
        .with_tag("best-practice")
        ,

        Rule::new(
            "xml-mixed-content",
            "kind == \"element\" && hasChild('#text') && countChildren('*') > 1",
            "Element has mixed content (text and child elements) - consider restructuring",
        )
        .with_severity(Severity::Info)
        .with_target(Some("element"), None)
        .with_tag("style")
        ,

        Rule::new(
            "xml-todo-comment",
            "kind == \"comment\" && attributes.text =~ /TODO|FIXME|HACK|XXX/i",
            "Found TODO/FIXME comment - ensure this is addressed",
        )
        .with_severity(Severity::Info)
        .with_target(Some("comment"), None)
        .with_tag("awareness")
        ,

        Rule::new(
            "xml-trailing-whitespace",
            "kind == \"text\" && attributes.value =~ /\\s+$/",
            "Text node has trailing whitespace",
        )
        .with_severity(Severity::Info)
        .with_target(Some("text"), None)
        .with_tag("style")
        ,

    ]
}

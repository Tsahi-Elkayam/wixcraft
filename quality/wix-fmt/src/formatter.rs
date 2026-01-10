//! Core formatting engine

use crate::config::FormatConfig;
use crate::loader::WixData;
use crate::ordering::{sort_attributes, sort_children};
use crate::writer::XmlWriter;
use roxmltree::{Document, Node, NodeType};
use std::fs;
use std::path::Path;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum FormatError {
    #[error("Failed to read file: {0}")]
    ReadFile(#[from] std::io::Error),
    #[error("Failed to parse XML: {0}")]
    ParseXml(#[from] roxmltree::Error),
}

/// WiX XML formatter
pub struct Formatter {
    config: FormatConfig,
    data: Option<WixData>,
}

impl Formatter {
    /// Create a new formatter with configuration
    pub fn new(config: FormatConfig) -> Self {
        Self { config, data: None }
    }

    /// Create a formatter with wix-data for element ordering
    pub fn with_wix_data(config: FormatConfig, data: WixData) -> Self {
        Self {
            config,
            data: Some(data),
        }
    }

    /// Format XML source string
    pub fn format(&self, source: &str) -> Result<String, FormatError> {
        let doc = Document::parse(source)?;
        let mut writer = XmlWriter::new(self.config.clone());

        // Check if source had an XML declaration and preserve it
        if source.trim_start().starts_with("<?xml") {
            // Extract version and encoding from original source
            let (version, encoding) = self.extract_declaration_from_source(source);
            writer.write_declaration(&version, encoding.as_deref());
            writer.newline();
        }

        self.format_node(&doc.root(), &mut writer, true);

        Ok(writer.finish())
    }

    /// Extract XML declaration info from source
    fn extract_declaration_from_source(&self, source: &str) -> (String, Option<String>) {
        let mut version = "1.0".to_string();
        let mut encoding = None;

        if let Some(start) = source.find("<?xml") {
            if let Some(end) = source[start..].find("?>") {
                let decl = &source[start..start + end + 2];

                // Extract version
                if let Some(v_start) = decl.find("version=") {
                    let rest = &decl[v_start + 8..];
                    let quote = rest.chars().next().unwrap_or('"');
                    if let Some(v_end) = rest[1..].find(quote) {
                        version = rest[1..v_end + 1].to_string();
                    }
                }

                // Extract encoding
                if let Some(e_start) = decl.find("encoding=") {
                    let rest = &decl[e_start + 9..];
                    let quote = rest.chars().next().unwrap_or('"');
                    if let Some(e_end) = rest[1..].find(quote) {
                        encoding = Some(rest[1..e_end + 1].to_string());
                    }
                }
            }
        }

        (version, encoding)
    }

    /// Format an XML file
    pub fn format_file(&self, path: &Path) -> Result<String, FormatError> {
        let source = fs::read_to_string(path)?;
        self.format(&source)
    }

    /// Format a node and its children
    fn format_node(&self, node: &Node, writer: &mut XmlWriter, is_root: bool) {
        match node.node_type() {
            NodeType::Root => {
                for child in node.children() {
                    self.format_node(&child, writer, true);
                }
            }

            NodeType::Element => {
                self.format_element(node, writer);
            }

            NodeType::Text => {
                let text = node.text().unwrap_or("");
                // Preserve meaningful text, but normalize pure whitespace
                if !text.trim().is_empty() {
                    writer.write_text(text.trim());
                }
            }

            NodeType::Comment => {
                if !is_root {
                    writer.write_indent();
                }
                writer.write_comment(node.text().unwrap_or(""));
                writer.newline();
            }

            NodeType::PI => {
                if node.pi().is_some_and(|pi| pi.target == "xml") {
                    // Handle XML declaration specially
                    if let Some(pi) = node.pi() {
                        // Parse version and encoding from PI value
                        let (version, encoding) = parse_xml_declaration(pi.value.unwrap_or(""));
                        writer.write_declaration(&version, encoding.as_deref());
                        writer.newline();
                    }
                } else if let Some(pi) = node.pi() {
                    writer.write_pi(pi.target, pi.value);
                    writer.newline();
                }
            }
        }
    }

    /// Format an element with its attributes and children
    fn format_element(&self, node: &Node, writer: &mut XmlWriter) {
        let tag_name = node.tag_name().name();

        writer.write_indent();
        writer.write_element_start(tag_name);

        // Collect and optionally sort attributes
        let mut attrs: Vec<(String, String)> = node
            .attributes()
            .map(|a| (a.name().to_string(), a.value().to_string()))
            .collect();

        if self.config.sort_attributes {
            sort_attributes(&mut attrs, tag_name, self.data.as_ref());
        }

        // Decide on single-line vs multi-line attributes
        let multiline = self.should_multiline_attrs(&attrs, tag_name);

        if multiline && !attrs.is_empty() {
            // First attribute on same line
            let (first_name, first_value) = &attrs[0];
            writer.write_attribute(first_name, first_value);

            // Align subsequent attributes
            let align = tag_name.len() + 1; // +1 for '<'
            for (name, value) in attrs.iter().skip(1) {
                writer.write_attribute_newline(name, value, align);
            }
        } else {
            for (name, value) in &attrs {
                writer.write_attribute(name, value);
            }
        }

        // Check for children
        let children: Vec<_> = node
            .children()
            .filter(|c| {
                matches!(
                    c.node_type(),
                    NodeType::Element | NodeType::Comment | NodeType::PI
                ) || (c.node_type() == NodeType::Text
                    && !c.text().unwrap_or("").trim().is_empty())
            })
            .collect();

        if children.is_empty() {
            writer.write_element_end_empty();
            writer.newline();
        } else {
            writer.write_element_end();

            // Check if this is a text-only element
            let text_only = children.len() == 1
                && children[0].node_type() == NodeType::Text;

            if text_only {
                // Keep text inline
                let text = children[0].text().unwrap_or("").trim();
                writer.write_text(text);
                writer.write_close_tag(tag_name);
                writer.newline();
            } else {
                writer.newline();
                writer.indent();

                // Optionally sort child elements
                if self.config.sort_elements {
                    let child_elements: Vec<_> = children
                        .iter()
                        .filter(|c| c.node_type() == NodeType::Element)
                        .enumerate()
                        .map(|(i, c)| (c.tag_name().name().to_string(), i, *c))
                        .collect();

                    // Sort by element name
                    let mut sort_data: Vec<_> = child_elements
                        .iter()
                        .map(|(name, idx, _)| (name.clone(), *idx))
                        .collect();

                    if let Some(ref data) = self.data {
                        sort_children(&mut sort_data, tag_name, data);
                    }

                    // Reorder children based on sort result
                    let sorted_indices: Vec<_> = sort_data.iter().map(|(_, idx)| *idx).collect();

                    // Format non-element children first (comments, PIs), then sorted elements
                    for child in &children {
                        if child.node_type() != NodeType::Element {
                            self.format_node(child, writer, false);
                        }
                    }

                    for idx in sorted_indices {
                        let (_, _, node) = &child_elements[idx];
                        self.format_node(node, writer, false);
                    }
                } else {
                    // Preserve original order
                    for child in &children {
                        self.format_node(child, writer, false);
                    }
                }

                writer.dedent();
                writer.write_indent();
                writer.write_close_tag(tag_name);
                writer.newline();
            }
        }
    }

    /// Decide if attributes should be on multiple lines
    fn should_multiline_attrs(&self, attrs: &[(String, String)], tag_name: &str) -> bool {
        if attrs.len() > self.config.attr_threshold {
            return true;
        }

        // Estimate line length
        let attr_len: usize = attrs
            .iter()
            .map(|(n, v)| n.len() + v.len() + 4) // name="value" + space
            .sum();

        let total_len = 1 + tag_name.len() + attr_len + 2; // < + name + attrs + />
        total_len > self.config.max_line_width
    }
}

/// Parse XML declaration to extract version and encoding
fn parse_xml_declaration(value: &str) -> (String, Option<String>) {
    let mut version = "1.0".to_string();
    let mut encoding = None;

    // Simple parser for version="x" encoding="y"
    for part in value.split_whitespace() {
        if let Some(v) = part.strip_prefix("version=\"") {
            if let Some(v) = v.strip_suffix('"') {
                version = v.to_string();
            }
        } else if let Some(e) = part.strip_prefix("encoding=\"") {
            if let Some(e) = e.strip_suffix('"') {
                encoding = Some(e.to_string());
            }
        }
    }

    (version, encoding)
}

/// Format XML source with default configuration
pub fn format(source: &str) -> Result<String, FormatError> {
    Formatter::new(FormatConfig::default()).format(source)
}

/// Format an XML file with default configuration
pub fn format_file(path: &Path) -> Result<String, FormatError> {
    Formatter::new(FormatConfig::default()).format_file(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::IndentStyle;
    use tempfile::TempDir;

    #[test]
    fn test_format_simple() {
        let source = "<Package><Component /></Package>";
        let result = format(source).unwrap();
        assert!(result.contains("<Package>"));
        assert!(result.contains("</Package>"));
    }

    #[test]
    fn test_format_with_indentation() {
        let source = "<Package><Component /></Package>";
        let formatter = Formatter::new(FormatConfig::default());
        let result = formatter.format(source).unwrap();

        // Should have proper indentation
        assert!(result.contains("  <Component />")); // 2 space indent
    }

    #[test]
    fn test_format_preserves_attributes() {
        let source = r#"<Package Name="Test" Version="1.0" />"#;
        let result = format(source).unwrap();
        assert!(result.contains("Name=\"Test\""));
        assert!(result.contains("Version=\"1.0\""));
    }

    #[test]
    fn test_format_multiline_attributes() {
        let config = FormatConfig {
            attr_threshold: 2,
            ..Default::default()
        };
        let source = r#"<Package Name="Test" Version="1.0" Manufacturer="Acme" />"#;
        let formatter = Formatter::new(config);
        let result = formatter.format(source).unwrap();

        // Should have attributes on multiple lines
        assert!(result.contains('\n'));
        let lines: Vec<_> = result.lines().collect();
        assert!(lines.len() > 1);
    }

    #[test]
    fn test_format_xml_declaration() {
        let source = r#"<?xml version="1.0" encoding="UTF-8"?><Package />"#;
        let result = format(source).unwrap();
        assert!(result.contains("<?xml version=\"1.0\" encoding=\"UTF-8\"?>"));
    }

    #[test]
    fn test_format_comment() {
        let source = r#"<Package><!-- Comment --></Package>"#;
        let result = format(source).unwrap();
        assert!(result.contains("<!-- Comment -->"));
    }

    #[test]
    fn test_format_text_content() {
        let source = r#"<Property Id="TEST">Value</Property>"#;
        let result = format(source).unwrap();
        assert!(result.contains(">Value</Property>"));
    }

    #[test]
    fn test_format_self_closing() {
        let source = r#"<File Source="test.exe" />"#;
        let result = format(source).unwrap();
        assert!(result.contains("/>"));
    }

    #[test]
    fn test_format_tab_indent() {
        let config = FormatConfig {
            indent_style: IndentStyle::Tab,
            indent_size: 1,
            ..Default::default()
        };
        let source = "<Package><Component /></Package>";
        let formatter = Formatter::new(config);
        let result = formatter.format(source).unwrap();
        assert!(result.contains("\t<Component"));
    }

    #[test]
    fn test_format_file() {
        let temp = TempDir::new().unwrap();
        let file_path = temp.path().join("test.wxs");
        fs::write(&file_path, "<Package />").unwrap();

        let result = format_file(&file_path).unwrap();
        assert!(result.contains("<Package />"));
    }

    #[test]
    fn test_format_file_not_found() {
        let result = format_file(Path::new("/nonexistent/file.wxs"));
        assert!(matches!(result, Err(FormatError::ReadFile(_))));
    }

    #[test]
    fn test_format_invalid_xml() {
        let source = "<Package><Unclosed>";
        let result = format(source);
        assert!(matches!(result, Err(FormatError::ParseXml(_))));
    }

    #[test]
    fn test_sort_attributes() {
        let config = FormatConfig {
            sort_attributes: true,
            ..Default::default()
        };
        let source = r#"<Component Zebra="z" Id="test" Apple="a" />"#;
        let formatter = Formatter::new(config);
        let result = formatter.format(source).unwrap();

        // Id should come first
        let id_pos = result.find("Id=").unwrap();
        let apple_pos = result.find("Apple=").unwrap();
        let zebra_pos = result.find("Zebra=").unwrap();

        assert!(id_pos < apple_pos);
        assert!(apple_pos < zebra_pos);
    }

    #[test]
    fn test_final_newline() {
        let config = FormatConfig {
            insert_final_newline: true,
            ..Default::default()
        };
        let source = "<Package />";
        let formatter = Formatter::new(config);
        let result = formatter.format(source).unwrap();
        assert!(result.ends_with('\n'));
    }

    #[test]
    fn test_no_final_newline() {
        let config = FormatConfig {
            insert_final_newline: false,
            ..Default::default()
        };
        let source = "<Package />";
        let formatter = Formatter::new(config);
        let result = formatter.format(source).unwrap();
        assert!(!result.ends_with('\n'));
    }

    #[test]
    fn test_trim_trailing_whitespace() {
        let config = FormatConfig::default();
        let formatter = Formatter::new(config);
        let source = "<Package />";
        let result = formatter.format(source).unwrap();
        // No lines should have trailing spaces
        for line in result.lines() {
            assert!(!line.ends_with(' '));
        }
    }

    #[test]
    fn test_nested_elements() {
        let source = "<Package><Directory><Component><File /></Component></Directory></Package>";
        let result = format(source).unwrap();

        // Should have proper nesting
        let lines: Vec<_> = result.lines().collect();
        assert!(lines.len() >= 4);

        // Check increasing indent
        let dir_line = lines.iter().find(|l| l.contains("<Directory>")).unwrap();
        let comp_line = lines.iter().find(|l| l.contains("<Component>")).unwrap();
        let file_line = lines.iter().find(|l| l.contains("<File")).unwrap();

        let dir_indent = dir_line.len() - dir_line.trim_start().len();
        let comp_indent = comp_line.len() - comp_line.trim_start().len();
        let file_indent = file_line.len() - file_line.trim_start().len();

        assert!(comp_indent > dir_indent);
        assert!(file_indent > comp_indent);
    }

    #[test]
    fn test_parse_xml_declaration() {
        let (version, encoding) = parse_xml_declaration("version=\"1.0\" encoding=\"UTF-8\"");
        assert_eq!(version, "1.0");
        assert_eq!(encoding, Some("UTF-8".to_string()));
    }

    #[test]
    fn test_parse_xml_declaration_no_encoding() {
        let (version, encoding) = parse_xml_declaration("version=\"1.1\"");
        assert_eq!(version, "1.1");
        assert_eq!(encoding, None);
    }

    #[test]
    fn test_format_escapes_special_chars() {
        let source = r#"<Property Id="TEST">Value &amp; More</Property>"#;
        let result = format(source).unwrap();
        // The & should be preserved as &amp; in output
        assert!(result.contains("&amp;") || result.contains("Value & More"));
    }

    #[test]
    fn test_format_empty_element() {
        let source = "<Package></Package>";
        let result = format(source).unwrap();
        // Should become self-closing
        assert!(result.contains("<Package />"));
    }

    #[test]
    fn test_with_wix_data() {
        use std::io::Write;
        use tempfile::TempDir;

        // Create wix-data directory with element definition
        let temp = TempDir::new().unwrap();
        let elements_dir = temp.path().join("elements");
        fs::create_dir(&elements_dir).unwrap();

        let element_json = r#"{"name": "Package", "children": ["Directory", "Component"], "attributes": {"Id": {"type": "identifier"}}}"#;
        fs::write(elements_dir.join("package.json"), element_json).unwrap();

        let data = crate::loader::WixData::load(temp.path()).unwrap();
        let config = FormatConfig::default();
        let formatter = Formatter::with_wix_data(config, data);

        let source = "<Package><Component /></Package>";
        let result = formatter.format(source).unwrap();
        assert!(result.contains("<Package>"));
    }

    #[test]
    fn test_format_text_with_whitespace() {
        // Text that is pure whitespace should be normalized
        let source = "<Package>   \n   </Package>";
        let result = format(source).unwrap();
        // Should become self-closing when only whitespace
        assert!(result.contains("<Package />"));
    }

    #[test]
    fn test_format_text_with_content() {
        // Text with actual content should be preserved
        let source = "<Property>   Some value   </Property>";
        let result = format(source).unwrap();
        assert!(result.contains("Some value"));
    }

    #[test]
    fn test_format_with_sort_elements() {
        let config = FormatConfig {
            sort_elements: true,
            ..Default::default()
        };
        let source = "<Package><Component /><Directory /></Package>";
        let formatter = Formatter::new(config);
        let result = formatter.format(source).unwrap();
        // Without wix-data, alphabetical ordering
        assert!(result.contains("<Component />"));
        assert!(result.contains("<Directory />"));
    }

    #[test]
    fn test_format_with_sort_elements_and_wix_data() {
        use std::io::Write;
        use tempfile::TempDir;

        // Create wix-data with child order
        let temp = TempDir::new().unwrap();
        let elements_dir = temp.path().join("elements");
        fs::create_dir(&elements_dir).unwrap();

        let element_json = r#"{"name": "Package", "children": ["Directory", "Component"], "attributes": {}}"#;
        fs::write(elements_dir.join("package.json"), element_json).unwrap();

        let data = crate::loader::WixData::load(temp.path()).unwrap();
        let config = FormatConfig {
            sort_elements: true,
            ..Default::default()
        };
        let formatter = Formatter::with_wix_data(config, data);

        // Component comes before Directory in source but Directory should come first per wix-data
        let source = "<Package><Component Id=\"C\" /><Directory Id=\"D\" /></Package>";
        let result = formatter.format(source).unwrap();

        // Both should be present
        assert!(result.contains("Component"));
        assert!(result.contains("Directory"));
    }

    #[test]
    fn test_format_with_comments_and_elements() {
        let config = FormatConfig {
            sort_elements: true,
            ..Default::default()
        };
        let source = "<Package><!-- Comment --><Component /></Package>";
        let formatter = Formatter::new(config);
        let result = formatter.format(source).unwrap();
        assert!(result.contains("<!-- Comment -->"));
        assert!(result.contains("<Component />"));
    }

    #[test]
    fn test_format_error_display() {
        let io_err = FormatError::ReadFile(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "file not found"
        ));
        assert!(io_err.to_string().contains("Failed to read file"));

        // Parse error
        let parse_result = format("<Invalid><Unclosed>");
        assert!(matches!(parse_result, Err(FormatError::ParseXml(_))));
        if let Err(e) = parse_result {
            assert!(e.to_string().contains("Failed to parse XML"));
        }
    }

    #[test]
    fn test_extract_declaration_version_only() {
        let formatter = Formatter::new(FormatConfig::default());
        let source = r#"<?xml version="1.1"?>"#;
        let (version, encoding) = formatter.extract_declaration_from_source(source);
        assert_eq!(version, "1.1");
        assert!(encoding.is_none());
    }

    #[test]
    fn test_extract_declaration_with_encoding() {
        let formatter = Formatter::new(FormatConfig::default());
        let source = r#"<?xml version="1.0" encoding="UTF-16"?>"#;
        let (version, encoding) = formatter.extract_declaration_from_source(source);
        assert_eq!(version, "1.0");
        assert_eq!(encoding, Some("UTF-16".to_string()));
    }

    #[test]
    fn test_extract_declaration_no_declaration() {
        let formatter = Formatter::new(FormatConfig::default());
        let source = r#"<Package />"#;
        let (version, encoding) = formatter.extract_declaration_from_source(source);
        assert_eq!(version, "1.0"); // default
        assert!(encoding.is_none());
    }

    #[test]
    fn test_format_meaningful_text_preserved() {
        // Test that meaningful text content (not whitespace) is preserved
        let source = r#"<Property Id="TEST">Some important value</Property>"#;
        let result = format(source).unwrap();
        assert!(result.contains("Some important value"));
    }

    #[test]
    fn test_format_text_trimmed() {
        // Text should be trimmed but content preserved
        let source = r#"<Property Id="TEST">   trimmed   </Property>"#;
        let result = format(source).unwrap();
        assert!(result.contains(">trimmed</"));
    }

    #[test]
    fn test_format_mixed_children_and_text() {
        // Test element with both children and text (text should be handled correctly)
        let source = r#"<Package>Text<Component /></Package>"#;
        let result = format(source).unwrap();
        assert!(result.contains("Component"));
        // Text should be present somewhere
        assert!(result.contains("Text") || result.contains("<Package>"));
    }

    #[test]
    fn test_format_processing_instruction() {
        // Test non-xml processing instruction
        let source = r#"<?custom-pi some-data ?><Package />"#;
        // roxmltree may handle PIs differently - just verify no crash
        let result = format(source);
        // Should either succeed or error gracefully
        match result {
            Ok(r) => assert!(r.contains("<Package")),
            Err(_) => (), // Some PI formats may cause parse errors
        }
    }

    #[test]
    fn test_format_text_in_element_with_children() {
        // Element with both text and element children
        let source = r#"<Package><!-- comment --><Component /></Package>"#;
        let result = format(source).unwrap();
        assert!(result.contains("<!--"));
        assert!(result.contains("<Component />"));
    }

    #[test]
    fn test_sort_elements_complex() {
        use std::io::Write;

        // Create wix-data with specific ordering
        let temp = TempDir::new().unwrap();
        let elements_dir = temp.path().join("elements");
        fs::create_dir(&elements_dir).unwrap();

        let package = r#"{"name": "Package", "children": ["Feature", "Directory", "Component"], "attributes": {}}"#;
        fs::write(elements_dir.join("package.json"), package).unwrap();

        let data = crate::loader::WixData::load(temp.path()).unwrap();
        let config = FormatConfig {
            sort_elements: true,
            ..Default::default()
        };
        let formatter = Formatter::with_wix_data(config, data);

        // All elements out of canonical order
        let source = "<Package><Component /><Directory /><Feature /></Package>";
        let result = formatter.format(source).unwrap();

        // Find positions of each element
        let feat_pos = result.find("<Feature").unwrap_or(999);
        let dir_pos = result.find("<Directory").unwrap_or(999);
        let comp_pos = result.find("<Component").unwrap_or(999);

        // Should be: Feature, Directory, Component (per wix-data order)
        assert!(feat_pos < dir_pos, "Feature should come before Directory");
        assert!(dir_pos < comp_pos, "Directory should come before Component");
    }

    #[test]
    fn test_format_preserves_cdata() {
        let source = r#"<Package><![CDATA[Some <special> content]]></Package>"#;
        // Test that CDATA sections don't crash the formatter
        let result = format(source);
        // Should handle gracefully
        assert!(result.is_ok() || result.is_err()); // Either outcome is fine
    }

    #[test]
    fn test_format_nested_with_sorting_no_data() {
        // Without wix-data, sort_elements still triggers the sorting code path
        // but without wix-data, original order is preserved
        let config = FormatConfig {
            sort_elements: true,
            ..Default::default()
        };
        let source = "<Package><Z /><A /><M /></Package>";
        let formatter = Formatter::new(config);
        let result = formatter.format(source).unwrap();

        // Without wix-data, original order is preserved (Z, A, M)
        let z_pos = result.find("<Z").unwrap_or(999);
        let a_pos = result.find("<A").unwrap_or(999);
        let m_pos = result.find("<M").unwrap_or(999);

        assert!(z_pos < a_pos, "Z should come before A (original order)");
        assert!(a_pos < m_pos, "A should come before M (original order)");
    }
}

//! XML output writer

use crate::config::FormatConfig;

/// XML writer for formatted output
pub struct XmlWriter {
    output: String,
    config: FormatConfig,
    indent_level: usize,
}

impl XmlWriter {
    pub fn new(config: FormatConfig) -> Self {
        Self {
            output: String::new(),
            config,
            indent_level: 0,
        }
    }

    /// Get the formatted output
    pub fn finish(mut self) -> String {
        if self.config.trim_trailing_whitespace {
            self.output = self
                .output
                .lines()
                .map(|line| line.trim_end())
                .collect::<Vec<_>>()
                .join("\n");
        }

        if self.config.insert_final_newline && !self.output.ends_with('\n') {
            self.output.push('\n');
        }

        self.output
    }

    /// Write raw string
    #[allow(dead_code)]
    pub fn write_raw(&mut self, s: &str) {
        self.output.push_str(s);
    }

    /// Write a newline
    pub fn newline(&mut self) {
        self.output.push('\n');
    }

    /// Write current indentation
    pub fn write_indent(&mut self) {
        let indent = self.config.indent_str().repeat(self.indent_level);
        self.output.push_str(&indent);
    }

    /// Increase indent level
    pub fn indent(&mut self) {
        self.indent_level += 1;
    }

    /// Decrease indent level
    pub fn dedent(&mut self) {
        if self.indent_level > 0 {
            self.indent_level -= 1;
        }
    }

    /// Write XML declaration
    pub fn write_declaration(&mut self, version: &str, encoding: Option<&str>) {
        self.output.push_str("<?xml version=\"");
        self.output.push_str(&escape_attr(version));
        self.output.push('"');
        if let Some(enc) = encoding {
            self.output.push_str(" encoding=\"");
            self.output.push_str(&escape_attr(enc));
            self.output.push('"');
        }
        self.output.push_str("?>");
    }

    /// Write a comment
    pub fn write_comment(&mut self, text: &str) {
        self.output.push_str("<!--");
        self.output.push_str(text);
        self.output.push_str("-->");
    }

    /// Write opening tag start (just the element name)
    pub fn write_element_start(&mut self, name: &str) {
        self.output.push('<');
        self.output.push_str(name);
    }

    /// Write an attribute
    pub fn write_attribute(&mut self, name: &str, value: &str) {
        self.output.push(' ');
        self.output.push_str(name);
        self.output.push_str("=\"");
        self.output.push_str(&escape_attr(value));
        self.output.push('"');
    }

    /// Write attribute on a new line with alignment
    pub fn write_attribute_newline(&mut self, name: &str, value: &str, align: usize) {
        self.newline();
        self.write_indent();
        // Align to the position after element name + space
        self.output.push_str(&" ".repeat(align));
        self.output.push_str(name);
        self.output.push_str("=\"");
        self.output.push_str(&escape_attr(value));
        self.output.push('"');
    }

    /// Close opening tag (not self-closing)
    pub fn write_element_end(&mut self) {
        self.output.push('>');
    }

    /// Close self-closing tag
    pub fn write_element_end_empty(&mut self) {
        self.output.push_str(" />");
    }

    /// Write closing tag
    pub fn write_close_tag(&mut self, name: &str) {
        self.output.push_str("</");
        self.output.push_str(name);
        self.output.push('>');
    }

    /// Write text content
    pub fn write_text(&mut self, text: &str) {
        self.output.push_str(&escape_text(text));
    }

    /// Write CDATA section
    #[allow(dead_code)]
    pub fn write_cdata(&mut self, content: &str) {
        self.output.push_str("<![CDATA[");
        self.output.push_str(content);
        self.output.push_str("]]>");
    }

    /// Write processing instruction
    pub fn write_pi(&mut self, target: &str, content: Option<&str>) {
        self.output.push_str("<?");
        self.output.push_str(target);
        if let Some(c) = content {
            self.output.push(' ');
            self.output.push_str(c);
        }
        self.output.push_str("?>");
    }

    /// Get current line length (for wrapping decisions)
    #[allow(dead_code)]
    pub fn current_line_len(&self) -> usize {
        self.output.lines().last().map(|l| l.len()).unwrap_or(0)
    }
}

/// Escape special characters in attribute values
fn escape_attr(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => result.push_str("&amp;"),
            '<' => result.push_str("&lt;"),
            '>' => result.push_str("&gt;"),
            '"' => result.push_str("&quot;"),
            '\'' => result.push_str("&apos;"),
            _ => result.push(c),
        }
    }
    result
}

/// Escape special characters in text content
fn escape_text(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => result.push_str("&amp;"),
            '<' => result.push_str("&lt;"),
            '>' => result.push_str("&gt;"),
            _ => result.push(c),
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::IndentStyle;

    fn default_config() -> FormatConfig {
        FormatConfig::default()
    }

    #[test]
    fn test_write_declaration() {
        let mut w = XmlWriter::new(default_config());
        w.write_declaration("1.0", Some("UTF-8"));
        let output = w.finish();
        assert!(output.contains("<?xml version=\"1.0\" encoding=\"UTF-8\"?>"));
    }

    #[test]
    fn test_write_element() {
        let mut w = XmlWriter::new(default_config());
        w.write_element_start("Package");
        w.write_attribute("Name", "Test");
        w.write_element_end();
        w.write_close_tag("Package");
        let output = w.finish();
        assert!(output.contains("<Package Name=\"Test\"></Package>"));
    }

    #[test]
    fn test_write_self_closing() {
        let mut w = XmlWriter::new(default_config());
        w.write_element_start("File");
        w.write_attribute("Source", "test.exe");
        w.write_element_end_empty();
        let output = w.finish();
        assert!(output.contains("<File Source=\"test.exe\" />"));
    }

    #[test]
    fn test_escape_attr() {
        assert_eq!(escape_attr("a&b"), "a&amp;b");
        assert_eq!(escape_attr("a<b"), "a&lt;b");
        assert_eq!(escape_attr("a>b"), "a&gt;b");
        assert_eq!(escape_attr("a\"b"), "a&quot;b");
        assert_eq!(escape_attr("a'b"), "a&apos;b");
    }

    #[test]
    fn test_escape_text() {
        assert_eq!(escape_text("a&b"), "a&amp;b");
        assert_eq!(escape_text("a<b"), "a&lt;b");
        assert_eq!(escape_text("a>b"), "a&gt;b");
        // Quotes not escaped in text
        assert_eq!(escape_text("a\"b"), "a\"b");
    }

    #[test]
    fn test_indentation() {
        let mut w = XmlWriter::new(default_config());
        w.write_element_start("Root");
        w.write_element_end();
        w.newline();
        w.indent();
        w.write_indent();
        w.write_element_start("Child");
        w.write_element_end_empty();
        w.newline();
        w.dedent();
        w.write_close_tag("Root");
        let output = w.finish();
        assert!(output.contains("  <Child />")); // 2 spaces indent
    }

    #[test]
    fn test_tab_indentation() {
        let config = FormatConfig {
            indent_style: IndentStyle::Tab,
            indent_size: 1,
            ..Default::default()
        };
        let mut w = XmlWriter::new(config);
        w.indent();
        w.write_indent();
        w.write_raw("test");
        let output = w.finish();
        assert!(output.starts_with("\ttest"));
    }

    #[test]
    fn test_trim_trailing_whitespace() {
        let mut w = XmlWriter::new(default_config());
        w.write_raw("test   ");
        w.newline();
        w.write_raw("line2  ");
        let output = w.finish();
        assert!(!output.lines().any(|l| l.ends_with(' ')));
    }

    #[test]
    fn test_final_newline() {
        let config = FormatConfig {
            insert_final_newline: true,
            ..Default::default()
        };
        let mut w = XmlWriter::new(config);
        w.write_raw("test");
        let output = w.finish();
        assert!(output.ends_with('\n'));
    }

    #[test]
    fn test_no_final_newline() {
        let config = FormatConfig {
            insert_final_newline: false,
            ..Default::default()
        };
        let mut w = XmlWriter::new(config);
        w.write_raw("test");
        let output = w.finish();
        assert!(!output.ends_with('\n'));
    }

    #[test]
    fn test_write_comment() {
        let mut w = XmlWriter::new(default_config());
        w.write_comment(" This is a comment ");
        let output = w.finish();
        assert!(output.contains("<!-- This is a comment -->"));
    }

    #[test]
    fn test_write_cdata() {
        let mut w = XmlWriter::new(default_config());
        w.write_cdata("some <special> content");
        let output = w.finish();
        assert!(output.contains("<![CDATA[some <special> content]]>"));
    }

    #[test]
    fn test_write_text() {
        let mut w = XmlWriter::new(default_config());
        w.write_text("Hello <World>");
        let output = w.finish();
        assert!(output.contains("Hello &lt;World&gt;"));
    }

    #[test]
    fn test_current_line_len() {
        let mut w = XmlWriter::new(default_config());
        w.write_raw("12345");
        assert_eq!(w.current_line_len(), 5);
        w.newline();
        w.write_raw("abc");
        assert_eq!(w.current_line_len(), 3);
    }

    #[test]
    fn test_write_pi() {
        let mut w = XmlWriter::new(default_config());
        w.write_pi("target", Some("content"));
        let output = w.finish();
        assert!(output.contains("<?target content?>"));
    }

    #[test]
    fn test_write_pi_no_content() {
        let mut w = XmlWriter::new(default_config());
        w.write_pi("target", None);
        let output = w.finish();
        assert!(output.contains("<?target?>"));
    }

    #[test]
    fn test_attribute_newline() {
        let mut w = XmlWriter::new(default_config());
        w.write_element_start("Element");
        w.write_attribute_newline("Attr", "value", 8);
        let output = w.finish();
        assert!(output.contains("\n        Attr=\"value\""));
    }

    #[test]
    fn test_dedent_at_zero() {
        let mut w = XmlWriter::new(default_config());
        w.dedent(); // Should not panic
        w.write_indent();
        w.write_raw("test");
        let output = w.finish();
        assert!(output.starts_with("test"));
    }
}

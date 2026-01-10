//! WiX document wrapper for XML parsing

use roxmltree::{Document, Node};
use std::path::{Path, PathBuf};

use super::types::Range;

/// A parsed WiX document
pub struct WixDocument<'a> {
    source: &'a str,
    doc: Document<'a>,
    file: PathBuf,
}

impl<'a> WixDocument<'a> {
    /// Parse a WiX source file
    pub fn parse(source: &'a str, file: &Path) -> Result<Self, String> {
        let doc = Document::parse(source).map_err(|e| format!("XML parse error: {}", e))?;
        Ok(Self {
            source,
            doc,
            file: file.to_path_buf(),
        })
    }

    /// Get the source text
    pub fn source(&self) -> &str {
        self.source
    }

    /// Get the file path
    pub fn file(&self) -> &Path {
        &self.file
    }

    /// Get the root node
    pub fn root(&self) -> Node<'_, '_> {
        self.doc.root()
    }

    /// Get range of a node
    pub fn node_range(&self, node: &Node) -> Range {
        let r = node.range();
        Range::from_offsets(self.source, r.start, r.end)
    }

    /// Get the line content at a given line number (1-based)
    pub fn line_content(&self, line: usize) -> Option<&str> {
        self.source.lines().nth(line.saturating_sub(1))
    }

    /// Find element at position (line, column are 1-based)
    pub fn element_at(&self, line: usize, column: usize) -> Option<Node<'_, '_>> {
        let offset = self.position_to_offset(line, column)?;
        self.element_at_offset(self.root(), offset)
    }

    fn element_at_offset<'b>(&self, node: Node<'b, 'b>, offset: usize) -> Option<Node<'b, 'b>> {
        if node.is_element() {
            let range = node.range();
            if offset >= range.start && offset < range.end {
                // Check children first (more specific)
                for child in node.children() {
                    if let Some(found) = self.element_at_offset(child, offset) {
                        return Some(found);
                    }
                }
                return Some(node);
            }
        } else {
            for child in node.children() {
                if let Some(found) = self.element_at_offset(child, offset) {
                    return Some(found);
                }
            }
        }
        None
    }

    fn position_to_offset(&self, line: usize, column: usize) -> Option<usize> {
        let mut current_line = 1;
        let mut current_col = 1;
        let mut offset = 0;

        for (i, ch) in self.source.char_indices() {
            if current_line == line && current_col == column {
                return Some(i);
            }

            if ch == '\n' {
                if current_line == line {
                    // Column is past end of line
                    return Some(i);
                }
                current_line += 1;
                current_col = 1;
            } else {
                current_col += 1;
            }
            offset = i + ch.len_utf8();
        }

        // End of file
        if current_line == line {
            Some(offset)
        } else {
            None
        }
    }
}

/// Iterator over all elements in document order
pub struct ElementIterator<'a, 'input> {
    stack: Vec<Node<'a, 'input>>,
}

impl<'a, 'input> ElementIterator<'a, 'input> {
    pub fn new(root: Node<'a, 'input>) -> Self {
        Self {
            stack: vec![root],
        }
    }
}

impl<'a, 'input> Iterator for ElementIterator<'a, 'input> {
    type Item = Node<'a, 'input>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(node) = self.stack.pop() {
            // Push children in reverse order so we process them left-to-right
            let children: Vec<_> = node.children().collect();
            for child in children.into_iter().rev() {
                self.stack.push(child);
            }

            if node.is_element() {
                return Some(node);
            }
        }
        None
    }
}

/// Extension trait for Node
pub trait NodeExt<'a, 'input> {
    /// Get the Id attribute value
    fn id(&self) -> Option<&'a str>;

    /// Iterate over all descendant elements
    fn descendants(&self) -> ElementIterator<'a, 'input>;

    /// Check if this is a reference element (ComponentRef, DirectoryRef, etc.)
    fn is_reference(&self) -> bool;

    /// Check if this is a definition element (Component, Directory, etc.)
    fn is_definition(&self) -> bool;
}

impl<'a, 'input> NodeExt<'a, 'input> for Node<'a, 'input> {
    fn id(&self) -> Option<&'a str> {
        self.attribute("Id").or_else(|| self.attribute("Name"))
    }

    fn descendants(&self) -> ElementIterator<'a, 'input> {
        ElementIterator::new(*self)
    }

    fn is_reference(&self) -> bool {
        if !self.is_element() {
            return false;
        }
        matches!(
            self.tag_name().name(),
            "ComponentRef"
                | "ComponentGroupRef"
                | "DirectoryRef"
                | "FeatureRef"
                | "FeatureGroupRef"
                | "PropertyRef"
                | "CustomActionRef"
                | "BinaryRef"
        )
    }

    fn is_definition(&self) -> bool {
        if !self.is_element() {
            return false;
        }
        matches!(
            self.tag_name().name(),
            "Component"
                | "ComponentGroup"
                | "Directory"
                | "StandardDirectory"
                | "Feature"
                | "FeatureGroup"
                | "Property"
                | "CustomAction"
                | "Binary"
                | "Fragment"
                | "Package"
                | "Module"
                | "Bundle"
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_document() {
        let source = "<Wix><Package Name=\"Test\" /></Wix>";
        let doc = WixDocument::parse(source, Path::new("test.wxs")).unwrap();
        assert_eq!(doc.source(), source);
    }

    #[test]
    fn test_parse_document_invalid() {
        let source = "<Wix><Invalid";
        let result = WixDocument::parse(source, Path::new("test.wxs"));
        assert!(result.is_err());
        let err = result.err().unwrap();
        assert!(err.contains("XML parse error"));
    }

    #[test]
    fn test_file_path() {
        let source = "<Wix />";
        let doc = WixDocument::parse(source, Path::new("my/path/test.wxs")).unwrap();
        assert_eq!(doc.file().to_str().unwrap(), "my/path/test.wxs");
    }

    #[test]
    fn test_element_at_position() {
        let source = "<Wix>\n  <Component Id=\"C1\" />\n</Wix>";
        let doc = WixDocument::parse(source, Path::new("test.wxs")).unwrap();

        let elem = doc.element_at(2, 4).unwrap();
        assert_eq!(elem.tag_name().name(), "Component");
    }

    #[test]
    fn test_element_at_position_wix() {
        let source = "<Wix>\n  <Component Id=\"C1\" />\n</Wix>";
        let doc = WixDocument::parse(source, Path::new("test.wxs")).unwrap();

        // Position on the first line should find Wix
        let elem = doc.element_at(1, 2).unwrap();
        assert_eq!(elem.tag_name().name(), "Wix");
    }

    #[test]
    fn test_element_at_position_invalid_line() {
        let source = "<Wix />";
        let doc = WixDocument::parse(source, Path::new("test.wxs")).unwrap();

        // Line doesn't exist
        let result = doc.element_at(100, 1);
        assert!(result.is_none());
    }

    #[test]
    fn test_element_at_position_past_end_of_line() {
        let source = "<Wix>\n<A />\n</Wix>";
        let doc = WixDocument::parse(source, Path::new("test.wxs")).unwrap();

        // Column past end of line 1 - should return end of line position
        let result = doc.element_at(1, 100);
        // This should still find the Wix element
        assert!(result.is_some());
    }

    #[test]
    fn test_element_at_end_of_file() {
        let source = "<Wix />";
        let doc = WixDocument::parse(source, Path::new("test.wxs")).unwrap();

        // Position at end of file
        let result = doc.element_at(1, 7);
        assert!(result.is_some());
    }

    #[test]
    fn test_node_id() {
        let source = "<Wix><Component Id=\"C1\" /></Wix>";
        let doc = WixDocument::parse(source, Path::new("test.wxs")).unwrap();

        for node in doc.root().descendants() {
            if node.tag_name().name() == "Component" {
                assert_eq!(node.attribute("Id"), Some("C1"));
            }
        }
    }

    #[test]
    fn test_node_id_from_name() {
        let source = "<Wix><Package Name=\"TestPkg\" /></Wix>";
        let doc = WixDocument::parse(source, Path::new("test.wxs")).unwrap();

        for node in doc.root().descendants() {
            if node.tag_name().name() == "Package" {
                // Uses Name attribute when Id is not present
                assert_eq!(NodeExt::id(&node), Some("TestPkg"));
            }
        }
    }

    #[test]
    fn test_is_reference() {
        let source = "<Wix><ComponentRef Id=\"C1\" /></Wix>";
        let doc = WixDocument::parse(source, Path::new("test.wxs")).unwrap();

        for node in doc.root().descendants() {
            if node.tag_name().name() == "ComponentRef" {
                assert!(node.is_reference());
            }
        }
    }

    #[test]
    fn test_is_reference_all_types() {
        let source = r#"<Wix>
            <ComponentRef Id="C1" />
            <ComponentGroupRef Id="CG1" />
            <DirectoryRef Id="D1" />
            <FeatureRef Id="F1" />
            <FeatureGroupRef Id="FG1" />
            <PropertyRef Id="P1" />
            <CustomActionRef Id="CA1" />
            <BinaryRef Id="B1" />
        </Wix>"#;
        let doc = WixDocument::parse(source, Path::new("test.wxs")).unwrap();

        let ref_count = doc.root().descendants().filter(|n| n.is_reference()).count();
        assert_eq!(ref_count, 8);
    }

    #[test]
    fn test_is_reference_non_reference() {
        let source = "<Wix><Component Id=\"C1\" /></Wix>";
        let doc = WixDocument::parse(source, Path::new("test.wxs")).unwrap();

        for node in doc.root().descendants() {
            if node.tag_name().name() == "Component" {
                assert!(!node.is_reference());
            }
        }
    }

    #[test]
    fn test_is_definition() {
        let source = "<Wix><Component Id=\"C1\" /></Wix>";
        let doc = WixDocument::parse(source, Path::new("test.wxs")).unwrap();

        for node in doc.root().descendants() {
            if node.tag_name().name() == "Component" {
                assert!(node.is_definition());
            }
        }
    }

    #[test]
    fn test_is_definition_all_types() {
        let source = r#"<Wix>
            <Component Id="C1" />
            <ComponentGroup Id="CG1" />
            <Directory Id="D1" />
            <StandardDirectory Id="SD1" />
            <Feature Id="F1" />
            <FeatureGroup Id="FG1" />
            <Property Id="P1" />
            <CustomAction Id="CA1" />
            <Binary Id="B1" />
            <Fragment Id="Frag1" />
            <Package Name="Pkg" />
            <Module Id="Mod1" />
            <Bundle Name="Bund1" />
        </Wix>"#;
        let doc = WixDocument::parse(source, Path::new("test.wxs")).unwrap();

        let def_count = doc.root().descendants().filter(|n| n.is_definition()).count();
        assert_eq!(def_count, 13);
    }

    #[test]
    fn test_is_definition_non_definition() {
        let source = "<Wix><File Id=\"F1\" /></Wix>";
        let doc = WixDocument::parse(source, Path::new("test.wxs")).unwrap();

        for node in doc.root().descendants() {
            if node.tag_name().name() == "File" {
                assert!(!node.is_definition());
            }
        }
    }

    #[test]
    fn test_line_content() {
        let source = "<Wix>\n<A />\n</Wix>";
        let doc = WixDocument::parse(source, Path::new("test.wxs")).unwrap();
        assert_eq!(doc.line_content(1), Some("<Wix>"));
        assert_eq!(doc.line_content(2), Some("<A />"));
        assert_eq!(doc.line_content(3), Some("</Wix>"));
        // Line 0 with saturating_sub gives 0, which returns first line
        assert_eq!(doc.line_content(0), Some("<Wix>"));
        assert_eq!(doc.line_content(4), None); // Beyond file
    }

    #[test]
    fn test_node_range() {
        let source = "<Wix><Component Id=\"C1\" /></Wix>";
        let doc = WixDocument::parse(source, Path::new("test.wxs")).unwrap();

        for node in doc.root().descendants() {
            if node.tag_name().name() == "Component" {
                let range = doc.node_range(&node);
                assert_eq!(range.start.line, 1);
                assert!(range.start.character > 1); // After <Wix>
            }
        }
    }

    #[test]
    fn test_element_iterator() {
        let source = "<Wix><A><B /></A><C /></Wix>";
        let doc = WixDocument::parse(source, Path::new("test.wxs")).unwrap();

        let elements: Vec<_> = ElementIterator::new(doc.root())
            .map(|n| n.tag_name().name())
            .collect();

        // Should be in document order
        assert_eq!(elements, vec!["Wix", "A", "B", "C"]);
    }

    #[test]
    fn test_descendants() {
        let source = "<Wix><A><B /></A></Wix>";
        let doc = WixDocument::parse(source, Path::new("test.wxs")).unwrap();

        let wix_node = doc.root().descendants().find(|n| n.tag_name().name() == "Wix").unwrap();
        let descendants: Vec<_> = wix_node.descendants()
            .map(|n| n.tag_name().name())
            .collect();

        assert_eq!(descendants, vec!["Wix", "A", "B"]);
    }

    #[test]
    fn test_nodeext_descendants() {
        let source = "<Wix><A><B /></A></Wix>";
        let doc = WixDocument::parse(source, Path::new("test.wxs")).unwrap();

        // Use NodeExt::descendants explicitly
        for node in doc.root().descendants() {
            if node.tag_name().name() == "A" {
                let desc: Vec<_> = NodeExt::descendants(&node)
                    .map(|n| n.tag_name().name())
                    .collect();
                assert_eq!(desc, vec!["A", "B"]);
            }
        }
    }

    #[test]
    fn test_element_at_position_last_line_without_newline() {
        // Test the edge case where position is at EOF with no trailing newline
        let source = "<Wix />";  // Single line, no newline at end
        let doc = WixDocument::parse(source, Path::new("test.wxs")).unwrap();

        // Position at the end of the single line
        let result = doc.element_at(1, 7);
        assert!(result.is_some());
    }

    #[test]
    fn test_element_at_position_past_end_no_newline() {
        // Test position past end of last line when no trailing newline
        // This triggers the EOF branch (line 104-105: return Some(offset) after loop)
        let source = "<Wix />";  // 7 chars, no trailing newline
        let doc = WixDocument::parse(source, Path::new("test.wxs")).unwrap();

        // Request column 100, well past the 7 characters
        // This triggers the position_to_offset EOF path (line 105)
        // But the offset (7) is past the element's range, so no element is found
        let result = doc.element_at(1, 100);
        // No element at position past end of file
        assert!(result.is_none());
    }
}

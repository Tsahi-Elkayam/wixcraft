//! Parser for extracting documentation from WiX files

use lazy_static::lazy_static;
use regex::Regex;
use roxmltree::{Document, Node};
use std::path::Path;

use crate::types::*;

lazy_static! {
    /// Pattern for doc comments: <!-- @doc: description -->
    static ref DOC_COMMENT_PATTERN: Regex = Regex::new(r"<!--\s*@doc:\s*(.+?)\s*-->").unwrap();

    /// Pattern for property references: [PropertyName]
    static ref PROPERTY_REF_PATTERN: Regex = Regex::new(r"\[([A-Z_][A-Z0-9_]*)\]").unwrap();
}

/// Parser for WiX documentation
pub struct DocsParser {
    include_private: bool,
}

impl DocsParser {
    pub fn new() -> Self {
        Self {
            include_private: false,
        }
    }

    pub fn with_include_private(mut self, include: bool) -> Self {
        self.include_private = include;
        self
    }

    /// Parse a single WiX file
    pub fn parse_file(&self, source: &str, path: &Path) -> Result<ParsedFile, String> {
        let doc = Document::parse(source)
            .map_err(|e| format!("XML parse error in {}: {}", path.display(), e))?;

        let mut parsed = ParsedFile {
            path: path.to_path_buf(),
            description: None,
            components: Vec::new(),
            features: Vec::new(),
            directories: Vec::new(),
            custom_actions: Vec::new(),
            properties: Vec::new(),
            property_usages: Vec::new(),
        };

        // Extract file-level doc comment (first comment before root element)
        if let Some(desc) = self.extract_file_doc(source) {
            parsed.description = Some(desc);
        }

        // Parse all elements
        self.parse_node(doc.root(), source, path, &mut parsed)?;

        Ok(parsed)
    }

    fn extract_file_doc(&self, source: &str) -> Option<String> {
        // Look for doc comment at the start of the file
        let trimmed = source.trim_start();
        if let Some(caps) = DOC_COMMENT_PATTERN.captures(trimmed) {
            if trimmed.starts_with("<!--") {
                return Some(caps[1].to_string());
            }
        }
        None
    }

    fn parse_node(
        &self,
        node: Node,
        source: &str,
        path: &Path,
        parsed: &mut ParsedFile,
    ) -> Result<(), String> {
        if node.is_element() {
            let tag = node.tag_name().name();
            let line = self.get_line_number(source, node.range().start);

            // Check if should skip private elements
            if !self.include_private {
                if let Some(id) = node.attribute("Id") {
                    if id.starts_with('_') {
                        return Ok(());
                    }
                }
            }

            // Get doc comment for this element
            let doc = self.get_element_doc(source, node.range().start);

            match tag {
                "Component" => {
                    if let Some(id) = node.attribute("Id") {
                        let mut comp = ComponentDocs::new(id, path.to_path_buf(), line);
                        comp.guid = node.attribute("Guid").map(|s| s.to_string());
                        comp.description = doc;

                        // Parse child elements
                        for child in node.children().filter(|n| n.is_element()) {
                            match child.tag_name().name() {
                                "File" => {
                                    comp.files.push(FileEntry {
                                        id: child.attribute("Id").map(|s| s.to_string()),
                                        source: child.attribute("Source").map(|s| s.to_string()),
                                        name: child.attribute("Name").map(|s| s.to_string()),
                                        description: self.get_element_doc(source, child.range().start),
                                    });
                                }
                                "RegistryKey" | "RegistryValue" => {
                                    if let Some(entry) = self.parse_registry_entry(&child) {
                                        comp.registry.push(entry);
                                    }
                                }
                                "ServiceInstall" => {
                                    if let Some(entry) = self.parse_service_entry(&child) {
                                        comp.services.push(entry);
                                    }
                                }
                                _ => {}
                            }
                        }

                        parsed.components.push(comp);
                    }
                }
                "Feature" => {
                    if let Some(id) = node.attribute("Id") {
                        let mut feature = FeatureDocs::new(id, path.to_path_buf(), line);
                        feature.title = node.attribute("Title").map(|s| s.to_string());
                        feature.level = node.attribute("Level").map(|s| s.to_string());
                        feature.description = doc;

                        // Get component references
                        for child in node.children().filter(|n| n.is_element()) {
                            match child.tag_name().name() {
                                "ComponentRef" | "ComponentGroupRef" => {
                                    if let Some(ref_id) = child.attribute("Id") {
                                        feature.components.push(ref_id.to_string());
                                    }
                                }
                                "Feature" => {
                                    if let Some(child_id) = child.attribute("Id") {
                                        feature.children.push(child_id.to_string());
                                    }
                                }
                                _ => {}
                            }
                        }

                        parsed.features.push(feature);
                    }
                }
                "Directory" | "StandardDirectory" => {
                    if let Some(id) = node.attribute("Id") {
                        let mut dir = DirectoryDocs::new(id, path.to_path_buf(), line);
                        dir.name = node.attribute("Name").map(|s| s.to_string());
                        dir.description = doc;

                        // Get child directories and components
                        for child in node.children().filter(|n| n.is_element()) {
                            match child.tag_name().name() {
                                "Directory" | "StandardDirectory" => {
                                    if let Some(child_id) = child.attribute("Id") {
                                        dir.children.push(child_id.to_string());
                                    }
                                }
                                "Component" => {
                                    if let Some(comp_id) = child.attribute("Id") {
                                        dir.components.push(comp_id.to_string());
                                    }
                                }
                                _ => {}
                            }
                        }

                        parsed.directories.push(dir);
                    }
                }
                "CustomAction" => {
                    if let Some(id) = node.attribute("Id") {
                        let mut ca = CustomActionDocs::new(id, path.to_path_buf(), line);
                        ca.description = doc;
                        ca.binary = node.attribute("BinaryRef").or(node.attribute("BinaryKey")).map(|s| s.to_string());
                        ca.dll_entry = node.attribute("DllEntry").map(|s| s.to_string());
                        ca.execute = node.attribute("Execute").map(|s| s.to_string());
                        ca.return_attr = node.attribute("Return").map(|s| s.to_string());
                        ca.impersonate = node.attribute("Impersonate").map(|s| s.to_string());

                        parsed.custom_actions.push(ca);
                    }
                }
                "Property" => {
                    if let Some(id) = node.attribute("Id") {
                        let mut prop = PropertyDocs::new(id, path.to_path_buf(), line);
                        prop.value = node.attribute("Value").map(|s| s.to_string());
                        prop.description = doc;
                        prop.secure = node.attribute("Secure").map(|v| v == "yes").unwrap_or(false);
                        prop.admin = node.attribute("Admin").map(|v| v == "yes").unwrap_or(false);
                        prop.hidden = node.attribute("Hidden").map(|v| v == "yes").unwrap_or(false);

                        parsed.properties.push(prop);
                    }
                }
                "Custom" => {
                    // Custom action scheduling
                    if let Some(action) = node.attribute("Action") {
                        let sequence = self.get_parent_sequence(&node);
                        if let Some(seq) = sequence {
                            let entry = ScheduleEntry {
                                sequence: seq,
                                condition: node.text().map(|s| s.trim().to_string()).filter(|s| !s.is_empty()),
                                before: node.attribute("Before").map(|s| s.to_string()),
                                after: node.attribute("After").map(|s| s.to_string()),
                            };

                            // Find the custom action and add schedule
                            for ca in &mut parsed.custom_actions {
                                if ca.id == action {
                                    ca.scheduled_in.push(entry.clone());
                                    break;
                                }
                            }
                        }
                    }
                }
                _ => {}
            }

            // Track property usages in conditions and values
            self.track_property_usages(&node, source, path, line, parsed);
        }

        // Recurse into children
        for child in node.children() {
            self.parse_node(child, source, path, parsed)?;
        }

        Ok(())
    }

    fn get_line_number(&self, source: &str, offset: usize) -> usize {
        source[..offset.min(source.len())].matches('\n').count() + 1
    }

    fn get_element_doc(&self, source: &str, offset: usize) -> Option<String> {
        // Look for doc comment immediately before this element
        let before = &source[..offset];
        let trimmed = before.trim_end();

        // Check if the last thing before this element is a doc comment
        if let Some(comment_end) = trimmed.rfind("-->") {
            let comment_start = trimmed[..comment_end].rfind("<!--")?;
            let comment = &trimmed[comment_start..comment_end + 3];

            if let Some(caps) = DOC_COMMENT_PATTERN.captures(comment) {
                return Some(caps[1].to_string());
            }
        }

        None
    }

    fn get_parent_sequence(&self, node: &Node) -> Option<String> {
        let mut current = node.parent();
        while let Some(parent) = current {
            if parent.is_element() {
                let name = parent.tag_name().name();
                if name.ends_with("Sequence") {
                    return Some(name.to_string());
                }
            }
            current = parent.parent();
        }
        None
    }

    fn parse_registry_entry(&self, node: &Node) -> Option<RegistryEntry> {
        let root = node.attribute("Root")?;
        let key = node.attribute("Key")?;

        Some(RegistryEntry {
            root: root.to_string(),
            key: key.to_string(),
            name: node.attribute("Name").map(|s| s.to_string()),
            value_type: node.attribute("Type").map(|s| s.to_string()),
            description: None,
        })
    }

    fn parse_service_entry(&self, node: &Node) -> Option<ServiceEntry> {
        let name = node.attribute("Name")?;

        Some(ServiceEntry {
            name: name.to_string(),
            display_name: node.attribute("DisplayName").map(|s| s.to_string()),
            service_type: node.attribute("Type").map(|s| s.to_string()),
            start_type: node.attribute("Start").map(|s| s.to_string()),
            description: node.attribute("Description").map(|s| s.to_string()),
        })
    }

    fn track_property_usages(
        &self,
        node: &Node,
        _source: &str,
        path: &Path,
        line: usize,
        parsed: &mut ParsedFile,
    ) {
        let tag = node.tag_name().name();

        // Check all attributes for property references
        for attr in node.attributes() {
            let value = attr.value();
            for caps in PROPERTY_REF_PATTERN.captures_iter(value) {
                let prop_name = &caps[1];
                parsed.property_usages.push(PropertyUsage {
                    context: format!("{}/@{}", tag, attr.name()),
                    element: tag.to_string(),
                    file: path.to_path_buf(),
                    line,
                });

                // Also track which property is used (by name)
                // This will be resolved later when building ProjectDocs
                let _ = prop_name; // Used in the pattern match
            }
        }

        // Check text content for conditions
        if let Some(text) = node.text() {
            for caps in PROPERTY_REF_PATTERN.captures_iter(text) {
                let _prop_name = &caps[1];
                parsed.property_usages.push(PropertyUsage {
                    context: format!("{}/text()", tag),
                    element: tag.to_string(),
                    file: path.to_path_buf(),
                    line,
                });
            }
        }
    }
}

impl Default for DocsParser {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of parsing a single file
#[derive(Debug, Clone)]
pub struct ParsedFile {
    pub path: std::path::PathBuf,
    pub description: Option<String>,
    pub components: Vec<ComponentDocs>,
    pub features: Vec<FeatureDocs>,
    pub directories: Vec<DirectoryDocs>,
    pub custom_actions: Vec<CustomActionDocs>,
    pub properties: Vec<PropertyDocs>,
    pub property_usages: Vec<PropertyUsage>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_parser_new() {
        let parser = DocsParser::new();
        assert!(!parser.include_private);
    }

    #[test]
    fn test_parser_with_include_private() {
        let parser = DocsParser::new().with_include_private(true);
        assert!(parser.include_private);
    }

    #[test]
    fn test_parser_default() {
        let parser = DocsParser::default();
        assert!(!parser.include_private);
    }

    #[test]
    fn test_parse_component() {
        let source = r#"<Wix>
            <!-- @doc: Main application component -->
            <Component Id="MainComponent" Guid="*">
                <File Id="MainExe" Source="app.exe" />
            </Component>
        </Wix>"#;

        let parser = DocsParser::new();
        let result = parser.parse_file(source, Path::new("test.wxs")).unwrap();

        assert_eq!(result.components.len(), 1);
        assert_eq!(result.components[0].id, "MainComponent");
        assert_eq!(result.components[0].guid, Some("*".to_string()));
        assert!(result.components[0].description.is_some());
        assert_eq!(result.components[0].files.len(), 1);
    }

    #[test]
    fn test_parse_feature() {
        let source = r#"<Wix>
            <!-- @doc: Main product feature -->
            <Feature Id="MainFeature" Title="Main" Level="1">
                <ComponentRef Id="C1" />
                <ComponentRef Id="C2" />
            </Feature>
        </Wix>"#;

        let parser = DocsParser::new();
        let result = parser.parse_file(source, Path::new("test.wxs")).unwrap();

        assert_eq!(result.features.len(), 1);
        assert_eq!(result.features[0].id, "MainFeature");
        assert_eq!(result.features[0].title, Some("Main".to_string()));
        assert_eq!(result.features[0].components.len(), 2);
    }

    #[test]
    fn test_parse_directory() {
        let source = r#"<Wix>
            <!-- @doc: Installation directory -->
            <Directory Id="INSTALLDIR" Name="MyApp">
                <Directory Id="BinDir" Name="bin" />
            </Directory>
        </Wix>"#;

        let parser = DocsParser::new();
        let result = parser.parse_file(source, Path::new("test.wxs")).unwrap();

        assert_eq!(result.directories.len(), 2);
        assert_eq!(result.directories[0].id, "INSTALLDIR");
        assert_eq!(result.directories[0].children.len(), 1);
    }

    #[test]
    fn test_parse_custom_action() {
        let source = r#"<Wix>
            <!-- @doc: Runs post-install configuration -->
            <CustomAction Id="CA_Config" BinaryRef="ConfigDll" DllEntry="Configure" Execute="deferred" />
        </Wix>"#;

        let parser = DocsParser::new();
        let result = parser.parse_file(source, Path::new("test.wxs")).unwrap();

        assert_eq!(result.custom_actions.len(), 1);
        assert_eq!(result.custom_actions[0].id, "CA_Config");
        assert_eq!(result.custom_actions[0].binary, Some("ConfigDll".to_string()));
        assert_eq!(result.custom_actions[0].execute, Some("deferred".to_string()));
    }

    #[test]
    fn test_parse_property() {
        let source = r#"<Wix>
            <!-- @doc: Installation path property -->
            <Property Id="INSTALLPATH" Value="C:\Program Files\MyApp" Secure="yes" />
        </Wix>"#;

        let parser = DocsParser::new();
        let result = parser.parse_file(source, Path::new("test.wxs")).unwrap();

        assert_eq!(result.properties.len(), 1);
        assert_eq!(result.properties[0].id, "INSTALLPATH");
        assert!(result.properties[0].secure);
    }

    #[test]
    fn test_skip_private_elements() {
        let source = r#"<Wix>
            <Component Id="_PrivateComponent" Guid="*" />
            <Component Id="PublicComponent" Guid="*" />
        </Wix>"#;

        let parser = DocsParser::new();
        let result = parser.parse_file(source, Path::new("test.wxs")).unwrap();

        assert_eq!(result.components.len(), 1);
        assert_eq!(result.components[0].id, "PublicComponent");
    }

    #[test]
    fn test_include_private_elements() {
        let source = r#"<Wix>
            <Component Id="_PrivateComponent" Guid="*" />
            <Component Id="PublicComponent" Guid="*" />
        </Wix>"#;

        let parser = DocsParser::new().with_include_private(true);
        let result = parser.parse_file(source, Path::new("test.wxs")).unwrap();

        assert_eq!(result.components.len(), 2);
    }

    #[test]
    fn test_parse_file_doc() {
        let source = r#"<!-- @doc: Main WiX source file -->
<Wix>
    <Component Id="C1" Guid="*" />
</Wix>"#;

        let parser = DocsParser::new();
        let result = parser.parse_file(source, Path::new("test.wxs")).unwrap();

        assert_eq!(result.description, Some("Main WiX source file".to_string()));
    }

    #[test]
    fn test_parse_registry() {
        let source = r#"<Wix>
            <Component Id="RegComp" Guid="*">
                <RegistryKey Root="HKLM" Key="SOFTWARE\MyApp">
                    <RegistryValue Name="Version" Type="string" Value="1.0" />
                </RegistryKey>
            </Component>
        </Wix>"#;

        let parser = DocsParser::new();
        let result = parser.parse_file(source, Path::new("test.wxs")).unwrap();

        assert_eq!(result.components.len(), 1);
        assert!(!result.components[0].registry.is_empty());
    }

    #[test]
    fn test_parse_service() {
        let source = r#"<Wix>
            <Component Id="ServiceComp" Guid="*">
                <ServiceInstall Name="MyService" DisplayName="My Service" Type="ownProcess" Start="auto" />
            </Component>
        </Wix>"#;

        let parser = DocsParser::new();
        let result = parser.parse_file(source, Path::new("test.wxs")).unwrap();

        assert_eq!(result.components.len(), 1);
        assert_eq!(result.components[0].services.len(), 1);
        assert_eq!(result.components[0].services[0].name, "MyService");
    }

    #[test]
    fn test_parse_nested_features() {
        let source = r#"<Wix>
            <Feature Id="ParentFeature" Level="1">
                <Feature Id="ChildFeature" Level="2" />
            </Feature>
        </Wix>"#;

        let parser = DocsParser::new();
        let result = parser.parse_file(source, Path::new("test.wxs")).unwrap();

        assert_eq!(result.features.len(), 2);
        assert!(result.features[0].children.contains(&"ChildFeature".to_string()));
    }

    #[test]
    fn test_parse_standard_directory() {
        let source = r#"<Wix>
            <StandardDirectory Id="ProgramFilesFolder">
                <Directory Id="INSTALLDIR" Name="MyApp" />
            </StandardDirectory>
        </Wix>"#;

        let parser = DocsParser::new();
        let result = parser.parse_file(source, Path::new("test.wxs")).unwrap();

        assert_eq!(result.directories.len(), 2);
    }

    #[test]
    fn test_parse_invalid_xml() {
        let source = r#"<Wix><Invalid"#;

        let parser = DocsParser::new();
        let result = parser.parse_file(source, Path::new("test.wxs"));

        assert!(result.is_err());
    }

    #[test]
    fn test_get_line_number() {
        let parser = DocsParser::new();
        let source = "line1\nline2\nline3";

        assert_eq!(parser.get_line_number(source, 0), 1);
        assert_eq!(parser.get_line_number(source, 6), 2);
        assert_eq!(parser.get_line_number(source, 12), 3);
    }

    #[test]
    fn test_component_group_ref() {
        let source = r#"<Wix>
            <Feature Id="MainFeature" Level="1">
                <ComponentGroupRef Id="CG1" />
            </Feature>
        </Wix>"#;

        let parser = DocsParser::new();
        let result = parser.parse_file(source, Path::new("test.wxs")).unwrap();

        assert_eq!(result.features[0].components.len(), 1);
        assert!(result.features[0].components.contains(&"CG1".to_string()));
    }

    #[test]
    fn test_parse_custom_action_with_binary_key() {
        let source = r#"<Wix>
            <CustomAction Id="CA_Test" BinaryKey="TestDll" DllEntry="Entry" Return="check" Impersonate="no" />
        </Wix>"#;

        let parser = DocsParser::new();
        let result = parser.parse_file(source, Path::new("test.wxs")).unwrap();

        assert_eq!(result.custom_actions.len(), 1);
        assert_eq!(result.custom_actions[0].binary, Some("TestDll".to_string()));
        assert_eq!(result.custom_actions[0].return_attr, Some("check".to_string()));
        assert_eq!(result.custom_actions[0].impersonate, Some("no".to_string()));
    }

    #[test]
    fn test_parse_property_flags() {
        let source = r#"<Wix>
            <Property Id="ADMINPROP" Admin="yes" Hidden="yes" />
        </Wix>"#;

        let parser = DocsParser::new();
        let result = parser.parse_file(source, Path::new("test.wxs")).unwrap();

        assert_eq!(result.properties.len(), 1);
        assert!(result.properties[0].admin);
        assert!(result.properties[0].hidden);
    }

    #[test]
    fn test_parse_file_with_name() {
        let source = r#"<Wix>
            <Component Id="C1" Guid="*">
                <File Id="F1" Source="source.exe" Name="target.exe" />
            </Component>
        </Wix>"#;

        let parser = DocsParser::new();
        let result = parser.parse_file(source, Path::new("test.wxs")).unwrap();

        assert_eq!(result.components[0].files[0].name, Some("target.exe".to_string()));
    }

    #[test]
    fn test_parse_registry_value_direct() {
        let source = r#"<Wix>
            <Component Id="C1" Guid="*">
                <RegistryValue Root="HKCU" Key="Software\Test" Name="Value" Type="integer" Value="1" />
            </Component>
        </Wix>"#;

        let parser = DocsParser::new();
        let result = parser.parse_file(source, Path::new("test.wxs")).unwrap();

        assert_eq!(result.components[0].registry.len(), 1);
        assert_eq!(result.components[0].registry[0].root, "HKCU");
    }

    #[test]
    fn test_parse_directory_with_components() {
        let source = r#"<Wix>
            <Directory Id="INSTALLDIR" Name="App">
                <Component Id="C1" Guid="*" />
                <Component Id="C2" Guid="*" />
            </Directory>
        </Wix>"#;

        let parser = DocsParser::new();
        let result = parser.parse_file(source, Path::new("test.wxs")).unwrap();

        assert_eq!(result.directories[0].components.len(), 2);
    }

    #[test]
    fn test_parse_custom_action_schedule() {
        let source = r#"<Wix>
            <CustomAction Id="CA_Test" Execute="immediate" />
            <InstallExecuteSequence>
                <Custom Action="CA_Test" Before="InstallFinalize">NOT Installed</Custom>
            </InstallExecuteSequence>
        </Wix>"#;

        let parser = DocsParser::new();
        let result = parser.parse_file(source, Path::new("test.wxs")).unwrap();

        assert_eq!(result.custom_actions.len(), 1);
        // Note: schedule parsing happens after CA is parsed
    }

    #[test]
    fn test_property_reference_in_attribute() {
        let source = r#"<Wix>
            <Component Id="C1" Directory="[INSTALLDIR]" Guid="*" />
        </Wix>"#;

        let parser = DocsParser::new();
        let result = parser.parse_file(source, Path::new("test.wxs")).unwrap();

        assert!(!result.property_usages.is_empty());
    }
}

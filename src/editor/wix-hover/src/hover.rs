//! Core hover logic

use crate::context::detect_hover_target;
use crate::loader::WixData;
use crate::types::{HoverInfo, HoverTarget};

/// Hover information provider
pub struct HoverProvider {
    data: WixData,
}

impl HoverProvider {
    /// Create a new hover provider with loaded wix-data
    pub fn new(data: WixData) -> Self {
        Self { data }
    }

    /// Get hover information for a position in the source
    pub fn hover(&self, source: &str, line: u32, column: u32) -> Option<HoverInfo> {
        let target = detect_hover_target(source, line, column);

        match target {
            HoverTarget::Element { name, range } => {
                self.hover_element(&name).map(|info| info.with_range(range))
            }

            HoverTarget::AttributeName {
                element,
                attribute,
                range,
            } => self
                .hover_attribute(&element, &attribute)
                .map(|info| info.with_range(range)),

            HoverTarget::AttributeValue {
                element,
                attribute,
                value,
                range,
            } => self
                .hover_value(&element, &attribute, &value)
                .map(|info| info.with_range(range)),

            HoverTarget::None => None,
        }
    }

    /// Generate hover content for an element
    fn hover_element(&self, name: &str) -> Option<HoverInfo> {
        let elem = self.data.get_element(name)?;

        let mut content = String::new();

        // Header
        content.push_str(&format!("### {}\n\n", elem.name));

        // Description
        if !elem.description.is_empty() {
            content.push_str(&elem.description);
            content.push_str("\n\n");
        }

        // Since version
        if let Some(ref since) = elem.since {
            content.push_str(&format!("**Since:** {}\n\n", since));
        }

        // Parents
        if !elem.parents.is_empty() {
            let parents = elem.parents.join(", ");
            content.push_str(&format!("**Parents:** {}\n\n", parents));
        }

        // Children
        if !elem.children.is_empty() {
            let children: Vec<_> = elem.children.iter().take(10).cloned().collect();
            let mut children_str = children.join(", ");
            if elem.children.len() > 10 {
                children_str.push_str(", ...");
            }
            content.push_str(&format!("**Children:** {}\n\n", children_str));
        }

        // Documentation link
        if let Some(ref url) = elem.documentation {
            content.push_str(&format!("[Documentation]({})\n", url));
        }

        Some(HoverInfo::new(content.trim().to_string()))
    }

    /// Generate hover content for an attribute
    fn hover_attribute(&self, element: &str, attribute: &str) -> Option<HoverInfo> {
        let attr = self.data.get_attribute(element, attribute)?;

        let mut content = String::new();

        // Header
        content.push_str(&format!("### {} ({})\n\n", attribute, element));

        // Type
        if !attr.attr_type.is_empty() {
            content.push_str(&format!("**Type:** {}\n\n", format_type(&attr.attr_type)));
        }

        // Required
        content.push_str(&format!(
            "**Required:** {}\n\n",
            if attr.required { "Yes" } else { "No" }
        ));

        // Description
        if !attr.description.is_empty() {
            content.push_str(&attr.description);
            content.push_str("\n\n");
        }

        // Default value
        if let Some(ref default) = attr.default {
            content.push_str(&format!("**Default:** `{}`\n\n", default));
        }

        // Enum values
        if let Some(ref values) = attr.values {
            content.push_str("**Values:**\n");
            for v in values {
                content.push_str(&format!("- `{}`\n", v));
            }
        }

        Some(HoverInfo::new(content.trim().to_string()))
    }

    /// Generate hover content for a value
    fn hover_value(&self, _element: &str, _attribute: &str, value: &str) -> Option<HoverInfo> {
        // Check standard directories
        if self.data.is_standard_directory(value) {
            return Some(self.hover_standard_directory(value));
        }

        // Check builtin properties
        if self.data.is_builtin_property(value) {
            return Some(self.hover_builtin_property(value));
        }

        // Check GUID format
        if value == "*" {
            return Some(HoverInfo::new(
                "### Auto-generated GUID\n\n\
                 The `*` value tells WiX to automatically generate a unique GUID \
                 at build time.\n\n\
                 This is the recommended approach for Component GUIDs."
                    .to_string(),
            ));
        }

        None
    }

    /// Hover info for standard directories
    fn hover_standard_directory(&self, name: &str) -> HoverInfo {
        let description = match name {
            "TARGETDIR" => "The root destination directory. Required for all installations.",
            "ProgramFilesFolder" => "The Program Files folder (e.g., `C:\\Program Files`).",
            "ProgramFiles64Folder" => "The 64-bit Program Files folder.",
            "CommonFilesFolder" => "The Common Files folder.",
            "CommonFiles64Folder" => "The 64-bit Common Files folder.",
            "SystemFolder" => "The Windows System32 folder.",
            "System64Folder" => "The 64-bit System folder.",
            "WindowsFolder" => "The Windows directory (e.g., `C:\\Windows`).",
            "TempFolder" => "The temporary files folder.",
            "LocalAppDataFolder" => "The local application data folder.",
            "AppDataFolder" => "The roaming application data folder.",
            "CommonAppDataFolder" => "The common application data folder (ProgramData).",
            "DesktopFolder" => "The user's desktop folder.",
            "StartMenuFolder" => "The Start Menu folder.",
            "ProgramMenuFolder" => "The Programs folder in the Start Menu.",
            "StartupFolder" => "The Startup folder.",
            "FontsFolder" => "The Fonts folder.",
            _ => "Standard Windows Installer directory.",
        };

        HoverInfo::new(format!(
            "### {} (Standard Directory)\n\n{}",
            name, description
        ))
    }

    /// Hover info for builtin properties
    fn hover_builtin_property(&self, name: &str) -> HoverInfo {
        let description = match name {
            "ProductName" => "The name of the product being installed.",
            "ProductCode" => "The unique GUID identifying this product.",
            "ProductVersion" => "The version of the product.",
            "Manufacturer" => "The manufacturer of the product.",
            "UpgradeCode" => "The GUID used for major upgrades.",
            "ProductLanguage" => "The language ID of the product.",
            "INSTALLFOLDER" => "The default installation directory (custom property).",
            "INSTALLDIR" => "Common name for the installation directory.",
            _ => "Built-in Windows Installer property.",
        };

        HoverInfo::new(format!(
            "### {} (Built-in Property)\n\n{}",
            name, description
        ))
    }
}

/// Format attribute type for display
fn format_type(type_name: &str) -> String {
    match type_name {
        "identifier" => "Identifier".to_string(),
        "guid" => "GUID".to_string(),
        "yesno" => "Yes/No".to_string(),
        "integer" => "Integer".to_string(),
        "string" => "String".to_string(),
        "path" => "Path".to_string(),
        "version" => "Version".to_string(),
        "enum" => "Enumeration".to_string(),
        _ => type_name.to_string(),
    }
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

        let component = r#"{
            "name": "Component",
            "description": "A component is a grouping of resources.",
            "documentation": "https://wixtoolset.org/docs/schema/wxs/component/",
            "since": "v3",
            "parents": ["Directory", "DirectoryRef"],
            "children": ["File", "RegistryKey", "ServiceInstall"],
            "attributes": {
                "Id": {"type": "identifier", "required": false, "description": "Component identifier"},
                "Guid": {"type": "guid", "required": true, "description": "Component GUID"},
                "Transitive": {"type": "yesno", "default": false, "description": "Whether transitive"}
            }
        }"#;
        fs::write(elements_dir.join("component.json"), component).unwrap();

        let feature = r#"{
            "name": "Feature",
            "description": "A feature for installation.",
            "parents": ["Package", "Feature"],
            "children": ["ComponentRef", "Feature"],
            "attributes": {
                "Id": {"type": "identifier", "required": true},
                "Display": {"type": "enum", "values": ["expand", "collapse", "hidden"], "default": "expand"}
            }
        }"#;
        fs::write(elements_dir.join("feature.json"), feature).unwrap();

        let keywords_dir = temp.path().join("keywords");
        fs::create_dir(&keywords_dir).unwrap();
        let keywords = r#"{
            "standardDirectories": ["ProgramFilesFolder", "SystemFolder", "TARGETDIR"],
            "builtinProperties": ["ProductName", "ProductVersion"],
            "elements": [],
            "preprocessorDirectives": []
        }"#;
        fs::write(keywords_dir.join("keywords.json"), keywords).unwrap();

        let data = WixData::load(temp.path()).unwrap();
        (temp, data)
    }

    #[test]
    fn test_hover_element() {
        let (_temp, data) = create_test_data();
        let provider = HoverProvider::new(data);

        let source = "<Component Guid=\"*\" />";
        let info = provider.hover(source, 1, 3).unwrap();

        assert!(info.contents.contains("### Component"));
        assert!(info.contents.contains("grouping of resources"));
        assert!(info.contents.contains("**Since:** v3"));
        assert!(info.contents.contains("**Parents:**"));
        assert!(info.contents.contains("**Children:**"));
        assert!(info.contents.contains("Documentation"));
    }

    #[test]
    fn test_hover_attribute() {
        let (_temp, data) = create_test_data();
        let provider = HoverProvider::new(data);

        let source = "<Component Guid=\"*\" />";
        let info = provider.hover(source, 1, 13).unwrap();

        assert!(info.contents.contains("### Guid (Component)"));
        assert!(info.contents.contains("**Type:** GUID"));
        assert!(info.contents.contains("**Required:** Yes"));
    }

    #[test]
    fn test_hover_enum_attribute() {
        let (_temp, data) = create_test_data();
        let provider = HoverProvider::new(data);

        let source = "<Feature Display=\"expand\" />";
        let info = provider.hover(source, 1, 11).unwrap();

        assert!(info.contents.contains("**Type:** Enumeration"));
        assert!(info.contents.contains("**Values:**"));
        assert!(info.contents.contains("`expand`"));
        assert!(info.contents.contains("`collapse`"));
    }

    #[test]
    fn test_hover_standard_directory() {
        let (_temp, data) = create_test_data();
        let provider = HoverProvider::new(data);

        let source = "<Directory Id=\"ProgramFilesFolder\" />";
        let info = provider.hover(source, 1, 20).unwrap();

        assert!(info.contents.contains("ProgramFilesFolder"));
        assert!(info.contents.contains("Standard Directory"));
        assert!(info.contents.contains("Program Files"));
    }

    #[test]
    fn test_hover_auto_guid() {
        let (_temp, data) = create_test_data();
        let provider = HoverProvider::new(data);

        let source = "<Component Guid=\"*\" />";
        let info = provider.hover(source, 1, 18).unwrap();

        assert!(info.contents.contains("Auto-generated GUID"));
    }

    #[test]
    fn test_hover_builtin_property() {
        let (_temp, data) = create_test_data();
        let provider = HoverProvider::new(data);

        let source = "<Property Id=\"ProductName\" />";
        let info = provider.hover(source, 1, 18).unwrap();

        assert!(info.contents.contains("ProductName"));
        assert!(info.contents.contains("Built-in Property"));
    }

    #[test]
    fn test_hover_unknown_element() {
        let (_temp, data) = create_test_data();
        let provider = HoverProvider::new(data);

        let source = "<Unknown />";
        let info = provider.hover(source, 1, 3);

        assert!(info.is_none());
    }

    #[test]
    fn test_hover_unknown_attribute() {
        let (_temp, data) = create_test_data();
        let provider = HoverProvider::new(data);

        let source = "<Component Unknown=\"x\" />";
        let info = provider.hover(source, 1, 13);

        assert!(info.is_none());
    }

    #[test]
    fn test_hover_outside_tag() {
        let (_temp, data) = create_test_data();
        let provider = HoverProvider::new(data);

        let source = "<Component />   ";
        let info = provider.hover(source, 1, 16);

        assert!(info.is_none());
    }

    #[test]
    fn test_format_type() {
        assert_eq!(format_type("guid"), "GUID");
        assert_eq!(format_type("yesno"), "Yes/No");
        assert_eq!(format_type("identifier"), "Identifier");
        assert_eq!(format_type("custom"), "custom");
    }

    #[test]
    fn test_hover_has_range() {
        let (_temp, data) = create_test_data();
        let provider = HoverProvider::new(data);

        let source = "<Component />";
        let info = provider.hover(source, 1, 3).unwrap();

        assert!(info.range.is_some());
        let range = info.range.unwrap();
        assert_eq!(range.start_line, 1);
    }

    #[test]
    fn test_hover_attribute_with_default() {
        let (_temp, data) = create_test_data();
        let provider = HoverProvider::new(data);

        let source = "<Feature Display=\"expand\" />";
        let info = provider.hover(source, 1, 11).unwrap();

        assert!(info.contents.contains("**Default:** `expand`"));
    }
}

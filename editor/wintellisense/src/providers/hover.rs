//! Hover information provider

use crate::loader::SchemaData;
use crate::types::{CursorContext, HoverInfo, HoverResult};

/// Get hover information at cursor
pub fn get_hover_info(schema: &SchemaData, ctx: &CursorContext, _source: &str) -> HoverResult {
    // Element hover
    if let Some(ref element) = ctx.current_element {
        // If we're on an attribute, show attribute info
        if let Some(ref attr_name) = ctx.current_attribute {
            if let Some(elem) = schema.get_element(element) {
                if let Some(attr) = elem.attributes.get(attr_name) {
                    return HoverResult::some(format_attribute_hover(
                        element, attr_name, attr,
                    ));
                }
            }
        }

        // Show element info
        if !ctx.in_attribute_value {
            if let Some(elem) = schema.get_element(element) {
                return HoverResult::some(format_element_hover(elem));
            }
        }
    }

    // Check if hovering on a word that matches a standard directory
    if let Some(ref word) = ctx.word_at_cursor {
        // Standard directory hover
        if schema.keywords.standard_directories.contains(word) {
            return HoverResult::some(format_directory_hover(word));
        }

        // Builtin property hover
        if schema.keywords.builtin_properties.contains(word) {
            return HoverResult::some(format_property_hover(word));
        }
    }

    HoverResult::none()
}

fn format_element_hover(elem: &crate::types::ElementDef) -> HoverInfo {
    let mut content = format!("## {}\n\n", elem.name);

    if !elem.description.is_empty() {
        content.push_str(&elem.description);
        content.push_str("\n\n");
    }

    if let Some(ref doc) = elem.documentation {
        content.push_str(doc);
        content.push_str("\n\n");
    }

    // Parents
    if !elem.parents.is_empty() {
        content.push_str("**Parents:** ");
        content.push_str(&elem.parents.join(", "));
        content.push_str("\n\n");
    }

    // Children
    if !elem.children.is_empty() {
        content.push_str("**Children:** ");
        content.push_str(&elem.children.join(", "));
        content.push_str("\n\n");
    }

    // Required attributes
    let required: Vec<_> = elem
        .attributes
        .iter()
        .filter(|(_, def)| def.required)
        .map(|(name, _)| name.as_str())
        .collect();

    if !required.is_empty() {
        content.push_str("**Required attributes:** ");
        content.push_str(&required.join(", "));
        content.push_str("\n\n");
    }

    // Link to docs
    content.push_str(&format!(
        "[WiX Documentation](https://wixtoolset.org/docs/schema/wxs/{})",
        elem.name.to_lowercase()
    ));

    HoverInfo::new(content)
}

fn format_attribute_hover(
    element: &str,
    attr_name: &str,
    attr: &crate::types::AttributeDef,
) -> HoverInfo {
    let mut content = format!("## {}.{}\n\n", element, attr_name);

    if !attr.description.is_empty() {
        content.push_str(&attr.description);
        content.push_str("\n\n");
    }

    // Type
    content.push_str(&format!("**Type:** `{}`\n\n", attr.attr_type));

    // Required
    if attr.required {
        content.push_str("**Required:** Yes\n\n");
    }

    // Default value
    if let Some(ref default) = attr.default {
        content.push_str(&format!("**Default:** `{}`\n\n", default));
    }

    // Allowed values
    if let Some(ref values) = attr.values {
        if !values.is_empty() {
            content.push_str("**Values:**\n");
            for value in values {
                content.push_str(&format!("- `{}`\n", value));
            }
            content.push('\n');
        }
    }

    HoverInfo::new(content)
}

fn format_directory_hover(dir: &str) -> HoverInfo {
    let description = match dir {
        "TARGETDIR" => "Root installation directory",
        "ProgramFilesFolder" => "Program Files folder (32-bit on 32-bit OS, 32-bit on 64-bit OS)",
        "ProgramFiles64Folder" => "Program Files folder (64-bit)",
        "ProgramFiles6432Folder" => "Program Files folder (32-bit or 64-bit depending on package)",
        "CommonFilesFolder" => "Common Files folder",
        "CommonFiles64Folder" => "Common Files folder (64-bit)",
        "ProgramMenuFolder" => "Start Menu Programs folder",
        "StartMenuFolder" => "Start Menu folder",
        "StartupFolder" => "Startup folder",
        "DesktopFolder" => "Desktop folder",
        "AppDataFolder" => "Application Data folder (roaming)",
        "LocalAppDataFolder" => "Local Application Data folder",
        "TempFolder" => "Temporary folder",
        "SystemFolder" => "System32 folder (32-bit)",
        "System64Folder" => "System32 folder (64-bit)",
        "WindowsFolder" => "Windows folder",
        "AdminToolsFolder" => "Administrative Tools folder",
        "FontsFolder" => "Fonts folder",
        "FavoritesFolder" => "Favorites folder",
        "PersonalFolder" => "Personal (My Documents) folder",
        "SendToFolder" => "SendTo folder",
        "TemplateFolder" => "Templates folder",
        _ => "Standard Windows Installer directory",
    };

    HoverInfo::new(format!(
        "## {}\n\n{}\n\n**Type:** Standard Directory\n\n[MSI Documentation](https://learn.microsoft.com/windows/win32/msi/property-reference)",
        dir, description
    ))
}

fn format_property_hover(prop: &str) -> HoverInfo {
    let description = match prop {
        "ProductCode" => "GUID uniquely identifying this product",
        "ProductName" => "Human-readable product name",
        "ProductVersion" => "Product version string (major.minor.build)",
        "Manufacturer" => "Company or individual publishing the product",
        "UpgradeCode" => "GUID shared by all versions for upgrade detection",
        "INSTALLFOLDER" => "Primary installation folder",
        "TARGETDIR" => "Root installation directory",
        "INSTALLDIR" => "Main installation directory (common convention)",
        "REINSTALL" => "Features to reinstall",
        "REINSTALLMODE" => "Reinstallation mode flags",
        "REBOOT" => "Reboot behavior control",
        "ADDLOCAL" => "Features to install locally",
        "REMOVE" => "Features to remove",
        "ALLUSERS" => "Per-machine (1) or per-user (empty) installation",
        "ARPPRODUCTICON" => "Product icon for Add/Remove Programs",
        "ARPHELPLINK" => "Support URL for Add/Remove Programs",
        _ => "Windows Installer property",
    };

    HoverInfo::new(format!(
        "## {}\n\n{}\n\n**Type:** Builtin Property\n\n[MSI Properties](https://learn.microsoft.com/windows/win32/msi/property-reference)",
        prop, description
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{AttributeDef, ElementDef};
    use std::collections::HashMap;

    fn create_test_schema() -> SchemaData {
        let mut elements = HashMap::new();
        let mut attrs = HashMap::new();

        attrs.insert(
            "Id".to_string(),
            AttributeDef {
                attr_type: "identifier".to_string(),
                required: true,
                description: "Component identifier".to_string(),
                ..Default::default()
            },
        );

        attrs.insert(
            "Guid".to_string(),
            AttributeDef {
                attr_type: "guid".to_string(),
                required: false,
                description: "Component GUID".to_string(),
                default: Some("*".to_string()),
                ..Default::default()
            },
        );

        elements.insert(
            "Component".to_string(),
            ElementDef {
                name: "Component".to_string(),
                description: "A component is a logical grouping of files and registry keys.".to_string(),
                parents: vec!["Package".to_string(), "Fragment".to_string()],
                children: vec!["File".to_string(), "RegistryKey".to_string()],
                attributes: attrs,
                ..Default::default()
            },
        );

        SchemaData {
            elements,
            ..Default::default()
        }
    }

    #[test]
    fn test_element_hover() {
        let schema = create_test_schema();
        let ctx = CursorContext {
            current_element: Some("Component".to_string()),
            ..Default::default()
        };

        let result = get_hover_info(&schema, &ctx, "");
        assert!(result.info.is_some());

        let info = result.info.unwrap();
        assert!(info.content.contains("## Component"));
        assert!(info.content.contains("logical grouping"));
    }

    #[test]
    fn test_attribute_hover() {
        let schema = create_test_schema();
        let ctx = CursorContext {
            current_element: Some("Component".to_string()),
            current_attribute: Some("Guid".to_string()),
            in_opening_tag: true,
            ..Default::default()
        };

        let result = get_hover_info(&schema, &ctx, "");
        assert!(result.info.is_some());

        let info = result.info.unwrap();
        assert!(info.content.contains("Component.Guid"));
        assert!(info.content.contains("guid"));
    }

    #[test]
    fn test_no_hover() {
        let schema = SchemaData::default();
        let ctx = CursorContext::default();

        let result = get_hover_info(&schema, &ctx, "");
        assert!(result.info.is_none());
    }

    #[test]
    fn test_directory_hover() {
        let mut schema = SchemaData::default();
        schema.keywords.standard_directories = vec!["ProgramFilesFolder".to_string()];

        let ctx = CursorContext {
            word_at_cursor: Some("ProgramFilesFolder".to_string()),
            ..Default::default()
        };

        let result = get_hover_info(&schema, &ctx, "");
        assert!(result.info.is_some());
        assert!(result.info.unwrap().content.contains("Program Files"));
    }
}

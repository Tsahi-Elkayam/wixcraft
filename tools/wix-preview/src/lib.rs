//! wix-preview - Preview installer layout without building
//!
//! Extracts and displays:
//! - File/directory layout
//! - Feature tree
//! - Registry entries
//! - Shortcuts
//! - Services

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Directory entry in the preview
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectoryEntry {
    pub id: String,
    pub name: String,
    pub path: String,
    pub children: Vec<DirectoryEntry>,
    pub files: Vec<FileEntry>,
}

/// File entry in the preview
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    pub id: String,
    pub name: String,
    pub source: String,
    pub key_path: bool,
    pub size: Option<u64>,
}

/// Registry entry preview
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryEntry {
    pub root: String,
    pub key: String,
    pub name: Option<String>,
    pub value: Option<String>,
    pub value_type: String,
}

/// Shortcut preview
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortcutEntry {
    pub id: String,
    pub name: String,
    pub target: String,
    pub directory: String,
    pub icon: Option<String>,
}

/// Service preview
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceEntry {
    pub id: String,
    pub name: String,
    pub display_name: Option<String>,
    pub start_type: String,
    pub account: Option<String>,
}

/// Feature preview
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureEntry {
    pub id: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub level: u32,
    pub children: Vec<FeatureEntry>,
    pub components: Vec<String>,
}

/// Complete installation preview
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct InstallPreview {
    pub product_name: Option<String>,
    pub product_version: Option<String>,
    pub manufacturer: Option<String>,
    pub directories: Vec<DirectoryEntry>,
    pub features: Vec<FeatureEntry>,
    pub registry: Vec<RegistryEntry>,
    pub shortcuts: Vec<ShortcutEntry>,
    pub services: Vec<ServiceEntry>,
    pub total_files: usize,
    pub total_size_estimate: u64,
}

/// Standard Windows directories
pub fn get_standard_directory(id: &str) -> Option<&'static str> {
    match id {
        "ProgramFilesFolder" => Some("C:\\Program Files"),
        "ProgramFiles64Folder" => Some("C:\\Program Files"),
        "ProgramFiles86Folder" => Some("C:\\Program Files (x86)"),
        "CommonFilesFolder" => Some("C:\\Program Files\\Common Files"),
        "SystemFolder" => Some("C:\\Windows\\System32"),
        "System64Folder" => Some("C:\\Windows\\System32"),
        "WindowsFolder" => Some("C:\\Windows"),
        "TempFolder" => Some("C:\\Users\\<user>\\AppData\\Local\\Temp"),
        "LocalAppDataFolder" => Some("C:\\Users\\<user>\\AppData\\Local"),
        "AppDataFolder" => Some("C:\\Users\\<user>\\AppData\\Roaming"),
        "CommonAppDataFolder" => Some("C:\\ProgramData"),
        "DesktopFolder" => Some("C:\\Users\\<user>\\Desktop"),
        "StartMenuFolder" => Some("C:\\Users\\<user>\\AppData\\Roaming\\Microsoft\\Windows\\Start Menu"),
        "ProgramMenuFolder" => Some("C:\\Users\\<user>\\AppData\\Roaming\\Microsoft\\Windows\\Start Menu\\Programs"),
        "StartupFolder" => Some("C:\\Users\\<user>\\AppData\\Roaming\\Microsoft\\Windows\\Start Menu\\Programs\\Startup"),
        "TARGETDIR" => Some("Installation Root"),
        "INSTALLDIR" => Some("[Installation Directory]"),
        _ => None,
    }
}

/// Preview generator
pub struct PreviewGenerator;

impl PreviewGenerator {
    /// Generate preview from WiX source
    pub fn generate(content: &str) -> InstallPreview {
        let mut preview = InstallPreview::default();

        if let Ok(doc) = roxmltree::Document::parse(content) {
            // Extract product info
            for node in doc.descendants() {
                match node.tag_name().name() {
                    "Package" | "Product" => {
                        preview.product_name = node.attribute("Name").map(String::from);
                        preview.product_version = node.attribute("Version").map(String::from);
                        preview.manufacturer = node.attribute("Manufacturer").map(String::from);
                    }
                    _ => {}
                }
            }

            // Build directory structure
            let mut dir_map: HashMap<String, DirectoryEntry> = HashMap::new();

            for node in doc.descendants() {
                match node.tag_name().name() {
                    "Directory" | "StandardDirectory" => {
                        let id = node.attribute("Id").unwrap_or("").to_string();
                        let name = node
                            .attribute("Name")
                            .map(String::from)
                            .unwrap_or_else(|| {
                                get_standard_directory(&id)
                                    .unwrap_or(&id)
                                    .to_string()
                            });

                        dir_map.insert(
                            id.clone(),
                            DirectoryEntry {
                                id: id.clone(),
                                name,
                                path: String::new(),
                                children: Vec::new(),
                                files: Vec::new(),
                            },
                        );
                    }
                    _ => {}
                }
            }

            // Extract files
            for node in doc.descendants() {
                if node.tag_name().name() == "File" {
                    let file = FileEntry {
                        id: node.attribute("Id").unwrap_or("").to_string(),
                        name: node
                            .attribute("Name")
                            .or_else(|| node.attribute("Source"))
                            .unwrap_or("")
                            .to_string(),
                        source: node.attribute("Source").unwrap_or("").to_string(),
                        key_path: node.attribute("KeyPath") == Some("yes"),
                        size: None,
                    };

                    preview.total_files += 1;

                    // Find parent directory
                    if let Some(parent) = find_parent_directory(&node) {
                        if let Some(dir) = dir_map.get_mut(parent) {
                            dir.files.push(file);
                        }
                    }
                }
            }

            // Extract features
            for node in doc.descendants() {
                if node.tag_name().name() == "Feature" {
                    if node.parent().map(|p| p.tag_name().name()) != Some("Feature") {
                        let feature = extract_feature(&node);
                        preview.features.push(feature);
                    }
                }
            }

            // Extract registry
            for node in doc.descendants() {
                if node.tag_name().name() == "RegistryKey" || node.tag_name().name() == "RegistryValue" {
                    let root = node.attribute("Root").unwrap_or("HKLM").to_string();
                    let key = node.attribute("Key").unwrap_or("").to_string();
                    let name = node.attribute("Name").map(String::from);
                    let value = node.attribute("Value").map(String::from);
                    let value_type = node.attribute("Type").unwrap_or("string").to_string();

                    preview.registry.push(RegistryEntry {
                        root,
                        key,
                        name,
                        value,
                        value_type,
                    });
                }
            }

            // Extract shortcuts
            for node in doc.descendants() {
                if node.tag_name().name() == "Shortcut" {
                    preview.shortcuts.push(ShortcutEntry {
                        id: node.attribute("Id").unwrap_or("").to_string(),
                        name: node.attribute("Name").unwrap_or("").to_string(),
                        target: node.attribute("Target").unwrap_or("[INSTALLDIR]").to_string(),
                        directory: node.attribute("Directory").unwrap_or("").to_string(),
                        icon: node.attribute("Icon").map(String::from),
                    });
                }
            }

            // Extract services
            for node in doc.descendants() {
                if node.tag_name().name() == "ServiceInstall" {
                    preview.services.push(ServiceEntry {
                        id: node.attribute("Id").unwrap_or("").to_string(),
                        name: node.attribute("Name").unwrap_or("").to_string(),
                        display_name: node.attribute("DisplayName").map(String::from),
                        start_type: node.attribute("Start").unwrap_or("auto").to_string(),
                        account: node.attribute("Account").map(String::from),
                    });
                }
            }

            // Convert dir_map to preview.directories
            for dir in dir_map.into_values() {
                preview.directories.push(dir);
            }
        }

        preview
    }

    /// Generate tree view of directories
    pub fn generate_tree(preview: &InstallPreview) -> String {
        let mut output = String::new();

        output.push_str(&format!(
            "{} v{}\n",
            preview.product_name.as_deref().unwrap_or("Unknown Product"),
            preview.product_version.as_deref().unwrap_or("0.0.0")
        ));
        output.push_str(&format!(
            "Manufacturer: {}\n",
            preview.manufacturer.as_deref().unwrap_or("Unknown")
        ));
        output.push_str(&format!("Total Files: {}\n\n", preview.total_files));

        output.push_str("Directory Structure:\n");
        output.push_str(&"=".repeat(50));
        output.push('\n');

        for dir in &preview.directories {
            output.push_str(&format_directory(dir, 0));
        }

        if !preview.shortcuts.is_empty() {
            output.push_str("\nShortcuts:\n");
            output.push_str(&"=".repeat(50));
            output.push('\n');
            for shortcut in &preview.shortcuts {
                output.push_str(&format!("  {} -> {}\n", shortcut.name, shortcut.target));
            }
        }

        if !preview.services.is_empty() {
            output.push_str("\nServices:\n");
            output.push_str(&"=".repeat(50));
            output.push('\n');
            for service in &preview.services {
                output.push_str(&format!(
                    "  {} ({}) - Start: {}\n",
                    service.name,
                    service.display_name.as_deref().unwrap_or(""),
                    service.start_type
                ));
            }
        }

        output
    }
}

fn find_parent_directory<'a>(node: &roxmltree::Node<'a, 'a>) -> Option<&'a str> {
    let mut current = node.parent();
    while let Some(parent) = current {
        if parent.tag_name().name() == "Directory" || parent.tag_name().name() == "DirectoryRef" {
            return parent.attribute("Id");
        }
        current = parent.parent();
    }
    None
}

fn extract_feature(node: &roxmltree::Node) -> FeatureEntry {
    let mut feature = FeatureEntry {
        id: node.attribute("Id").unwrap_or("").to_string(),
        title: node.attribute("Title").map(String::from),
        description: node.attribute("Description").map(String::from),
        level: node.attribute("Level").and_then(|l| l.parse().ok()).unwrap_or(1),
        children: Vec::new(),
        components: Vec::new(),
    };

    // Extract child features
    for child in node.children() {
        if child.tag_name().name() == "Feature" {
            feature.children.push(extract_feature(&child));
        } else if child.tag_name().name() == "ComponentRef" {
            if let Some(id) = child.attribute("Id") {
                feature.components.push(id.to_string());
            }
        }
    }

    feature
}

fn format_directory(dir: &DirectoryEntry, indent: usize) -> String {
    let mut output = String::new();
    let prefix = "  ".repeat(indent);

    output.push_str(&format!("{}[{}] {}\n", prefix, dir.id, dir.name));

    for file in &dir.files {
        output.push_str(&format!(
            "{}  - {}{}\n",
            prefix,
            file.name,
            if file.key_path { " (KeyPath)" } else { "" }
        ));
    }

    for child in &dir.children {
        output.push_str(&format_directory(child, indent + 1));
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_standard_directory() {
        assert!(get_standard_directory("ProgramFilesFolder").is_some());
        assert!(get_standard_directory("UnknownFolder").is_none());
    }

    #[test]
    fn test_generate_preview() {
        let content = r#"
        <Wix xmlns="http://wixtoolset.org/schemas/v4/wxs">
            <Package Name="TestApp" Version="1.0.0" Manufacturer="Test">
                <Directory Id="TARGETDIR" Name="SourceDir">
                    <Directory Id="ProgramFilesFolder">
                        <Directory Id="INSTALLDIR" Name="TestApp">
                            <Component Id="MainComp">
                                <File Id="MainExe" Source="app.exe" KeyPath="yes" />
                            </Component>
                        </Directory>
                    </Directory>
                </Directory>
                <Feature Id="MainFeature" Title="Main">
                    <ComponentRef Id="MainComp" />
                </Feature>
            </Package>
        </Wix>
        "#;

        let preview = PreviewGenerator::generate(content);
        assert_eq!(preview.product_name, Some("TestApp".to_string()));
        assert_eq!(preview.total_files, 1);
    }

    #[test]
    fn test_generate_tree() {
        let preview = InstallPreview {
            product_name: Some("TestApp".to_string()),
            product_version: Some("1.0.0".to_string()),
            manufacturer: Some("Test".to_string()),
            total_files: 5,
            ..Default::default()
        };

        let tree = PreviewGenerator::generate_tree(&preview);
        assert!(tree.contains("TestApp"));
        assert!(tree.contains("1.0.0"));
    }
}

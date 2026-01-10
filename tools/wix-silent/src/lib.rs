//! wix-silent - Silent install parameter generator
//!
//! Extracts and documents MSI public properties for silent installation

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// MSI property information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MsiProperty {
    pub name: String,
    pub default_value: Option<String>,
    pub description: Option<String>,
    pub secure: bool,
    pub public: bool,
    pub required: bool,
    pub property_type: PropertyType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PropertyType {
    String,
    Path,
    Boolean,
    Integer,
    Guid,
    Feature,
    Unknown,
}

impl std::fmt::Display for PropertyType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PropertyType::String => write!(f, "string"),
            PropertyType::Path => write!(f, "path"),
            PropertyType::Boolean => write!(f, "boolean (0/1)"),
            PropertyType::Integer => write!(f, "integer"),
            PropertyType::Guid => write!(f, "GUID"),
            PropertyType::Feature => write!(f, "feature state"),
            PropertyType::Unknown => write!(f, "unknown"),
        }
    }
}

/// Standard MSI properties used in silent installs
pub fn get_standard_properties() -> Vec<MsiProperty> {
    vec![
        MsiProperty {
            name: "TARGETDIR".to_string(),
            default_value: None,
            description: Some("Root installation directory".to_string()),
            secure: false,
            public: true,
            required: false,
            property_type: PropertyType::Path,
        },
        MsiProperty {
            name: "INSTALLDIR".to_string(),
            default_value: None,
            description: Some("Main installation directory".to_string()),
            secure: false,
            public: true,
            required: false,
            property_type: PropertyType::Path,
        },
        MsiProperty {
            name: "ALLUSERS".to_string(),
            default_value: Some("1".to_string()),
            description: Some("Install for all users (1) or current user only (empty)".to_string()),
            secure: false,
            public: true,
            required: false,
            property_type: PropertyType::Boolean,
        },
        MsiProperty {
            name: "MSIINSTALLPERUSER".to_string(),
            default_value: None,
            description: Some("Per-user installation (1) vs per-machine (0)".to_string()),
            secure: false,
            public: true,
            required: false,
            property_type: PropertyType::Boolean,
        },
        MsiProperty {
            name: "REINSTALLMODE".to_string(),
            default_value: Some("omus".to_string()),
            description: Some("Reinstall mode flags (o=older, m=missing, u=user, s=shortcut)".to_string()),
            secure: false,
            public: true,
            required: false,
            property_type: PropertyType::String,
        },
        MsiProperty {
            name: "REINSTALL".to_string(),
            default_value: None,
            description: Some("Features to reinstall (ALL or feature list)".to_string()),
            secure: false,
            public: true,
            required: false,
            property_type: PropertyType::Feature,
        },
        MsiProperty {
            name: "ADDLOCAL".to_string(),
            default_value: None,
            description: Some("Features to install locally (ALL or comma-separated list)".to_string()),
            secure: false,
            public: true,
            required: false,
            property_type: PropertyType::Feature,
        },
        MsiProperty {
            name: "REMOVE".to_string(),
            default_value: None,
            description: Some("Features to remove (ALL or comma-separated list)".to_string()),
            secure: false,
            public: true,
            required: false,
            property_type: PropertyType::Feature,
        },
        MsiProperty {
            name: "TRANSFORMS".to_string(),
            default_value: None,
            description: Some("Transform files to apply (.mst)".to_string()),
            secure: false,
            public: true,
            required: false,
            property_type: PropertyType::Path,
        },
        MsiProperty {
            name: "REBOOT".to_string(),
            default_value: None,
            description: Some("Reboot behavior: Force, Suppress, ReallySuppress".to_string()),
            secure: false,
            public: true,
            required: false,
            property_type: PropertyType::String,
        },
        MsiProperty {
            name: "ARPNOREMOVE".to_string(),
            default_value: None,
            description: Some("Hide Remove button in Add/Remove Programs (1)".to_string()),
            secure: false,
            public: true,
            required: false,
            property_type: PropertyType::Boolean,
        },
        MsiProperty {
            name: "ARPNOREPAIR".to_string(),
            default_value: None,
            description: Some("Hide Repair button in Add/Remove Programs (1)".to_string()),
            secure: false,
            public: true,
            required: false,
            property_type: PropertyType::Boolean,
        },
        MsiProperty {
            name: "ARPNOMODIFY".to_string(),
            default_value: None,
            description: Some("Hide Modify button in Add/Remove Programs (1)".to_string()),
            secure: false,
            public: true,
            required: false,
            property_type: PropertyType::Boolean,
        },
    ]
}

/// Silent install analyzer
pub struct SilentAnalyzer;

impl Default for SilentAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl SilentAnalyzer {
    pub fn new() -> Self {
        Self
    }

    /// Extract properties from WiX source
    pub fn extract_properties(&self, content: &str) -> Vec<MsiProperty> {
        let mut properties = Vec::new();

        if let Ok(doc) = roxmltree::Document::parse(content) {
            for node in doc.descendants() {
                if node.tag_name().name() == "Property" {
                    if let Some(id) = node.attribute("Id") {
                        // Public properties are UPPERCASE
                        let is_public = id.chars().all(|c| c.is_uppercase() || c.is_numeric() || c == '_');
                        let secure = node.attribute("Secure") == Some("yes");

                        let prop = MsiProperty {
                            name: id.to_string(),
                            default_value: node.attribute("Value").map(String::from),
                            description: extract_comment_before(&node, content),
                            secure,
                            public: is_public,
                            required: false,
                            property_type: infer_property_type(id, node.attribute("Value")),
                        };

                        if is_public {
                            properties.push(prop);
                        }
                    }
                }
            }
        }

        properties
    }

    /// Extract features from WiX source
    pub fn extract_features(&self, content: &str) -> Vec<FeatureInfo> {
        let mut features = Vec::new();

        if let Ok(doc) = roxmltree::Document::parse(content) {
            for node in doc.descendants() {
                if node.tag_name().name() == "Feature" {
                    if let Some(id) = node.attribute("Id") {
                        let level: u32 = node
                            .attribute("Level")
                            .and_then(|l| l.parse().ok())
                            .unwrap_or(1);

                        features.push(FeatureInfo {
                            id: id.to_string(),
                            title: node.attribute("Title").map(String::from),
                            description: node.attribute("Description").map(String::from),
                            level,
                            default_install: level > 0,
                        });
                    }
                }
            }
        }

        features
    }

    /// Generate silent install documentation
    pub fn generate_docs(&self, properties: &[MsiProperty], features: &[FeatureInfo]) -> String {
        let mut doc = String::new();

        doc.push_str("# Silent Installation Guide\n\n");

        // Basic silent install
        doc.push_str("## Basic Silent Install\n\n");
        doc.push_str("```cmd\n");
        doc.push_str("msiexec /i package.msi /qn /l*v install.log\n");
        doc.push_str("```\n\n");

        // UI levels
        doc.push_str("## UI Levels\n\n");
        doc.push_str("| Flag | Description |\n");
        doc.push_str("|------|-------------|\n");
        doc.push_str("| /qn | No UI (silent) |\n");
        doc.push_str("| /qb | Basic UI (progress bar only) |\n");
        doc.push_str("| /qr | Reduced UI |\n");
        doc.push_str("| /qf | Full UI |\n\n");

        // Properties
        if !properties.is_empty() {
            doc.push_str("## Public Properties\n\n");
            doc.push_str("| Property | Type | Default | Description |\n");
            doc.push_str("|----------|------|---------|-------------|\n");

            for prop in properties {
                doc.push_str(&format!(
                    "| {} | {} | {} | {} |\n",
                    prop.name,
                    prop.property_type,
                    prop.default_value.as_deref().unwrap_or("-"),
                    prop.description.as_deref().unwrap_or("-")
                ));
            }
            doc.push('\n');
        }

        // Features
        if !features.is_empty() {
            doc.push_str("## Features\n\n");
            doc.push_str("| Feature | Title | Default |\n");
            doc.push_str("|---------|-------|----------|\n");

            for feature in features {
                doc.push_str(&format!(
                    "| {} | {} | {} |\n",
                    feature.id,
                    feature.title.as_deref().unwrap_or("-"),
                    if feature.default_install {
                        "Installed"
                    } else {
                        "Not installed"
                    }
                ));
            }
            doc.push('\n');

            doc.push_str("### Feature Selection Examples\n\n");
            doc.push_str("```cmd\n");
            doc.push_str("# Install all features\n");
            doc.push_str("msiexec /i package.msi /qn ADDLOCAL=ALL\n\n");
            doc.push_str("# Install specific features\n");
            let feature_list: Vec<&str> = features.iter().take(2).map(|f| f.id.as_str()).collect();
            doc.push_str(&format!(
                "msiexec /i package.msi /qn ADDLOCAL={}\n",
                feature_list.join(",")
            ));
            doc.push_str("```\n\n");
        }

        // Common examples
        doc.push_str("## Common Examples\n\n");
        doc.push_str("```cmd\n");
        doc.push_str("# Silent install with logging\n");
        doc.push_str("msiexec /i package.msi /qn /l*v install.log\n\n");
        doc.push_str("# Install to custom directory\n");
        doc.push_str("msiexec /i package.msi /qn INSTALLDIR=\"C:\\MyApp\"\n\n");
        doc.push_str("# Per-user installation\n");
        doc.push_str("msiexec /i package.msi /qn ALLUSERS=\"\" MSIINSTALLPERUSER=1\n\n");
        doc.push_str("# Silent uninstall\n");
        doc.push_str("msiexec /x package.msi /qn\n\n");
        doc.push_str("# Repair installation\n");
        doc.push_str("msiexec /f package.msi /qn\n");
        doc.push_str("```\n");

        doc
    }

    /// Generate command line for silent install
    pub fn generate_command(
        &self,
        msi_path: &str,
        properties: &HashMap<String, String>,
        features: Option<&[String]>,
        log_file: Option<&str>,
    ) -> String {
        let mut cmd = format!("msiexec /i \"{}\" /qn", msi_path);

        // Add logging
        if let Some(log) = log_file {
            cmd.push_str(&format!(" /l*v \"{}\"", log));
        }

        // Add properties
        for (name, value) in properties {
            if value.contains(' ') {
                cmd.push_str(&format!(" {}=\"{}\"", name, value));
            } else {
                cmd.push_str(&format!(" {}={}", name, value));
            }
        }

        // Add features
        if let Some(feats) = features {
            if !feats.is_empty() {
                cmd.push_str(&format!(" ADDLOCAL={}", feats.join(",")));
            }
        }

        cmd
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureInfo {
    pub id: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub level: u32,
    pub default_install: bool,
}

fn infer_property_type(name: &str, value: Option<&str>) -> PropertyType {
    let name_upper = name.to_uppercase();

    // Check name patterns
    if name_upper.ends_with("DIR") || name_upper.ends_with("PATH") || name_upper.ends_with("FOLDER")
    {
        return PropertyType::Path;
    }

    if name_upper.contains("GUID") || name_upper.contains("CODE") {
        return PropertyType::Guid;
    }

    // Check value patterns
    if let Some(v) = value {
        if v == "0" || v == "1" {
            return PropertyType::Boolean;
        }
        if v.parse::<i64>().is_ok() {
            return PropertyType::Integer;
        }
        if v.starts_with('{') && v.ends_with('}') && v.len() == 38 {
            return PropertyType::Guid;
        }
    }

    PropertyType::String
}

fn extract_comment_before(node: &roxmltree::Node, content: &str) -> Option<String> {
    // Try to find a comment before this node
    let start = node.range().start;
    if start < 10 {
        return None;
    }

    let before = &content[..start];
    if let Some(comment_end) = before.rfind("-->") {
        if let Some(comment_start) = before[..comment_end].rfind("<!--") {
            let comment = &before[comment_start + 4..comment_end].trim();
            if !comment.is_empty() {
                return Some(comment.to_string());
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_properties() {
        let content = r#"
        <Wix xmlns="http://wixtoolset.org/schemas/v4/wxs">
            <Package>
                <!-- Installation directory -->
                <Property Id="INSTALLDIR" Value="C:\MyApp" />
                <Property Id="MY_OPTION" Value="1" />
                <Property Id="privateProperty" Value="hidden" />
            </Package>
        </Wix>
        "#;

        let analyzer = SilentAnalyzer::new();
        let props = analyzer.extract_properties(content);

        assert_eq!(props.len(), 2); // Only public (UPPERCASE) properties
        assert!(props.iter().any(|p| p.name == "INSTALLDIR"));
        assert!(props.iter().any(|p| p.name == "MY_OPTION"));
    }

    #[test]
    fn test_extract_features() {
        let content = r#"
        <Wix xmlns="http://wixtoolset.org/schemas/v4/wxs">
            <Package>
                <Feature Id="MainFeature" Title="Main" Level="1" />
                <Feature Id="OptionalFeature" Title="Optional" Level="0" />
            </Package>
        </Wix>
        "#;

        let analyzer = SilentAnalyzer::new();
        let features = analyzer.extract_features(content);

        assert_eq!(features.len(), 2);
        assert!(features[0].default_install);
        assert!(!features[1].default_install);
    }

    #[test]
    fn test_generate_command() {
        let analyzer = SilentAnalyzer::new();
        let mut props = HashMap::new();
        props.insert("INSTALLDIR".to_string(), "C:\\Program Files\\MyApp".to_string());

        let cmd = analyzer.generate_command(
            "myapp.msi",
            &props,
            Some(&["Feature1".to_string(), "Feature2".to_string()]),
            Some("install.log"),
        );

        assert!(cmd.contains("msiexec /i"));
        assert!(cmd.contains("/qn"));
        assert!(cmd.contains("INSTALLDIR="));
        assert!(cmd.contains("ADDLOCAL=Feature1,Feature2"));
        assert!(cmd.contains("/l*v"));
    }

    #[test]
    fn test_infer_property_type() {
        assert_eq!(
            infer_property_type("INSTALLDIR", None),
            PropertyType::Path
        );
        assert_eq!(
            infer_property_type("MYPATH", None),
            PropertyType::Path
        );
        assert_eq!(
            infer_property_type("OPTION", Some("1")),
            PropertyType::Boolean
        );
        assert_eq!(
            infer_property_type("COUNT", Some("42")),
            PropertyType::Integer
        );
    }

    #[test]
    fn test_standard_properties() {
        let props = get_standard_properties();
        assert!(!props.is_empty());
        assert!(props.iter().any(|p| p.name == "INSTALLDIR"));
        assert!(props.iter().any(|p| p.name == "ALLUSERS"));
    }

    #[test]
    fn test_property_type_display() {
        assert_eq!(format!("{}", PropertyType::Path), "path");
        assert_eq!(format!("{}", PropertyType::Boolean), "boolean (0/1)");
    }
}

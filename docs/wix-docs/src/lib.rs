//! WiX Documentation Generator
//!
//! A doxygen-like tool for generating documentation from WiX installer projects.
//!
//! # Features
//!
//! - Extracts documentation from WiX source files
//! - Parses `<!-- @doc: description -->` comments
//! - Generates HTML, Markdown, or JSON output
//! - Cross-references between elements
//! - Documents components, features, custom actions, properties
//!
//! # Example
//!
//! ```no_run
//! use wix_docs::{generate_docs, DocsConfig, OutputFormat};
//! use std::path::Path;
//!
//! let config = DocsConfig {
//!     format: OutputFormat::Html,
//!     output_dir: Path::new("docs").to_path_buf(),
//!     ..Default::default()
//! };
//!
//! generate_docs(Path::new("src"), &config).unwrap();
//! ```

pub mod generator;
pub mod parser;
pub mod types;

pub use generator::DocsGenerator;
pub use parser::{DocsParser, ParsedFile};
pub use types::*;

use std::path::Path;
use walkdir::WalkDir;

/// Generate documentation for a WiX project
pub fn generate_docs(source_dir: &Path, config: &DocsConfig) -> Result<ProjectDocs, String> {
    let parser = DocsParser::new().with_include_private(config.include_private);
    let mut project = ProjectDocs::new(
        config.project_name.clone().unwrap_or_else(|| {
            source_dir
                .file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| "WiX Project".to_string())
        }),
    );

    // Find all .wxs files
    let wxs_files: Vec<_> = WalkDir::new(source_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .map(|ext| ext == "wxs" || ext == "wxi")
                .unwrap_or(false)
        })
        .collect();

    if wxs_files.is_empty() {
        return Err(format!(
            "No WiX files found in {}",
            source_dir.display()
        ));
    }

    // Parse each file
    for entry in wxs_files {
        let path = entry.path();

        // Check exclusion
        if config.include_private {
            // Skip private files starting with _
            if path
                .file_name()
                .map(|s| s.to_string_lossy().starts_with('_'))
                .unwrap_or(false)
            {
                continue;
            }
        }

        let source = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;

        let parsed = parser.parse_file(&source, path)?;

        // Add file info
        let mut file_docs = FileDocs::new(path.to_path_buf());
        file_docs.description = parsed.description;
        for comp in &parsed.components {
            file_docs.elements.push(format!("Component:{}", comp.id));
        }
        for feature in &parsed.features {
            file_docs.elements.push(format!("Feature:{}", feature.id));
        }
        project.files.push(file_docs);

        // Merge parsed content
        project.components.extend(parsed.components);
        project.features.extend(parsed.features);
        project.directories.extend(parsed.directories);
        project.custom_actions.extend(parsed.custom_actions);
        project.properties.extend(parsed.properties);
    }

    // Build cross-references
    build_references(&mut project);

    // Extract package info if available
    extract_package_info(&mut project);

    // Generate output
    let generator = DocsGenerator::new(config.clone());
    generator.generate(&project)?;

    Ok(project)
}

/// Build cross-references between elements
fn build_references(project: &mut ProjectDocs) {
    // Build feature -> component references
    for feature in &project.features {
        for comp_ref in &feature.components {
            project.references.push(Reference {
                from_type: "Feature".to_string(),
                from_id: feature.id.clone(),
                to_type: "Component".to_string(),
                to_id: comp_ref.clone(),
                ref_type: ReferenceType::Includes,
            });

            // Also update component's included_in_features
            for comp in &mut project.components {
                if &comp.id == comp_ref {
                    if !comp.included_in_features.contains(&feature.id) {
                        comp.included_in_features.push(feature.id.clone());
                    }
                }
            }
        }
    }

    // Build directory parent-child references
    for dir in &project.directories {
        for child_id in &dir.children {
            project.references.push(Reference {
                from_type: "Directory".to_string(),
                from_id: dir.id.clone(),
                to_type: "Directory".to_string(),
                to_id: child_id.clone(),
                ref_type: ReferenceType::Parent,
            });
        }
    }

    // Build feature parent-child references
    for feature in &project.features {
        for child_id in &feature.children {
            project.references.push(Reference {
                from_type: "Feature".to_string(),
                from_id: feature.id.clone(),
                to_type: "Feature".to_string(),
                to_id: child_id.clone(),
                ref_type: ReferenceType::Parent,
            });
        }
    }
}

/// Extract package information from parsed content
fn extract_package_info(project: &mut ProjectDocs) {
    // Look for Package element info in properties or other sources
    for prop in &project.properties {
        if prop.id == "ProductVersion" {
            if let Some(value) = &prop.value {
                project.version = Some(value.clone());
            }
        }
        if prop.id == "ProductName" {
            if let Some(value) = &prop.value {
                project.description = Some(value.clone());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_wxs(dir: &Path, name: &str, content: &str) {
        let path = dir.join(name);
        fs::write(&path, content).unwrap();
    }

    #[test]
    fn test_generate_docs_basic() {
        let temp_dir = TempDir::new().unwrap();
        let source_dir = temp_dir.path().join("src");
        let output_dir = temp_dir.path().join("docs");
        fs::create_dir_all(&source_dir).unwrap();

        create_test_wxs(
            &source_dir,
            "product.wxs",
            r#"<!-- @doc: Main product file -->
<Wix>
    <!-- @doc: Main application component -->
    <Component Id="MainComponent" Guid="*">
        <File Id="MainExe" Source="app.exe" />
    </Component>

    <!-- @doc: Main feature -->
    <Feature Id="MainFeature" Title="Main" Level="1">
        <ComponentRef Id="MainComponent" />
    </Feature>
</Wix>"#,
        );

        let config = DocsConfig {
            format: OutputFormat::Html,
            output_dir: output_dir.clone(),
            ..Default::default()
        };

        let project = generate_docs(&source_dir, &config).unwrap();

        assert_eq!(project.components.len(), 1);
        assert_eq!(project.features.len(), 1);
        assert!(output_dir.join("index.html").exists());
    }

    #[test]
    fn test_generate_docs_no_files() {
        let temp_dir = TempDir::new().unwrap();
        let source_dir = temp_dir.path().join("empty");
        fs::create_dir_all(&source_dir).unwrap();

        let config = DocsConfig::default();
        let result = generate_docs(&source_dir, &config);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("No WiX files found"));
    }

    #[test]
    fn test_generate_docs_markdown() {
        let temp_dir = TempDir::new().unwrap();
        let source_dir = temp_dir.path().join("src");
        let output_dir = temp_dir.path().join("docs");
        fs::create_dir_all(&source_dir).unwrap();

        create_test_wxs(
            &source_dir,
            "product.wxs",
            r#"<Wix>
    <Component Id="C1" Guid="*" />
</Wix>"#,
        );

        let config = DocsConfig {
            format: OutputFormat::Markdown,
            output_dir: output_dir.clone(),
            ..Default::default()
        };

        generate_docs(&source_dir, &config).unwrap();

        assert!(output_dir.join("README.md").exists());
    }

    #[test]
    fn test_generate_docs_json() {
        let temp_dir = TempDir::new().unwrap();
        let source_dir = temp_dir.path().join("src");
        let output_dir = temp_dir.path().join("docs");
        fs::create_dir_all(&source_dir).unwrap();

        create_test_wxs(
            &source_dir,
            "product.wxs",
            r#"<Wix>
    <Component Id="C1" Guid="*" />
</Wix>"#,
        );

        let config = DocsConfig {
            format: OutputFormat::Json,
            output_dir: output_dir.clone(),
            ..Default::default()
        };

        generate_docs(&source_dir, &config).unwrap();

        assert!(output_dir.join("docs.json").exists());
    }

    #[test]
    fn test_build_references() {
        let mut project = ProjectDocs::new("Test");

        project.features.push(FeatureDocs {
            id: "F1".to_string(),
            title: None,
            level: None,
            description: None,
            file: std::path::PathBuf::from("test.wxs"),
            line: 1,
            components: vec!["C1".to_string()],
            children: Vec::new(),
            parent: None,
        });

        project.components.push(ComponentDocs::new(
            "C1",
            std::path::PathBuf::from("test.wxs"),
            5,
        ));

        build_references(&mut project);

        assert!(!project.references.is_empty());
        assert!(project.components[0].included_in_features.contains(&"F1".to_string()));
    }

    #[test]
    fn test_extract_package_info() {
        let mut project = ProjectDocs::new("Test");

        project.properties.push(PropertyDocs {
            id: "ProductVersion".to_string(),
            value: Some("1.2.3".to_string()),
            description: None,
            file: std::path::PathBuf::from("test.wxs"),
            line: 1,
            secure: false,
            admin: false,
            hidden: false,
            used_in: Vec::new(),
        });

        extract_package_info(&mut project);

        assert_eq!(project.version, Some("1.2.3".to_string()));
    }

    #[test]
    fn test_multiple_files() {
        let temp_dir = TempDir::new().unwrap();
        let source_dir = temp_dir.path().join("src");
        let output_dir = temp_dir.path().join("docs");
        fs::create_dir_all(&source_dir).unwrap();

        create_test_wxs(
            &source_dir,
            "product.wxs",
            r#"<Wix><Component Id="C1" Guid="*" /></Wix>"#,
        );

        create_test_wxs(
            &source_dir,
            "files.wxs",
            r#"<Wix><Component Id="C2" Guid="*" /></Wix>"#,
        );

        let config = DocsConfig {
            format: OutputFormat::Html,
            output_dir: output_dir.clone(),
            ..Default::default()
        };

        let project = generate_docs(&source_dir, &config).unwrap();

        assert_eq!(project.components.len(), 2);
        assert_eq!(project.files.len(), 2);
    }

    #[test]
    fn test_wxi_files() {
        let temp_dir = TempDir::new().unwrap();
        let source_dir = temp_dir.path().join("src");
        let output_dir = temp_dir.path().join("docs");
        fs::create_dir_all(&source_dir).unwrap();

        create_test_wxs(
            &source_dir,
            "common.wxi",
            r#"<Wix><Property Id="CommonProp" Value="test" /></Wix>"#,
        );

        let config = DocsConfig {
            format: OutputFormat::Html,
            output_dir: output_dir.clone(),
            ..Default::default()
        };

        let project = generate_docs(&source_dir, &config).unwrap();

        assert_eq!(project.properties.len(), 1);
    }

    #[test]
    fn test_custom_project_name() {
        let temp_dir = TempDir::new().unwrap();
        let source_dir = temp_dir.path().join("src");
        let output_dir = temp_dir.path().join("docs");
        fs::create_dir_all(&source_dir).unwrap();

        create_test_wxs(
            &source_dir,
            "product.wxs",
            r#"<Wix><Component Id="C1" Guid="*" /></Wix>"#,
        );

        let config = DocsConfig {
            format: OutputFormat::Html,
            output_dir,
            project_name: Some("MyCustomProject".to_string()),
            ..Default::default()
        };

        let project = generate_docs(&source_dir, &config).unwrap();

        assert_eq!(project.name, "MyCustomProject");
    }
}

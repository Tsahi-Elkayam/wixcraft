//! Documentation output generators

use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

use crate::types::*;

/// Generator for documentation output
pub struct DocsGenerator {
    config: DocsConfig,
}

impl DocsGenerator {
    pub fn new(config: DocsConfig) -> Self {
        Self { config }
    }

    /// Generate documentation for a project
    pub fn generate(&self, project: &ProjectDocs) -> Result<(), String> {
        fs::create_dir_all(&self.config.output_dir)
            .map_err(|e| format!("Failed to create output directory: {}", e))?;

        match self.config.format {
            OutputFormat::Html => self.generate_html(project),
            OutputFormat::Markdown => self.generate_markdown(project),
            OutputFormat::Json => self.generate_json(project),
        }
    }

    fn generate_html(&self, project: &ProjectDocs) -> Result<(), String> {
        // Generate index.html
        let index_path = self.config.output_dir.join("index.html");
        let index_content = self.render_html_index(project);
        self.write_file(&index_path, &index_content)?;

        // Generate component pages
        let components_dir = self.config.output_dir.join("components");
        fs::create_dir_all(&components_dir)
            .map_err(|e| format!("Failed to create components directory: {}", e))?;

        for component in &project.components {
            let path = components_dir.join(format!("{}.html", component.id));
            let content = self.render_html_component(component, project);
            self.write_file(&path, &content)?;
        }

        // Generate feature pages
        let features_dir = self.config.output_dir.join("features");
        fs::create_dir_all(&features_dir)
            .map_err(|e| format!("Failed to create features directory: {}", e))?;

        for feature in &project.features {
            let path = features_dir.join(format!("{}.html", feature.id));
            let content = self.render_html_feature(feature, project);
            self.write_file(&path, &content)?;
        }

        // Generate custom actions page
        if !project.custom_actions.is_empty() {
            let ca_path = self.config.output_dir.join("custom-actions.html");
            let content = self.render_html_custom_actions(project);
            self.write_file(&ca_path, &content)?;
        }

        // Generate properties page
        if !project.properties.is_empty() {
            let props_path = self.config.output_dir.join("properties.html");
            let content = self.render_html_properties(project);
            self.write_file(&props_path, &content)?;
        }

        // Generate CSS
        let css_path = self.config.output_dir.join("style.css");
        self.write_file(&css_path, CSS_CONTENT)?;

        Ok(())
    }

    fn generate_markdown(&self, project: &ProjectDocs) -> Result<(), String> {
        // Generate README.md
        let readme_path = self.config.output_dir.join("README.md");
        let readme_content = self.render_markdown_index(project);
        self.write_file(&readme_path, &readme_content)?;

        // Generate components.md
        if !project.components.is_empty() {
            let path = self.config.output_dir.join("components.md");
            let content = self.render_markdown_components(project);
            self.write_file(&path, &content)?;
        }

        // Generate features.md
        if !project.features.is_empty() {
            let path = self.config.output_dir.join("features.md");
            let content = self.render_markdown_features(project);
            self.write_file(&path, &content)?;
        }

        // Generate custom-actions.md
        if !project.custom_actions.is_empty() {
            let path = self.config.output_dir.join("custom-actions.md");
            let content = self.render_markdown_custom_actions(project);
            self.write_file(&path, &content)?;
        }

        // Generate properties.md
        if !project.properties.is_empty() {
            let path = self.config.output_dir.join("properties.md");
            let content = self.render_markdown_properties(project);
            self.write_file(&path, &content)?;
        }

        Ok(())
    }

    fn generate_json(&self, project: &ProjectDocs) -> Result<(), String> {
        let json_path = self.config.output_dir.join("docs.json");
        let content = serde_json::to_string_pretty(project)
            .map_err(|e| format!("Failed to serialize JSON: {}", e))?;
        self.write_file(&json_path, &content)
    }

    fn write_file(&self, path: &Path, content: &str) -> Result<(), String> {
        let mut file = File::create(path)
            .map_err(|e| format!("Failed to create {}: {}", path.display(), e))?;
        file.write_all(content.as_bytes())
            .map_err(|e| format!("Failed to write {}: {}", path.display(), e))
    }

    // HTML rendering methods

    fn render_html_index(&self, project: &ProjectDocs) -> String {
        let mut html = String::new();
        html.push_str(&self.html_header(&project.name));

        html.push_str("<div class=\"container\">\n");
        html.push_str(&format!("<h1>{}</h1>\n", escape_html(&project.name)));

        if let Some(desc) = &project.description {
            html.push_str(&format!("<p class=\"description\">{}</p>\n", escape_html(desc)));
        }

        if let Some(version) = &project.version {
            html.push_str(&format!("<p class=\"version\">Version: {}</p>\n", escape_html(version)));
        }

        // Table of contents
        if self.config.generate_toc {
            html.push_str("<nav class=\"toc\">\n<h2>Contents</h2>\n<ul>\n");

            if !project.features.is_empty() {
                html.push_str(&format!(
                    "<li><a href=\"#features\">Features</a> ({})</li>\n",
                    project.features.len()
                ));
            }
            if !project.components.is_empty() {
                html.push_str(&format!(
                    "<li><a href=\"#components\">Components</a> ({})</li>\n",
                    project.components.len()
                ));
            }
            if !project.custom_actions.is_empty() {
                html.push_str(&format!(
                    "<li><a href=\"custom-actions.html\">Custom Actions</a> ({})</li>\n",
                    project.custom_actions.len()
                ));
            }
            if !project.properties.is_empty() {
                html.push_str(&format!(
                    "<li><a href=\"properties.html\">Properties</a> ({})</li>\n",
                    project.properties.len()
                ));
            }

            html.push_str("</ul>\n</nav>\n");
        }

        // Features summary
        if !project.features.is_empty() {
            html.push_str("<section id=\"features\">\n<h2>Features</h2>\n");
            html.push_str("<table>\n<thead><tr><th>Id</th><th>Title</th><th>Level</th><th>Components</th></tr></thead>\n<tbody>\n");

            for feature in &project.features {
                html.push_str(&format!(
                    "<tr><td><a href=\"features/{}.html\">{}</a></td><td>{}</td><td>{}</td><td>{}</td></tr>\n",
                    feature.id,
                    escape_html(&feature.id),
                    feature.title.as_deref().map(escape_html).unwrap_or_default(),
                    feature.level.as_deref().unwrap_or("-"),
                    feature.components.len()
                ));
            }

            html.push_str("</tbody>\n</table>\n</section>\n");
        }

        // Components summary
        if !project.components.is_empty() {
            html.push_str("<section id=\"components\">\n<h2>Components</h2>\n");
            html.push_str("<table>\n<thead><tr><th>Id</th><th>Description</th><th>Files</th></tr></thead>\n<tbody>\n");

            for component in &project.components {
                html.push_str(&format!(
                    "<tr><td><a href=\"components/{}.html\">{}</a></td><td>{}</td><td>{}</td></tr>\n",
                    component.id,
                    escape_html(&component.id),
                    component.description.as_deref().map(escape_html).unwrap_or_default(),
                    component.files.len()
                ));
            }

            html.push_str("</tbody>\n</table>\n</section>\n");
        }

        html.push_str("</div>\n");
        html.push_str(&self.html_footer());
        html
    }

    fn render_html_component(&self, component: &ComponentDocs, _project: &ProjectDocs) -> String {
        let mut html = String::new();
        html.push_str(&self.html_header(&format!("Component: {}", component.id)));

        html.push_str("<div class=\"container\">\n");
        html.push_str(&format!("<h1>Component: {}</h1>\n", escape_html(&component.id)));
        html.push_str("<p><a href=\"../index.html\">&larr; Back to index</a></p>\n");

        if let Some(desc) = &component.description {
            html.push_str(&format!("<p class=\"description\">{}</p>\n", escape_html(desc)));
        }

        // Details
        html.push_str("<h2>Details</h2>\n<dl>\n");
        if let Some(guid) = &component.guid {
            html.push_str(&format!("<dt>GUID</dt><dd><code>{}</code></dd>\n", escape_html(guid)));
        }
        html.push_str(&format!(
            "<dt>Source</dt><dd>{}:{}</dd>\n",
            component.file.display(),
            component.line
        ));
        html.push_str("</dl>\n");

        // Files
        if !component.files.is_empty() {
            html.push_str("<h2>Files</h2>\n<table>\n<thead><tr><th>Id</th><th>Source</th><th>Description</th></tr></thead>\n<tbody>\n");
            for file in &component.files {
                html.push_str(&format!(
                    "<tr><td>{}</td><td>{}</td><td>{}</td></tr>\n",
                    file.id.as_deref().unwrap_or("-"),
                    file.source.as_deref().map(escape_html).unwrap_or_default(),
                    file.description.as_deref().map(escape_html).unwrap_or_default()
                ));
            }
            html.push_str("</tbody>\n</table>\n");
        }

        // Registry
        if !component.registry.is_empty() {
            html.push_str("<h2>Registry Entries</h2>\n<table>\n<thead><tr><th>Root</th><th>Key</th><th>Name</th><th>Type</th></tr></thead>\n<tbody>\n");
            for reg in &component.registry {
                html.push_str(&format!(
                    "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>\n",
                    escape_html(&reg.root),
                    escape_html(&reg.key),
                    reg.name.as_deref().unwrap_or("-"),
                    reg.value_type.as_deref().unwrap_or("-")
                ));
            }
            html.push_str("</tbody>\n</table>\n");
        }

        // Services
        if !component.services.is_empty() {
            html.push_str("<h2>Services</h2>\n<table>\n<thead><tr><th>Name</th><th>Display Name</th><th>Type</th><th>Start</th></tr></thead>\n<tbody>\n");
            for svc in &component.services {
                html.push_str(&format!(
                    "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>\n",
                    escape_html(&svc.name),
                    svc.display_name.as_deref().map(escape_html).unwrap_or_default(),
                    svc.service_type.as_deref().unwrap_or("-"),
                    svc.start_type.as_deref().unwrap_or("-")
                ));
            }
            html.push_str("</tbody>\n</table>\n");
        }

        // Included in features
        if !component.included_in_features.is_empty() {
            html.push_str("<h2>Included in Features</h2>\n<ul>\n");
            for feature_id in &component.included_in_features {
                html.push_str(&format!(
                    "<li><a href=\"../features/{}.html\">{}</a></li>\n",
                    feature_id,
                    escape_html(feature_id)
                ));
            }
            html.push_str("</ul>\n");
        }

        html.push_str("</div>\n");
        html.push_str(&self.html_footer());
        html
    }

    fn render_html_feature(&self, feature: &FeatureDocs, _project: &ProjectDocs) -> String {
        let mut html = String::new();
        html.push_str(&self.html_header(&format!("Feature: {}", feature.id)));

        html.push_str("<div class=\"container\">\n");
        html.push_str(&format!("<h1>Feature: {}</h1>\n", escape_html(&feature.id)));
        html.push_str("<p><a href=\"../index.html\">&larr; Back to index</a></p>\n");

        if let Some(title) = &feature.title {
            html.push_str(&format!("<p class=\"title\"><strong>{}</strong></p>\n", escape_html(title)));
        }

        if let Some(desc) = &feature.description {
            html.push_str(&format!("<p class=\"description\">{}</p>\n", escape_html(desc)));
        }

        // Details
        html.push_str("<h2>Details</h2>\n<dl>\n");
        if let Some(level) = &feature.level {
            html.push_str(&format!("<dt>Install Level</dt><dd>{}</dd>\n", escape_html(level)));
        }
        html.push_str(&format!(
            "<dt>Source</dt><dd>{}:{}</dd>\n",
            feature.file.display(),
            feature.line
        ));
        html.push_str("</dl>\n");

        // Components
        if !feature.components.is_empty() {
            html.push_str("<h2>Components</h2>\n<ul>\n");
            for comp_id in &feature.components {
                html.push_str(&format!(
                    "<li><a href=\"../components/{}.html\">{}</a></li>\n",
                    comp_id,
                    escape_html(comp_id)
                ));
            }
            html.push_str("</ul>\n");
        }

        // Child features
        if !feature.children.is_empty() {
            html.push_str("<h2>Child Features</h2>\n<ul>\n");
            for child_id in &feature.children {
                html.push_str(&format!(
                    "<li><a href=\"{}.html\">{}</a></li>\n",
                    child_id,
                    escape_html(child_id)
                ));
            }
            html.push_str("</ul>\n");
        }

        html.push_str("</div>\n");
        html.push_str(&self.html_footer());
        html
    }

    fn render_html_custom_actions(&self, project: &ProjectDocs) -> String {
        let mut html = String::new();
        html.push_str(&self.html_header("Custom Actions"));

        html.push_str("<div class=\"container\">\n");
        html.push_str("<h1>Custom Actions</h1>\n");
        html.push_str("<p><a href=\"index.html\">&larr; Back to index</a></p>\n");

        for ca in &project.custom_actions {
            html.push_str(&format!("<h2 id=\"{}\">{}</h2>\n", ca.id, escape_html(&ca.id)));

            if let Some(desc) = &ca.description {
                html.push_str(&format!("<p class=\"description\">{}</p>\n", escape_html(desc)));
            }

            html.push_str("<dl>\n");
            if let Some(binary) = &ca.binary {
                html.push_str(&format!("<dt>Binary</dt><dd>{}</dd>\n", escape_html(binary)));
            }
            if let Some(dll_entry) = &ca.dll_entry {
                html.push_str(&format!("<dt>DLL Entry</dt><dd>{}</dd>\n", escape_html(dll_entry)));
            }
            if let Some(execute) = &ca.execute {
                html.push_str(&format!("<dt>Execute</dt><dd>{}</dd>\n", escape_html(execute)));
            }
            html.push_str(&format!(
                "<dt>Source</dt><dd>{}:{}</dd>\n",
                ca.file.display(),
                ca.line
            ));
            html.push_str("</dl>\n");

            if !ca.scheduled_in.is_empty() {
                html.push_str("<h3>Scheduling</h3>\n<table>\n<thead><tr><th>Sequence</th><th>Condition</th><th>Position</th></tr></thead>\n<tbody>\n");
                for sched in &ca.scheduled_in {
                    let position = match (&sched.before, &sched.after) {
                        (Some(b), _) => format!("Before {}", b),
                        (_, Some(a)) => format!("After {}", a),
                        _ => "-".to_string(),
                    };
                    html.push_str(&format!(
                        "<tr><td>{}</td><td>{}</td><td>{}</td></tr>\n",
                        escape_html(&sched.sequence),
                        sched.condition.as_deref().map(escape_html).unwrap_or_default(),
                        escape_html(&position)
                    ));
                }
                html.push_str("</tbody>\n</table>\n");
            }
        }

        html.push_str("</div>\n");
        html.push_str(&self.html_footer());
        html
    }

    fn render_html_properties(&self, project: &ProjectDocs) -> String {
        let mut html = String::new();
        html.push_str(&self.html_header("Properties"));

        html.push_str("<div class=\"container\">\n");
        html.push_str("<h1>Properties</h1>\n");
        html.push_str("<p><a href=\"index.html\">&larr; Back to index</a></p>\n");

        html.push_str("<table>\n<thead><tr><th>Id</th><th>Value</th><th>Flags</th><th>Description</th></tr></thead>\n<tbody>\n");

        for prop in &project.properties {
            let mut flags = Vec::new();
            if prop.secure { flags.push("Secure"); }
            if prop.admin { flags.push("Admin"); }
            if prop.hidden { flags.push("Hidden"); }

            html.push_str(&format!(
                "<tr><td><code>{}</code></td><td>{}</td><td>{}</td><td>{}</td></tr>\n",
                escape_html(&prop.id),
                prop.value.as_deref().map(escape_html).unwrap_or_default(),
                flags.join(", "),
                prop.description.as_deref().map(escape_html).unwrap_or_default()
            ));
        }

        html.push_str("</tbody>\n</table>\n");
        html.push_str("</div>\n");
        html.push_str(&self.html_footer());
        html
    }

    fn html_header(&self, title: &str) -> String {
        format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{}</title>
    <link rel="stylesheet" href="style.css">
</head>
<body>
"#,
            escape_html(title)
        )
    }

    fn html_footer(&self) -> String {
        format!(
            r#"<footer>
    <p>Generated by wix-docs</p>
</footer>
</body>
</html>
"#
        )
    }

    // Markdown rendering methods

    fn render_markdown_index(&self, project: &ProjectDocs) -> String {
        let mut md = String::new();
        md.push_str(&format!("# {}\n\n", project.name));

        if let Some(desc) = &project.description {
            md.push_str(&format!("{}\n\n", desc));
        }

        if let Some(version) = &project.version {
            md.push_str(&format!("**Version:** {}\n\n", version));
        }

        md.push_str("## Contents\n\n");

        if !project.features.is_empty() {
            md.push_str(&format!("- [Features](features.md) ({})\n", project.features.len()));
        }
        if !project.components.is_empty() {
            md.push_str(&format!("- [Components](components.md) ({})\n", project.components.len()));
        }
        if !project.custom_actions.is_empty() {
            md.push_str(&format!("- [Custom Actions](custom-actions.md) ({})\n", project.custom_actions.len()));
        }
        if !project.properties.is_empty() {
            md.push_str(&format!("- [Properties](properties.md) ({})\n", project.properties.len()));
        }

        md
    }

    fn render_markdown_components(&self, project: &ProjectDocs) -> String {
        let mut md = String::new();
        md.push_str("# Components\n\n");

        for component in &project.components {
            md.push_str(&format!("## {}\n\n", component.id));

            if let Some(desc) = &component.description {
                md.push_str(&format!("{}\n\n", desc));
            }

            if let Some(guid) = &component.guid {
                md.push_str(&format!("**GUID:** `{}`\n\n", guid));
            }

            if !component.files.is_empty() {
                md.push_str("### Files\n\n");
                md.push_str("| Id | Source |\n|---|---|\n");
                for file in &component.files {
                    md.push_str(&format!(
                        "| {} | {} |\n",
                        file.id.as_deref().unwrap_or("-"),
                        file.source.as_deref().unwrap_or("-")
                    ));
                }
                md.push('\n');
            }
        }

        md
    }

    fn render_markdown_features(&self, project: &ProjectDocs) -> String {
        let mut md = String::new();
        md.push_str("# Features\n\n");

        for feature in &project.features {
            md.push_str(&format!("## {}\n\n", feature.id));

            if let Some(title) = &feature.title {
                md.push_str(&format!("**{}**\n\n", title));
            }

            if let Some(desc) = &feature.description {
                md.push_str(&format!("{}\n\n", desc));
            }

            if !feature.components.is_empty() {
                md.push_str("### Components\n\n");
                for comp_id in &feature.components {
                    md.push_str(&format!("- {}\n", comp_id));
                }
                md.push('\n');
            }
        }

        md
    }

    fn render_markdown_custom_actions(&self, project: &ProjectDocs) -> String {
        let mut md = String::new();
        md.push_str("# Custom Actions\n\n");

        for ca in &project.custom_actions {
            md.push_str(&format!("## {}\n\n", ca.id));

            if let Some(desc) = &ca.description {
                md.push_str(&format!("{}\n\n", desc));
            }

            if let Some(binary) = &ca.binary {
                md.push_str(&format!("- **Binary:** {}\n", binary));
            }
            if let Some(execute) = &ca.execute {
                md.push_str(&format!("- **Execute:** {}\n", execute));
            }
            md.push('\n');
        }

        md
    }

    fn render_markdown_properties(&self, project: &ProjectDocs) -> String {
        let mut md = String::new();
        md.push_str("# Properties\n\n");
        md.push_str("| Id | Value | Flags | Description |\n");
        md.push_str("|---|---|---|---|\n");

        for prop in &project.properties {
            let mut flags = Vec::new();
            if prop.secure { flags.push("Secure"); }
            if prop.admin { flags.push("Admin"); }
            if prop.hidden { flags.push("Hidden"); }

            md.push_str(&format!(
                "| `{}` | {} | {} | {} |\n",
                prop.id,
                prop.value.as_deref().unwrap_or("-"),
                if flags.is_empty() { "-".to_string() } else { flags.join(", ") },
                prop.description.as_deref().unwrap_or("-")
            ));
        }

        md
    }
}

/// Escape HTML special characters
fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// CSS content for HTML output
const CSS_CONTENT: &str = r#"
:root {
    --primary: #2563eb;
    --bg: #ffffff;
    --text: #1f2937;
    --border: #e5e7eb;
    --code-bg: #f3f4f6;
}

* {
    box-sizing: border-box;
    margin: 0;
    padding: 0;
}

body {
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
    line-height: 1.6;
    color: var(--text);
    background: var(--bg);
}

.container {
    max-width: 1200px;
    margin: 0 auto;
    padding: 2rem;
}

h1 {
    font-size: 2rem;
    margin-bottom: 1rem;
    color: var(--primary);
}

h2 {
    font-size: 1.5rem;
    margin: 2rem 0 1rem;
    padding-bottom: 0.5rem;
    border-bottom: 1px solid var(--border);
}

h3 {
    font-size: 1.25rem;
    margin: 1.5rem 0 0.75rem;
}

p {
    margin-bottom: 1rem;
}

a {
    color: var(--primary);
    text-decoration: none;
}

a:hover {
    text-decoration: underline;
}

table {
    width: 100%;
    border-collapse: collapse;
    margin: 1rem 0;
}

th, td {
    padding: 0.75rem;
    text-align: left;
    border-bottom: 1px solid var(--border);
}

th {
    background: var(--code-bg);
    font-weight: 600;
}

tr:hover {
    background: var(--code-bg);
}

code {
    background: var(--code-bg);
    padding: 0.2rem 0.4rem;
    border-radius: 4px;
    font-family: 'SF Mono', Consolas, monospace;
    font-size: 0.9em;
}

dl {
    margin: 1rem 0;
}

dt {
    font-weight: 600;
    margin-top: 0.5rem;
}

dd {
    margin-left: 1rem;
    color: #6b7280;
}

ul {
    margin: 1rem 0;
    padding-left: 2rem;
}

li {
    margin: 0.25rem 0;
}

.toc {
    background: var(--code-bg);
    padding: 1rem 1.5rem;
    border-radius: 8px;
    margin: 1rem 0;
}

.toc h2 {
    margin-top: 0;
    border: none;
}

.description {
    color: #6b7280;
    font-style: italic;
}

footer {
    margin-top: 3rem;
    padding: 1rem 2rem;
    border-top: 1px solid var(--border);
    text-align: center;
    color: #9ca3af;
    font-size: 0.875rem;
}
"#;

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_project() -> ProjectDocs {
        let mut project = ProjectDocs::new("TestProject")
            .with_description("A test project")
            .with_version("1.0.0");

        project.components.push(ComponentDocs {
            id: "C1".to_string(),
            guid: Some("*".to_string()),
            description: Some("Main component".to_string()),
            file: std::path::PathBuf::from("test.wxs"),
            line: 10,
            files: vec![FileEntry {
                id: Some("F1".to_string()),
                source: Some("app.exe".to_string()),
                name: None,
                description: None,
            }],
            registry: Vec::new(),
            services: Vec::new(),
            included_in_features: vec!["MainFeature".to_string()],
        });

        project.features.push(FeatureDocs {
            id: "MainFeature".to_string(),
            title: Some("Main Feature".to_string()),
            level: Some("1".to_string()),
            description: Some("The main feature".to_string()),
            file: std::path::PathBuf::from("test.wxs"),
            line: 5,
            components: vec!["C1".to_string()],
            children: Vec::new(),
            parent: None,
        });

        project.properties.push(PropertyDocs {
            id: "INSTALLDIR".to_string(),
            value: Some("C:\\Program Files\\Test".to_string()),
            description: Some("Install directory".to_string()),
            file: std::path::PathBuf::from("test.wxs"),
            line: 3,
            secure: true,
            admin: false,
            hidden: false,
            used_in: Vec::new(),
        });

        project
    }

    #[test]
    fn test_generate_html() {
        let temp_dir = TempDir::new().unwrap();
        let config = DocsConfig {
            format: OutputFormat::Html,
            output_dir: temp_dir.path().to_path_buf(),
            ..Default::default()
        };

        let generator = DocsGenerator::new(config);
        let project = create_test_project();

        generator.generate(&project).unwrap();

        assert!(temp_dir.path().join("index.html").exists());
        assert!(temp_dir.path().join("style.css").exists());
        assert!(temp_dir.path().join("components").exists());
        assert!(temp_dir.path().join("features").exists());
    }

    #[test]
    fn test_generate_markdown() {
        let temp_dir = TempDir::new().unwrap();
        let config = DocsConfig {
            format: OutputFormat::Markdown,
            output_dir: temp_dir.path().to_path_buf(),
            ..Default::default()
        };

        let generator = DocsGenerator::new(config);
        let project = create_test_project();

        generator.generate(&project).unwrap();

        assert!(temp_dir.path().join("README.md").exists());
        assert!(temp_dir.path().join("components.md").exists());
        assert!(temp_dir.path().join("features.md").exists());
    }

    #[test]
    fn test_generate_json() {
        let temp_dir = TempDir::new().unwrap();
        let config = DocsConfig {
            format: OutputFormat::Json,
            output_dir: temp_dir.path().to_path_buf(),
            ..Default::default()
        };

        let generator = DocsGenerator::new(config);
        let project = create_test_project();

        generator.generate(&project).unwrap();

        assert!(temp_dir.path().join("docs.json").exists());

        let content = fs::read_to_string(temp_dir.path().join("docs.json")).unwrap();
        let parsed: ProjectDocs = serde_json::from_str(&content).unwrap();
        assert_eq!(parsed.name, "TestProject");
    }

    #[test]
    fn test_escape_html() {
        assert_eq!(escape_html("<div>"), "&lt;div&gt;");
        assert_eq!(escape_html("a & b"), "a &amp; b");
        assert_eq!(escape_html("\"quoted\""), "&quot;quoted&quot;");
    }

    #[test]
    fn test_html_header() {
        let config = DocsConfig::default();
        let generator = DocsGenerator::new(config);
        let header = generator.html_header("Test Title");

        assert!(header.contains("<title>Test Title</title>"));
        assert!(header.contains("<!DOCTYPE html>"));
    }

    #[test]
    fn test_html_footer() {
        let config = DocsConfig::default();
        let generator = DocsGenerator::new(config);
        let footer = generator.html_footer();

        assert!(footer.contains("wix-docs"));
        assert!(footer.contains("</body>"));
    }

    #[test]
    fn test_render_markdown_index() {
        let config = DocsConfig::default();
        let generator = DocsGenerator::new(config);
        let project = create_test_project();

        let md = generator.render_markdown_index(&project);

        assert!(md.contains("# TestProject"));
        assert!(md.contains("A test project"));
        assert!(md.contains("**Version:** 1.0.0"));
    }

    #[test]
    fn test_empty_project() {
        let temp_dir = TempDir::new().unwrap();
        let config = DocsConfig {
            format: OutputFormat::Html,
            output_dir: temp_dir.path().to_path_buf(),
            ..Default::default()
        };

        let generator = DocsGenerator::new(config);
        let project = ProjectDocs::new("EmptyProject");

        generator.generate(&project).unwrap();

        assert!(temp_dir.path().join("index.html").exists());
    }

    #[test]
    fn test_custom_actions_html() {
        let temp_dir = TempDir::new().unwrap();
        let config = DocsConfig {
            format: OutputFormat::Html,
            output_dir: temp_dir.path().to_path_buf(),
            ..Default::default()
        };

        let generator = DocsGenerator::new(config);
        let mut project = ProjectDocs::new("TestProject");

        project.custom_actions.push(CustomActionDocs {
            id: "CA_Test".to_string(),
            description: Some("Test action".to_string()),
            file: std::path::PathBuf::from("test.wxs"),
            line: 20,
            binary: Some("TestDll".to_string()),
            dll_entry: Some("EntryPoint".to_string()),
            script: None,
            execute: Some("deferred".to_string()),
            return_attr: None,
            impersonate: None,
            scheduled_in: vec![ScheduleEntry {
                sequence: "InstallExecuteSequence".to_string(),
                condition: Some("NOT Installed".to_string()),
                before: None,
                after: Some("InstallFiles".to_string()),
            }],
        });

        generator.generate(&project).unwrap();

        assert!(temp_dir.path().join("custom-actions.html").exists());
    }
}

//! Content-based file type detection for code files
//!
//! Detects programming languages and file types using multiple strategies:
//! - File extension
//! - Shebang line (#!/...)
//! - Content patterns and fingerprints
//! - XML namespace detection
//!
//! # Example
//!
//! ```
//! use code_detector::{detect, detect_from_content, Language};
//!
//! // Detect from file path and content
//! let lang = detect("example.wxs", "<Wix xmlns='...'>");
//! assert_eq!(lang, Language::Wix);
//!
//! // Detect from content only
//! let lang = detect_from_content("#!/bin/bash\necho hello");
//! assert_eq!(lang, Language::Bash);
//! ```

use serde::{Deserialize, Serialize};
use std::path::Path;

/// Supported languages/file types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    // Installer/Config
    Wix,
    Ansible,
    Docker,
    Dockerfile,
    DockerCompose,
    Kubernetes,
    Terraform,

    // CI/CD
    Jenkins,
    GitHubActions,
    GitLabCI,
    AzurePipelines,

    // Programming
    Rust,
    Go,
    Python,
    JavaScript,
    TypeScript,
    Java,
    CSharp,
    Cpp,
    C,
    Ruby,
    Php,
    Swift,
    Kotlin,
    Scala,

    // Scripting
    Bash,
    PowerShell,
    Batch,
    Groovy,
    Lua,
    Perl,

    // Data/Config
    Json,
    Yaml,
    Toml,
    Xml,
    Html,
    Css,
    Markdown,
    Ini,
    Properties,

    // Other
    Makefile,
    Cmake,
    Sql,
    GraphQL,
    Protobuf,

    /// Unknown or unsupported file type
    #[default]
    Unknown,
}

impl Language {
    /// Get the language name as a string
    pub fn as_str(&self) -> &'static str {
        match self {
            Language::Wix => "wix",
            Language::Ansible => "ansible",
            Language::Docker => "docker",
            Language::Dockerfile => "dockerfile",
            Language::DockerCompose => "docker-compose",
            Language::Kubernetes => "kubernetes",
            Language::Terraform => "terraform",
            Language::Jenkins => "jenkins",
            Language::GitHubActions => "github-actions",
            Language::GitLabCI => "gitlab-ci",
            Language::AzurePipelines => "azure-pipelines",
            Language::Rust => "rust",
            Language::Go => "go",
            Language::Python => "python",
            Language::JavaScript => "javascript",
            Language::TypeScript => "typescript",
            Language::Java => "java",
            Language::CSharp => "csharp",
            Language::Cpp => "cpp",
            Language::C => "c",
            Language::Ruby => "ruby",
            Language::Php => "php",
            Language::Swift => "swift",
            Language::Kotlin => "kotlin",
            Language::Scala => "scala",
            Language::Bash => "bash",
            Language::PowerShell => "powershell",
            Language::Batch => "batch",
            Language::Groovy => "groovy",
            Language::Lua => "lua",
            Language::Perl => "perl",
            Language::Json => "json",
            Language::Yaml => "yaml",
            Language::Toml => "toml",
            Language::Xml => "xml",
            Language::Html => "html",
            Language::Css => "css",
            Language::Markdown => "markdown",
            Language::Ini => "ini",
            Language::Properties => "properties",
            Language::Makefile => "makefile",
            Language::Cmake => "cmake",
            Language::Sql => "sql",
            Language::GraphQL => "graphql",
            Language::Protobuf => "protobuf",
            Language::Unknown => "unknown",
        }
    }

    /// Get display name for the language
    pub fn display_name(&self) -> &'static str {
        match self {
            Language::Wix => "WiX",
            Language::Ansible => "Ansible",
            Language::Docker => "Docker",
            Language::Dockerfile => "Dockerfile",
            Language::DockerCompose => "Docker Compose",
            Language::Kubernetes => "Kubernetes",
            Language::Terraform => "Terraform",
            Language::Jenkins => "Jenkinsfile",
            Language::GitHubActions => "GitHub Actions",
            Language::GitLabCI => "GitLab CI",
            Language::AzurePipelines => "Azure Pipelines",
            Language::Rust => "Rust",
            Language::Go => "Go",
            Language::Python => "Python",
            Language::JavaScript => "JavaScript",
            Language::TypeScript => "TypeScript",
            Language::Java => "Java",
            Language::CSharp => "C#",
            Language::Cpp => "C++",
            Language::C => "C",
            Language::Ruby => "Ruby",
            Language::Php => "PHP",
            Language::Swift => "Swift",
            Language::Kotlin => "Kotlin",
            Language::Scala => "Scala",
            Language::Bash => "Bash",
            Language::PowerShell => "PowerShell",
            Language::Batch => "Batch",
            Language::Groovy => "Groovy",
            Language::Lua => "Lua",
            Language::Perl => "Perl",
            Language::Json => "JSON",
            Language::Yaml => "YAML",
            Language::Toml => "TOML",
            Language::Xml => "XML",
            Language::Html => "HTML",
            Language::Css => "CSS",
            Language::Markdown => "Markdown",
            Language::Ini => "INI",
            Language::Properties => "Properties",
            Language::Makefile => "Makefile",
            Language::Cmake => "CMake",
            Language::Sql => "SQL",
            Language::GraphQL => "GraphQL",
            Language::Protobuf => "Protocol Buffers",
            Language::Unknown => "Unknown",
        }
    }

    /// Get common file extensions for this language
    pub fn extensions(&self) -> &'static [&'static str] {
        match self {
            Language::Wix => &["wxs", "wxi", "wxl"],
            Language::Ansible => &["yml", "yaml"],
            Language::Docker => &[],
            Language::Dockerfile => &["dockerfile"],
            Language::DockerCompose => &["yml", "yaml"],
            Language::Kubernetes => &["yml", "yaml"],
            Language::Terraform => &["tf", "tfvars"],
            Language::Jenkins => &["jenkinsfile", "groovy"],
            Language::GitHubActions => &["yml", "yaml"],
            Language::GitLabCI => &["yml", "yaml"],
            Language::AzurePipelines => &["yml", "yaml"],
            Language::Rust => &["rs"],
            Language::Go => &["go"],
            Language::Python => &["py", "pyw", "pyi"],
            Language::JavaScript => &["js", "mjs", "cjs"],
            Language::TypeScript => &["ts", "mts", "cts"],
            Language::Java => &["java"],
            Language::CSharp => &["cs"],
            Language::Cpp => &["cpp", "cc", "cxx", "hpp", "hxx", "h"],
            Language::C => &["c", "h"],
            Language::Ruby => &["rb", "rake", "gemspec"],
            Language::Php => &["php", "phtml"],
            Language::Swift => &["swift"],
            Language::Kotlin => &["kt", "kts"],
            Language::Scala => &["scala", "sc"],
            Language::Bash => &["sh", "bash", "zsh"],
            Language::PowerShell => &["ps1", "psm1", "psd1"],
            Language::Batch => &["bat", "cmd"],
            Language::Groovy => &["groovy", "gvy", "gy", "gsh"],
            Language::Lua => &["lua"],
            Language::Perl => &["pl", "pm", "t"],
            Language::Json => &["json", "jsonc"],
            Language::Yaml => &["yml", "yaml"],
            Language::Toml => &["toml"],
            Language::Xml => &["xml", "xsl", "xslt", "xsd"],
            Language::Html => &["html", "htm", "xhtml"],
            Language::Css => &["css", "scss", "sass", "less"],
            Language::Markdown => &["md", "markdown"],
            Language::Ini => &["ini", "cfg", "conf"],
            Language::Properties => &["properties"],
            Language::Makefile => &["makefile", "mk"],
            Language::Cmake => &["cmake"],
            Language::Sql => &["sql"],
            Language::GraphQL => &["graphql", "gql"],
            Language::Protobuf => &["proto"],
            Language::Unknown => &[],
        }
    }
}

/// Detection confidence level
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Confidence {
    /// Low confidence - guessed from extension only
    Low,
    /// Medium confidence - pattern match or filename
    Medium,
    /// High confidence - shebang or strong content fingerprint
    High,
    /// Certain - unambiguous detection (e.g., XML namespace)
    Certain,
}

/// Detection result with confidence
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DetectionResult {
    pub language: Language,
    pub confidence: Confidence,
    pub reason: &'static str,
}

impl DetectionResult {
    pub fn new(language: Language, confidence: Confidence, reason: &'static str) -> Self {
        Self {
            language,
            confidence,
            reason,
        }
    }
}

/// Detect language from file path and content
pub fn detect(path: &str, content: &str) -> Language {
    detect_with_confidence(path, content).language
}

/// Detect language from content only (no file path)
pub fn detect_from_content(content: &str) -> Language {
    detect_content_only(content).language
}

/// Detect language with confidence information
pub fn detect_with_confidence(path: &str, content: &str) -> DetectionResult {
    // Try content-based detection first (highest confidence)
    let content_result = detect_content_only(content);
    if content_result.confidence >= Confidence::High {
        return content_result;
    }

    // Try filename-based detection
    let filename_result = detect_from_filename(path);

    // If we have a filename result and content result agrees or is unknown, use filename
    if filename_result.language != Language::Unknown {
        // Check if content detection gives more specific result
        if content_result.language != Language::Unknown
            && content_result.confidence > filename_result.confidence
        {
            return content_result;
        }

        // Refine YAML/JSON based on content
        if filename_result.language == Language::Yaml {
            if let Some(refined) = refine_yaml_type(path, content) {
                return refined;
            }
        }

        return filename_result;
    }

    // Fall back to content detection
    if content_result.language != Language::Unknown {
        return content_result;
    }

    DetectionResult::new(Language::Unknown, Confidence::Low, "No detection")
}

/// Detect from filename/path only
fn detect_from_filename(path: &str) -> DetectionResult {
    let path = Path::new(path);
    let filename = path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_lowercase();

    // Check exact filename matches first
    match filename.as_str() {
        "dockerfile" => return DetectionResult::new(Language::Dockerfile, Confidence::Certain, "Dockerfile filename"),
        "docker-compose.yml" | "docker-compose.yaml" | "compose.yml" | "compose.yaml" => {
            return DetectionResult::new(Language::DockerCompose, Confidence::Certain, "Docker Compose filename");
        }
        "jenkinsfile" => return DetectionResult::new(Language::Jenkins, Confidence::Certain, "Jenkinsfile filename"),
        "makefile" | "gnumakefile" => return DetectionResult::new(Language::Makefile, Confidence::Certain, "Makefile filename"),
        "cmakelists.txt" => return DetectionResult::new(Language::Cmake, Confidence::Certain, "CMakeLists.txt filename"),
        "cargo.toml" => return DetectionResult::new(Language::Toml, Confidence::Certain, "Cargo.toml filename"),
        "package.json" => return DetectionResult::new(Language::Json, Confidence::Certain, "package.json filename"),
        "tsconfig.json" => return DetectionResult::new(Language::Json, Confidence::Certain, "tsconfig.json filename"),
        ".gitlab-ci.yml" => return DetectionResult::new(Language::GitLabCI, Confidence::Certain, "GitLab CI filename"),
        "azure-pipelines.yml" | "azure-pipelines.yaml" => {
            return DetectionResult::new(Language::AzurePipelines, Confidence::Certain, "Azure Pipelines filename");
        }
        _ => {}
    }

    // Check for GitHub Actions path
    if path.to_string_lossy().contains(".github/workflows/") {
        if filename.ends_with(".yml") || filename.ends_with(".yaml") {
            return DetectionResult::new(Language::GitHubActions, Confidence::Certain, "GitHub Actions path");
        }
    }

    // Check extension
    let ext = path.extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    match ext.as_str() {
        "wxs" | "wxi" | "wxl" => DetectionResult::new(Language::Wix, Confidence::High, "WiX extension"),
        "rs" => DetectionResult::new(Language::Rust, Confidence::High, "Rust extension"),
        "go" => DetectionResult::new(Language::Go, Confidence::High, "Go extension"),
        "py" | "pyw" | "pyi" => DetectionResult::new(Language::Python, Confidence::High, "Python extension"),
        "js" | "mjs" | "cjs" => DetectionResult::new(Language::JavaScript, Confidence::High, "JavaScript extension"),
        "ts" | "mts" | "cts" => DetectionResult::new(Language::TypeScript, Confidence::High, "TypeScript extension"),
        "tsx" => DetectionResult::new(Language::TypeScript, Confidence::High, "TypeScript JSX extension"),
        "jsx" => DetectionResult::new(Language::JavaScript, Confidence::High, "JavaScript JSX extension"),
        "java" => DetectionResult::new(Language::Java, Confidence::High, "Java extension"),
        "cs" => DetectionResult::new(Language::CSharp, Confidence::High, "C# extension"),
        "cpp" | "cc" | "cxx" | "hpp" | "hxx" => DetectionResult::new(Language::Cpp, Confidence::High, "C++ extension"),
        "c" => DetectionResult::new(Language::C, Confidence::Medium, "C extension"),
        "h" => DetectionResult::new(Language::C, Confidence::Low, "C/C++ header"),
        "rb" | "rake" | "gemspec" => DetectionResult::new(Language::Ruby, Confidence::High, "Ruby extension"),
        "php" | "phtml" => DetectionResult::new(Language::Php, Confidence::High, "PHP extension"),
        "swift" => DetectionResult::new(Language::Swift, Confidence::High, "Swift extension"),
        "kt" | "kts" => DetectionResult::new(Language::Kotlin, Confidence::High, "Kotlin extension"),
        "scala" | "sc" => DetectionResult::new(Language::Scala, Confidence::High, "Scala extension"),
        "sh" | "bash" | "zsh" => DetectionResult::new(Language::Bash, Confidence::High, "Shell extension"),
        "ps1" | "psm1" | "psd1" => DetectionResult::new(Language::PowerShell, Confidence::High, "PowerShell extension"),
        "bat" | "cmd" => DetectionResult::new(Language::Batch, Confidence::High, "Batch extension"),
        "groovy" | "gvy" | "gy" | "gsh" => DetectionResult::new(Language::Groovy, Confidence::High, "Groovy extension"),
        "lua" => DetectionResult::new(Language::Lua, Confidence::High, "Lua extension"),
        "pl" | "pm" => DetectionResult::new(Language::Perl, Confidence::High, "Perl extension"),
        "json" | "jsonc" => DetectionResult::new(Language::Json, Confidence::High, "JSON extension"),
        "yml" | "yaml" => DetectionResult::new(Language::Yaml, Confidence::Medium, "YAML extension"),
        "toml" => DetectionResult::new(Language::Toml, Confidence::High, "TOML extension"),
        "xml" | "xsl" | "xslt" | "xsd" => DetectionResult::new(Language::Xml, Confidence::Medium, "XML extension"),
        "html" | "htm" | "xhtml" => DetectionResult::new(Language::Html, Confidence::High, "HTML extension"),
        "css" | "scss" | "sass" | "less" => DetectionResult::new(Language::Css, Confidence::High, "CSS extension"),
        "md" | "markdown" => DetectionResult::new(Language::Markdown, Confidence::High, "Markdown extension"),
        "ini" | "cfg" => DetectionResult::new(Language::Ini, Confidence::Medium, "INI extension"),
        "conf" => DetectionResult::new(Language::Ini, Confidence::Low, "Config extension"),
        "properties" => DetectionResult::new(Language::Properties, Confidence::High, "Properties extension"),
        "mk" => DetectionResult::new(Language::Makefile, Confidence::High, "Makefile extension"),
        "cmake" => DetectionResult::new(Language::Cmake, Confidence::High, "CMake extension"),
        "sql" => DetectionResult::new(Language::Sql, Confidence::High, "SQL extension"),
        "graphql" | "gql" => DetectionResult::new(Language::GraphQL, Confidence::High, "GraphQL extension"),
        "proto" => DetectionResult::new(Language::Protobuf, Confidence::High, "Protobuf extension"),
        "tf" | "tfvars" => DetectionResult::new(Language::Terraform, Confidence::High, "Terraform extension"),
        _ => DetectionResult::new(Language::Unknown, Confidence::Low, "Unknown extension"),
    }
}

/// Detect from content only
fn detect_content_only(content: &str) -> DetectionResult {
    let content = content.trim();
    if content.is_empty() {
        return DetectionResult::new(Language::Unknown, Confidence::Low, "Empty content");
    }

    // Check shebang first
    if let Some(result) = detect_shebang(content) {
        return result;
    }

    // Check XML/HTML
    if let Some(result) = detect_xml_type(content) {
        return result;
    }

    // Check JSON
    if (content.starts_with('{') && content.ends_with('}'))
        || (content.starts_with('[') && content.ends_with(']'))
    {
        return DetectionResult::new(Language::Json, Confidence::Medium, "JSON structure");
    }

    // Check for specific content patterns
    if let Some(result) = detect_content_patterns(content) {
        return result;
    }

    DetectionResult::new(Language::Unknown, Confidence::Low, "No content match")
}

/// Detect language from shebang line
fn detect_shebang(content: &str) -> Option<DetectionResult> {
    let first_line = content.lines().next()?;
    if !first_line.starts_with("#!") {
        return None;
    }

    let shebang = first_line.to_lowercase();

    if shebang.contains("python") {
        Some(DetectionResult::new(Language::Python, Confidence::Certain, "Python shebang"))
    } else if shebang.contains("bash") {
        Some(DetectionResult::new(Language::Bash, Confidence::Certain, "Bash shebang"))
    } else if shebang.contains("/sh") {
        Some(DetectionResult::new(Language::Bash, Confidence::High, "Shell shebang"))
    } else if shebang.contains("zsh") {
        Some(DetectionResult::new(Language::Bash, Confidence::Certain, "Zsh shebang"))
    } else if shebang.contains("node") || shebang.contains("nodejs") {
        Some(DetectionResult::new(Language::JavaScript, Confidence::Certain, "Node.js shebang"))
    } else if shebang.contains("ruby") {
        Some(DetectionResult::new(Language::Ruby, Confidence::Certain, "Ruby shebang"))
    } else if shebang.contains("perl") {
        Some(DetectionResult::new(Language::Perl, Confidence::Certain, "Perl shebang"))
    } else if shebang.contains("php") {
        Some(DetectionResult::new(Language::Php, Confidence::Certain, "PHP shebang"))
    } else if shebang.contains("lua") {
        Some(DetectionResult::new(Language::Lua, Confidence::Certain, "Lua shebang"))
    } else if shebang.contains("groovy") {
        Some(DetectionResult::new(Language::Groovy, Confidence::Certain, "Groovy shebang"))
    } else if shebang.contains("pwsh") || shebang.contains("powershell") {
        Some(DetectionResult::new(Language::PowerShell, Confidence::Certain, "PowerShell shebang"))
    } else {
        None
    }
}

/// Detect XML-based file types
fn detect_xml_type(content: &str) -> Option<DetectionResult> {
    // Check for XML declaration or root element
    let is_xml = content.starts_with("<?xml")
        || content.starts_with("<!")
        || (content.starts_with('<') && !content.starts_with("<!DOCTYPE html"));

    if !is_xml {
        // Check for HTML doctype
        if content.to_lowercase().starts_with("<!doctype html") {
            return Some(DetectionResult::new(Language::Html, Confidence::Certain, "HTML doctype"));
        }
        return None;
    }

    // Check for WiX namespace
    if content.contains("http://wixtoolset.org/")
        || content.contains("http://schemas.microsoft.com/wix/")
        || content.contains("<Wix")
        || content.contains("<Include")
        || content.contains("<Fragment")
    {
        return Some(DetectionResult::new(Language::Wix, Confidence::Certain, "WiX namespace/element"));
    }

    // Check for HTML
    if content.to_lowercase().contains("<html") {
        return Some(DetectionResult::new(Language::Html, Confidence::High, "HTML element"));
    }

    // Generic XML
    Some(DetectionResult::new(Language::Xml, Confidence::Medium, "XML structure"))
}

/// Detect content patterns
fn detect_content_patterns(content: &str) -> Option<DetectionResult> {
    let lower = content.to_lowercase();

    // Ansible patterns
    if lower.contains("hosts:") && (lower.contains("tasks:") || lower.contains("roles:")) {
        return Some(DetectionResult::new(Language::Ansible, Confidence::High, "Ansible playbook structure"));
    }
    if lower.contains("ansible.builtin") || lower.contains("become:") && lower.contains("tasks:") {
        return Some(DetectionResult::new(Language::Ansible, Confidence::High, "Ansible module pattern"));
    }

    // Kubernetes patterns
    if lower.contains("apiversion:") && lower.contains("kind:") {
        return Some(DetectionResult::new(Language::Kubernetes, Confidence::High, "Kubernetes manifest"));
    }

    // Terraform patterns
    if content.contains("resource \"") || content.contains("provider \"") || content.contains("variable \"") {
        return Some(DetectionResult::new(Language::Terraform, Confidence::High, "Terraform block"));
    }

    // Jenkins pipeline
    if content.contains("pipeline {") || content.contains("node {") && content.contains("stage(") {
        return Some(DetectionResult::new(Language::Jenkins, Confidence::High, "Jenkins pipeline"));
    }

    // Rust patterns
    if content.contains("fn main()") || content.contains("pub fn ") || content.contains("impl ") {
        return Some(DetectionResult::new(Language::Rust, Confidence::Medium, "Rust syntax"));
    }

    // Go patterns
    if content.contains("package main") || content.contains("func main()") {
        return Some(DetectionResult::new(Language::Go, Confidence::High, "Go syntax"));
    }

    // Python patterns
    if content.contains("def ") && content.contains("):") || content.contains("import ") && content.contains("from ") {
        return Some(DetectionResult::new(Language::Python, Confidence::Medium, "Python syntax"));
    }

    // PowerShell patterns
    if content.contains("$PSVersionTable") || content.contains("Write-Host") || content.contains("Get-") {
        return Some(DetectionResult::new(Language::PowerShell, Confidence::High, "PowerShell cmdlet"));
    }

    None
}

/// Refine YAML type based on path and content
fn refine_yaml_type(path: &str, content: &str) -> Option<DetectionResult> {
    let lower_path = path.to_lowercase();
    let lower_content = content.to_lowercase();

    // Docker Compose
    if lower_path.contains("docker-compose") || lower_path.contains("compose.y") {
        return Some(DetectionResult::new(Language::DockerCompose, Confidence::Certain, "Docker Compose filename"));
    }
    if lower_content.contains("services:") && (lower_content.contains("image:") || lower_content.contains("build:")) {
        return Some(DetectionResult::new(Language::DockerCompose, Confidence::High, "Docker Compose structure"));
    }

    // GitHub Actions
    if lower_path.contains(".github/workflows/") {
        return Some(DetectionResult::new(Language::GitHubActions, Confidence::Certain, "GitHub Actions path"));
    }
    if lower_content.contains("runs-on:") && lower_content.contains("steps:") {
        return Some(DetectionResult::new(Language::GitHubActions, Confidence::High, "GitHub Actions structure"));
    }

    // GitLab CI
    if lower_path.contains(".gitlab-ci") {
        return Some(DetectionResult::new(Language::GitLabCI, Confidence::Certain, "GitLab CI filename"));
    }
    if lower_content.contains("stages:") && (lower_content.contains("script:") || lower_content.contains("image:")) {
        return Some(DetectionResult::new(Language::GitLabCI, Confidence::Medium, "GitLab CI structure"));
    }

    // Azure Pipelines
    if lower_path.contains("azure-pipeline") {
        return Some(DetectionResult::new(Language::AzurePipelines, Confidence::Certain, "Azure Pipelines filename"));
    }
    if lower_content.contains("trigger:") && lower_content.contains("pool:") {
        return Some(DetectionResult::new(Language::AzurePipelines, Confidence::High, "Azure Pipelines structure"));
    }

    // Ansible
    if lower_content.contains("hosts:") && (lower_content.contains("tasks:") || lower_content.contains("roles:")) {
        return Some(DetectionResult::new(Language::Ansible, Confidence::High, "Ansible playbook"));
    }
    if lower_content.contains("ansible.builtin") {
        return Some(DetectionResult::new(Language::Ansible, Confidence::High, "Ansible module"));
    }

    // Kubernetes
    if lower_content.contains("apiversion:") && lower_content.contains("kind:") {
        return Some(DetectionResult::new(Language::Kubernetes, Confidence::High, "Kubernetes manifest"));
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_wix_by_extension() {
        assert_eq!(detect("product.wxs", ""), Language::Wix);
        assert_eq!(detect("include.wxi", ""), Language::Wix);
        assert_eq!(detect("strings.wxl", ""), Language::Wix);
    }

    #[test]
    fn test_detect_wix_by_content() {
        let content = r#"<?xml version="1.0"?>
<Wix xmlns="http://wixtoolset.org/schemas/v4/wxs">
  <Package Name="Test" />
</Wix>"#;
        assert_eq!(detect("unknown.xml", content), Language::Wix);
        assert_eq!(detect_from_content(content), Language::Wix);
    }

    #[test]
    fn test_detect_shebang() {
        assert_eq!(detect_from_content("#!/bin/bash\necho hello"), Language::Bash);
        assert_eq!(detect_from_content("#!/usr/bin/env python3\nprint('hi')"), Language::Python);
        assert_eq!(detect_from_content("#!/usr/bin/node\nconsole.log('hi')"), Language::JavaScript);
        assert_eq!(detect_from_content("#!/usr/bin/env ruby\nputs 'hi'"), Language::Ruby);
    }

    #[test]
    fn test_detect_dockerfile() {
        assert_eq!(detect("Dockerfile", "FROM ubuntu"), Language::Dockerfile);
        assert_eq!(detect("dockerfile", "FROM alpine"), Language::Dockerfile);
    }

    #[test]
    fn test_detect_docker_compose() {
        let content = "services:\n  web:\n    image: nginx";
        assert_eq!(detect("docker-compose.yml", content), Language::DockerCompose);
        assert_eq!(detect("compose.yaml", content), Language::DockerCompose);
    }

    #[test]
    fn test_detect_github_actions() {
        let content = "name: CI\non: push\njobs:\n  build:\n    runs-on: ubuntu-latest\n    steps:";
        assert_eq!(detect(".github/workflows/ci.yml", content), Language::GitHubActions);
    }

    #[test]
    fn test_detect_kubernetes() {
        let content = "apiVersion: v1\nkind: Pod\nmetadata:\n  name: test";
        assert_eq!(detect("pod.yaml", content), Language::Kubernetes);
    }

    #[test]
    fn test_detect_ansible() {
        let content = "- hosts: all\n  tasks:\n    - name: Install";
        assert_eq!(detect("playbook.yml", content), Language::Ansible);
    }

    #[test]
    fn test_detect_terraform() {
        let content = r#"resource "aws_instance" "example" {
  ami = "ami-12345"
}"#;
        assert_eq!(detect("main.tf", content), Language::Terraform);
        assert_eq!(detect_from_content(content), Language::Terraform);
    }

    #[test]
    fn test_detect_jenkinsfile() {
        let content = "pipeline {\n  agent any\n  stages {\n    stage('Build') {";
        assert_eq!(detect("Jenkinsfile", content), Language::Jenkins);
        assert_eq!(detect_from_content(content), Language::Jenkins);
    }

    #[test]
    fn test_detect_programming_languages() {
        assert_eq!(detect("main.rs", ""), Language::Rust);
        assert_eq!(detect("main.go", ""), Language::Go);
        assert_eq!(detect("app.py", ""), Language::Python);
        assert_eq!(detect("index.js", ""), Language::JavaScript);
        assert_eq!(detect("app.ts", ""), Language::TypeScript);
        assert_eq!(detect("Main.java", ""), Language::Java);
        assert_eq!(detect("Program.cs", ""), Language::CSharp);
        assert_eq!(detect("main.cpp", ""), Language::Cpp);
        assert_eq!(detect("app.rb", ""), Language::Ruby);
    }

    #[test]
    fn test_detect_json() {
        let content = r#"{"name": "test", "version": "1.0"}"#;
        assert_eq!(detect("package.json", content), Language::Json);
        assert_eq!(detect_from_content(content), Language::Json);
    }

    #[test]
    fn test_detect_html() {
        let content = "<!DOCTYPE html>\n<html><head></head><body></body></html>";
        assert_eq!(detect("index.html", content), Language::Html);
        assert_eq!(detect_from_content(content), Language::Html);
    }

    #[test]
    fn test_confidence_levels() {
        // Shebang should be Certain
        let result = detect_with_confidence("script", "#!/bin/bash\necho hi");
        assert_eq!(result.confidence, Confidence::Certain);

        // Extension should be High
        let result = detect_with_confidence("main.rs", "");
        assert_eq!(result.confidence, Confidence::High);

        // YAML extension alone is Medium (could be many things)
        let result = detect_with_confidence("config.yml", "key: value");
        assert!(result.confidence >= Confidence::Medium);
    }

    #[test]
    fn test_language_as_str() {
        assert_eq!(Language::Wix.as_str(), "wix");
        assert_eq!(Language::Rust.as_str(), "rust");
        assert_eq!(Language::GitHubActions.as_str(), "github-actions");
    }

    #[test]
    fn test_language_display_name() {
        assert_eq!(Language::Wix.display_name(), "WiX");
        assert_eq!(Language::CSharp.display_name(), "C#");
        assert_eq!(Language::GitHubActions.display_name(), "GitHub Actions");
    }

    #[test]
    fn test_language_extensions() {
        assert!(Language::Wix.extensions().contains(&"wxs"));
        assert!(Language::Rust.extensions().contains(&"rs"));
        assert!(Language::Python.extensions().contains(&"py"));
    }
}

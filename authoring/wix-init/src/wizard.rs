//! wix-wizard - Interactive wizard for WiX project setup
//!
//! Provides step-by-step project configuration and generation.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Wizard step definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WizardStep {
    pub id: String,
    pub title: String,
    pub description: String,
    pub questions: Vec<Question>,
}

/// Question types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QuestionType {
    Text,
    Number,
    Boolean,
    Choice(Vec<String>),
    MultiChoice(Vec<String>),
    Path,
    Guid,
}

/// A wizard question
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Question {
    pub id: String,
    pub prompt: String,
    pub question_type: QuestionType,
    pub required: bool,
    pub default: Option<String>,
    pub validation: Option<String>,
    pub help: Option<String>,
}

impl Question {
    pub fn text(id: &str, prompt: &str) -> Self {
        Self {
            id: id.to_string(),
            prompt: prompt.to_string(),
            question_type: QuestionType::Text,
            required: true,
            default: None,
            validation: None,
            help: None,
        }
    }

    pub fn boolean(id: &str, prompt: &str, default: bool) -> Self {
        Self {
            id: id.to_string(),
            prompt: prompt.to_string(),
            question_type: QuestionType::Boolean,
            required: true,
            default: Some(default.to_string()),
            validation: None,
            help: None,
        }
    }

    pub fn choice(id: &str, prompt: &str, options: Vec<&str>) -> Self {
        Self {
            id: id.to_string(),
            prompt: prompt.to_string(),
            question_type: QuestionType::Choice(options.into_iter().map(String::from).collect()),
            required: true,
            default: None,
            validation: None,
            help: None,
        }
    }

    pub fn path(id: &str, prompt: &str) -> Self {
        Self {
            id: id.to_string(),
            prompt: prompt.to_string(),
            question_type: QuestionType::Path,
            required: true,
            default: None,
            validation: None,
            help: None,
        }
    }

    pub fn with_default(mut self, default: &str) -> Self {
        self.default = Some(default.to_string());
        self
    }

    pub fn with_help(mut self, help: &str) -> Self {
        self.help = Some(help.to_string());
        self
    }

    pub fn optional(mut self) -> Self {
        self.required = false;
        self
    }
}

/// Wizard answer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Answer {
    pub question_id: String,
    pub value: AnswerValue,
}

/// Answer value types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnswerValue {
    Text(String),
    Number(i64),
    Boolean(bool),
    Choices(Vec<String>),
    Path(PathBuf),
}

impl AnswerValue {
    pub fn as_str(&self) -> Option<&str> {
        match self {
            AnswerValue::Text(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            AnswerValue::Boolean(b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_path(&self) -> Option<&PathBuf> {
        match self {
            AnswerValue::Path(p) => Some(p),
            _ => None,
        }
    }
}

/// Project configuration from wizard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub name: String,
    pub version: String,
    pub manufacturer: String,
    pub description: Option<String>,
    pub project_type: ProjectType,
    pub source_directory: PathBuf,
    pub output_directory: PathBuf,
    pub features: Vec<FeatureConfig>,
    pub ui_type: String,
    pub include_license: bool,
    pub upgrade_code: Option<String>,
    pub additional_options: HashMap<String, String>,
}

/// Project types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProjectType {
    /// Simple application installer
    Application,
    /// Windows Service installer
    Service,
    /// Library/SDK installer
    Library,
    /// Web application installer
    WebApp,
    /// Driver installer
    Driver,
    /// Bundle/Bootstrapper
    Bundle,
}

impl ProjectType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ProjectType::Application => "application",
            ProjectType::Service => "service",
            ProjectType::Library => "library",
            ProjectType::WebApp => "webapp",
            ProjectType::Driver => "driver",
            ProjectType::Bundle => "bundle",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "application" | "app" => Some(ProjectType::Application),
            "service" | "svc" => Some(ProjectType::Service),
            "library" | "lib" => Some(ProjectType::Library),
            "webapp" | "web" => Some(ProjectType::WebApp),
            "driver" | "drv" => Some(ProjectType::Driver),
            "bundle" | "bootstrapper" => Some(ProjectType::Bundle),
            _ => None,
        }
    }
}

/// Feature configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureConfig {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub level: u32,
    pub directory: Option<String>,
}

/// Wizard definition
pub struct Wizard {
    pub steps: Vec<WizardStep>,
    pub answers: HashMap<String, AnswerValue>,
}

impl Default for Wizard {
    fn default() -> Self {
        Self::new()
    }
}

impl Wizard {
    pub fn new() -> Self {
        Self {
            steps: Self::default_steps(),
            answers: HashMap::new(),
        }
    }

    fn default_steps() -> Vec<WizardStep> {
        vec![
            WizardStep {
                id: "basic".to_string(),
                title: "Basic Information".to_string(),
                description: "Enter basic product information".to_string(),
                questions: vec![
                    Question::text("name", "Product name:")
                        .with_help("The display name of your product"),
                    Question::text("version", "Version:")
                        .with_default("1.0.0")
                        .with_help("Semantic version (e.g., 1.0.0)"),
                    Question::text("manufacturer", "Manufacturer/Company:")
                        .with_help("Your company or organization name"),
                    Question::text("description", "Description:")
                        .optional()
                        .with_help("Brief description of the product"),
                ],
            },
            WizardStep {
                id: "type".to_string(),
                title: "Project Type".to_string(),
                description: "Select the type of installer".to_string(),
                questions: vec![
                    Question::choice(
                        "project_type",
                        "What type of project?",
                        vec!["Application", "Service", "Library", "WebApp", "Driver", "Bundle"],
                    ),
                ],
            },
            WizardStep {
                id: "paths".to_string(),
                title: "Paths".to_string(),
                description: "Configure source and output paths".to_string(),
                questions: vec![
                    Question::path("source_dir", "Source directory:")
                        .with_default(".")
                        .with_help("Directory containing files to install"),
                    Question::path("output_dir", "Output directory:")
                        .with_default("./output")
                        .with_help("Where to generate the installer"),
                ],
            },
            WizardStep {
                id: "ui".to_string(),
                title: "User Interface".to_string(),
                description: "Configure installer UI".to_string(),
                questions: vec![
                    Question::choice(
                        "ui_type",
                        "UI Type:",
                        vec!["Minimal", "InstallDir", "FeatureTree", "Mondo"],
                    )
                    .with_default("InstallDir"),
                    Question::boolean("include_license", "Include license dialog?", true),
                ],
            },
            WizardStep {
                id: "features".to_string(),
                title: "Features".to_string(),
                description: "Configure product features".to_string(),
                questions: vec![
                    Question::boolean("multi_feature", "Multiple features?", false)
                        .with_help("Enable if your product has optional components"),
                ],
            },
        ]
    }

    pub fn set_answer(&mut self, question_id: &str, value: AnswerValue) {
        self.answers.insert(question_id.to_string(), value);
    }

    pub fn get_answer(&self, question_id: &str) -> Option<&AnswerValue> {
        self.answers.get(question_id)
    }

    pub fn get_text(&self, question_id: &str) -> Option<&str> {
        self.answers.get(question_id).and_then(|v| v.as_str())
    }

    pub fn get_bool(&self, question_id: &str) -> Option<bool> {
        self.answers.get(question_id).and_then(|v| v.as_bool())
    }

    pub fn build_config(&self) -> Result<ProjectConfig, String> {
        let name = self.get_text("name")
            .ok_or("Product name is required")?
            .to_string();
        let version = self.get_text("version")
            .unwrap_or("1.0.0")
            .to_string();
        let manufacturer = self.get_text("manufacturer")
            .ok_or("Manufacturer is required")?
            .to_string();

        let project_type_str = self.get_text("project_type").unwrap_or("Application");
        let project_type = ProjectType::from_str(project_type_str)
            .unwrap_or(ProjectType::Application);

        let source_dir = self.answers.get("source_dir")
            .and_then(|v| v.as_path())
            .cloned()
            .unwrap_or_else(|| PathBuf::from("."));

        let output_dir = self.answers.get("output_dir")
            .and_then(|v| v.as_path())
            .cloned()
            .unwrap_or_else(|| PathBuf::from("./output"));

        let ui_type = self.get_text("ui_type")
            .unwrap_or("Minimal")
            .to_string();

        let include_license = self.get_bool("include_license").unwrap_or(false);

        Ok(ProjectConfig {
            name,
            version,
            manufacturer,
            description: self.get_text("description").map(String::from),
            project_type,
            source_directory: source_dir,
            output_directory: output_dir,
            features: vec![FeatureConfig {
                id: "MainFeature".to_string(),
                title: "Main Application".to_string(),
                description: None,
                level: 1,
                directory: Some("INSTALLFOLDER".to_string()),
            }],
            ui_type,
            include_license,
            upgrade_code: None,
            additional_options: HashMap::new(),
        })
    }

    pub fn step_count(&self) -> usize {
        self.steps.len()
    }

    pub fn get_step(&self, index: usize) -> Option<&WizardStep> {
        self.steps.get(index)
    }
}

/// Project generator from wizard config
pub struct ProjectGenerator;

impl ProjectGenerator {
    /// Generate WiX source from config
    pub fn generate_wxs(config: &ProjectConfig) -> String {
        let mut wxs = String::new();
        wxs.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
        wxs.push_str("<Wix xmlns=\"http://wixtoolset.org/schemas/v4/wxs\">\n");
        wxs.push_str(&format!(
            "    <Package Name=\"{}\"\n",
            config.name
        ));
        wxs.push_str(&format!(
            "             Manufacturer=\"{}\"\n",
            config.manufacturer
        ));
        wxs.push_str(&format!(
            "             Version=\"{}\"\n",
            config.version
        ));

        if let Some(ref upgrade_code) = config.upgrade_code {
            wxs.push_str(&format!(
                "             UpgradeCode=\"{}\">\n",
                upgrade_code
            ));
        } else {
            wxs.push_str("             UpgradeCode=\"PUT-GUID-HERE\">\n");
        }

        wxs.push_str("\n        <MajorUpgrade DowngradeErrorMessage=\"A newer version is already installed.\" />\n");

        // UI
        wxs.push_str(&format!("\n        <UIRef Id=\"WixUI_{}\" />\n", config.ui_type));

        // Features
        wxs.push_str("\n        <Feature Id=\"ProductFeature\" Title=\"Main Feature\" Level=\"1\">\n");
        wxs.push_str("            <ComponentGroupRef Id=\"ProductComponents\" />\n");
        wxs.push_str("        </Feature>\n");

        // Directory structure
        wxs.push_str("\n        <StandardDirectory Id=\"ProgramFilesFolder\">\n");
        wxs.push_str(&format!(
            "            <Directory Id=\"INSTALLFOLDER\" Name=\"{}\" />\n",
            config.name
        ));
        wxs.push_str("        </StandardDirectory>\n");

        // Component group placeholder
        wxs.push_str("\n        <ComponentGroup Id=\"ProductComponents\" Directory=\"INSTALLFOLDER\">\n");
        wxs.push_str("            <!-- Add components here -->\n");
        wxs.push_str("        </ComponentGroup>\n");

        wxs.push_str("\n    </Package>\n");
        wxs.push_str("</Wix>\n");

        wxs
    }

    /// Generate project summary
    pub fn generate_summary(config: &ProjectConfig) -> String {
        let mut summary = String::new();
        summary.push_str("=== WiX Project Summary ===\n\n");
        summary.push_str(&format!("Product: {}\n", config.name));
        summary.push_str(&format!("Version: {}\n", config.version));
        summary.push_str(&format!("Manufacturer: {}\n", config.manufacturer));
        if let Some(ref desc) = config.description {
            summary.push_str(&format!("Description: {}\n", desc));
        }
        summary.push_str(&format!("Type: {:?}\n", config.project_type));
        summary.push_str(&format!("Source: {}\n", config.source_directory.display()));
        summary.push_str(&format!("Output: {}\n", config.output_directory.display()));
        summary.push_str(&format!("UI: {}\n", config.ui_type));
        summary.push_str(&format!("License: {}\n", if config.include_license { "Yes" } else { "No" }));
        summary
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wizard_creation() {
        let wizard = Wizard::new();
        assert!(!wizard.steps.is_empty());
    }

    #[test]
    fn test_wizard_step_count() {
        let wizard = Wizard::new();
        assert_eq!(wizard.step_count(), 5);
    }

    #[test]
    fn test_question_text() {
        let q = Question::text("name", "Enter name:");
        assert_eq!(q.id, "name");
        assert!(q.required);
    }

    #[test]
    fn test_question_boolean() {
        let q = Question::boolean("flag", "Enable?", true);
        assert_eq!(q.default, Some("true".to_string()));
    }

    #[test]
    fn test_question_choice() {
        let q = Question::choice("type", "Select:", vec!["A", "B", "C"]);
        if let QuestionType::Choice(opts) = q.question_type {
            assert_eq!(opts.len(), 3);
        } else {
            panic!("Expected Choice type");
        }
    }

    #[test]
    fn test_question_with_default() {
        let q = Question::text("name", "Name:").with_default("Default");
        assert_eq!(q.default, Some("Default".to_string()));
    }

    #[test]
    fn test_question_with_help() {
        let q = Question::text("name", "Name:").with_help("Enter your name");
        assert_eq!(q.help, Some("Enter your name".to_string()));
    }

    #[test]
    fn test_question_optional() {
        let q = Question::text("name", "Name:").optional();
        assert!(!q.required);
    }

    #[test]
    fn test_set_and_get_answer() {
        let mut wizard = Wizard::new();
        wizard.set_answer("name", AnswerValue::Text("MyApp".to_string()));
        assert_eq!(wizard.get_text("name"), Some("MyApp"));
    }

    #[test]
    fn test_answer_value_as_str() {
        let value = AnswerValue::Text("test".to_string());
        assert_eq!(value.as_str(), Some("test"));

        let bool_value = AnswerValue::Boolean(true);
        assert_eq!(bool_value.as_str(), None);
    }

    #[test]
    fn test_answer_value_as_bool() {
        let value = AnswerValue::Boolean(true);
        assert_eq!(value.as_bool(), Some(true));
    }

    #[test]
    fn test_project_type_from_str() {
        assert_eq!(ProjectType::from_str("application"), Some(ProjectType::Application));
        assert_eq!(ProjectType::from_str("service"), Some(ProjectType::Service));
        assert_eq!(ProjectType::from_str("invalid"), None);
    }

    #[test]
    fn test_project_type_as_str() {
        assert_eq!(ProjectType::Application.as_str(), "application");
        assert_eq!(ProjectType::Service.as_str(), "service");
    }

    #[test]
    fn test_build_config() {
        let mut wizard = Wizard::new();
        wizard.set_answer("name", AnswerValue::Text("TestApp".to_string()));
        wizard.set_answer("version", AnswerValue::Text("2.0.0".to_string()));
        wizard.set_answer("manufacturer", AnswerValue::Text("TestCorp".to_string()));

        let config = wizard.build_config().unwrap();
        assert_eq!(config.name, "TestApp");
        assert_eq!(config.version, "2.0.0");
        assert_eq!(config.manufacturer, "TestCorp");
    }

    #[test]
    fn test_build_config_defaults() {
        let mut wizard = Wizard::new();
        wizard.set_answer("name", AnswerValue::Text("App".to_string()));
        wizard.set_answer("manufacturer", AnswerValue::Text("Corp".to_string()));

        let config = wizard.build_config().unwrap();
        assert_eq!(config.version, "1.0.0");
        assert_eq!(config.project_type, ProjectType::Application);
    }

    #[test]
    fn test_build_config_missing_name() {
        let wizard = Wizard::new();
        let result = wizard.build_config();
        assert!(result.is_err());
    }

    #[test]
    fn test_generate_wxs() {
        let config = ProjectConfig {
            name: "TestApp".to_string(),
            version: "1.0.0".to_string(),
            manufacturer: "TestCorp".to_string(),
            description: None,
            project_type: ProjectType::Application,
            source_directory: PathBuf::from("."),
            output_directory: PathBuf::from("./out"),
            features: vec![],
            ui_type: "InstallDir".to_string(),
            include_license: false,
            upgrade_code: None,
            additional_options: HashMap::new(),
        };

        let wxs = ProjectGenerator::generate_wxs(&config);
        assert!(wxs.contains("TestApp"));
        assert!(wxs.contains("TestCorp"));
        assert!(wxs.contains("WixUI_InstallDir"));
    }

    #[test]
    fn test_generate_summary() {
        let config = ProjectConfig {
            name: "TestApp".to_string(),
            version: "1.0.0".to_string(),
            manufacturer: "TestCorp".to_string(),
            description: Some("Test description".to_string()),
            project_type: ProjectType::Application,
            source_directory: PathBuf::from("."),
            output_directory: PathBuf::from("./out"),
            features: vec![],
            ui_type: "InstallDir".to_string(),
            include_license: true,
            upgrade_code: None,
            additional_options: HashMap::new(),
        };

        let summary = ProjectGenerator::generate_summary(&config);
        assert!(summary.contains("TestApp"));
        assert!(summary.contains("TestCorp"));
        assert!(summary.contains("Test description"));
    }

    #[test]
    fn test_feature_config() {
        let feature = FeatureConfig {
            id: "Feature1".to_string(),
            title: "Main Feature".to_string(),
            description: Some("Description".to_string()),
            level: 1,
            directory: Some("INSTALLFOLDER".to_string()),
        };
        assert_eq!(feature.id, "Feature1");
        assert_eq!(feature.level, 1);
    }
}

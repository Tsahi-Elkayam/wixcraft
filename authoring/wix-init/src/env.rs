//! wix-env - Environment variable helper for WiX installers
//!
//! Generates WiX components for setting environment variables.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Environment variable scope
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EnvScope {
    /// Machine-wide (HKLM)
    Machine,
    /// Per-user (HKCU)
    User,
}

impl EnvScope {
    pub fn as_str(&self) -> &'static str {
        match self {
            EnvScope::Machine => "machine",
            EnvScope::User => "user",
        }
    }

    pub fn registry_root(&self) -> &'static str {
        match self {
            EnvScope::Machine => "HKLM",
            EnvScope::User => "HKCU",
        }
    }
}

/// Environment variable action
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EnvAction {
    /// Set the variable (overwrite existing)
    Set,
    /// Create only if it doesn't exist
    Create,
    /// Remove the variable
    Remove,
}

impl EnvAction {
    pub fn as_str(&self) -> &'static str {
        match self {
            EnvAction::Set => "set",
            EnvAction::Create => "create",
            EnvAction::Remove => "remove",
        }
    }
}

/// How to handle existing PATH values
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PathAction {
    /// Prepend to existing PATH
    Prepend,
    /// Append to existing PATH
    Append,
    /// Replace entire PATH (dangerous!)
    Replace,
}

/// Environment variable definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvVariable {
    pub name: String,
    pub value: String,
    pub scope: EnvScope,
    pub action: EnvAction,
    pub separator: Option<String>,
    pub part: Option<String>,
    pub permanent: bool,
}

impl EnvVariable {
    /// Create a new environment variable
    pub fn new(name: &str, value: &str) -> Self {
        Self {
            name: name.to_string(),
            value: value.to_string(),
            scope: EnvScope::Machine,
            action: EnvAction::Set,
            separator: None,
            part: None,
            permanent: false,
        }
    }

    /// Set scope to machine-wide
    pub fn machine(mut self) -> Self {
        self.scope = EnvScope::Machine;
        self
    }

    /// Set scope to per-user
    pub fn user(mut self) -> Self {
        self.scope = EnvScope::User;
        self
    }

    /// Set action to create only
    pub fn create_only(mut self) -> Self {
        self.action = EnvAction::Create;
        self
    }

    /// Set action to remove
    pub fn remove(mut self) -> Self {
        self.action = EnvAction::Remove;
        self
    }

    /// Make permanent (survives uninstall)
    pub fn permanent(mut self) -> Self {
        self.permanent = true;
        self
    }

    /// Append to existing value
    pub fn append_to_existing(mut self, separator: &str) -> Self {
        self.separator = Some(separator.to_string());
        self.part = Some("last".to_string());
        self
    }

    /// Prepend to existing value
    pub fn prepend_to_existing(mut self, separator: &str) -> Self {
        self.separator = Some(separator.to_string());
        self.part = Some("first".to_string());
        self
    }
}

/// PATH entry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathEntry {
    pub directory: String,
    pub scope: EnvScope,
    pub action: PathAction,
}

impl PathEntry {
    pub fn new(directory: &str) -> Self {
        Self {
            directory: directory.to_string(),
            scope: EnvScope::Machine,
            action: PathAction::Append,
        }
    }

    pub fn machine(mut self) -> Self {
        self.scope = EnvScope::Machine;
        self
    }

    pub fn user(mut self) -> Self {
        self.scope = EnvScope::User;
        self
    }

    pub fn prepend(mut self) -> Self {
        self.action = PathAction::Prepend;
        self
    }

    pub fn append(mut self) -> Self {
        self.action = PathAction::Append;
        self
    }
}

/// Environment configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EnvConfig {
    pub variables: Vec<EnvVariable>,
    pub path_entries: Vec<PathEntry>,
}

impl EnvConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_variable(&mut self, var: EnvVariable) -> &mut Self {
        self.variables.push(var);
        self
    }

    pub fn add_path(&mut self, entry: PathEntry) -> &mut Self {
        self.path_entries.push(entry);
        self
    }

    pub fn var(mut self, name: &str, value: &str) -> Self {
        self.variables.push(EnvVariable::new(name, value));
        self
    }

    pub fn path(mut self, directory: &str) -> Self {
        self.path_entries.push(PathEntry::new(directory));
        self
    }
}

/// WiX environment component generator
pub struct EnvGenerator;

impl EnvGenerator {
    /// Generate WiX Environment element
    pub fn generate_element(var: &EnvVariable) -> String {
        let mut attrs = vec![
            format!("Id=\"Env_{}\"", Self::sanitize_id(&var.name)),
            format!("Name=\"{}\"", var.name),
            format!("Value=\"{}\"", var.value),
            format!("System=\"{}\"", if var.scope == EnvScope::Machine { "yes" } else { "no" }),
            format!("Action=\"{}\"", var.action.as_str()),
        ];

        if var.permanent {
            attrs.push("Permanent=\"yes\"".to_string());
        }

        if let Some(ref sep) = var.separator {
            attrs.push(format!("Separator=\"{}\"", sep));
        }

        if let Some(ref part) = var.part {
            attrs.push(format!("Part=\"{}\"", part));
        }

        format!("<Environment {} />", attrs.join(" "))
    }

    /// Generate WiX component for environment variable
    pub fn generate_component(var: &EnvVariable, component_id: &str) -> String {
        format!(
            r#"<Component Id="{}" Guid="*">
    {}
</Component>"#,
            component_id,
            Self::generate_element(var)
        )
    }

    /// Generate PATH modification element
    pub fn generate_path_element(entry: &PathEntry) -> String {
        let part = match entry.action {
            PathAction::Prepend => "first",
            PathAction::Append => "last",
            PathAction::Replace => "all",
        };

        format!(
            r#"<Environment Id="Env_PATH_{}" Name="PATH" Value="{}" System="{}" Action="set" Part="{}" Separator=";" />"#,
            Self::sanitize_id(&entry.directory),
            entry.directory,
            if entry.scope == EnvScope::Machine { "yes" } else { "no" },
            part
        )
    }

    /// Generate full component group for environment config
    pub fn generate_component_group(config: &EnvConfig, group_id: &str) -> String {
        let mut components = Vec::new();

        // Generate variable components
        for (i, var) in config.variables.iter().enumerate() {
            let comp_id = format!("EnvVar_{}", i);
            components.push(Self::generate_component(var, &comp_id));
        }

        // Generate PATH components
        for (i, entry) in config.path_entries.iter().enumerate() {
            let comp_id = format!("EnvPath_{}", i);
            let element = Self::generate_path_element(entry);
            components.push(format!(
                r#"<Component Id="{}" Guid="*">
    {}
</Component>"#,
                comp_id, element
            ));
        }

        format!(
            r#"<ComponentGroup Id="{}">
    {}
</ComponentGroup>"#,
            group_id,
            components.join("\n    ")
        )
    }

    fn sanitize_id(name: &str) -> String {
        name.chars()
            .map(|c| if c.is_alphanumeric() || c == '_' { c } else { '_' })
            .collect()
    }
}

/// Common environment patterns
pub struct EnvPatterns;

impl EnvPatterns {
    /// Add directory to PATH
    pub fn add_to_path(directory: &str) -> EnvVariable {
        EnvVariable::new("PATH", directory)
            .append_to_existing(";")
    }

    /// Set JAVA_HOME
    pub fn java_home(directory: &str) -> EnvVariable {
        EnvVariable::new("JAVA_HOME", directory)
    }

    /// Set application home directory
    pub fn app_home(app_name: &str, directory: &str) -> EnvVariable {
        EnvVariable::new(&format!("{}_HOME", app_name.to_uppercase()), directory)
    }

    /// Set application data directory
    pub fn app_data(app_name: &str, directory: &str) -> EnvVariable {
        EnvVariable::new(&format!("{}_DATA", app_name.to_uppercase()), directory)
            .user()
    }

    /// Create config directory variable
    pub fn config_dir(app_name: &str, directory: &str) -> EnvVariable {
        EnvVariable::new(&format!("{}_CONFIG", app_name.to_uppercase()), directory)
    }

    /// Create standard application environment
    pub fn standard_app_env(app_name: &str, install_dir: &str) -> EnvConfig {
        let mut config = EnvConfig::new();
        config.add_variable(Self::app_home(app_name, install_dir));
        config.add_path(PathEntry::new(&format!("{}\\bin", install_dir)));
        config
    }
}

/// Environment variable parser
pub struct EnvParser;

impl EnvParser {
    /// Parse environment variables from a .env file format
    pub fn parse_env_file(content: &str) -> HashMap<String, String> {
        let mut vars = HashMap::new();

        for line in content.lines() {
            let line = line.trim();

            // Skip empty lines and comments
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // Parse KEY=VALUE
            if let Some(eq_pos) = line.find('=') {
                let key = line[..eq_pos].trim();
                let value = line[eq_pos + 1..].trim();

                // Remove quotes if present
                let value = value
                    .strip_prefix('"').and_then(|v| v.strip_suffix('"'))
                    .or_else(|| value.strip_prefix('\'').and_then(|v| v.strip_suffix('\'')))
                    .unwrap_or(value);

                vars.insert(key.to_string(), value.to_string());
            }
        }

        vars
    }

    /// Convert parsed vars to EnvConfig
    pub fn to_env_config(vars: &HashMap<String, String>, scope: EnvScope) -> EnvConfig {
        let mut config = EnvConfig::new();

        for (name, value) in vars {
            let mut var = EnvVariable::new(name, value);
            var.scope = scope;
            config.add_variable(var);
        }

        config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_env_scope_as_str() {
        assert_eq!(EnvScope::Machine.as_str(), "machine");
        assert_eq!(EnvScope::User.as_str(), "user");
    }

    #[test]
    fn test_env_scope_registry_root() {
        assert_eq!(EnvScope::Machine.registry_root(), "HKLM");
        assert_eq!(EnvScope::User.registry_root(), "HKCU");
    }

    #[test]
    fn test_env_action_as_str() {
        assert_eq!(EnvAction::Set.as_str(), "set");
        assert_eq!(EnvAction::Create.as_str(), "create");
        assert_eq!(EnvAction::Remove.as_str(), "remove");
    }

    #[test]
    fn test_env_variable_new() {
        let var = EnvVariable::new("MY_VAR", "my_value");
        assert_eq!(var.name, "MY_VAR");
        assert_eq!(var.value, "my_value");
        assert_eq!(var.scope, EnvScope::Machine);
    }

    #[test]
    fn test_env_variable_user() {
        let var = EnvVariable::new("MY_VAR", "value").user();
        assert_eq!(var.scope, EnvScope::User);
    }

    #[test]
    fn test_env_variable_permanent() {
        let var = EnvVariable::new("MY_VAR", "value").permanent();
        assert!(var.permanent);
    }

    #[test]
    fn test_env_variable_append() {
        let var = EnvVariable::new("PATH", "C:\\bin").append_to_existing(";");
        assert_eq!(var.separator, Some(";".to_string()));
        assert_eq!(var.part, Some("last".to_string()));
    }

    #[test]
    fn test_env_variable_prepend() {
        let var = EnvVariable::new("PATH", "C:\\bin").prepend_to_existing(";");
        assert_eq!(var.part, Some("first".to_string()));
    }

    #[test]
    fn test_path_entry_new() {
        let entry = PathEntry::new("C:\\Program Files\\App\\bin");
        assert_eq!(entry.directory, "C:\\Program Files\\App\\bin");
        assert_eq!(entry.action, PathAction::Append);
    }

    #[test]
    fn test_path_entry_prepend() {
        let entry = PathEntry::new("C:\\bin").prepend();
        assert_eq!(entry.action, PathAction::Prepend);
    }

    #[test]
    fn test_env_config_builder() {
        let config = EnvConfig::new()
            .var("MY_VAR", "value")
            .path("C:\\bin");
        assert_eq!(config.variables.len(), 1);
        assert_eq!(config.path_entries.len(), 1);
    }

    #[test]
    fn test_generate_element() {
        let var = EnvVariable::new("MY_VAR", "my_value");
        let xml = EnvGenerator::generate_element(&var);
        assert!(xml.contains("MY_VAR"));
        assert!(xml.contains("my_value"));
        assert!(xml.contains("System=\"yes\""));
    }

    #[test]
    fn test_generate_element_user() {
        let var = EnvVariable::new("MY_VAR", "value").user();
        let xml = EnvGenerator::generate_element(&var);
        assert!(xml.contains("System=\"no\""));
    }

    #[test]
    fn test_generate_component() {
        let var = EnvVariable::new("MY_VAR", "value");
        let xml = EnvGenerator::generate_component(&var, "EnvComp1");
        assert!(xml.contains("<Component Id=\"EnvComp1\""));
        assert!(xml.contains("<Environment"));
    }

    #[test]
    fn test_generate_path_element() {
        let entry = PathEntry::new("C:\\bin");
        let xml = EnvGenerator::generate_path_element(&entry);
        assert!(xml.contains("PATH"));
        assert!(xml.contains("C:\\bin"));
        assert!(xml.contains("Part=\"last\""));
    }

    #[test]
    fn test_generate_path_element_prepend() {
        let entry = PathEntry::new("C:\\bin").prepend();
        let xml = EnvGenerator::generate_path_element(&entry);
        assert!(xml.contains("Part=\"first\""));
    }

    #[test]
    fn test_generate_component_group() {
        let mut config = EnvConfig::new();
        config.add_variable(EnvVariable::new("VAR1", "val1"));
        config.add_path(PathEntry::new("C:\\bin"));

        let xml = EnvGenerator::generate_component_group(&config, "EnvGroup");
        assert!(xml.contains("<ComponentGroup Id=\"EnvGroup\""));
        assert!(xml.contains("VAR1"));
        assert!(xml.contains("PATH"));
    }

    #[test]
    fn test_env_patterns_add_to_path() {
        let var = EnvPatterns::add_to_path("C:\\bin");
        assert_eq!(var.name, "PATH");
        assert_eq!(var.separator, Some(";".to_string()));
    }

    #[test]
    fn test_env_patterns_java_home() {
        let var = EnvPatterns::java_home("C:\\Java\\jdk");
        assert_eq!(var.name, "JAVA_HOME");
    }

    #[test]
    fn test_env_patterns_app_home() {
        let var = EnvPatterns::app_home("myapp", "C:\\MyApp");
        assert_eq!(var.name, "MYAPP_HOME");
    }

    #[test]
    fn test_env_patterns_standard_app() {
        let config = EnvPatterns::standard_app_env("myapp", "C:\\MyApp");
        assert_eq!(config.variables.len(), 1);
        assert_eq!(config.path_entries.len(), 1);
    }

    #[test]
    fn test_parse_env_file() {
        let content = r#"
# Comment
KEY1=value1
KEY2="quoted value"
KEY3='single quoted'
EMPTY=
"#;
        let vars = EnvParser::parse_env_file(content);
        assert_eq!(vars.get("KEY1"), Some(&"value1".to_string()));
        assert_eq!(vars.get("KEY2"), Some(&"quoted value".to_string()));
        assert_eq!(vars.get("KEY3"), Some(&"single quoted".to_string()));
    }

    #[test]
    fn test_parse_env_file_skip_comments() {
        let content = "# comment\nKEY=value";
        let vars = EnvParser::parse_env_file(content);
        assert_eq!(vars.len(), 1);
        assert!(vars.contains_key("KEY"));
    }

    #[test]
    fn test_to_env_config() {
        let mut vars = HashMap::new();
        vars.insert("KEY1".to_string(), "val1".to_string());
        vars.insert("KEY2".to_string(), "val2".to_string());

        let config = EnvParser::to_env_config(&vars, EnvScope::User);
        assert_eq!(config.variables.len(), 2);
        assert!(config.variables.iter().all(|v| v.scope == EnvScope::User));
    }

    #[test]
    fn test_sanitize_id() {
        assert_eq!(EnvGenerator::sanitize_id("MY_VAR"), "MY_VAR");
        assert_eq!(EnvGenerator::sanitize_id("my-var"), "my_var");
        assert_eq!(EnvGenerator::sanitize_id("path.to.var"), "path_to_var");
    }
}

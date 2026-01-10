//! wix-env - Environment variable and PATH configuration helper
//!
//! Generates WiX XML for:
//! - Setting environment variables (user/system)
//! - Modifying PATH variable
//! - Removing variables on uninstall

use serde::{Deserialize, Serialize};

/// Environment variable scope
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EnvScope {
    /// User environment (HKCU)
    User,
    /// System environment (HKLM) - requires admin
    System,
}

impl std::fmt::Display for EnvScope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EnvScope::User => write!(f, "user"),
            EnvScope::System => write!(f, "machine"),
        }
    }
}

/// Environment variable action
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EnvAction {
    /// Set variable (overwrite if exists)
    Set,
    /// Create variable (only if not exists)
    Create,
    /// Append to variable (with separator)
    Append,
    /// Prepend to variable (with separator)
    Prepend,
}

impl std::fmt::Display for EnvAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EnvAction::Set => write!(f, "set"),
            EnvAction::Create => write!(f, "create"),
            EnvAction::Append => write!(f, "set"),
            EnvAction::Prepend => write!(f, "set"),
        }
    }
}

/// Environment variable definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvVariable {
    pub name: String,
    pub value: String,
    pub scope: EnvScope,
    pub action: EnvAction,
    pub separator: Option<String>,
    pub permanent: bool,
}

impl EnvVariable {
    pub fn new(name: &str, value: &str) -> Self {
        Self {
            name: name.to_string(),
            value: value.to_string(),
            scope: EnvScope::User,
            action: EnvAction::Set,
            separator: None,
            permanent: false,
        }
    }

    pub fn system(mut self) -> Self {
        self.scope = EnvScope::System;
        self
    }

    pub fn user(mut self) -> Self {
        self.scope = EnvScope::User;
        self
    }

    pub fn append(mut self, separator: &str) -> Self {
        self.action = EnvAction::Append;
        self.separator = Some(separator.to_string());
        self
    }

    pub fn prepend(mut self, separator: &str) -> Self {
        self.action = EnvAction::Prepend;
        self.separator = Some(separator.to_string());
        self
    }

    pub fn permanent(mut self) -> Self {
        self.permanent = true;
        self
    }
}

/// PATH modification entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathEntry {
    pub path: String,
    pub scope: EnvScope,
    pub prepend: bool,
    pub permanent: bool,
}

impl PathEntry {
    pub fn new(path: &str) -> Self {
        Self {
            path: path.to_string(),
            scope: EnvScope::User,
            prepend: false,
            permanent: false,
        }
    }

    pub fn system(mut self) -> Self {
        self.scope = EnvScope::System;
        self
    }

    pub fn prepend(mut self) -> Self {
        self.prepend = true;
        self
    }

    pub fn permanent(mut self) -> Self {
        self.permanent = true;
        self
    }
}

/// WiX XML generator for environment configuration
pub struct EnvGenerator {
    variables: Vec<EnvVariable>,
    path_entries: Vec<PathEntry>,
    component_id: String,
}

impl Default for EnvGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl EnvGenerator {
    pub fn new() -> Self {
        Self {
            variables: Vec::new(),
            path_entries: Vec::new(),
            component_id: "EnvVars".to_string(),
        }
    }

    pub fn with_component_id(mut self, id: &str) -> Self {
        self.component_id = id.to_string();
        self
    }

    pub fn add_variable(&mut self, var: EnvVariable) {
        self.variables.push(var);
    }

    pub fn add_path(&mut self, entry: PathEntry) {
        self.path_entries.push(entry);
    }

    /// Generate WiX v4 XML for environment configuration
    pub fn generate_wix4(&self) -> String {
        let mut xml = String::new();
        xml.push_str(&format!(
            r#"<Component Id="{}" Guid="*">
"#,
            self.component_id
        ));

        // Generate environment variables
        for (i, var) in self.variables.iter().enumerate() {
            let id = format!("Env_{}", i);
            let value = self.format_value(var);
            let permanent = if var.permanent { "yes" } else { "no" };

            xml.push_str(&format!(
                r#"    <Environment Id="{}" Name="{}" Value="{}" Action="{}" System="{}" Permanent="{}" />
"#,
                id,
                var.name,
                escape_xml(&value),
                var.action,
                if var.scope == EnvScope::System { "yes" } else { "no" },
                permanent
            ));
        }

        // Generate PATH entries
        for (i, entry) in self.path_entries.iter().enumerate() {
            let id = format!("Path_{}", i);
            let value = if entry.prepend {
                format!("[~];{}", entry.path)
            } else {
                format!("{};[~]", entry.path)
            };
            let permanent = if entry.permanent { "yes" } else { "no" };

            xml.push_str(&format!(
                r#"    <Environment Id="{}" Name="PATH" Value="{}" Action="set" System="{}" Permanent="{}" Part="{}" />
"#,
                id,
                escape_xml(&value),
                if entry.scope == EnvScope::System { "yes" } else { "no" },
                permanent,
                if entry.prepend { "first" } else { "last" }
            ));
        }

        xml.push_str("</Component>\n");
        xml
    }

    /// Generate WiX v3 XML for environment configuration
    pub fn generate_wix3(&self) -> String {
        let mut xml = String::new();
        xml.push_str(&format!(
            r#"<Component Id="{}" Guid="*">
"#,
            self.component_id
        ));

        // Generate environment variables
        for (i, var) in self.variables.iter().enumerate() {
            let id = format!("Env_{}", i);
            let value = self.format_value(var);
            let permanent = if var.permanent { "yes" } else { "no" };

            xml.push_str(&format!(
                r#"    <Environment Id="{}" Name="{}" Value="{}" Action="{}" System="{}" Permanent="{}" />
"#,
                id,
                var.name,
                escape_xml(&value),
                var.action,
                if var.scope == EnvScope::System { "yes" } else { "no" },
                permanent
            ));
        }

        // Generate PATH entries
        for (i, entry) in self.path_entries.iter().enumerate() {
            let id = format!("Path_{}", i);
            let value = if entry.prepend {
                format!("[~];{}", entry.path)
            } else {
                format!("{};[~]", entry.path)
            };
            let permanent = if entry.permanent { "yes" } else { "no" };

            xml.push_str(&format!(
                r#"    <Environment Id="{}" Name="PATH" Value="{}" Action="set" System="{}" Permanent="{}" Part="{}" />
"#,
                id,
                escape_xml(&value),
                if entry.scope == EnvScope::System { "yes" } else { "no" },
                permanent,
                if entry.prepend { "first" } else { "last" }
            ));
        }

        xml.push_str("</Component>\n");
        xml
    }

    fn format_value(&self, var: &EnvVariable) -> String {
        match var.action {
            EnvAction::Append => {
                let sep = var.separator.as_deref().unwrap_or(";");
                format!("[~]{}{}", sep, var.value)
            }
            EnvAction::Prepend => {
                let sep = var.separator.as_deref().unwrap_or(";");
                format!("{}{}[~]", var.value, sep)
            }
            _ => var.value.clone(),
        }
    }
}

/// Common environment variable templates
pub struct EnvTemplates;

impl EnvTemplates {
    /// Add installation directory to PATH
    pub fn add_install_dir_to_path(scope: EnvScope) -> PathEntry {
        let mut entry = PathEntry::new("[INSTALLDIR]");
        if scope == EnvScope::System {
            entry = entry.system();
        }
        entry
    }

    /// Add bin subdirectory to PATH
    pub fn add_bin_to_path(scope: EnvScope) -> PathEntry {
        let mut entry = PathEntry::new("[INSTALLDIR]bin");
        if scope == EnvScope::System {
            entry = entry.system();
        }
        entry
    }

    /// Set HOME directory variable
    pub fn set_home_var(name: &str, scope: EnvScope) -> EnvVariable {
        let mut var = EnvVariable::new(name, "[INSTALLDIR]");
        if scope == EnvScope::System {
            var = var.system();
        }
        var
    }

    /// Set application data directory
    pub fn set_app_data_var(name: &str, app_name: &str) -> EnvVariable {
        EnvVariable::new(name, &format!("[LocalAppDataFolder]{}", app_name))
    }

    /// Common pattern: JAVA_HOME style
    pub fn java_home_style(var_name: &str, scope: EnvScope) -> (EnvVariable, PathEntry) {
        let home_var = if scope == EnvScope::System {
            EnvVariable::new(var_name, "[INSTALLDIR]").system()
        } else {
            EnvVariable::new(var_name, "[INSTALLDIR]")
        };

        let path_var = format!("[{}]bin", var_name);
        let path_entry = if scope == EnvScope::System {
            PathEntry::new(&path_var).system()
        } else {
            PathEntry::new(&path_var)
        };

        (home_var, path_entry)
    }
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

/// Parse environment variable definition from string
/// Format: NAME=value[@scope][:action]
pub fn parse_env_definition(s: &str) -> anyhow::Result<EnvVariable> {
    let parts: Vec<&str> = s.splitn(2, '=').collect();
    if parts.len() != 2 {
        anyhow::bail!("Invalid format. Use: NAME=value[@scope][:action]");
    }

    let name = parts[0].trim();
    let mut value_part = parts[1].to_string();
    let mut scope = EnvScope::User;
    let mut action = EnvAction::Set;

    // Parse action suffix
    if value_part.ends_with(":append") {
        action = EnvAction::Append;
        value_part = value_part.trim_end_matches(":append").to_string();
    } else if value_part.ends_with(":prepend") {
        action = EnvAction::Prepend;
        value_part = value_part.trim_end_matches(":prepend").to_string();
    } else if value_part.ends_with(":create") {
        action = EnvAction::Create;
        value_part = value_part.trim_end_matches(":create").to_string();
    }

    // Parse scope suffix
    if value_part.ends_with("@system") {
        scope = EnvScope::System;
        value_part = value_part.trim_end_matches("@system").to_string();
    } else if value_part.ends_with("@user") {
        scope = EnvScope::User;
        value_part = value_part.trim_end_matches("@user").to_string();
    }

    let mut var = EnvVariable::new(name, &value_part);
    var.scope = scope;
    var.action = action;
    if action == EnvAction::Append || action == EnvAction::Prepend {
        var.separator = Some(";".to_string());
    }

    Ok(var)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_env_variable_builder() {
        let var = EnvVariable::new("MY_VAR", "my_value")
            .system()
            .permanent();

        assert_eq!(var.name, "MY_VAR");
        assert_eq!(var.value, "my_value");
        assert_eq!(var.scope, EnvScope::System);
        assert!(var.permanent);
    }

    #[test]
    fn test_path_entry_builder() {
        let entry = PathEntry::new("[INSTALLDIR]bin")
            .system()
            .prepend()
            .permanent();

        assert_eq!(entry.path, "[INSTALLDIR]bin");
        assert_eq!(entry.scope, EnvScope::System);
        assert!(entry.prepend);
        assert!(entry.permanent);
    }

    #[test]
    fn test_generate_wix4() {
        let mut gen = EnvGenerator::new();
        gen.add_variable(EnvVariable::new("MY_HOME", "[INSTALLDIR]"));
        gen.add_path(PathEntry::new("[INSTALLDIR]bin"));

        let xml = gen.generate_wix4();
        assert!(xml.contains("Environment"));
        assert!(xml.contains("MY_HOME"));
        assert!(xml.contains("PATH"));
    }

    #[test]
    fn test_append_format() {
        let var = EnvVariable::new("PATH", "[INSTALLDIR]")
            .append(";");

        let gen = EnvGenerator::new();
        let value = gen.format_value(&var);
        assert_eq!(value, "[~];[INSTALLDIR]");
    }

    #[test]
    fn test_prepend_format() {
        let var = EnvVariable::new("PATH", "[INSTALLDIR]")
            .prepend(";");

        let gen = EnvGenerator::new();
        let value = gen.format_value(&var);
        assert_eq!(value, "[INSTALLDIR];[~]");
    }

    #[test]
    fn test_parse_env_definition() {
        let var = parse_env_definition("MY_VAR=value@system").unwrap();
        assert_eq!(var.name, "MY_VAR");
        assert_eq!(var.value, "value");
        assert_eq!(var.scope, EnvScope::System);
    }

    #[test]
    fn test_parse_env_with_action() {
        let var = parse_env_definition("PATH=[INSTALLDIR]:append").unwrap();
        assert_eq!(var.name, "PATH");
        assert_eq!(var.action, EnvAction::Append);
    }

    #[test]
    fn test_java_home_template() {
        let (home, path) = EnvTemplates::java_home_style("JAVA_HOME", EnvScope::System);
        assert_eq!(home.name, "JAVA_HOME");
        assert_eq!(home.scope, EnvScope::System);
        assert!(path.path.contains("JAVA_HOME"));
    }

    #[test]
    fn test_escape_xml() {
        assert_eq!(escape_xml("<test>"), "&lt;test&gt;");
        assert_eq!(escape_xml("a & b"), "a &amp; b");
    }

    #[test]
    fn test_scope_display() {
        assert_eq!(format!("{}", EnvScope::User), "user");
        assert_eq!(format!("{}", EnvScope::System), "machine");
    }

    #[test]
    fn test_action_display() {
        assert_eq!(format!("{}", EnvAction::Set), "set");
        assert_eq!(format!("{}", EnvAction::Create), "create");
    }
}

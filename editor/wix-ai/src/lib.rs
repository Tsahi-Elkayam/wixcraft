//! wix-ai - AI-assisted WiX code generation
//!
//! Provides pattern-based code generation from natural language prompts:
//! - Intent recognition for common WiX tasks
//! - Template expansion with context
//! - Code completion suggestions
//! - Best practice recommendations

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// User intent categories
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Intent {
    /// Create a new installer project
    CreateProject,
    /// Add a file to install
    AddFile,
    /// Create a shortcut
    CreateShortcut,
    /// Add registry entry
    AddRegistry,
    /// Create environment variable
    AddEnvironment,
    /// Install a service
    InstallService,
    /// Add custom action
    AddCustomAction,
    /// Create feature
    CreateFeature,
    /// Add prerequisite check
    AddPrerequisite,
    /// Upgrade handling
    AddUpgrade,
    /// Unknown intent
    Unknown,
}

/// Code generation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationResult {
    pub intent: Intent,
    pub code: String,
    pub explanation: String,
    pub suggestions: Vec<String>,
    pub references: Vec<String>,
}

/// Code template
#[derive(Debug, Clone)]
pub struct Template {
    pub name: &'static str,
    pub description: &'static str,
    pub keywords: &'static [&'static str],
    pub code: &'static str,
    pub variables: &'static [&'static str],
}

/// WiX AI code generator
pub struct WixAi {
    templates: Vec<Template>,
}

impl Default for WixAi {
    fn default() -> Self {
        Self::new()
    }
}

impl WixAi {
    pub fn new() -> Self {
        Self {
            templates: Self::load_templates(),
        }
    }

    fn load_templates() -> Vec<Template> {
        vec![
            Template {
                name: "basic_installer",
                description: "Create a basic MSI installer",
                keywords: &["create", "new", "installer", "msi", "basic", "simple", "project"],
                code: r#"<Wix xmlns="http://wixtoolset.org/schemas/v4/wxs">
  <Package Name="{{NAME}}" Version="{{VERSION}}" Manufacturer="{{MANUFACTURER}}"
           UpgradeCode="{{UPGRADE_CODE}}">

    <MajorUpgrade DowngradeErrorMessage="A newer version is already installed." />
    <MediaTemplate EmbedCab="yes" />

    <StandardDirectory Id="ProgramFiles64Folder">
      <Directory Id="INSTALLDIR" Name="{{NAME}}">
        <Component Id="MainComponent" Guid="*">
          <File Id="MainExe" Source="{{SOURCE_PATH}}" KeyPath="yes" />
        </Component>
      </Directory>
    </StandardDirectory>

    <Feature Id="MainFeature" Title="Main Application" Level="1">
      <ComponentRef Id="MainComponent" />
    </Feature>

  </Package>
</Wix>"#,
                variables: &["NAME", "VERSION", "MANUFACTURER", "UPGRADE_CODE", "SOURCE_PATH"],
            },
            Template {
                name: "file_component",
                description: "Add a file to the installer",
                keywords: &["file", "add", "install", "copy", "include"],
                code: r#"<Component Id="{{COMPONENT_ID}}" Guid="*">
  <File Id="{{FILE_ID}}" Source="{{SOURCE}}" KeyPath="yes" />
</Component>"#,
                variables: &["COMPONENT_ID", "FILE_ID", "SOURCE"],
            },
            Template {
                name: "shortcut",
                description: "Create a Start Menu shortcut",
                keywords: &["shortcut", "start menu", "link", "desktop", "icon"],
                code: r#"<StandardDirectory Id="ProgramMenuFolder">
  <Directory Id="ApplicationProgramsFolder" Name="{{APP_NAME}}">
    <Component Id="ShortcutComponent" Guid="*">
      <Shortcut Id="{{SHORTCUT_ID}}"
                Name="{{SHORTCUT_NAME}}"
                Target="[INSTALLDIR]{{TARGET_FILE}}"
                WorkingDirectory="INSTALLDIR"
                Icon="{{ICON_ID}}" />
      <RemoveFolder Id="CleanUpShortcut" Directory="ApplicationProgramsFolder" On="uninstall" />
      <RegistryValue Root="HKCU" Key="Software\{{MANUFACTURER}}\{{APP_NAME}}"
                     Name="installed" Type="integer" Value="1" KeyPath="yes" />
    </Component>
  </Directory>
</StandardDirectory>"#,
                variables: &["APP_NAME", "SHORTCUT_ID", "SHORTCUT_NAME", "TARGET_FILE", "ICON_ID", "MANUFACTURER"],
            },
            Template {
                name: "registry_entry",
                description: "Add a registry entry",
                keywords: &["registry", "reg", "hkey", "hklm", "hkcu", "key", "value"],
                code: r#"<Component Id="{{COMPONENT_ID}}" Guid="*">
  <RegistryKey Root="{{ROOT}}" Key="{{KEY}}">
    <RegistryValue Name="{{VALUE_NAME}}" Type="{{VALUE_TYPE}}" Value="{{VALUE}}" KeyPath="yes" />
  </RegistryKey>
</Component>"#,
                variables: &["COMPONENT_ID", "ROOT", "KEY", "VALUE_NAME", "VALUE_TYPE", "VALUE"],
            },
            Template {
                name: "environment_variable",
                description: "Set an environment variable",
                keywords: &["environment", "env", "variable", "path", "java_home", "set"],
                code: r#"<Component Id="{{COMPONENT_ID}}" Guid="*">
  <Environment Id="{{ENV_ID}}"
               Name="{{ENV_NAME}}"
               Value="{{ENV_VALUE}}"
               Permanent="no"
               Part="all"
               Action="set"
               System="{{SYSTEM}}" />
</Component>"#,
                variables: &["COMPONENT_ID", "ENV_ID", "ENV_NAME", "ENV_VALUE", "SYSTEM"],
            },
            Template {
                name: "service_install",
                description: "Install a Windows service",
                keywords: &["service", "windows service", "daemon", "background", "install service"],
                code: r#"<Component Id="{{COMPONENT_ID}}" Guid="*">
  <File Id="{{SERVICE_EXE_ID}}" Source="{{SERVICE_EXE_PATH}}" KeyPath="yes" />
  <ServiceInstall Id="{{SERVICE_ID}}"
                  Name="{{SERVICE_NAME}}"
                  DisplayName="{{DISPLAY_NAME}}"
                  Description="{{DESCRIPTION}}"
                  Start="auto"
                  Type="ownProcess"
                  ErrorControl="normal"
                  Account="LocalSystem" />
  <ServiceControl Id="{{SERVICE_ID}}_Control"
                  Name="{{SERVICE_NAME}}"
                  Start="install"
                  Stop="both"
                  Remove="uninstall"
                  Wait="yes" />
</Component>"#,
                variables: &["COMPONENT_ID", "SERVICE_EXE_ID", "SERVICE_EXE_PATH", "SERVICE_ID", "SERVICE_NAME", "DISPLAY_NAME", "DESCRIPTION"],
            },
            Template {
                name: "custom_action",
                description: "Add a custom action",
                keywords: &["custom action", "ca", "script", "execute", "run", "command"],
                code: r#"<!-- Immediate custom action to set property -->
<SetProperty Id="{{CA_ID}}" Value="[INSTALLDIR]" Sequence="execute" Before="{{BEFORE_ACTION}}" />

<!-- OR: Deferred custom action -->
<CustomAction Id="{{CA_ID}}"
              BinaryRef="{{BINARY_REF}}"
              DllEntry="{{DLL_ENTRY}}"
              Execute="deferred"
              Impersonate="no" />

<InstallExecuteSequence>
  <Custom Action="{{CA_ID}}" Before="{{BEFORE_ACTION}}">NOT Installed</Custom>
</InstallExecuteSequence>"#,
                variables: &["CA_ID", "BINARY_REF", "DLL_ENTRY", "BEFORE_ACTION"],
            },
            Template {
                name: "feature",
                description: "Create a feature for optional components",
                keywords: &["feature", "optional", "component", "selectable", "tree"],
                code: r#"<Feature Id="{{FEATURE_ID}}"
         Title="{{FEATURE_TITLE}}"
         Description="{{FEATURE_DESCRIPTION}}"
         Level="{{LEVEL}}"
         AllowAbsent="yes"
         AllowAdvertise="no">
  <ComponentRef Id="{{COMPONENT_REF}}" />
  <!-- Add more ComponentRef elements as needed -->
</Feature>"#,
                variables: &["FEATURE_ID", "FEATURE_TITLE", "FEATURE_DESCRIPTION", "LEVEL", "COMPONENT_REF"],
            },
            Template {
                name: "prerequisite",
                description: "Add a prerequisite check",
                keywords: &["prerequisite", "require", "check", "dotnet", ".net", "runtime", "condition"],
                code: r#"<!-- .NET Runtime Check Example -->
<Property Id="NETFRAMEWORK48">
  <RegistrySearch Id="NetFramework48"
                  Root="HKLM"
                  Key="SOFTWARE\Microsoft\NET Framework Setup\NDP\v4\Full"
                  Name="Release"
                  Type="raw" />
</Property>

<Launch Condition="Installed OR NETFRAMEWORK48 >= 528040"
        Message="This application requires .NET Framework 4.8. Please install it first." />"#,
                variables: &[],
            },
            Template {
                name: "major_upgrade",
                description: "Handle upgrades from previous versions",
                keywords: &["upgrade", "update", "major", "version", "previous", "uninstall old"],
                code: r#"<!-- Allow upgrades, prevent downgrades -->
<MajorUpgrade
  Schedule="afterInstallInitialize"
  DowngradeErrorMessage="A newer version of [ProductName] is already installed."
  AllowSameVersionUpgrades="yes" />

<!-- OR: Manual upgrade handling -->
<Upgrade Id="{{UPGRADE_CODE}}">
  <UpgradeVersion Minimum="0.0.0" Maximum="{{CURRENT_VERSION}}"
                  IncludeMinimum="yes" IncludeMaximum="no"
                  Property="PREVIOUSVERSIONSINSTALLED" />
</Upgrade>

<InstallExecuteSequence>
  <RemoveExistingProducts After="InstallInitialize" />
</InstallExecuteSequence>"#,
                variables: &["UPGRADE_CODE", "CURRENT_VERSION"],
            },
        ]
    }

    /// Detect intent from natural language prompt
    pub fn detect_intent(&self, prompt: &str) -> Intent {
        let prompt_lower = prompt.to_lowercase();

        if prompt_lower.contains("create") && (prompt_lower.contains("project") || prompt_lower.contains("installer") || prompt_lower.contains("new")) {
            Intent::CreateProject
        } else if prompt_lower.contains("shortcut") || prompt_lower.contains("start menu") || prompt_lower.contains("desktop icon") {
            Intent::CreateShortcut
        } else if prompt_lower.contains("registry") || prompt_lower.contains("reg key") || prompt_lower.contains("hklm") || prompt_lower.contains("hkcu") {
            Intent::AddRegistry
        } else if prompt_lower.contains("environment") || prompt_lower.contains("env var") || prompt_lower.contains("path variable") {
            Intent::AddEnvironment
        } else if prompt_lower.contains("service") || prompt_lower.contains("daemon") || prompt_lower.contains("background process") {
            Intent::InstallService
        } else if prompt_lower.contains("custom action") || prompt_lower.contains("run script") || prompt_lower.contains("execute command") {
            Intent::AddCustomAction
        } else if prompt_lower.contains("feature") || prompt_lower.contains("optional") || prompt_lower.contains("selectable") {
            Intent::CreateFeature
        } else if prompt_lower.contains("prerequisite") || prompt_lower.contains("require") || prompt_lower.contains(".net") || prompt_lower.contains("runtime") {
            Intent::AddPrerequisite
        } else if prompt_lower.contains("upgrade") || prompt_lower.contains("update") || prompt_lower.contains("previous version") {
            Intent::AddUpgrade
        } else if prompt_lower.contains("file") || prompt_lower.contains("install") || prompt_lower.contains("add") || prompt_lower.contains("copy") {
            Intent::AddFile
        } else {
            Intent::Unknown
        }
    }

    /// Generate code from prompt
    pub fn generate(&self, prompt: &str, variables: &HashMap<String, String>) -> GenerationResult {
        let intent = self.detect_intent(prompt);

        let template = match intent {
            Intent::CreateProject => self.templates.iter().find(|t| t.name == "basic_installer"),
            Intent::AddFile => self.templates.iter().find(|t| t.name == "file_component"),
            Intent::CreateShortcut => self.templates.iter().find(|t| t.name == "shortcut"),
            Intent::AddRegistry => self.templates.iter().find(|t| t.name == "registry_entry"),
            Intent::AddEnvironment => self.templates.iter().find(|t| t.name == "environment_variable"),
            Intent::InstallService => self.templates.iter().find(|t| t.name == "service_install"),
            Intent::AddCustomAction => self.templates.iter().find(|t| t.name == "custom_action"),
            Intent::CreateFeature => self.templates.iter().find(|t| t.name == "feature"),
            Intent::AddPrerequisite => self.templates.iter().find(|t| t.name == "prerequisite"),
            Intent::AddUpgrade => self.templates.iter().find(|t| t.name == "major_upgrade"),
            Intent::Unknown => None,
        };

        match template {
            Some(tmpl) => {
                let mut code = tmpl.code.to_string();

                // Substitute variables
                for (key, value) in variables {
                    code = code.replace(&format!("{{{{{}}}}}", key), value);
                }

                // List remaining variables
                let remaining: Vec<String> = tmpl.variables.iter()
                    .filter(|v| code.contains(&format!("{{{{{}}}}}", v)))
                    .map(|v| v.to_string())
                    .collect();

                let suggestions = if !remaining.is_empty() {
                    vec![format!("Provide values for: {}", remaining.join(", "))]
                } else {
                    vec!["Code is complete and ready to use".to_string()]
                };

                GenerationResult {
                    intent,
                    code,
                    explanation: tmpl.description.to_string(),
                    suggestions,
                    references: vec![
                        "https://wixtoolset.org/docs/".to_string(),
                        "https://docs.firegiant.com/wix/".to_string(),
                    ],
                }
            }
            None => GenerationResult {
                intent: Intent::Unknown,
                code: String::new(),
                explanation: "Could not understand the request. Try being more specific.".to_string(),
                suggestions: vec![
                    "Try: 'create new installer project'".to_string(),
                    "Try: 'add a file to install'".to_string(),
                    "Try: 'create start menu shortcut'".to_string(),
                    "Try: 'add registry key'".to_string(),
                    "Try: 'install windows service'".to_string(),
                ],
                references: Vec::new(),
            },
        }
    }

    /// List available templates
    pub fn list_templates(&self) -> Vec<(&str, &str)> {
        self.templates.iter()
            .map(|t| (t.name, t.description))
            .collect()
    }

    /// Get template by name
    pub fn get_template(&self, name: &str) -> Option<&Template> {
        self.templates.iter().find(|t| t.name == name)
    }

    /// Suggest completions for partial WiX code
    pub fn suggest_completions(&self, context: &str) -> Vec<String> {
        let context_lower = context.to_lowercase();
        let mut suggestions = Vec::new();

        if context_lower.contains("<component") && !context_lower.contains("</component") {
            suggestions.push("  <File Id=\"\" Source=\"\" KeyPath=\"yes\" />".to_string());
            suggestions.push("</Component>".to_string());
        }

        if context_lower.contains("<directory") && !context_lower.contains("</directory") {
            suggestions.push("  <Component Id=\"\" Guid=\"*\">".to_string());
            suggestions.push("</Directory>".to_string());
        }

        if context_lower.contains("<feature") && !context_lower.contains("</feature") {
            suggestions.push("  <ComponentRef Id=\"\" />".to_string());
            suggestions.push("</Feature>".to_string());
        }

        if context_lower.contains("<package") && !context_lower.contains("</package") {
            suggestions.push("  <MajorUpgrade DowngradeErrorMessage=\"\" />".to_string());
            suggestions.push("  <MediaTemplate EmbedCab=\"yes\" />".to_string());
            suggestions.push("</Package>".to_string());
        }

        suggestions
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_intent_project() {
        let ai = WixAi::new();
        assert_eq!(ai.detect_intent("create new installer project"), Intent::CreateProject);
        assert_eq!(ai.detect_intent("I want to create a new MSI"), Intent::CreateProject);
    }

    #[test]
    fn test_detect_intent_shortcut() {
        let ai = WixAi::new();
        assert_eq!(ai.detect_intent("add start menu shortcut"), Intent::CreateShortcut);
        assert_eq!(ai.detect_intent("create desktop icon"), Intent::CreateShortcut);
    }

    #[test]
    fn test_detect_intent_service() {
        let ai = WixAi::new();
        assert_eq!(ai.detect_intent("install windows service"), Intent::InstallService);
    }

    #[test]
    fn test_generate_with_variables() {
        let ai = WixAi::new();
        let mut vars = HashMap::new();
        vars.insert("COMPONENT_ID".to_string(), "MainComp".to_string());
        vars.insert("FILE_ID".to_string(), "MainExe".to_string());
        vars.insert("SOURCE".to_string(), "app.exe".to_string());

        let result = ai.generate("add file to install", &vars);
        assert_eq!(result.intent, Intent::AddFile);
        assert!(result.code.contains("MainComp"));
        assert!(result.code.contains("app.exe"));
    }

    #[test]
    fn test_list_templates() {
        let ai = WixAi::new();
        let templates = ai.list_templates();
        assert!(!templates.is_empty());
        assert!(templates.iter().any(|(name, _)| *name == "basic_installer"));
    }

    #[test]
    fn test_suggest_completions() {
        let ai = WixAi::new();
        let suggestions = ai.suggest_completions("<Component Id=\"Test\">");
        assert!(!suggestions.is_empty());
        assert!(suggestions.iter().any(|s| s.contains("</Component>")));
    }
}

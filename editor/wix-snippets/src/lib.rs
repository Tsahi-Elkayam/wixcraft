//! wix-snippets - Snippet library for common WiX patterns
//!
//! Provides code snippets for VS Code, Sublime Text, and other editors.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A code snippet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snippet {
    pub prefix: String,
    pub body: Vec<String>,
    pub description: String,
    pub scope: Option<String>,
}

impl Snippet {
    pub fn new(prefix: &str, body: Vec<&str>, description: &str) -> Self {
        Self {
            prefix: prefix.to_string(),
            body: body.into_iter().map(String::from).collect(),
            description: description.to_string(),
            scope: Some("xml".to_string()),
        }
    }

    pub fn to_vscode_format(&self) -> serde_json::Value {
        serde_json::json!({
            "prefix": self.prefix,
            "body": self.body,
            "description": self.description
        })
    }

    pub fn to_sublime_format(&self) -> String {
        let body = self.body.join("\n");
        format!(
            "<snippet>\n\
             \t<content><![CDATA[\n{}\n]]></content>\n\
             \t<tabTrigger>{}</tabTrigger>\n\
             \t<scope>text.xml</scope>\n\
             \t<description>{}</description>\n\
             </snippet>",
            body, self.prefix, self.description
        )
    }
}

/// Snippet library
pub struct SnippetLibrary {
    snippets: HashMap<String, Snippet>,
}

impl SnippetLibrary {
    pub fn new() -> Self {
        let mut lib = Self {
            snippets: HashMap::new(),
        };
        lib.load_default_snippets();
        lib
    }

    fn load_default_snippets(&mut self) {
        // Product snippet
        self.add(Snippet::new(
            "wix-product",
            vec![
                "<Wix xmlns=\"http://wixtoolset.org/schemas/v4/wxs\">",
                "\t<Package Name=\"${1:ProductName}\"",
                "\t         Manufacturer=\"${2:Manufacturer}\"",
                "\t         Version=\"${3:1.0.0}\"",
                "\t         UpgradeCode=\"${4:PUT-GUID-HERE}\">",
                "\t\t$0",
                "\t</Package>",
                "</Wix>",
            ],
            "WiX v4 Product template",
        ));

        // Component snippet
        self.add(Snippet::new(
            "wix-component",
            vec![
                "<Component Id=\"${1:ComponentId}\" Guid=\"*\">",
                "\t<File Source=\"${2:SourcePath}\" />",
                "\t$0",
                "</Component>",
            ],
            "Component with file",
        ));

        // Directory snippet
        self.add(Snippet::new(
            "wix-directory",
            vec![
                "<Directory Id=\"${1:DirectoryId}\" Name=\"${2:DirectoryName}\">",
                "\t$0",
                "</Directory>",
            ],
            "Directory element",
        ));

        // Feature snippet
        self.add(Snippet::new(
            "wix-feature",
            vec![
                "<Feature Id=\"${1:FeatureId}\" Title=\"${2:Feature Title}\" Level=\"1\">",
                "\t<ComponentGroupRef Id=\"${3:ComponentGroupId}\" />",
                "\t$0",
                "</Feature>",
            ],
            "Feature element",
        ));

        // Registry snippet
        self.add(Snippet::new(
            "wix-registry",
            vec![
                "<RegistryKey Root=\"${1|HKLM,HKCU,HKCR|}\" Key=\"${2:Software\\\\Company\\\\Product}\">",
                "\t<RegistryValue Name=\"${3:ValueName}\" Type=\"${4|string,integer,binary|}\" Value=\"${5:Value}\" />",
                "</RegistryKey>",
            ],
            "Registry key with value",
        ));

        // Shortcut snippet
        self.add(Snippet::new(
            "wix-shortcut",
            vec![
                "<Shortcut Id=\"${1:ShortcutId}\"",
                "\t        Name=\"${2:Shortcut Name}\"",
                "\t        Directory=\"${3:ProgramMenuFolder}\"",
                "\t        Target=\"[${4:INSTALLFOLDER}]${5:app.exe}\"",
                "\t        WorkingDirectory=\"${4:INSTALLFOLDER}\" />",
            ],
            "Shortcut element",
        ));

        // Service install snippet
        self.add(Snippet::new(
            "wix-service",
            vec![
                "<ServiceInstall Id=\"${1:ServiceId}\"",
                "\t             Name=\"${2:ServiceName}\"",
                "\t             DisplayName=\"${3:Service Display Name}\"",
                "\t             Type=\"ownProcess\"",
                "\t             Start=\"auto\"",
                "\t             ErrorControl=\"normal\" />",
                "<ServiceControl Id=\"${1:ServiceId}Control\"",
                "\t             Name=\"${2:ServiceName}\"",
                "\t             Start=\"install\"",
                "\t             Stop=\"both\"",
                "\t             Remove=\"uninstall\" />",
            ],
            "Windows service installation",
        ));

        // Custom action snippet
        self.add(Snippet::new(
            "wix-customaction",
            vec![
                "<CustomAction Id=\"${1:ActionId}\"",
                "\t           DllEntry=\"${2:EntryPoint}\"",
                "\t           BinaryRef=\"${3:BinaryId}\"",
                "\t           Execute=\"${4|deferred,immediate|}\"",
                "\t           Return=\"${5|check,ignore|}\" />",
            ],
            "Custom action",
        ));

        // UI reference snippet
        self.add(Snippet::new(
            "wix-ui",
            vec![
                "<UI>",
                "\t<UIRef Id=\"WixUI_${1|Minimal,InstallDir,FeatureTree,Mondo|}\" />",
                "</UI>",
            ],
            "Standard UI reference",
        ));

        // Property snippet
        self.add(Snippet::new(
            "wix-property",
            vec![
                "<Property Id=\"${1:PROPERTYNAME}\" Value=\"${2:DefaultValue}\" />",
            ],
            "Property definition",
        ));

        // Condition snippet
        self.add(Snippet::new(
            "wix-condition",
            vec![
                "<Launch Condition=\"${1:Condition}\" Message=\"${2:Error message}\" />",
            ],
            "Launch condition",
        ));

        // MajorUpgrade snippet
        self.add(Snippet::new(
            "wix-upgrade",
            vec![
                "<MajorUpgrade",
                "\tDowngradeErrorMessage=\"A newer version of [ProductName] is already installed.\"",
                "\tAllowSameVersionUpgrades=\"yes\" />",
            ],
            "Major upgrade configuration",
        ));

        // ComponentGroup snippet
        self.add(Snippet::new(
            "wix-componentgroup",
            vec![
                "<ComponentGroup Id=\"${1:GroupId}\" Directory=\"${2:INSTALLFOLDER}\">",
                "\t$0",
                "</ComponentGroup>",
            ],
            "Component group",
        ));

        // Fragment snippet
        self.add(Snippet::new(
            "wix-fragment",
            vec![
                "<Fragment>",
                "\t$0",
                "</Fragment>",
            ],
            "Fragment element",
        ));

        // StandardDirectory snippet
        self.add(Snippet::new(
            "wix-stddir",
            vec![
                "<StandardDirectory Id=\"${1|ProgramFilesFolder,ProgramFiles64Folder,CommonFilesFolder,LocalAppDataFolder,AppDataFolder,DesktopFolder,ProgramMenuFolder,StartupFolder|}\">",
                "\t$0",
                "</StandardDirectory>",
            ],
            "Standard directory reference",
        ));
    }

    pub fn add(&mut self, snippet: Snippet) {
        self.snippets.insert(snippet.prefix.clone(), snippet);
    }

    pub fn get(&self, prefix: &str) -> Option<&Snippet> {
        self.snippets.get(prefix)
    }

    pub fn all(&self) -> impl Iterator<Item = &Snippet> {
        self.snippets.values()
    }

    pub fn to_vscode_json(&self) -> String {
        let mut map = serde_json::Map::new();
        for (name, snippet) in &self.snippets {
            map.insert(name.clone(), snippet.to_vscode_format());
        }
        serde_json::to_string_pretty(&map).unwrap()
    }

    pub fn count(&self) -> usize {
        self.snippets.len()
    }
}

impl Default for SnippetLibrary {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snippet_library_creation() {
        let lib = SnippetLibrary::new();
        assert!(lib.count() > 0);
    }

    #[test]
    fn test_get_snippet() {
        let lib = SnippetLibrary::new();
        let snippet = lib.get("wix-product");
        assert!(snippet.is_some());
    }

    #[test]
    fn test_vscode_format() {
        let lib = SnippetLibrary::new();
        let json = lib.to_vscode_json();
        assert!(json.contains("wix-product"));
        assert!(json.contains("prefix"));
    }

    #[test]
    fn test_sublime_format() {
        let snippet = Snippet::new("test", vec!["<Test />"], "Test snippet");
        let sublime = snippet.to_sublime_format();
        assert!(sublime.contains("<snippet>"));
        assert!(sublime.contains("tabTrigger"));
    }

    #[test]
    fn test_all_snippets_have_body() {
        let lib = SnippetLibrary::new();
        for snippet in lib.all() {
            assert!(!snippet.body.is_empty());
            assert!(!snippet.prefix.is_empty());
            assert!(!snippet.description.is_empty());
        }
    }
}

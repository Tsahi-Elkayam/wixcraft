//! wix-sublime - Sublime Text package generator for WiX
//!
//! Generates Sublime Text package with language support.

use serde::{Deserialize, Serialize};

/// Sublime Text syntax definition (YAML format as string)
pub struct SublimeSyntax;

impl SublimeSyntax {
    pub fn generate() -> String {
        r#"%YAML 1.2
---
name: WiX
file_extensions:
  - wxs
  - wxi
  - wxl
scope: text.xml.wix
contexts:
  main:
    - include: xml
    - include: wix-elements
    - include: wix-attributes
    - include: guid
    - include: preprocessor
    - include: property-reference

  xml:
    - match: '<!--'
      push: comment
    - match: '<\?'
      push: processing-instruction
    - match: '<!\[CDATA\['
      push: cdata

  comment:
    - meta_scope: comment.block.xml
    - match: '-->'
      pop: true

  processing-instruction:
    - meta_scope: meta.tag.preprocessor.xml
    - match: '\?>'
      pop: true

  cdata:
    - meta_scope: string.unquoted.cdata.xml
    - match: '\]\]>'
      pop: true

  wix-elements:
    - match: '\b(Wix|Package|Product|Fragment|Component|ComponentGroup|Directory|DirectoryRef|Feature|FeatureRef|File|Property|Registry|RegistryKey|RegistryValue|Shortcut|ServiceInstall|ServiceControl|CustomAction|Binary|UI|UIRef|Media|MajorUpgrade|StandardDirectory)\b'
      scope: entity.name.tag.wix

  wix-attributes:
    - match: '\b(Id|Name|Value|Source|Directory|Guid|Type|Key|Root|Action|Execute|Return|Level|Title|Description|Manufacturer|Version|UpgradeCode)\b'
      scope: entity.other.attribute-name.wix

  guid:
    - match: '\{[0-9A-Fa-f]{8}-[0-9A-Fa-f]{4}-[0-9A-Fa-f]{4}-[0-9A-Fa-f]{4}-[0-9A-Fa-f]{12}\}'
      scope: constant.other.guid.wix

  preprocessor:
    - match: '\$\(var\.[A-Za-z_][A-Za-z0-9_]*\)'
      scope: variable.other.preprocessor.wix

  property-reference:
    - match: '\[[A-Z_][A-Z0-9_]*\]'
      scope: variable.other.property.wix
"#.to_string()
    }
}

/// Sublime Text settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SublimeSettings {
    pub extensions: Vec<String>,
    pub tab_size: u32,
    pub translate_tabs_to_spaces: bool,
}

impl SublimeSettings {
    pub fn wix_settings() -> Self {
        Self {
            extensions: vec!["wxs".to_string(), "wxi".to_string(), "wxl".to_string()],
            tab_size: 2,
            translate_tabs_to_spaces: true,
        }
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap()
    }
}

/// Sublime Text build system
pub struct SublimeBuildSystem;

impl SublimeBuildSystem {
    pub fn generate() -> String {
        r#"{
    "shell_cmd": "wix build \"${file}\"",
    "file_regex": "^(.+?):(\\d+):(\\d+): (.+)$",
    "working_dir": "${file_path}",
    "selector": "text.xml.wix",
    "variants": [
        {
            "name": "Build Release",
            "shell_cmd": "wix build -arch x64 -o \"${file_base_name}.msi\" \"${file}\""
        },
        {
            "name": "Validate",
            "shell_cmd": "wix-lint \"${file}\""
        }
    ]
}"#.to_string()
    }
}

/// Sublime Text completions
pub struct SublimeCompletions;

impl SublimeCompletions {
    pub fn generate() -> String {
        r#"{
    "scope": "text.xml.wix",
    "completions": [
        { "trigger": "component\tWiX Component", "contents": "<Component Id=\"${1:ComponentId}\" Guid=\"*\">\n\t<File Source=\"${2:SourcePath}\" />\n</Component>" },
        { "trigger": "directory\tWiX Directory", "contents": "<Directory Id=\"${1:DirectoryId}\" Name=\"${2:Name}\">\n\t$0\n</Directory>" },
        { "trigger": "feature\tWiX Feature", "contents": "<Feature Id=\"${1:FeatureId}\" Title=\"${2:Title}\" Level=\"1\">\n\t<ComponentGroupRef Id=\"${3:GroupId}\" />\n</Feature>" },
        { "trigger": "file\tWiX File", "contents": "<File Source=\"${1:SourcePath}\" />" },
        { "trigger": "property\tWiX Property", "contents": "<Property Id=\"${1:PROPERTYNAME}\" Value=\"${2:Value}\" />" },
        { "trigger": "registry\tWiX Registry", "contents": "<RegistryKey Root=\"${1:HKLM}\" Key=\"${2:Software\\\\Company}\">\n\t<RegistryValue Name=\"${3:Name}\" Type=\"string\" Value=\"${4:Value}\" />\n</RegistryKey>" },
        { "trigger": "shortcut\tWiX Shortcut", "contents": "<Shortcut Id=\"${1:ShortcutId}\" Name=\"${2:Name}\" Directory=\"ProgramMenuFolder\" Target=\"[INSTALLFOLDER]${3:app.exe}\" />" },
        { "trigger": "customaction\tWiX Custom Action", "contents": "<CustomAction Id=\"${1:ActionId}\" DllEntry=\"${2:EntryPoint}\" BinaryRef=\"${3:BinaryId}\" Execute=\"deferred\" Return=\"check\" />" },
        { "trigger": "majorupgrade\tWiX Major Upgrade", "contents": "<MajorUpgrade DowngradeErrorMessage=\"A newer version is already installed.\" />" }
    ]
}"#.to_string()
    }
}

/// Package generator
pub struct PackageGenerator;

impl PackageGenerator {
    pub fn generate_syntax() -> String {
        SublimeSyntax::generate()
    }

    pub fn generate_settings() -> String {
        SublimeSettings::wix_settings().to_json()
    }

    pub fn generate_build_system() -> String {
        SublimeBuildSystem::generate()
    }

    pub fn generate_completions() -> String {
        SublimeCompletions::generate()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_syntax_generation() {
        let syntax = SublimeSyntax::generate();
        assert!(syntax.contains("name: WiX"));
        assert!(syntax.contains("wxs"));
    }

    #[test]
    fn test_settings() {
        let settings = SublimeSettings::wix_settings();
        assert!(settings.extensions.contains(&"wxs".to_string()));
    }

    #[test]
    fn test_build_system() {
        let build = SublimeBuildSystem::generate();
        assert!(build.contains("wix build"));
    }

    #[test]
    fn test_completions() {
        let completions = SublimeCompletions::generate();
        assert!(completions.contains("component"));
    }
}

//! wix-vscode - VS Code extension generator for WiX
//!
//! Generates VS Code extension package with language support.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// VS Code extension manifest (package.json)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct ExtensionManifest {
    pub name: String,
    #[serde(rename = "displayName")]
    pub display_name: String,
    pub description: String,
    pub version: String,
    pub publisher: String,
    pub engines: Engines,
    pub categories: Vec<String>,
    pub activationEvents: Vec<String>,
    pub main: Option<String>,
    pub contributes: Contributes,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Engines {
    pub vscode: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contributes {
    pub languages: Vec<Language>,
    pub grammars: Vec<Grammar>,
    pub snippets: Vec<SnippetContrib>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub configuration: Option<Configuration>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Language {
    pub id: String,
    pub aliases: Vec<String>,
    pub extensions: Vec<String>,
    pub configuration: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Grammar {
    pub language: String,
    #[serde(rename = "scopeName")]
    pub scope_name: String,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnippetContrib {
    pub language: String,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Configuration {
    pub title: String,
    pub properties: HashMap<String, ConfigProperty>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigProperty {
    #[serde(rename = "type")]
    pub prop_type: String,
    pub default: serde_json::Value,
    pub description: String,
}

impl ExtensionManifest {
    pub fn new() -> Self {
        let mut properties = HashMap::new();
        properties.insert(
            "wix.enableLsp".to_string(),
            ConfigProperty {
                prop_type: "boolean".to_string(),
                default: serde_json::Value::Bool(true),
                description: "Enable WiX Language Server".to_string(),
            },
        );
        properties.insert(
            "wix.lspPath".to_string(),
            ConfigProperty {
                prop_type: "string".to_string(),
                default: serde_json::Value::String("wix-lsp".to_string()),
                description: "Path to wix-lsp executable".to_string(),
            },
        );

        Self {
            name: "wix-language".to_string(),
            display_name: "WiX Language Support".to_string(),
            description: "WiX Toolset language support with IntelliSense, snippets, and more".to_string(),
            version: "1.0.0".to_string(),
            publisher: "wixcraft".to_string(),
            engines: Engines {
                vscode: "^1.60.0".to_string(),
            },
            categories: vec![
                "Programming Languages".to_string(),
                "Snippets".to_string(),
                "Linters".to_string(),
            ],
            activationEvents: vec![
                "onLanguage:wix".to_string(),
            ],
            main: Some("./out/extension.js".to_string()),
            contributes: Contributes {
                languages: vec![Language {
                    id: "wix".to_string(),
                    aliases: vec!["WiX".to_string(), "wix".to_string()],
                    extensions: vec![".wxs".to_string(), ".wxi".to_string(), ".wxl".to_string()],
                    configuration: "./language-configuration.json".to_string(),
                }],
                grammars: vec![Grammar {
                    language: "wix".to_string(),
                    scope_name: "text.xml.wix".to_string(),
                    path: "./syntaxes/wix.tmLanguage.json".to_string(),
                }],
                snippets: vec![SnippetContrib {
                    language: "wix".to_string(),
                    path: "./snippets/wix.json".to_string(),
                }],
                configuration: Some(Configuration {
                    title: "WiX".to_string(),
                    properties,
                }),
            },
        }
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap()
    }
}

impl Default for ExtensionManifest {
    fn default() -> Self {
        Self::new()
    }
}

/// Generate extension TypeScript entry point
pub fn generate_extension_ts() -> String {
    r#"import * as vscode from 'vscode';
import * as path from 'path';
import { LanguageClient, LanguageClientOptions, ServerOptions } from 'vscode-languageclient/node';

let client: LanguageClient;

export function activate(context: vscode.ExtensionContext) {
    const config = vscode.workspace.getConfiguration('wix');

    if (!config.get('enableLsp', true)) {
        return;
    }

    const serverPath = config.get<string>('lspPath', 'wix-lsp');

    const serverOptions: ServerOptions = {
        run: { command: serverPath, args: ['--stdio'] },
        debug: { command: serverPath, args: ['--stdio', '--log-level', 'debug'] }
    };

    const clientOptions: LanguageClientOptions = {
        documentSelector: [{ scheme: 'file', language: 'wix' }],
        synchronize: {
            fileEvents: vscode.workspace.createFileSystemWatcher('**/*.{wxs,wxi,wxl}')
        }
    };

    client = new LanguageClient(
        'wixLanguageServer',
        'WiX Language Server',
        serverOptions,
        clientOptions
    );

    client.start();
}

export function deactivate(): Thenable<void> | undefined {
    if (!client) {
        return undefined;
    }
    return client.stop();
}
"#.to_string()
}

/// Generate extension files
pub struct ExtensionGenerator;

impl ExtensionGenerator {
    pub fn generate_package_json() -> String {
        ExtensionManifest::new().to_json()
    }

    pub fn generate_extension_ts() -> String {
        generate_extension_ts()
    }

    pub fn generate_tsconfig() -> String {
        r#"{
    "compilerOptions": {
        "module": "commonjs",
        "target": "ES2020",
        "outDir": "out",
        "lib": ["ES2020"],
        "sourceMap": true,
        "rootDir": "src",
        "strict": true
    },
    "exclude": ["node_modules", ".vscode-test"]
}"#.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manifest_creation() {
        let manifest = ExtensionManifest::new();
        assert_eq!(manifest.name, "wix-language");
    }

    #[test]
    fn test_manifest_json() {
        let manifest = ExtensionManifest::new();
        let json = manifest.to_json();
        assert!(json.contains("wix-language"));
        assert!(json.contains(".wxs"));
    }

    #[test]
    fn test_extension_ts() {
        let ts = generate_extension_ts();
        assert!(ts.contains("LanguageClient"));
        assert!(ts.contains("activate"));
    }

    #[test]
    fn test_generator() {
        let json = ExtensionGenerator::generate_package_json();
        assert!(json.contains("contributes"));
    }
}

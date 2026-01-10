//! wix-wdac - WDAC (Windows Defender Application Control) compatibility checker
//!
//! Analyzes WiX projects for WDAC policy compatibility issues:
//! - Unsigned custom action DLLs
//! - Unsigned executables
//! - Script-based custom actions (VBScript, JScript)
//! - PowerShell custom actions without constrained language mode

use serde::{Deserialize, Serialize};

/// WDAC compatibility issue severity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity {
    /// Will be blocked by WDAC
    Blocker,
    /// May be blocked depending on policy
    Warning,
    /// Informational - consider for stricter policies
    Info,
}

/// Type of WDAC issue
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum IssueType {
    /// Unsigned custom action DLL
    UnsignedCustomAction,
    /// Unsigned executable
    UnsignedExecutable,
    /// VBScript custom action
    VBScriptAction,
    /// JScript custom action
    JScriptAction,
    /// PowerShell custom action
    PowerShellAction,
    /// Unsigned WiX extension DLL
    UnsignedWixExtension,
    /// Script file reference
    ScriptFile,
    /// Embedded binary without signature
    EmbeddedBinary,
}

/// A WDAC compatibility issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WdacIssue {
    pub issue_type: IssueType,
    pub severity: Severity,
    pub file: String,
    pub element: Option<String>,
    pub line: Option<usize>,
    pub message: String,
    pub recommendation: String,
}

/// WDAC analysis result
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WdacAnalysis {
    pub issues: Vec<WdacIssue>,
    pub custom_actions: Vec<CustomActionInfo>,
    pub executables: Vec<ExecutableInfo>,
    pub scripts: Vec<ScriptInfo>,
    pub is_wdac_compatible: bool,
    pub blocker_count: usize,
    pub warning_count: usize,
}

/// Custom action information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomActionInfo {
    pub id: String,
    pub action_type: CustomActionType,
    pub source: Option<String>,
    pub target: Option<String>,
    pub is_signed: Option<bool>,
    pub line: Option<usize>,
}

/// Custom action type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CustomActionType {
    /// Native DLL (Type 1, 17)
    NativeDll,
    /// Managed DLL (.NET)
    ManagedDll,
    /// VBScript (Type 6, 22, 38, 54)
    VBScript,
    /// JScript (Type 5, 21, 37, 53)
    JScript,
    /// PowerShell
    PowerShell,
    /// Executable (Type 2, 18, 34, 50)
    Executable,
    /// Property set
    PropertySet,
    /// Directory set
    DirectorySet,
    /// Unknown
    Unknown,
}

/// Executable information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutableInfo {
    pub id: String,
    pub source: String,
    pub is_signed: Option<bool>,
    pub line: Option<usize>,
}

/// Script information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptInfo {
    pub id: String,
    pub script_type: String,
    pub source: Option<String>,
    pub line: Option<usize>,
}

/// Known unsigned WiX extension DLLs
pub const UNSIGNED_WIX_DLLS: &[&str] = &[
    "wixca.dll",
    "WixCA.dll",
    "firewall.dll",
    "netfx.dll",
    "iis.dll",
    "sql.dll",
    "util.dll",
];

/// WDAC analyzer
pub struct WdacAnalyzer;

impl WdacAnalyzer {
    /// Analyze WiX source for WDAC compatibility
    pub fn analyze(content: &str) -> WdacAnalysis {
        let mut analysis = WdacAnalysis::default();

        if let Ok(doc) = roxmltree::Document::parse(content) {
            // Analyze custom actions
            for node in doc.descendants() {
                if node.tag_name().name() == "CustomAction" {
                    let ca = Self::analyze_custom_action(&node);

                    // Check for WDAC issues
                    match ca.action_type {
                        CustomActionType::VBScript => {
                            analysis.issues.push(WdacIssue {
                                issue_type: IssueType::VBScriptAction,
                                severity: Severity::Blocker,
                                file: String::new(),
                                element: Some(ca.id.clone()),
                                line: ca.line,
                                message: format!("VBScript custom action '{}' will be blocked by WDAC", ca.id),
                                recommendation: "Convert to a signed native or managed DLL custom action".to_string(),
                            });
                        }
                        CustomActionType::JScript => {
                            analysis.issues.push(WdacIssue {
                                issue_type: IssueType::JScriptAction,
                                severity: Severity::Blocker,
                                file: String::new(),
                                element: Some(ca.id.clone()),
                                line: ca.line,
                                message: format!("JScript custom action '{}' will be blocked by WDAC", ca.id),
                                recommendation: "Convert to a signed native or managed DLL custom action".to_string(),
                            });
                        }
                        CustomActionType::PowerShell => {
                            analysis.issues.push(WdacIssue {
                                issue_type: IssueType::PowerShellAction,
                                severity: Severity::Warning,
                                file: String::new(),
                                element: Some(ca.id.clone()),
                                line: ca.line,
                                message: format!("PowerShell custom action '{}' may be blocked by WDAC", ca.id),
                                recommendation: "Use constrained language mode or convert to signed DLL".to_string(),
                            });
                        }
                        CustomActionType::NativeDll | CustomActionType::ManagedDll => {
                            if ca.is_signed == Some(false) {
                                analysis.issues.push(WdacIssue {
                                    issue_type: IssueType::UnsignedCustomAction,
                                    severity: Severity::Blocker,
                                    file: String::new(),
                                    element: Some(ca.id.clone()),
                                    line: ca.line,
                                    message: format!("Unsigned custom action DLL '{}' will be blocked by WDAC", ca.id),
                                    recommendation: "Sign the custom action DLL with a trusted certificate".to_string(),
                                });
                            }
                        }
                        _ => {}
                    }

                    analysis.custom_actions.push(ca);
                }

                // Check for Binary elements (potential unsigned DLLs)
                if node.tag_name().name() == "Binary" {
                    if let Some(source) = node.attribute("SourceFile").or(node.attribute("Source")) {
                        let source_lower = source.to_lowercase();
                        if source_lower.ends_with(".dll") || source_lower.ends_with(".exe") {
                            // Check if it's a known unsigned WiX DLL
                            // Handle both Unix and Windows path separators
                            let filename = source
                                .rsplit(|c| c == '/' || c == '\\')
                                .next()
                                .unwrap_or(source);

                            if UNSIGNED_WIX_DLLS.iter().any(|&dll| filename.eq_ignore_ascii_case(dll)) {
                                analysis.issues.push(WdacIssue {
                                    issue_type: IssueType::UnsignedWixExtension,
                                    severity: Severity::Blocker,
                                    file: source.to_string(),
                                    element: node.attribute("Id").map(String::from),
                                    line: None,
                                    message: format!("WiX extension DLL '{}' is not code-signed", filename),
                                    recommendation: "Add hash rule to WDAC policy or request signed version from WiX team".to_string(),
                                });
                            }
                        }
                    }
                }

                // Check for File elements with executables
                if node.tag_name().name() == "File" {
                    if let Some(source) = node.attribute("Source") {
                        let source_lower = source.to_lowercase();
                        if source_lower.ends_with(".exe") || source_lower.ends_with(".dll") {
                            analysis.executables.push(ExecutableInfo {
                                id: node.attribute("Id").unwrap_or("").to_string(),
                                source: source.to_string(),
                                is_signed: None, // Would need actual file to check
                                line: None,
                            });
                        }

                        // Check for script files
                        if source_lower.ends_with(".vbs") || source_lower.ends_with(".js")
                            || source_lower.ends_with(".ps1") || source_lower.ends_with(".bat")
                            || source_lower.ends_with(".cmd") {
                            // Extract extension without Path (cross-platform)
                            let ext = source.rsplit('.').next().unwrap_or("unknown");
                            analysis.scripts.push(ScriptInfo {
                                id: node.attribute("Id").unwrap_or("").to_string(),
                                script_type: ext.to_string(),
                                source: Some(source.to_string()),
                                line: None,
                            });

                            analysis.issues.push(WdacIssue {
                                issue_type: IssueType::ScriptFile,
                                severity: Severity::Warning,
                                file: source.to_string(),
                                element: node.attribute("Id").map(String::from),
                                line: None,
                                message: format!("Script file '{}' may be blocked by WDAC", source),
                                recommendation: "Consider converting to compiled code or adding to WDAC allow list".to_string(),
                            });
                        }
                    }
                }
            }
        }

        // Calculate summary
        analysis.blocker_count = analysis.issues.iter()
            .filter(|i| i.severity == Severity::Blocker)
            .count();
        analysis.warning_count = analysis.issues.iter()
            .filter(|i| i.severity == Severity::Warning)
            .count();
        analysis.is_wdac_compatible = analysis.blocker_count == 0;

        analysis
    }

    fn analyze_custom_action(node: &roxmltree::Node) -> CustomActionInfo {
        let id = node.attribute("Id").unwrap_or("").to_string();
        let binary_key = node.attribute("BinaryKey").or(node.attribute("BinaryRef"));
        let dll_entry = node.attribute("DllEntry");
        let exe_command = node.attribute("ExeCommand");
        let script = node.attribute("Script");
        let value = node.attribute("Value");
        let directory = node.attribute("Directory");
        let property = node.attribute("Property");

        let action_type = if script.is_some() {
            match script.unwrap().to_lowercase().as_str() {
                "vbscript" => CustomActionType::VBScript,
                "jscript" => CustomActionType::JScript,
                _ => CustomActionType::Unknown,
            }
        } else if dll_entry.is_some() {
            // Check if it's managed by looking for common patterns
            CustomActionType::NativeDll
        } else if exe_command.is_some() {
            CustomActionType::Executable
        } else if property.is_some() && value.is_some() {
            CustomActionType::PropertySet
        } else if property.is_some() && directory.is_some() {
            CustomActionType::DirectorySet
        } else {
            CustomActionType::Unknown
        };

        CustomActionInfo {
            id,
            action_type,
            source: binary_key.map(String::from),
            target: dll_entry.or(exe_command).map(String::from),
            is_signed: None,
            line: None,
        }
    }

    /// Generate WDAC policy rules for allowing WiX components
    pub fn generate_allow_rules(analysis: &WdacAnalysis) -> String {
        let mut rules = String::new();
        rules.push_str("<!-- WDAC Allow Rules for WiX Installer -->\n");
        rules.push_str("<!-- Add these to your WDAC policy XML -->\n\n");

        rules.push_str("<FileRules>\n");

        // Add rules for unsigned WiX DLLs
        for issue in &analysis.issues {
            if issue.issue_type == IssueType::UnsignedWixExtension {
                rules.push_str(&format!(
                    "  <!-- {} -->\n",
                    issue.message
                ));
                rules.push_str(&format!(
                    "  <Allow ID=\"ID_ALLOW_{}\" FriendlyName=\"{}\" Hash=\"[CALCULATE_HASH]\" />\n",
                    issue.element.as_deref().unwrap_or("UNKNOWN"),
                    issue.file
                ));
            }
        }

        rules.push_str("</FileRules>\n");
        rules
    }

    /// Generate recommendations report
    pub fn generate_report(analysis: &WdacAnalysis) -> String {
        let mut report = String::new();

        report.push_str("WDAC Compatibility Report\n");
        report.push_str(&"=".repeat(50));
        report.push('\n');

        if analysis.is_wdac_compatible {
            report.push_str("\nStatus: COMPATIBLE\n");
            report.push_str("No blocking issues found.\n");
        } else {
            report.push_str("\nStatus: NOT COMPATIBLE\n");
            report.push_str(&format!("Blockers: {}\n", analysis.blocker_count));
            report.push_str(&format!("Warnings: {}\n", analysis.warning_count));
        }

        report.push_str(&format!("\nCustom Actions: {}\n", analysis.custom_actions.len()));
        report.push_str(&format!("Executables: {}\n", analysis.executables.len()));
        report.push_str(&format!("Scripts: {}\n", analysis.scripts.len()));

        if !analysis.issues.is_empty() {
            report.push_str("\nIssues:\n");
            report.push_str(&"-".repeat(50));
            report.push('\n');

            for (i, issue) in analysis.issues.iter().enumerate() {
                report.push_str(&format!(
                    "\n{}. [{}] {}\n",
                    i + 1,
                    match issue.severity {
                        Severity::Blocker => "BLOCKER",
                        Severity::Warning => "WARNING",
                        Severity::Info => "INFO",
                    },
                    issue.message
                ));
                report.push_str(&format!("   Recommendation: {}\n", issue.recommendation));
            }
        }

        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyze_vbscript_action() {
        let content = r#"
        <Wix xmlns="http://wixtoolset.org/schemas/v4/wxs">
            <Package>
                <CustomAction Id="SetProps" Script="vbscript">
                    MsgBox "Hello"
                </CustomAction>
            </Package>
        </Wix>
        "#;

        let analysis = WdacAnalyzer::analyze(content);
        assert!(!analysis.is_wdac_compatible);
        assert_eq!(analysis.blocker_count, 1);
        assert!(analysis.issues.iter().any(|i| i.issue_type == IssueType::VBScriptAction));
    }

    #[test]
    fn test_analyze_jscript_action() {
        let content = r#"
        <Wix xmlns="http://wixtoolset.org/schemas/v4/wxs">
            <Package>
                <CustomAction Id="RunScript" Script="jscript">
                    WScript.Echo("Hello");
                </CustomAction>
            </Package>
        </Wix>
        "#;

        let analysis = WdacAnalyzer::analyze(content);
        assert!(!analysis.is_wdac_compatible);
        assert!(analysis.issues.iter().any(|i| i.issue_type == IssueType::JScriptAction));
    }

    #[test]
    fn test_analyze_unsigned_wix_dll() {
        let content = r#"
        <Wix xmlns="http://wixtoolset.org/schemas/v4/wxs">
            <Package>
                <Binary Id="WixCA" SourceFile="$(WixToolPath)\wixca.dll" />
            </Package>
        </Wix>
        "#;

        let analysis = WdacAnalyzer::analyze(content);
        assert!(analysis.issues.iter().any(|i| i.issue_type == IssueType::UnsignedWixExtension));
    }

    #[test]
    fn test_analyze_script_file() {
        let content = r#"
        <Wix xmlns="http://wixtoolset.org/schemas/v4/wxs">
            <Package>
                <Component>
                    <File Id="Script" Source="setup.ps1" />
                </Component>
            </Package>
        </Wix>
        "#;

        let analysis = WdacAnalyzer::analyze(content);
        assert!(analysis.issues.iter().any(|i| i.issue_type == IssueType::ScriptFile));
    }

    #[test]
    fn test_clean_project() {
        let content = r#"
        <Wix xmlns="http://wixtoolset.org/schemas/v4/wxs">
            <Package Name="Test" Version="1.0.0" Manufacturer="Test">
                <Component>
                    <File Id="MainExe" Source="app.exe" />
                </Component>
            </Package>
        </Wix>
        "#;

        let analysis = WdacAnalyzer::analyze(content);
        assert_eq!(analysis.blocker_count, 0);
    }

    #[test]
    fn test_generate_report() {
        let content = r#"
        <Wix xmlns="http://wixtoolset.org/schemas/v4/wxs">
            <Package>
                <CustomAction Id="VBS" Script="vbscript">test</CustomAction>
            </Package>
        </Wix>
        "#;

        let analysis = WdacAnalyzer::analyze(content);
        let report = WdacAnalyzer::generate_report(&analysis);
        assert!(report.contains("NOT COMPATIBLE"));
        assert!(report.contains("BLOCKER"));
    }
}

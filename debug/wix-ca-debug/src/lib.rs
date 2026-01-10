//! wix-ca-debug - Custom Action debugger and analyzer for WiX/MSI
//!
//! Helps developers debug and analyze WiX custom actions by:
//! - Analyzing custom action definitions and scheduling
//! - Detecting common security and configuration issues
//! - Generating debug helper code
//! - Providing debugging guidance

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Custom action execution context
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionContext {
    /// Runs during UI sequence (client process)
    Immediate,
    /// Runs during execute sequence with user impersonation
    Deferred,
    /// Runs during execute sequence as SYSTEM
    DeferredNoImpersonate,
    /// Runs during rollback
    Rollback,
    /// Runs during commit
    Commit,
}

/// Custom action type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CustomActionType {
    /// DLL custom action (Type 1, 17)
    Dll,
    /// EXE custom action (Type 2, 18, 34, 50)
    Exe,
    /// Script custom action (Type 5, 6, 21, 22, 37, 38, 53, 54)
    Script,
    /// Property set (Type 51)
    PropertySet,
    /// Directory set (Type 35)
    DirectorySet,
    /// Error (Type 19)
    Error,
    /// Nested install (Type 7, 23, 39)
    NestedInstall,
    /// Unknown
    Unknown,
}

/// Severity of an issue
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity {
    Critical,
    Error,
    Warning,
    Info,
}

/// A detected issue with a custom action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomActionIssue {
    pub custom_action_id: String,
    pub severity: Severity,
    pub category: String,
    pub message: String,
    pub suggestion: String,
    pub line: Option<usize>,
}

/// Parsed custom action definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomActionDef {
    pub id: String,
    pub action_type: CustomActionType,
    pub execution: ExecutionContext,
    pub binary_key: Option<String>,
    pub dll_entry: Option<String>,
    pub exe_command: Option<String>,
    pub script: Option<String>,
    pub property: Option<String>,
    pub value: Option<String>,
    pub directory: Option<String>,
    pub file_key: Option<String>,
    pub impersonate: bool,
    pub execute: String,
    pub return_type: String,
    pub sequence_tables: Vec<SequenceEntry>,
    pub line: Option<usize>,
    pub issues: Vec<CustomActionIssue>,
}

/// Sequence table entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SequenceEntry {
    pub table: String,
    pub sequence: Option<i32>,
    pub condition: Option<String>,
    pub before: Option<String>,
    pub after: Option<String>,
}

/// Analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResult {
    pub custom_actions: Vec<CustomActionDef>,
    pub issues: Vec<CustomActionIssue>,
    pub summary: AnalysisSummary,
}

/// Summary statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisSummary {
    pub total_custom_actions: usize,
    pub deferred_count: usize,
    pub immediate_count: usize,
    pub elevated_count: usize,
    pub critical_issues: usize,
    pub error_issues: usize,
    pub warning_issues: usize,
    pub info_issues: usize,
}

/// Custom Action Analyzer
pub struct CustomActionAnalyzer {
    security_patterns: Vec<(Regex, String, Severity)>,
}

impl Default for CustomActionAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl CustomActionAnalyzer {
    pub fn new() -> Self {
        let security_patterns = vec![
            // Dangerous executables in custom actions
            (
                Regex::new(r"(?i)(cmd\.exe|powershell\.exe|wscript\.exe|cscript\.exe)").unwrap(),
                "Running shell/script interpreters can be exploited for privilege escalation".to_string(),
                Severity::Warning,
            ),
            // Temp directory usage
            (
                Regex::new(r"(?i)(%temp%|\[TempFolder\]|\\temp\\)").unwrap(),
                "Writing to TEMP folder can be exploited (DLL hijacking, file replacement)".to_string(),
                Severity::Warning,
            ),
            // Network operations
            (
                Regex::new(r"(?i)(http://|https://|ftp://|\\\\[a-z])").unwrap(),
                "Network operations in custom actions can fail or be intercepted".to_string(),
                Severity::Info,
            ),
        ];

        Self { security_patterns }
    }

    /// Analyze a WiX XML file for custom actions
    pub fn analyze(&self, xml_content: &str) -> AnalysisResult {
        let mut custom_actions = Vec::new();
        let mut all_issues = Vec::new();

        // Parse XML
        let doc = match roxmltree::Document::parse(xml_content) {
            Ok(d) => d,
            Err(_) => {
                return AnalysisResult {
                    custom_actions: Vec::new(),
                    issues: vec![CustomActionIssue {
                        custom_action_id: String::new(),
                        severity: Severity::Error,
                        category: "Parse".to_string(),
                        message: "Failed to parse XML".to_string(),
                        suggestion: "Check XML syntax".to_string(),
                        line: None,
                    }],
                    summary: AnalysisSummary {
                        total_custom_actions: 0,
                        deferred_count: 0,
                        immediate_count: 0,
                        elevated_count: 0,
                        critical_issues: 1,
                        error_issues: 0,
                        warning_issues: 0,
                        info_issues: 0,
                    },
                };
            }
        };

        // Find all CustomAction elements
        let ca_elements: Vec<_> = doc
            .descendants()
            .filter(|n| n.has_tag_name("CustomAction"))
            .collect();

        // Find all sequence references
        let mut sequence_refs: HashMap<String, Vec<SequenceEntry>> = HashMap::new();

        for seq_table in &[
            "InstallUISequence",
            "InstallExecuteSequence",
            "AdminUISequence",
            "AdminExecuteSequence",
            "AdvertiseExecuteSequence",
        ] {
            for node in doc.descendants() {
                if node.has_tag_name("Custom") {
                    if let Some(parent) = node.parent() {
                        if parent.has_tag_name(*seq_table) {
                            if let Some(action) = node.attribute("Action") {
                                let entry = SequenceEntry {
                                    table: seq_table.to_string(),
                                    sequence: node
                                        .attribute("Sequence")
                                        .and_then(|s| s.parse().ok()),
                                    condition: node.text().map(|s| s.to_string()),
                                    before: node.attribute("Before").map(|s| s.to_string()),
                                    after: node.attribute("After").map(|s| s.to_string()),
                                };
                                sequence_refs
                                    .entry(action.to_string())
                                    .or_default()
                                    .push(entry);
                            }
                        }
                    }
                }
            }
        }

        // Process each CustomAction
        for ca_node in ca_elements {
            let id = ca_node.attribute("Id").unwrap_or("").to_string();
            if id.is_empty() {
                continue;
            }

            let mut ca_def = self.parse_custom_action(&ca_node, &id);
            ca_def.sequence_tables = sequence_refs.remove(&id).unwrap_or_default();

            // Analyze for issues
            let issues = self.analyze_custom_action(&ca_def);
            ca_def.issues = issues.clone();
            all_issues.extend(issues);

            custom_actions.push(ca_def);
        }

        // Generate summary
        let summary = AnalysisSummary {
            total_custom_actions: custom_actions.len(),
            deferred_count: custom_actions
                .iter()
                .filter(|ca| {
                    matches!(
                        ca.execution,
                        ExecutionContext::Deferred | ExecutionContext::DeferredNoImpersonate
                    )
                })
                .count(),
            immediate_count: custom_actions
                .iter()
                .filter(|ca| matches!(ca.execution, ExecutionContext::Immediate))
                .count(),
            elevated_count: custom_actions
                .iter()
                .filter(|ca| matches!(ca.execution, ExecutionContext::DeferredNoImpersonate))
                .count(),
            critical_issues: all_issues
                .iter()
                .filter(|i| matches!(i.severity, Severity::Critical))
                .count(),
            error_issues: all_issues
                .iter()
                .filter(|i| matches!(i.severity, Severity::Error))
                .count(),
            warning_issues: all_issues
                .iter()
                .filter(|i| matches!(i.severity, Severity::Warning))
                .count(),
            info_issues: all_issues
                .iter()
                .filter(|i| matches!(i.severity, Severity::Info))
                .count(),
        };

        AnalysisResult {
            custom_actions,
            issues: all_issues,
            summary,
        }
    }

    fn parse_custom_action(&self, node: &roxmltree::Node, id: &str) -> CustomActionDef {
        let binary_key = node.attribute("BinaryKey").or(node.attribute("BinaryRef"));
        let dll_entry = node.attribute("DllEntry");
        let exe_command = node.attribute("ExeCommand");
        let script = node.attribute("Script");
        let property = node.attribute("Property");
        let value = node.attribute("Value");
        let directory = node.attribute("Directory");
        let file_key = node.attribute("FileKey");
        let impersonate = node.attribute("Impersonate").map(|v| v == "yes").unwrap_or(true);
        let execute = node.attribute("Execute").unwrap_or("immediate").to_string();
        let return_type = node.attribute("Return").unwrap_or("check").to_string();

        // Determine action type
        let action_type = if dll_entry.is_some() {
            CustomActionType::Dll
        } else if exe_command.is_some() {
            CustomActionType::Exe
        } else if script.is_some() {
            CustomActionType::Script
        } else if property.is_some() && value.is_some() {
            CustomActionType::PropertySet
        } else if directory.is_some() {
            CustomActionType::DirectorySet
        } else {
            CustomActionType::Unknown
        };

        // Determine execution context
        let execution = match execute.as_str() {
            "deferred" => {
                if impersonate {
                    ExecutionContext::Deferred
                } else {
                    ExecutionContext::DeferredNoImpersonate
                }
            }
            "rollback" => ExecutionContext::Rollback,
            "commit" => ExecutionContext::Commit,
            _ => ExecutionContext::Immediate,
        };

        // Get line number from node position
        let pos = node.document().text_pos_at(node.range().start);
        let line = Some(pos.row as usize);

        CustomActionDef {
            id: id.to_string(),
            action_type,
            execution,
            binary_key: binary_key.map(|s| s.to_string()),
            dll_entry: dll_entry.map(|s| s.to_string()),
            exe_command: exe_command.map(|s| s.to_string()),
            script: script.map(|s| s.to_string()),
            property: property.map(|s| s.to_string()),
            value: value.map(|s| s.to_string()),
            directory: directory.map(|s| s.to_string()),
            file_key: file_key.map(|s| s.to_string()),
            impersonate,
            execute,
            return_type,
            sequence_tables: Vec::new(),
            line,
            issues: Vec::new(),
        }
    }

    fn analyze_custom_action(&self, ca: &CustomActionDef) -> Vec<CustomActionIssue> {
        let mut issues = Vec::new();

        // Check for elevated custom actions (security concern)
        if matches!(ca.execution, ExecutionContext::DeferredNoImpersonate) {
            issues.push(CustomActionIssue {
                custom_action_id: ca.id.clone(),
                severity: Severity::Warning,
                category: "Security".to_string(),
                message: "Custom action runs elevated (Impersonate=no)".to_string(),
                suggestion: "Ensure this action cannot be exploited for privilege escalation. \
                    Avoid running shell interpreters or writing to user-writable directories."
                    .to_string(),
                line: ca.line,
            });
        }

        // Check for EXE custom actions (harder to debug)
        if matches!(ca.action_type, CustomActionType::Exe) {
            issues.push(CustomActionIssue {
                custom_action_id: ca.id.clone(),
                severity: Severity::Info,
                category: "Debug".to_string(),
                message: "EXE custom actions are harder to debug than DLL custom actions".to_string(),
                suggestion: "Consider adding logging to the executable or using a wrapper script \
                    that logs output."
                    .to_string(),
                line: ca.line,
            });

            // Check for dangerous executables
            if let Some(ref cmd) = ca.exe_command {
                for (pattern, msg, severity) in &self.security_patterns {
                    if pattern.is_match(cmd) {
                        issues.push(CustomActionIssue {
                            custom_action_id: ca.id.clone(),
                            severity: *severity,
                            category: "Security".to_string(),
                            message: msg.clone(),
                            suggestion: "Review the command for security implications.".to_string(),
                            line: ca.line,
                        });
                    }
                }
            }
        }

        // Check for deferred actions without data property
        if matches!(
            ca.execution,
            ExecutionContext::Deferred | ExecutionContext::DeferredNoImpersonate
        ) && ca.property.is_none()
        {
            // Check if there's a SetProperty action to pass data
            if matches!(ca.action_type, CustomActionType::Dll) {
                issues.push(CustomActionIssue {
                    custom_action_id: ca.id.clone(),
                    severity: Severity::Info,
                    category: "Debug".to_string(),
                    message: "Deferred custom action may need a SetProperty action to pass data"
                        .to_string(),
                    suggestion: format!(
                        "If this action needs data from properties, create a Type 51 action \
                        with Property=\"{}\" to pass CustomActionData.",
                        ca.id
                    ),
                    line: ca.line,
                });
            }
        }

        // Check for async custom actions
        if ca.return_type == "asyncNoWait" || ca.return_type == "asyncWait" {
            issues.push(CustomActionIssue {
                custom_action_id: ca.id.clone(),
                severity: Severity::Warning,
                category: "Debug".to_string(),
                message: "Async custom actions are very difficult to debug".to_string(),
                suggestion: "Use Return=\"check\" during development. \
                    Async actions don't report errors to the installer."
                    .to_string(),
                line: ca.line,
            });
        }

        // Check for script custom actions (security and debugging concerns)
        if matches!(ca.action_type, CustomActionType::Script) {
            issues.push(CustomActionIssue {
                custom_action_id: ca.id.clone(),
                severity: Severity::Warning,
                category: "Security".to_string(),
                message: "Script custom actions (VBScript/JScript) are deprecated and insecure"
                    .to_string(),
                suggestion: "Consider migrating to a DLL custom action using C# or C++. \
                    Scripts can be modified in the MSI and run with elevated privileges."
                    .to_string(),
                line: ca.line,
            });
        }

        // Check if not scheduled anywhere
        if ca.sequence_tables.is_empty() {
            issues.push(CustomActionIssue {
                custom_action_id: ca.id.clone(),
                severity: Severity::Warning,
                category: "Config".to_string(),
                message: "Custom action is defined but not scheduled in any sequence".to_string(),
                suggestion: "Add a <Custom> element to schedule this action in \
                    InstallExecuteSequence or InstallUISequence."
                    .to_string(),
                line: ca.line,
            });
        }

        issues
    }

    /// Generate debug helper code for a custom action
    pub fn generate_debug_helper(&self, ca: &CustomActionDef, language: &str) -> String {
        match language {
            "csharp" | "c#" => self.generate_csharp_debug(ca),
            "cpp" | "c++" => self.generate_cpp_debug(ca),
            "vbscript" | "vbs" => self.generate_vbscript_debug(ca),
            _ => "Unsupported language".to_string(),
        }
    }

    fn generate_csharp_debug(&self, ca: &CustomActionDef) -> String {
        format!(
            r#"// Debug helper for custom action: {}
// Add this at the start of your custom action method

#if DEBUG
// Option 1: Message box to attach debugger
System.Windows.Forms.MessageBox.Show(
    $"Attach debugger to process {{System.Diagnostics.Process.GetCurrentProcess().Id}}\n" +
    "Custom Action: {}\n" +
    "Execution: {}\n" +
    "Click OK when ready.",
    "Debug Custom Action",
    System.Windows.Forms.MessageBoxButtons.OK);

// Option 2: Automatic debugger break (requires debugger already attached)
// System.Diagnostics.Debugger.Break();

// Option 3: Wait for debugger to attach
// while (!System.Diagnostics.Debugger.IsAttached)
//     System.Threading.Thread.Sleep(100);
// System.Diagnostics.Debugger.Break();
#endif

// Logging helper
session.Log("=== Custom Action {} Starting ===");
session.Log($"Execution context: {}");
session.Log($"CustomActionData: {{session.CustomActionData}}");

// Your custom action code here...

session.Log("=== Custom Action {} Completed ===");
"#,
            ca.id,
            ca.id,
            ca.execute,
            ca.id,
            ca.execute,
            ca.id
        )
    }

    fn generate_cpp_debug(&self, ca: &CustomActionDef) -> String {
        format!(
            r#"// Debug helper for custom action: {}
// Add this at the start of your custom action function

#ifdef _DEBUG
// Option 1: Message box to attach debugger
{{
    WCHAR szMsg[256];
    swprintf_s(szMsg, L"Attach debugger to PID: %d\nCustom Action: {}\nClick OK when ready.",
        GetCurrentProcessId());
    MessageBoxW(NULL, szMsg, L"Debug Custom Action", MB_OK);
}}

// Option 2: Debug break (requires debugger attached)
// __debugbreak();

// Option 3: Wait for debugger
// while (!IsDebuggerPresent()) Sleep(100);
// __debugbreak();
#endif

// Logging helper
WcaLog(LOGMSG_STANDARD, "=== Custom Action {} Starting ===");
WcaLog(LOGMSG_STANDARD, "Execution context: {}");

LPWSTR pwzData = NULL;
hr = WcaGetProperty(L"CustomActionData", &pwzData);
if (SUCCEEDED(hr) && pwzData)
{{
    WcaLog(LOGMSG_STANDARD, "CustomActionData: %ls", pwzData);
    ReleaseStr(pwzData);
}}

// Your custom action code here...

WcaLog(LOGMSG_STANDARD, "=== Custom Action {} Completed ===");
"#,
            ca.id, ca.id, ca.id, ca.execute, ca.id
        )
    }

    fn generate_vbscript_debug(&self, ca: &CustomActionDef) -> String {
        format!(
            r#"' Debug helper for custom action: {}
' Add this at the start of your VBScript

' Option 1: Message box to attach debugger
MsgBox "Attach debugger now." & vbCrLf & _
       "Custom Action: {}" & vbCrLf & _
       "Execution: {}" & vbCrLf & _
       "Click OK when ready.", vbOKOnly, "Debug Custom Action"

' Option 2: Script debugger break (requires script debugger)
' Stop

' Logging helper
Sub LogMessage(msg)
    Dim record
    Set record = Session.Installer.CreateRecord(1)
    record.StringData(0) = "[1]"
    record.StringData(1) = msg
    Session.Message &H04000000, record
End Sub

LogMessage "=== Custom Action {} Starting ==="
LogMessage "Execution context: {}"

' For deferred actions, get CustomActionData
If Session.Property("CustomActionData") <> "" Then
    LogMessage "CustomActionData: " & Session.Property("CustomActionData")
End If

' Your custom action code here...

LogMessage "=== Custom Action {} Completed ==="
"#,
            ca.id, ca.id, ca.execute, ca.id, ca.execute, ca.id
        )
    }

    /// Generate debugging guide for a custom action
    pub fn generate_debug_guide(&self, ca: &CustomActionDef) -> String {
        let mut guide = String::new();

        guide.push_str(&format!("# Debugging Guide: {}\n\n", ca.id));

        guide.push_str("## Custom Action Details\n\n");
        guide.push_str(&format!("- **Type**: {:?}\n", ca.action_type));
        guide.push_str(&format!("- **Execution**: {:?}\n", ca.execution));
        guide.push_str(&format!("- **Impersonate**: {}\n", if ca.impersonate { "Yes" } else { "No (runs as SYSTEM)" }));
        guide.push_str(&format!("- **Return**: {}\n", ca.return_type));

        if let Some(ref binary) = ca.binary_key {
            guide.push_str(&format!("- **Binary**: {}\n", binary));
        }
        if let Some(ref entry) = ca.dll_entry {
            guide.push_str(&format!("- **DLL Entry**: {}\n", entry));
        }
        if let Some(ref cmd) = ca.exe_command {
            guide.push_str(&format!("- **Command**: {}\n", cmd));
        }

        guide.push_str("\n## Debugging Steps\n\n");

        match ca.execution {
            ExecutionContext::Immediate => {
                guide.push_str("### Immediate Custom Action\n\n");
                guide.push_str("This action runs in the client process during the UI sequence.\n\n");
                guide.push_str("1. **Attach debugger**: Attach to `msiexec.exe` (the client process)\n");
                guide.push_str("2. **Set breakpoint**: Set a breakpoint in your custom action code\n");
                guide.push_str("3. **Run installer**: Start the installation with `msiexec /i package.msi`\n");
                guide.push_str("4. **Verbose logging**: Use `msiexec /i package.msi /l*v install.log`\n\n");
            }
            ExecutionContext::Deferred | ExecutionContext::DeferredNoImpersonate => {
                guide.push_str("### Deferred Custom Action\n\n");
                guide.push_str("This action runs during the execute sequence");
                if !ca.impersonate {
                    guide.push_str(" **as NT AUTHORITY\\SYSTEM**");
                }
                guide.push_str(".\n\n");

                guide.push_str("**Challenge**: Deferred actions run in a separate server process.\n\n");

                guide.push_str("1. **Enable verbose logging**:\n");
                guide.push_str("   ```\n");
                guide.push_str("   msiexec /i package.msi /l*v install.log\n");
                guide.push_str("   ```\n\n");

                guide.push_str("2. **Add debug message box** (see generated code below)\n\n");

                guide.push_str("3. **Attach to correct process**:\n");
                if ca.impersonate {
                    guide.push_str("   - Attach to the `msiexec.exe` process running as the user\n");
                } else {
                    guide.push_str("   - Attach to the `msiexec.exe` process running as SYSTEM\n");
                    guide.push_str("   - You may need to debug as Administrator\n");
                }
                guide.push_str("\n");

                guide.push_str("4. **Passing data**: Use CustomActionData property:\n");
                guide.push_str("   ```xml\n");
                guide.push_str(&format!("   <SetProperty Id=\"{}\" Before=\"{}\"\n", ca.id, ca.id));
                guide.push_str("                 Value=\"[INSTALLFOLDER];[ProductVersion]\" Sequence=\"execute\" />\n");
                guide.push_str("   ```\n\n");
            }
            ExecutionContext::Rollback => {
                guide.push_str("### Rollback Custom Action\n\n");
                guide.push_str("This action only runs if the installation fails and rolls back.\n\n");
                guide.push_str("1. **Trigger rollback**: Cause the installation to fail after this action\n");
                guide.push_str("2. **Check logs**: Rollback actions are logged with `Rollback:` prefix\n\n");
            }
            ExecutionContext::Commit => {
                guide.push_str("### Commit Custom Action\n\n");
                guide.push_str("This action runs after successful installation commit.\n\n");
                guide.push_str("1. **Note**: Commit actions run after InstallFinalize\n");
                guide.push_str("2. **Errors**: Failures don't cause rollback at this point\n\n");
            }
        }

        guide.push_str("## Verbose Logging Tips\n\n");
        guide.push_str("```\n");
        guide.push_str("msiexec /i package.msi /l*v install.log\n");
        guide.push_str("```\n\n");
        guide.push_str("In the log, search for:\n");
        guide.push_str(&format!("- `Action start.*{}` - When your action starts\n", ca.id));
        guide.push_str("- `return value 3` - Custom action failures (search upward from here)\n");
        guide.push_str("- `CustomActionData` - Data passed to deferred actions\n\n");

        if matches!(
            ca.execution,
            ExecutionContext::Deferred | ExecutionContext::DeferredNoImpersonate
        ) {
            guide.push_str("## Force Log Flush\n\n");
            guide.push_str("If the custom action crashes, logs may not be flushed. Use:\n");
            guide.push_str("```\n");
            guide.push_str("msiexec /i package.msi /l*v! install.log\n");
            guide.push_str("```\n");
            guide.push_str("The `!` forces immediate flush (slower but catches crashes).\n\n");
        }

        guide
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyze_dll_custom_action() {
        let xml = r#"
        <Wix xmlns="http://wixtoolset.org/schemas/v4/wxs">
          <CustomAction Id="MyAction" BinaryKey="MyBinary" DllEntry="MyFunction" Execute="deferred" Impersonate="no" />
        </Wix>
        "#;

        let analyzer = CustomActionAnalyzer::new();
        let result = analyzer.analyze(xml);

        assert_eq!(result.custom_actions.len(), 1);
        assert_eq!(result.custom_actions[0].id, "MyAction");
        assert!(matches!(
            result.custom_actions[0].action_type,
            CustomActionType::Dll
        ));
        assert!(matches!(
            result.custom_actions[0].execution,
            ExecutionContext::DeferredNoImpersonate
        ));
    }

    #[test]
    fn test_analyze_exe_custom_action() {
        let xml = r#"
        <Wix xmlns="http://wixtoolset.org/schemas/v4/wxs">
          <CustomAction Id="RunCmd" FileKey="MyExe" ExeCommand="/install" Execute="immediate" />
        </Wix>
        "#;

        let analyzer = CustomActionAnalyzer::new();
        let result = analyzer.analyze(xml);

        assert_eq!(result.custom_actions.len(), 1);
        assert!(matches!(
            result.custom_actions[0].action_type,
            CustomActionType::Exe
        ));
    }

    #[test]
    fn test_security_warning_elevated() {
        let xml = r#"
        <Wix xmlns="http://wixtoolset.org/schemas/v4/wxs">
          <CustomAction Id="ElevatedAction" BinaryKey="MyBinary" DllEntry="Func" Execute="deferred" Impersonate="no" />
        </Wix>
        "#;

        let analyzer = CustomActionAnalyzer::new();
        let result = analyzer.analyze(xml);

        assert!(result.issues.iter().any(|i| i.category == "Security"));
    }

    #[test]
    fn test_unscheduled_warning() {
        let xml = r#"
        <Wix xmlns="http://wixtoolset.org/schemas/v4/wxs">
          <CustomAction Id="UnusedAction" BinaryKey="MyBinary" DllEntry="Func" />
        </Wix>
        "#;

        let analyzer = CustomActionAnalyzer::new();
        let result = analyzer.analyze(xml);

        assert!(result
            .issues
            .iter()
            .any(|i| i.message.contains("not scheduled")));
    }

    #[test]
    fn test_generate_csharp_debug() {
        let ca = CustomActionDef {
            id: "TestAction".to_string(),
            action_type: CustomActionType::Dll,
            execution: ExecutionContext::Deferred,
            binary_key: Some("TestBinary".to_string()),
            dll_entry: Some("TestEntry".to_string()),
            exe_command: None,
            script: None,
            property: None,
            value: None,
            directory: None,
            file_key: None,
            impersonate: false,
            execute: "deferred".to_string(),
            return_type: "check".to_string(),
            sequence_tables: Vec::new(),
            line: Some(10),
            issues: Vec::new(),
        };

        let analyzer = CustomActionAnalyzer::new();
        let code = analyzer.generate_debug_helper(&ca, "csharp");

        assert!(code.contains("TestAction"));
        assert!(code.contains("MessageBox"));
        assert!(code.contains("Debugger.Break"));
    }

    #[test]
    fn test_analyze_property_set() {
        let xml = r#"
        <Wix xmlns="http://wixtoolset.org/schemas/v4/wxs">
          <CustomAction Id="SetMyProp" Property="MYPROP" Value="[INSTALLFOLDER]" />
        </Wix>
        "#;

        let analyzer = CustomActionAnalyzer::new();
        let result = analyzer.analyze(xml);

        assert_eq!(result.custom_actions.len(), 1);
        assert!(matches!(
            result.custom_actions[0].action_type,
            CustomActionType::PropertySet
        ));
    }

    #[test]
    fn test_summary_counts() {
        let xml = r#"
        <Wix xmlns="http://wixtoolset.org/schemas/v4/wxs">
          <CustomAction Id="CA1" BinaryKey="B" DllEntry="F" Execute="immediate" />
          <CustomAction Id="CA2" BinaryKey="B" DllEntry="F" Execute="deferred" Impersonate="yes" />
          <CustomAction Id="CA3" BinaryKey="B" DllEntry="F" Execute="deferred" Impersonate="no" />
        </Wix>
        "#;

        let analyzer = CustomActionAnalyzer::new();
        let result = analyzer.analyze(xml);

        assert_eq!(result.summary.total_custom_actions, 3);
        assert_eq!(result.summary.immediate_count, 1);
        assert_eq!(result.summary.deferred_count, 2);
        assert_eq!(result.summary.elevated_count, 1);
    }
}

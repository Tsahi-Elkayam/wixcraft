//! wix-security - MSI security scanner for privilege escalation vulnerabilities
//!
//! Detects common security issues in WiX source files and compiled MSI packages
//! that could lead to privilege escalation attacks.
//!
//! # Vulnerability Categories
//!
//! - **Elevated Custom Actions**: CAs running as SYSTEM without impersonation
//! - **Temp Folder Risks**: Binaries extracted to writable locations
//! - **DLL Hijacking**: Unsafe DLL search paths
//! - **Repair Vulnerabilities**: Elevated operations triggered by standard users
//! - **Path Traversal**: Unsafe path handling in custom actions

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Security vulnerability finding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityFinding {
    /// Unique identifier for the vulnerability type
    pub id: String,
    /// Severity level
    pub severity: Severity,
    /// Short title
    pub title: String,
    /// Detailed description
    pub description: String,
    /// Location in source (file:line or table:row)
    pub location: String,
    /// Affected element or component
    pub affected: String,
    /// Related CVE if applicable
    pub cve: Option<String>,
    /// How to fix
    pub remediation: String,
    /// CVSS-like score (0-10)
    pub score: f32,
}

/// Severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::Critical => write!(f, "CRITICAL"),
            Severity::High => write!(f, "HIGH"),
            Severity::Medium => write!(f, "MEDIUM"),
            Severity::Low => write!(f, "LOW"),
            Severity::Info => write!(f, "INFO"),
        }
    }
}

/// Scan results summary
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ScanResult {
    pub findings: Vec<SecurityFinding>,
    pub critical_count: usize,
    pub high_count: usize,
    pub medium_count: usize,
    pub low_count: usize,
    pub info_count: usize,
    pub files_scanned: usize,
}

impl ScanResult {
    pub fn add_finding(&mut self, finding: SecurityFinding) {
        match finding.severity {
            Severity::Critical => self.critical_count += 1,
            Severity::High => self.high_count += 1,
            Severity::Medium => self.medium_count += 1,
            Severity::Low => self.low_count += 1,
            Severity::Info => self.info_count += 1,
        }
        self.findings.push(finding);
    }

    pub fn total_findings(&self) -> usize {
        self.findings.len()
    }

    pub fn has_critical(&self) -> bool {
        self.critical_count > 0
    }

    pub fn has_high_or_critical(&self) -> bool {
        self.critical_count > 0 || self.high_count > 0
    }

    pub fn max_score(&self) -> f32 {
        self.findings.iter().map(|f| f.score).fold(0.0, f32::max)
    }
}

/// Security scanner for WiX/MSI files
pub struct SecurityScanner {
    rules: Vec<SecurityRule>,
}

struct SecurityRule {
    id: &'static str,
    title: &'static str,
    check: Box<dyn Fn(&str, &str) -> Vec<SecurityFinding> + Send + Sync>,
}

impl Default for SecurityScanner {
    fn default() -> Self {
        Self::new()
    }
}

impl SecurityScanner {
    pub fn new() -> Self {
        let rules = vec![
            // Rule 1: Elevated custom actions without impersonation
            SecurityRule {
                id: "SEC001",
                title: "Elevated Custom Action",
                check: Box::new(check_elevated_custom_actions),
            },
            // Rule 2: Custom actions in deferred context
            SecurityRule {
                id: "SEC002",
                title: "Deferred Elevated Custom Action",
                check: Box::new(check_deferred_elevated),
            },
            // Rule 3: Executable custom actions
            SecurityRule {
                id: "SEC003",
                title: "Executable Custom Action",
                check: Box::new(check_executable_custom_actions),
            },
            // Rule 4: Script custom actions
            SecurityRule {
                id: "SEC004",
                title: "Script Custom Action",
                check: Box::new(check_script_custom_actions),
            },
            // Rule 5: Temp folder usage
            SecurityRule {
                id: "SEC005",
                title: "Temp Folder Usage",
                check: Box::new(check_temp_folder_usage),
            },
            // Rule 6: Writable install locations
            SecurityRule {
                id: "SEC006",
                title: "Writable Install Location",
                check: Box::new(check_writable_locations),
            },
            // Rule 7: Command execution
            SecurityRule {
                id: "SEC007",
                title: "Command Execution",
                check: Box::new(check_command_execution),
            },
            // Rule 8: Service installation as LocalSystem
            SecurityRule {
                id: "SEC008",
                title: "Service as LocalSystem",
                check: Box::new(check_service_accounts),
            },
            // Rule 9: Registry modification in HKLM
            SecurityRule {
                id: "SEC009",
                title: "Privileged Registry Access",
                check: Box::new(check_registry_access),
            },
            // Rule 10: Binary table extraction
            SecurityRule {
                id: "SEC010",
                title: "Binary Extraction Risk",
                check: Box::new(check_binary_extraction),
            },
            // Rule 11: SetProperty to sensitive values
            SecurityRule {
                id: "SEC011",
                title: "Sensitive Property Modification",
                check: Box::new(check_sensitive_properties),
            },
            // Rule 12: Unquoted paths
            SecurityRule {
                id: "SEC012",
                title: "Unquoted Path",
                check: Box::new(check_unquoted_paths),
            },
        ];

        Self { rules }
    }

    /// Scan WiX source file content
    pub fn scan_source(&self, content: &str, filename: &str) -> ScanResult {
        let mut result = ScanResult::default();
        result.files_scanned = 1;

        for rule in &self.rules {
            let findings = (rule.check)(content, filename);
            for finding in findings {
                result.add_finding(finding);
            }
        }

        // Sort by severity
        result.findings.sort_by(|a, b| {
            let severity_order = |s: &Severity| match s {
                Severity::Critical => 0,
                Severity::High => 1,
                Severity::Medium => 2,
                Severity::Low => 3,
                Severity::Info => 4,
            };
            severity_order(&a.severity).cmp(&severity_order(&b.severity))
        });

        result
    }

    /// Scan multiple files
    pub fn scan_files(&self, files: &[&Path]) -> ScanResult {
        let mut combined = ScanResult::default();

        for file in files {
            if let Ok(content) = std::fs::read_to_string(file) {
                let filename = file.to_string_lossy();
                let result = self.scan_source(&content, &filename);

                combined.files_scanned += 1;
                combined.critical_count += result.critical_count;
                combined.high_count += result.high_count;
                combined.medium_count += result.medium_count;
                combined.low_count += result.low_count;
                combined.info_count += result.info_count;
                combined.findings.extend(result.findings);
            }
        }

        combined
    }
}

// Security check implementations

fn check_elevated_custom_actions(content: &str, filename: &str) -> Vec<SecurityFinding> {
    let mut findings = Vec::new();

    // Parse XML to find CustomAction elements
    if let Ok(doc) = roxmltree::Document::parse(content) {
        for node in doc.descendants() {
            if node.tag_name().name() == "CustomAction" {
                let id = node.attribute("Id").unwrap_or("unknown");
                let impersonate = node.attribute("Impersonate");
                let execute = node.attribute("Execute");

                // Check for deferred actions without impersonation (run as SYSTEM)
                if execute == Some("deferred") && impersonate != Some("yes") {
                    let has_dll = node.attribute("DllEntry").is_some()
                        || node.attribute("BinaryRef").is_some();
                    let has_exe = node.attribute("ExeCommand").is_some()
                        || node.attribute("FileRef").is_some();

                    if has_dll || has_exe {
                        let line = get_line_number(content, node);
                        findings.push(SecurityFinding {
                            id: "SEC001".to_string(),
                            severity: Severity::High,
                            title: "Elevated Custom Action Without Impersonation".to_string(),
                            description: format!(
                                "Custom action '{}' runs in deferred context without Impersonate=\"yes\", \
                                meaning it executes as NT AUTHORITY\\SYSTEM. This can be exploited \
                                for privilege escalation via MSI repair attacks.",
                                id
                            ),
                            location: format!("{}:{}", filename, line),
                            affected: id.to_string(),
                            cve: Some("CVE-2024-38014".to_string()),
                            remediation: "Add Impersonate=\"yes\" to run as the installing user, \
                                or ensure the custom action cannot be exploited during repair.".to_string(),
                            score: 7.8,
                        });
                    }
                }
            }
        }
    }

    findings
}

fn check_deferred_elevated(content: &str, filename: &str) -> Vec<SecurityFinding> {
    let mut findings = Vec::new();

    if let Ok(doc) = roxmltree::Document::parse(content) {
        for node in doc.descendants() {
            if node.tag_name().name() == "CustomAction" {
                let id = node.attribute("Id").unwrap_or("unknown");
                let execute = node.attribute("Execute");
                let impersonate = node.attribute("Impersonate");

                // Deferred + no impersonate + between InstallInitialize and InstallFinalize = elevated
                if execute == Some("deferred") && impersonate == Some("no") {
                    let line = get_line_number(content, node);
                    findings.push(SecurityFinding {
                        id: "SEC002".to_string(),
                        severity: Severity::Critical,
                        title: "Explicitly Elevated Custom Action".to_string(),
                        description: format!(
                            "Custom action '{}' has Execute=\"deferred\" with Impersonate=\"no\", \
                            explicitly running as SYSTEM. This is a high-risk pattern for \
                            privilege escalation attacks.",
                            id
                        ),
                        location: format!("{}:{}", filename, line),
                        affected: id.to_string(),
                        cve: Some("CVE-2024-38014".to_string()),
                        remediation: "Remove Impersonate=\"no\" or change to Impersonate=\"yes\". \
                            Review if SYSTEM privileges are actually required.".to_string(),
                        score: 9.0,
                    });
                }
            }
        }
    }

    findings
}

fn check_executable_custom_actions(content: &str, filename: &str) -> Vec<SecurityFinding> {
    let mut findings = Vec::new();

    if let Ok(doc) = roxmltree::Document::parse(content) {
        for node in doc.descendants() {
            if node.tag_name().name() == "CustomAction" {
                let id = node.attribute("Id").unwrap_or("unknown");
                let exe_command = node.attribute("ExeCommand");
                let file_ref = node.attribute("FileRef");

                if exe_command.is_some() || file_ref.is_some() {
                    let line = get_line_number(content, node);
                    let cmd = exe_command.or(file_ref).unwrap_or("");

                    // Check for dangerous executables
                    let dangerous = ["cmd.exe", "powershell", "cscript", "wscript", "mshta", "net.exe", "net1.exe"];
                    let is_dangerous = dangerous.iter().any(|d| cmd.to_lowercase().contains(d));

                    if is_dangerous {
                        findings.push(SecurityFinding {
                            id: "SEC003".to_string(),
                            severity: Severity::High,
                            title: "Dangerous Executable in Custom Action".to_string(),
                            description: format!(
                                "Custom action '{}' executes a potentially dangerous command: '{}'. \
                                Command interpreters can be exploited for privilege escalation.",
                                id, cmd
                            ),
                            location: format!("{}:{}", filename, line),
                            affected: id.to_string(),
                            cve: None,
                            remediation: "Avoid using command interpreters in custom actions. \
                                Use a compiled DLL with specific functionality instead.".to_string(),
                            score: 7.5,
                        });
                    }
                }
            }
        }
    }

    findings
}

fn check_script_custom_actions(content: &str, filename: &str) -> Vec<SecurityFinding> {
    let mut findings = Vec::new();

    if let Ok(doc) = roxmltree::Document::parse(content) {
        for node in doc.descendants() {
            if node.tag_name().name() == "CustomAction" {
                let id = node.attribute("Id").unwrap_or("unknown");
                let script = node.attribute("Script");
                let vbscript = node.attribute("VBScriptCall");
                let jscript = node.attribute("JScriptCall");

                if script.is_some() || vbscript.is_some() || jscript.is_some() {
                    let line = get_line_number(content, node);
                    let script_type = if vbscript.is_some() { "VBScript" }
                        else if jscript.is_some() { "JScript" }
                        else { script.unwrap_or("Script") };

                    findings.push(SecurityFinding {
                        id: "SEC004".to_string(),
                        severity: Severity::Medium,
                        title: "Script-Based Custom Action".to_string(),
                        description: format!(
                            "Custom action '{}' uses {} which can be modified by attackers \
                            if the MSI is not properly signed or the script is stored insecurely.",
                            id, script_type
                        ),
                        location: format!("{}:{}", filename, line),
                        affected: id.to_string(),
                        cve: None,
                        remediation: "Prefer compiled DLLs over scripts. If scripts are necessary, \
                            ensure the MSI is digitally signed.".to_string(),
                        score: 5.5,
                    });
                }
            }
        }
    }

    findings
}

fn check_temp_folder_usage(content: &str, filename: &str) -> Vec<SecurityFinding> {
    let mut findings = Vec::new();

    // Check for TempFolder references
    let temp_patterns = [
        r"\[TempFolder\]",
        r"\[%TEMP%\]",
        r"\[%TMP%\]",
        r"TempFolder",
    ];

    for (line_num, line) in content.lines().enumerate() {
        for pattern in &temp_patterns {
            if let Ok(re) = Regex::new(pattern) {
                if re.is_match(line) {
                    // Check if it's in a custom action or file operation context
                    if line.contains("CustomAction") || line.contains("Directory") || line.contains("Property") {
                        findings.push(SecurityFinding {
                            id: "SEC005".to_string(),
                            severity: Severity::Medium,
                            title: "Temp Folder Used for Extraction".to_string(),
                            description: "Files extracted to the temp folder can be replaced by attackers \
                                before execution. This is a common privilege escalation vector.".to_string(),
                            location: format!("{}:{}", filename, line_num + 1),
                            affected: line.trim().to_string(),
                            cve: Some("CVE-2023-26078".to_string()),
                            remediation: "Extract files to a secure location with restricted permissions, \
                                not the user's temp folder.".to_string(),
                            score: 6.5,
                        });
                        break;
                    }
                }
            }
        }
    }

    findings
}

fn check_writable_locations(content: &str, filename: &str) -> Vec<SecurityFinding> {
    let mut findings = Vec::new();

    // Writable locations that shouldn't contain executables
    let writable_patterns = [
        (r"\[AppDataFolder\]", "AppData"),
        (r"\[LocalAppDataFolder\]", "LocalAppData"),
        (r"\[CommonAppDataFolder\]", "ProgramData"),
        (r"C:\\Users\\Public", "Public folder"),
    ];

    if let Ok(doc) = roxmltree::Document::parse(content) {
        for node in doc.descendants() {
            // Check File elements
            if node.tag_name().name() == "File" {
                let source = node.attribute("Source").unwrap_or("");
                if source.ends_with(".exe") || source.ends_with(".dll") {
                    // Check parent directory
                    if let Some(parent) = node.parent() {
                        if parent.tag_name().name() == "Component" {
                            if let Some(dir_parent) = parent.parent() {
                                let dir_id = dir_parent.attribute("Id").unwrap_or("");

                                for (pattern, name) in &writable_patterns {
                                    if dir_id.contains(pattern.trim_matches(|c| c == '[' || c == ']')) {
                                        let line = get_line_number(content, node);
                                        findings.push(SecurityFinding {
                                            id: "SEC006".to_string(),
                                            severity: Severity::Medium,
                                            title: "Executable in Writable Location".to_string(),
                                            description: format!(
                                                "Executable '{}' is installed to {} which is writable by users. \
                                                This could allow DLL hijacking or executable replacement.",
                                                source, name
                                            ),
                                            location: format!("{}:{}", filename, line),
                                            affected: source.to_string(),
                                            cve: None,
                                            remediation: "Install executables to Program Files with proper ACLs.".to_string(),
                                            score: 5.0,
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    findings
}

fn check_command_execution(content: &str, filename: &str) -> Vec<SecurityFinding> {
    let mut findings = Vec::new();

    // Patterns indicating command execution
    let cmd_patterns = [
        (r"cmd\s*/c", "cmd /c command execution"),
        (r"cmd\.exe", "cmd.exe invocation"),
        (r"powershell.*-[eE]", "PowerShell encoded command"),
        (r"powershell.*-[cC]ommand", "PowerShell command execution"),
        (r"msiexec.*REINSTALL", "Repair trigger"),
        (r"net\s+user", "User account manipulation"),
        (r"net\s+localgroup", "Group membership manipulation"),
        (r"sc\s+(create|config)", "Service manipulation"),
        (r"reg\s+(add|delete)", "Registry manipulation via reg.exe"),
    ];

    for (line_num, line) in content.lines().enumerate() {
        for (pattern, desc) in &cmd_patterns {
            if let Ok(re) = Regex::new(&format!("(?i){}", pattern)) {
                if re.is_match(line) {
                    findings.push(SecurityFinding {
                        id: "SEC007".to_string(),
                        severity: Severity::High,
                        title: "Command Line Execution Detected".to_string(),
                        description: format!(
                            "Detected {}: This pattern can be exploited if the custom action \
                            runs elevated.",
                            desc
                        ),
                        location: format!("{}:{}", filename, line_num + 1),
                        affected: line.trim().to_string(),
                        cve: None,
                        remediation: "Avoid shell commands in custom actions. Use Windows API calls \
                            in compiled code instead.".to_string(),
                        score: 7.0,
                    });
                    break;
                }
            }
        }
    }

    findings
}

fn check_service_accounts(content: &str, filename: &str) -> Vec<SecurityFinding> {
    let mut findings = Vec::new();

    if let Ok(doc) = roxmltree::Document::parse(content) {
        for node in doc.descendants() {
            if node.tag_name().name() == "ServiceInstall" {
                let name = node.attribute("Name").unwrap_or("unknown");
                let account = node.attribute("Account");

                // LocalSystem is the default if not specified
                let is_localsystem = account.is_none()
                    || account == Some("LocalSystem")
                    || account == Some("NT AUTHORITY\\SYSTEM");

                if is_localsystem {
                    let line = get_line_number(content, node);
                    findings.push(SecurityFinding {
                        id: "SEC008".to_string(),
                        severity: Severity::Medium,
                        title: "Service Running as LocalSystem".to_string(),
                        description: format!(
                            "Service '{}' runs as LocalSystem (SYSTEM), which has full system privileges. \
                            If the service is compromised, attackers gain complete control.",
                            name
                        ),
                        location: format!("{}:{}", filename, line),
                        affected: name.to_string(),
                        cve: None,
                        remediation: "Use a dedicated service account with minimum required privileges, \
                            such as LocalService or NetworkService.".to_string(),
                        score: 5.5,
                    });
                }
            }
        }
    }

    findings
}

fn check_registry_access(content: &str, filename: &str) -> Vec<SecurityFinding> {
    let mut findings = Vec::new();

    if let Ok(doc) = roxmltree::Document::parse(content) {
        for node in doc.descendants() {
            if node.tag_name().name() == "RegistryKey" || node.tag_name().name() == "RegistryValue" {
                let root = node.attribute("Root").unwrap_or("");
                let key = node.attribute("Key").unwrap_or("");

                // Check for sensitive registry locations
                let sensitive_keys = [
                    ("HKLM\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Run", "Auto-run"),
                    ("HKLM\\SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\\Image File Execution Options", "IFEO hijacking"),
                    ("HKLM\\SYSTEM\\CurrentControlSet\\Services", "Service configuration"),
                    ("HKLM\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Policies", "System policies"),
                ];

                if root == "HKLM" {
                    for (sensitive_key, desc) in &sensitive_keys {
                        if key.to_lowercase().contains(&sensitive_key.to_lowercase().replace("HKLM\\", "")) {
                            let line = get_line_number(content, node);
                            findings.push(SecurityFinding {
                                id: "SEC009".to_string(),
                                severity: Severity::Medium,
                                title: "Sensitive Registry Modification".to_string(),
                                description: format!(
                                    "Modifying {} registry key: {}. This location is commonly targeted \
                                    for persistence and privilege escalation.",
                                    desc, key
                                ),
                                location: format!("{}:{}", filename, line),
                                affected: format!("{}\\{}", root, key),
                                cve: None,
                                remediation: "Ensure registry modifications are necessary and properly secured.".to_string(),
                                score: 5.0,
                            });
                        }
                    }
                }
            }
        }
    }

    findings
}

fn check_binary_extraction(content: &str, filename: &str) -> Vec<SecurityFinding> {
    let mut findings = Vec::new();

    if let Ok(doc) = roxmltree::Document::parse(content) {
        for node in doc.descendants() {
            if node.tag_name().name() == "Binary" {
                let id = node.attribute("Id").unwrap_or("unknown");
                let source = node.attribute("SourceFile")
                    .or_else(|| node.attribute("Source"))
                    .unwrap_or("");

                // Binaries in the Binary table are extracted at runtime
                if source.ends_with(".exe") || source.ends_with(".dll") {
                    let line = get_line_number(content, node);
                    findings.push(SecurityFinding {
                        id: "SEC010".to_string(),
                        severity: Severity::Low,
                        title: "Binary Table Extraction".to_string(),
                        description: format!(
                            "Binary '{}' ({}) is stored in the Binary table and extracted at runtime. \
                            The extraction location should be verified as secure.",
                            id, source
                        ),
                        location: format!("{}:{}", filename, line),
                        affected: id.to_string(),
                        cve: None,
                        remediation: "Ensure binaries are extracted to secure locations and verify \
                            integrity before execution.".to_string(),
                        score: 3.5,
                    });
                }
            }
        }
    }

    findings
}

fn check_sensitive_properties(content: &str, filename: &str) -> Vec<SecurityFinding> {
    let mut findings = Vec::new();

    // Sensitive properties that could be exploited
    let sensitive_props = [
        ("ALLUSERS", "Installation scope"),
        ("MSIINSTALLPERUSER", "Per-user vs per-machine"),
        ("AdminToolsFolder", "Admin tools location"),
        ("SystemFolder", "System32 location"),
        ("WindowsFolder", "Windows location"),
    ];

    if let Ok(doc) = roxmltree::Document::parse(content) {
        for node in doc.descendants() {
            if node.tag_name().name() == "SetProperty" || node.tag_name().name() == "Property" {
                let id = node.attribute("Id").unwrap_or("");
                let action = node.attribute("Action").unwrap_or("");

                for (prop, desc) in &sensitive_props {
                    if id == *prop || action == *prop {
                        let line = get_line_number(content, node);
                        findings.push(SecurityFinding {
                            id: "SEC011".to_string(),
                            severity: Severity::Info,
                            title: "Sensitive Property Modification".to_string(),
                            description: format!(
                                "Property '{}' ({}) can affect installation security context.",
                                prop, desc
                            ),
                            location: format!("{}:{}", filename, line),
                            affected: id.to_string(),
                            cve: None,
                            remediation: "Review property usage to ensure it cannot be exploited.".to_string(),
                            score: 2.0,
                        });
                    }
                }
            }
        }
    }

    findings
}

fn check_unquoted_paths(content: &str, filename: &str) -> Vec<SecurityFinding> {
    let mut findings = Vec::new();

    // Look for paths with spaces that aren't quoted
    let path_pattern = Regex::new(r#"(?i)(ExeCommand|Value|Target|Source)\s*=\s*"([^"]*\\[^"]*\s[^"]*\.exe)"#).unwrap();

    for (line_num, line) in content.lines().enumerate() {
        if let Some(caps) = path_pattern.captures(line) {
            let attr = caps.get(1).map(|m| m.as_str()).unwrap_or("");
            let path = caps.get(2).map(|m| m.as_str()).unwrap_or("");

            // Check if the path has spaces and isn't properly quoted within the value
            if path.contains(' ') && !path.starts_with('"') && !path.starts_with('\'') {
                findings.push(SecurityFinding {
                    id: "SEC012".to_string(),
                    severity: Severity::Medium,
                    title: "Unquoted Path with Spaces".to_string(),
                    description: format!(
                        "Path in {} attribute contains spaces but may not be properly quoted: '{}'. \
                        Unquoted paths can lead to arbitrary code execution.",
                        attr, path
                    ),
                    location: format!("{}:{}", filename, line_num + 1),
                    affected: path.to_string(),
                    cve: None,
                    remediation: "Ensure paths with spaces are properly quoted.".to_string(),
                    score: 5.5,
                });
            }
        }
    }

    findings
}

// Helper function to get approximate line number
fn get_line_number(content: &str, node: roxmltree::Node) -> usize {
    let pos = node.range().start;
    content[..pos].matches('\n').count() + 1
}

/// Generate SARIF output for CI/CD integration
pub fn to_sarif(result: &ScanResult, tool_name: &str) -> serde_json::Value {
    let results: Vec<serde_json::Value> = result.findings.iter().map(|f| {
        serde_json::json!({
            "ruleId": f.id,
            "level": match f.severity {
                Severity::Critical | Severity::High => "error",
                Severity::Medium => "warning",
                Severity::Low | Severity::Info => "note",
            },
            "message": {
                "text": f.description
            },
            "locations": [{
                "physicalLocation": {
                    "artifactLocation": {
                        "uri": f.location.split(':').next().unwrap_or(&f.location)
                    },
                    "region": {
                        "startLine": f.location.split(':').nth(1)
                            .and_then(|s| s.parse::<u32>().ok())
                            .unwrap_or(1)
                    }
                }
            }]
        })
    }).collect();

    let rules: Vec<serde_json::Value> = result.findings.iter()
        .map(|f| (&f.id, &f.title, &f.remediation))
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .map(|(id, title, help)| {
            serde_json::json!({
                "id": id,
                "name": title,
                "helpUri": format!("https://wixcraft.dev/security/{}", id),
                "help": {
                    "text": help
                }
            })
        })
        .collect();

    serde_json::json!({
        "$schema": "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json",
        "version": "2.1.0",
        "runs": [{
            "tool": {
                "driver": {
                    "name": tool_name,
                    "version": env!("CARGO_PKG_VERSION"),
                    "rules": rules
                }
            },
            "results": results
        }]
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_elevated_custom_action_detection() {
        let content = r#"
        <Wix xmlns="http://wixtoolset.org/schemas/v4/wxs">
            <CustomAction Id="CA_Install" DllEntry="Install" BinaryRef="CustomActions" Execute="deferred" />
        </Wix>
        "#;

        let scanner = SecurityScanner::new();
        let result = scanner.scan_source(content, "test.wxs");

        assert!(result.findings.iter().any(|f| f.id == "SEC001"));
    }

    #[test]
    fn test_explicit_no_impersonate_detection() {
        let content = r#"
        <Wix xmlns="http://wixtoolset.org/schemas/v4/wxs">
            <CustomAction Id="CA_Elevated" DllEntry="Run" BinaryRef="CA" Execute="deferred" Impersonate="no" />
        </Wix>
        "#;

        let scanner = SecurityScanner::new();
        let result = scanner.scan_source(content, "test.wxs");

        assert!(result.findings.iter().any(|f| f.id == "SEC002"));
        assert!(result.has_critical());
    }

    #[test]
    fn test_safe_custom_action_no_finding() {
        let content = r#"
        <Wix xmlns="http://wixtoolset.org/schemas/v4/wxs">
            <CustomAction Id="CA_Safe" DllEntry="Run" BinaryRef="CA" Execute="deferred" Impersonate="yes" />
        </Wix>
        "#;

        let scanner = SecurityScanner::new();
        let result = scanner.scan_source(content, "test.wxs");

        // Should not have SEC001 or SEC002
        assert!(!result.findings.iter().any(|f| f.id == "SEC001" || f.id == "SEC002"));
    }

    #[test]
    fn test_dangerous_command_detection() {
        let content = r#"
        <Wix xmlns="http://wixtoolset.org/schemas/v4/wxs">
            <CustomAction Id="CA_Cmd" ExeCommand="cmd.exe /c echo test" Execute="deferred" />
        </Wix>
        "#;

        let scanner = SecurityScanner::new();
        let result = scanner.scan_source(content, "test.wxs");

        assert!(result.findings.iter().any(|f| f.id == "SEC003"));
    }

    #[test]
    fn test_script_detection() {
        let content = r#"
        <Wix xmlns="http://wixtoolset.org/schemas/v4/wxs">
            <CustomAction Id="CA_Script" Script="vbscript">
                MsgBox "Hello"
            </CustomAction>
        </Wix>
        "#;

        let scanner = SecurityScanner::new();
        let result = scanner.scan_source(content, "test.wxs");

        assert!(result.findings.iter().any(|f| f.id == "SEC004"));
    }

    #[test]
    fn test_service_localsystem_detection() {
        let content = r#"
        <Wix xmlns="http://wixtoolset.org/schemas/v4/wxs">
            <ServiceInstall Name="MyService" Type="ownProcess" Start="auto" />
        </Wix>
        "#;

        let scanner = SecurityScanner::new();
        let result = scanner.scan_source(content, "test.wxs");

        assert!(result.findings.iter().any(|f| f.id == "SEC008"));
    }

    #[test]
    fn test_temp_folder_detection() {
        let content = r#"
        <Wix xmlns="http://wixtoolset.org/schemas/v4/wxs">
            <CustomAction Id="CA_Temp" Property="TEMPFILE" Value="[TempFolder]extract.exe" />
        </Wix>
        "#;

        let scanner = SecurityScanner::new();
        let result = scanner.scan_source(content, "test.wxs");

        assert!(result.findings.iter().any(|f| f.id == "SEC005"));
    }

    #[test]
    fn test_command_execution_patterns() {
        let content = r#"
        <CustomAction Id="CA_Net" Value="net user admin Password123 /add" />
        "#;

        let scanner = SecurityScanner::new();
        let result = scanner.scan_source(content, "test.wxs");

        assert!(result.findings.iter().any(|f| f.id == "SEC007"));
    }

    #[test]
    fn test_sarif_output() {
        let mut result = ScanResult::default();
        result.add_finding(SecurityFinding {
            id: "SEC001".to_string(),
            severity: Severity::High,
            title: "Test".to_string(),
            description: "Test description".to_string(),
            location: "test.wxs:10".to_string(),
            affected: "CA_Test".to_string(),
            cve: None,
            remediation: "Fix it".to_string(),
            score: 7.0,
        });

        let sarif = to_sarif(&result, "wix-security");
        assert!(sarif["runs"][0]["results"].as_array().unwrap().len() == 1);
    }

    #[test]
    fn test_severity_display() {
        assert_eq!(format!("{}", Severity::Critical), "CRITICAL");
        assert_eq!(format!("{}", Severity::High), "HIGH");
        assert_eq!(format!("{}", Severity::Medium), "MEDIUM");
    }

    #[test]
    fn test_scan_result_counts() {
        let mut result = ScanResult::default();

        result.add_finding(SecurityFinding {
            id: "TEST1".to_string(),
            severity: Severity::Critical,
            title: "Critical".to_string(),
            description: "".to_string(),
            location: "".to_string(),
            affected: "".to_string(),
            cve: None,
            remediation: "".to_string(),
            score: 9.0,
        });

        result.add_finding(SecurityFinding {
            id: "TEST2".to_string(),
            severity: Severity::High,
            title: "High".to_string(),
            description: "".to_string(),
            location: "".to_string(),
            affected: "".to_string(),
            cve: None,
            remediation: "".to_string(),
            score: 7.0,
        });

        assert_eq!(result.critical_count, 1);
        assert_eq!(result.high_count, 1);
        assert_eq!(result.total_findings(), 2);
        assert!(result.has_critical());
        assert!(result.has_high_or_critical());
        assert_eq!(result.max_score(), 9.0);
    }
}

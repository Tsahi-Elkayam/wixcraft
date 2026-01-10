//! msi-repair - MSI repair analyzer and troubleshooter
//!
//! Analyzes MSI installations for repair-related issues and provides
//! troubleshooting guidance for common repair problems.

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Severity of a repair issue
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity {
    Critical,
    Error,
    Warning,
    Info,
}

/// Category of repair issue
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum IssueCategory {
    SourceMissing,
    CacheMissing,
    CacheCorrupt,
    ComponentIssue,
    RegistryIssue,
    PermissionIssue,
    UacIssue,
    SecurityUpdate,
    CustomAction,
    Unknown,
}

/// A detected repair issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepairIssue {
    pub severity: Severity,
    pub category: IssueCategory,
    pub message: String,
    pub details: String,
    pub suggestion: String,
    pub kb_article: Option<String>,
}

/// Information about an installed product
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledProduct {
    pub product_code: String,
    pub product_name: String,
    pub version: String,
    pub install_date: Option<String>,
    pub install_source: Option<String>,
    pub local_package: Option<PathBuf>,
    pub publisher: Option<String>,
}

/// Repair analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepairAnalysis {
    pub product: InstalledProduct,
    pub issues: Vec<RepairIssue>,
    pub repair_command: String,
    pub recommendations: Vec<String>,
}

/// Log entry from MSI repair
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub line_number: usize,
    pub timestamp: Option<String>,
    pub level: String,
    pub message: String,
    pub context: Option<String>,
}

/// Result of log analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogAnalysisResult {
    pub issues: Vec<RepairIssue>,
    pub entries: Vec<LogEntry>,
    pub repair_type: String,
    pub success: bool,
    pub error_code: Option<u32>,
    pub failed_action: Option<String>,
}

/// Known repair-related error patterns
pub struct RepairPatterns {
    patterns: Vec<(Regex, IssueCategory, Severity, String, String)>,
}

impl Default for RepairPatterns {
    fn default() -> Self {
        Self::new()
    }
}

impl RepairPatterns {
    pub fn new() -> Self {
        let patterns = vec![
            // Source missing errors
            (
                Regex::new(r"(?i)the feature you are trying to use is on a network resource that is unavailable").unwrap(),
                IssueCategory::SourceMissing,
                Severity::Error,
                "Installation source is no longer available".to_string(),
                "Point to the original installation media or provide an alternate source path using SOURCELIST property".to_string(),
            ),
            (
                Regex::new(r"(?i)error 1706").unwrap(),
                IssueCategory::SourceMissing,
                Severity::Error,
                "Error 1706: No valid source could be found".to_string(),
                "Use 'msiexec /i package.msi REINSTALL=ALL REINSTALLMODE=vomus' with the original MSI".to_string(),
            ),
            // Cache issues
            (
                Regex::new(r"(?i)the cached MSI package.*is missing").unwrap(),
                IssueCategory::CacheMissing,
                Severity::Error,
                "Cached MSI package is missing from Windows Installer cache".to_string(),
                "Recache the MSI using 'msiexec /fv package.msi' with the original installation media".to_string(),
            ),
            (
                Regex::new(r"(?i)windows installer cache.*corrupt").unwrap(),
                IssueCategory::CacheCorrupt,
                Severity::Error,
                "Windows Installer cache may be corrupted".to_string(),
                "Run the Windows Installer Repair Tool or Microsoft Fixit".to_string(),
            ),
            // Component issues
            (
                Regex::new(r"(?i)error 1316.*network error|error.*specified account already exists").unwrap(),
                IssueCategory::ComponentIssue,
                Severity::Error,
                "Component registration conflict detected".to_string(),
                "Use Windows Installer CleanUp or manually clean registry entries".to_string(),
            ),
            (
                Regex::new(r"(?i)error 1309.*error reading from file").unwrap(),
                IssueCategory::SourceMissing,
                Severity::Error,
                "Cannot read from installation source".to_string(),
                "Verify the source path is accessible and files are not corrupted".to_string(),
            ),
            // Permission issues
            (
                Regex::new(r"(?i)error 1925.*you do not have sufficient privileges").unwrap(),
                IssueCategory::PermissionIssue,
                Severity::Error,
                "Insufficient privileges for repair operation".to_string(),
                "Run the repair as Administrator or adjust folder permissions".to_string(),
            ),
            (
                Regex::new(r"(?i)access denied|permission denied").unwrap(),
                IssueCategory::PermissionIssue,
                Severity::Warning,
                "Access denied during repair operation".to_string(),
                "Check file and registry permissions. Run as Administrator if needed".to_string(),
            ),
            // UAC issues (August 2024+ security updates)
            (
                Regex::new(r"(?i)User Account Control.*repair").unwrap(),
                IssueCategory::UacIssue,
                Severity::Warning,
                "UAC prompt triggered during silent repair".to_string(),
                "This may be caused by Windows security updates (KB5041585). Consider using scheduled task or explicit elevation".to_string(),
            ),
            (
                Regex::new(r"(?i)elevation required|requires elevation").unwrap(),
                IssueCategory::UacIssue,
                Severity::Warning,
                "Repair operation requires elevation".to_string(),
                "Run repair from elevated command prompt or use 'runas /user:Administrator'".to_string(),
            ),
            // Custom action failures
            (
                Regex::new(r"(?i)custom action.*returned actual error code.*1603").unwrap(),
                IssueCategory::CustomAction,
                Severity::Error,
                "Custom action failed during repair".to_string(),
                "Check the MSI log for the specific custom action that failed. May need to clean up state before repair".to_string(),
            ),
            // Return value 3
            (
                Regex::new(r"(?i)return value 3").unwrap(),
                IssueCategory::Unknown,
                Severity::Error,
                "An action returned error code 3 (failure)".to_string(),
                "Search upward from this line in the log to find the failing action".to_string(),
            ),
            // General errors
            (
                Regex::new(r"(?i)error 1603.*fatal error during installation").unwrap(),
                IssueCategory::Unknown,
                Severity::Critical,
                "Fatal error during repair/installation".to_string(),
                "Check preceding log entries for the root cause. Common causes: locked files, permission issues, custom action failures".to_string(),
            ),
        ];

        Self { patterns }
    }

    /// Check a log line against known patterns
    pub fn check_line(&self, line: &str) -> Option<RepairIssue> {
        for (pattern, category, severity, message, suggestion) in &self.patterns {
            if pattern.is_match(line) {
                return Some(RepairIssue {
                    severity: *severity,
                    category: *category,
                    message: message.clone(),
                    details: line.to_string(),
                    suggestion: suggestion.clone(),
                    kb_article: self.get_kb_article(*category),
                });
            }
        }
        None
    }

    fn get_kb_article(&self, category: IssueCategory) -> Option<String> {
        match category {
            IssueCategory::SourceMissing => Some("https://support.microsoft.com/kb/555175".to_string()),
            IssueCategory::CacheMissing => Some("https://support.microsoft.com/kb/2667628".to_string()),
            IssueCategory::UacIssue => Some("https://support.microsoft.com/kb/5041585".to_string()),
            _ => None,
        }
    }
}

/// MSI Repair Analyzer
pub struct RepairAnalyzer {
    patterns: RepairPatterns,
}

impl Default for RepairAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl RepairAnalyzer {
    pub fn new() -> Self {
        Self {
            patterns: RepairPatterns::new(),
        }
    }

    /// Analyze an MSI repair log file
    pub fn analyze_log(&self, content: &str) -> LogAnalysisResult {
        let mut issues = Vec::new();
        let mut entries = Vec::new();
        let mut success = true;
        let mut error_code: Option<u32> = None;
        let mut failed_action: Option<String> = None;
        let mut repair_type = "unknown".to_string();

        let lines: Vec<&str> = content.lines().collect();

        for (i, line) in lines.iter().enumerate() {
            // Check for repair type
            if line.contains("REINSTALL=") || line.contains("REINSTALLMODE=") {
                repair_type = "repair".to_string();
            } else if line.contains("Reconfiguring") {
                repair_type = "reconfigure".to_string();
            }

            // Check against patterns
            if let Some(issue) = self.patterns.check_line(line) {
                if matches!(issue.severity, Severity::Error | Severity::Critical) {
                    success = false;
                }
                issues.push(issue);
            }

            // Check for error codes
            let error_re = Regex::new(r"(?i)error\s*(\d+)").unwrap();
            if let Some(caps) = error_re.captures(line) {
                if let Some(code) = caps.get(1) {
                    if let Ok(c) = code.as_str().parse::<u32>() {
                        if c >= 1600 && c < 1700 {
                            error_code = Some(c);
                        }
                    }
                }
            }

            // Check for failed actions
            let action_re = Regex::new(r"Action\s+(\w+)\s+returned").unwrap();
            if line.contains("return value 3") {
                if let Some(caps) = action_re.captures(line) {
                    failed_action = caps.get(1).map(|m| m.as_str().to_string());
                }
            }

            // Create log entry for important lines
            if line.contains("error") || line.contains("Error") || line.contains("warning") ||
               line.contains("Action") || line.contains("return value") {
                entries.push(LogEntry {
                    line_number: i + 1,
                    timestamp: extract_timestamp(line),
                    level: if line.to_lowercase().contains("error") {
                        "ERROR".to_string()
                    } else if line.to_lowercase().contains("warning") {
                        "WARNING".to_string()
                    } else {
                        "INFO".to_string()
                    },
                    message: line.to_string(),
                    context: if i > 0 { Some(lines[i - 1].to_string()) } else { None },
                });
            }
        }

        // Check for success indicators
        if content.contains("Installation success or error status: 0") {
            success = true;
        } else if content.contains("Installation failed") ||
                  content.contains("Installation success or error status:") {
            success = false;
        }

        LogAnalysisResult {
            issues,
            entries,
            repair_type,
            success,
            error_code,
            failed_action,
        }
    }

    /// Generate repair troubleshooting guide
    pub fn generate_troubleshooting_guide(&self, issues: &[RepairIssue]) -> String {
        let mut guide = String::new();

        guide.push_str("# MSI Repair Troubleshooting Guide\n\n");

        if issues.is_empty() {
            guide.push_str("No specific issues detected. General repair steps:\n\n");
            guide.push_str("1. Run repair from elevated command prompt\n");
            guide.push_str("2. Use verbose logging: `msiexec /fa package.msi /l*v repair.log`\n");
            guide.push_str("3. Ensure original source media is available\n");
            guide.push_str("4. Check Windows Event Viewer for additional errors\n\n");
            return guide;
        }

        // Group issues by category
        let mut by_category: HashMap<IssueCategory, Vec<&RepairIssue>> = HashMap::new();
        for issue in issues {
            by_category.entry(issue.category).or_default().push(issue);
        }

        // Source issues
        if let Some(source_issues) = by_category.get(&IssueCategory::SourceMissing) {
            guide.push_str("## Source Availability Issues\n\n");
            guide.push_str("The original installation source is not available.\n\n");
            guide.push_str("### Solutions:\n\n");
            guide.push_str("1. **Provide alternate source:**\n");
            guide.push_str("   ```\n");
            guide.push_str("   msiexec /i {ProductCode} REINSTALL=ALL REINSTALLMODE=vomus \\\n");
            guide.push_str("           SOURCELIST=\"C:\\Path\\To\\Source\"\n");
            guide.push_str("   ```\n\n");
            guide.push_str("2. **Recache the MSI:**\n");
            guide.push_str("   ```\n");
            guide.push_str("   msiexec /fv path\\to\\original.msi\n");
            guide.push_str("   ```\n\n");
            guide.push_str("3. **Uninstall and reinstall** (if source available)\n\n");

            for issue in source_issues {
                if let Some(ref kb) = issue.kb_article {
                    guide.push_str(&format!("Reference: {}\n\n", kb));
                }
            }
        }

        // Cache issues
        if let Some(cache_issues) = by_category.get(&IssueCategory::CacheMissing) {
            guide.push_str("## Windows Installer Cache Issues\n\n");
            guide.push_str("The cached MSI package is missing or corrupted.\n\n");
            guide.push_str("### Cache Location:\n");
            guide.push_str("- `C:\\Windows\\Installer\\` (hidden, system folder)\n\n");
            guide.push_str("### Solutions:\n\n");
            guide.push_str("1. **Use Microsoft Fixit Tool:**\n");
            guide.push_str("   Download from: https://support.microsoft.com/fixit\n\n");
            guide.push_str("2. **Recache with original MSI:**\n");
            guide.push_str("   ```\n");
            guide.push_str("   msiexec /fv path\\to\\original.msi\n");
            guide.push_str("   ```\n\n");
            guide.push_str("3. **Use PatchCleaner** to identify orphaned cache entries\n\n");

            for issue in cache_issues {
                if let Some(ref kb) = issue.kb_article {
                    guide.push_str(&format!("Reference: {}\n\n", kb));
                }
            }
        }

        // UAC issues
        if let Some(uac_issues) = by_category.get(&IssueCategory::UacIssue) {
            guide.push_str("## UAC/Elevation Issues\n\n");
            guide.push_str("Recent Windows updates may cause unexpected UAC prompts during repair.\n\n");
            guide.push_str("### Affected Updates:\n");
            guide.push_str("- KB5041585 (August 2024)\n");
            guide.push_str("- Related security updates\n\n");
            guide.push_str("### Solutions:\n\n");
            guide.push_str("1. **Use scheduled task with highest privileges:**\n");
            guide.push_str("   - Create task running as SYSTEM\n");
            guide.push_str("   - Set \"Run with highest privileges\"\n\n");
            guide.push_str("2. **Explicit elevation:**\n");
            guide.push_str("   ```\n");
            guide.push_str("   runas /user:Administrator \"msiexec /fa {ProductCode}\"\n");
            guide.push_str("   ```\n\n");
            guide.push_str("3. **For enterprise deployment:**\n");
            guide.push_str("   - Use SCCM/Intune with system context\n");
            guide.push_str("   - Deploy via Group Policy\n\n");

            for issue in uac_issues {
                if let Some(ref kb) = issue.kb_article {
                    guide.push_str(&format!("Reference: {}\n\n", kb));
                }
            }
        }

        // Permission issues
        if by_category.contains_key(&IssueCategory::PermissionIssue) {
            guide.push_str("## Permission Issues\n\n");
            guide.push_str("### Solutions:\n\n");
            guide.push_str("1. Run repair from elevated command prompt\n");
            guide.push_str("2. Take ownership of affected files/folders\n");
            guide.push_str("3. Check NTFS permissions on installation folder\n");
            guide.push_str("4. Temporarily disable antivirus software\n\n");
        }

        // General repair commands
        guide.push_str("## Repair Command Reference\n\n");
        guide.push_str("| Command | Description |\n");
        guide.push_str("|---------|-------------|\n");
        guide.push_str("| `msiexec /fa {code}` | Repair all files |\n");
        guide.push_str("| `msiexec /fo {code}` | Repair if file missing |\n");
        guide.push_str("| `msiexec /fe {code}` | Reinstall if older/equal |\n");
        guide.push_str("| `msiexec /fd {code}` | Reinstall if different |\n");
        guide.push_str("| `msiexec /fp {code}` | Reinstall if missing |\n");
        guide.push_str("| `msiexec /fc {code}` | Verify checksum |\n");
        guide.push_str("| `msiexec /fs {code}` | All shortcuts |\n");
        guide.push_str("| `msiexec /fu {code}` | User registry |\n");
        guide.push_str("| `msiexec /fm {code}` | Machine registry |\n");
        guide.push_str("| `msiexec /fv {code}` | Recache MSI |\n\n");

        guide.push_str("## Verbose Logging\n\n");
        guide.push_str("Always use verbose logging for troubleshooting:\n");
        guide.push_str("```\n");
        guide.push_str("msiexec /fa {ProductCode} /l*v repair.log\n");
        guide.push_str("```\n\n");

        guide
    }

    /// Get common repair solutions for an error code
    pub fn get_solutions_for_error(&self, error_code: u32) -> Vec<String> {
        match error_code {
            1603 => vec![
                "Check for locked files in the installation directory".to_string(),
                "Review custom action logs for failures".to_string(),
                "Ensure sufficient disk space".to_string(),
                "Run repair from clean boot (minimal services)".to_string(),
                "Check Windows Event Viewer for related errors".to_string(),
            ],
            1606 => vec![
                "Run 'subinacl' to reset registry permissions".to_string(),
                "Check user profile path for special characters".to_string(),
            ],
            1612 => vec![
                "The installation source is not available".to_string(),
                "Register the source location using SOURCELIST property".to_string(),
            ],
            1618 => vec![
                "Another installation is in progress".to_string(),
                "Wait for other installation to complete".to_string(),
                "Check for hung msiexec.exe processes".to_string(),
            ],
            1619 => vec![
                "MSI package could not be opened".to_string(),
                "Verify file is not corrupted".to_string(),
                "Check file permissions".to_string(),
            ],
            1638 => vec![
                "Another version is already installed".to_string(),
                "Uninstall existing version first".to_string(),
                "Use major upgrade handling".to_string(),
            ],
            1706 => vec![
                "No valid source could be found".to_string(),
                "Provide SOURCELIST property with path to MSI".to_string(),
                "Administrative install to network share".to_string(),
            ],
            _ => vec![
                format!("Search Microsoft KB for error {}", error_code),
                "Check MSI verbose log for details".to_string(),
            ],
        }
    }
}

fn extract_timestamp(line: &str) -> Option<String> {
    let time_re = Regex::new(r"(\d{2}:\d{2}:\d{2})").unwrap();
    time_re.captures(line).map(|c| c.get(1).unwrap().as_str().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_source_missing() {
        let log = "Error: The feature you are trying to use is on a network resource that is unavailable";
        let analyzer = RepairAnalyzer::new();
        let result = analyzer.analyze_log(log);

        assert!(!result.issues.is_empty());
        assert!(matches!(result.issues[0].category, IssueCategory::SourceMissing));
    }

    #[test]
    fn test_detect_error_1706() {
        let log = "Error 1706. No valid source could be found for product";
        let analyzer = RepairAnalyzer::new();
        let result = analyzer.analyze_log(log);

        assert!(!result.issues.is_empty());
        assert!(matches!(result.issues[0].category, IssueCategory::SourceMissing));
    }

    #[test]
    fn test_detect_uac_issue() {
        let log = "User Account Control dialog appeared during silent repair";
        let analyzer = RepairAnalyzer::new();
        let result = analyzer.analyze_log(log);

        assert!(!result.issues.is_empty());
        assert!(matches!(result.issues[0].category, IssueCategory::UacIssue));
    }

    #[test]
    fn test_detect_success() {
        let log = "Installation success or error status: 0";
        let analyzer = RepairAnalyzer::new();
        let result = analyzer.analyze_log(log);

        assert!(result.success);
    }

    #[test]
    fn test_detect_failure() {
        let log = "Installation failed\nError 1603: Fatal error during installation";
        let analyzer = RepairAnalyzer::new();
        let result = analyzer.analyze_log(log);

        assert!(!result.success);
        assert!(!result.issues.is_empty());
    }

    #[test]
    fn test_error_code_extraction() {
        let log = "Error 1603: A fatal error occurred during installation.";
        let analyzer = RepairAnalyzer::new();
        let result = analyzer.analyze_log(log);

        assert_eq!(result.error_code, Some(1603));
    }

    #[test]
    fn test_solutions_for_1603() {
        let analyzer = RepairAnalyzer::new();
        let solutions = analyzer.get_solutions_for_error(1603);

        assert!(!solutions.is_empty());
        assert!(solutions.iter().any(|s| s.contains("locked files")));
    }

    #[test]
    fn test_troubleshooting_guide() {
        let issues = vec![RepairIssue {
            severity: Severity::Error,
            category: IssueCategory::SourceMissing,
            message: "Source missing".to_string(),
            details: "Test".to_string(),
            suggestion: "Fix it".to_string(),
            kb_article: Some("https://support.microsoft.com/kb/555175".to_string()),
        }];

        let analyzer = RepairAnalyzer::new();
        let guide = analyzer.generate_troubleshooting_guide(&issues);

        assert!(guide.contains("Source Availability"));
        assert!(guide.contains("SOURCELIST"));
    }
}

//! Error decoder and log analyzer for WiX/MSI
//!
//! Provides human-readable explanations for cryptic MSI error codes
//! and analyzes installation logs to identify issues.
//!
//! # Example
//!
//! ```
//! use wix_doctor::{ErrorDecoder, LogAnalyzer};
//!
//! // Decode an error code
//! let decoder = ErrorDecoder::new();
//! if let Some(info) = decoder.decode(1603) {
//!     println!("{}: {}", info.code, info.description);
//! }
//!
//! // Analyze a log file
//! let analyzer = LogAnalyzer::new();
//! let issues = analyzer.analyze("MSI (s) (A8:6C): Note: ...");
//! ```

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// MSI error information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorInfo {
    /// Error code
    pub code: u32,
    /// Short description
    pub description: String,
    /// Detailed explanation
    pub explanation: String,
    /// Common causes
    pub causes: Vec<String>,
    /// Suggested fixes
    pub fixes: Vec<String>,
    /// Related error codes
    pub related: Vec<u32>,
}

/// Error decoder for MSI/WiX error codes
pub struct ErrorDecoder {
    errors: HashMap<u32, ErrorInfo>,
}

impl Default for ErrorDecoder {
    fn default() -> Self {
        Self::new()
    }
}

impl ErrorDecoder {
    pub fn new() -> Self {
        let mut errors = HashMap::new();

        // Common MSI error codes
        errors.insert(1601, ErrorInfo {
            code: 1601,
            description: "The Windows Installer service could not be accessed".to_string(),
            explanation: "The Windows Installer service is not running or is disabled.".to_string(),
            causes: vec![
                "Windows Installer service is stopped".to_string(),
                "Windows Installer service is disabled".to_string(),
                "Insufficient permissions to access the service".to_string(),
            ],
            fixes: vec![
                "Start the Windows Installer service (msiexec /regserver)".to_string(),
                "Run the installer as Administrator".to_string(),
                "Re-register the Windows Installer: msiexec /unregister && msiexec /regserver".to_string(),
            ],
            related: vec![1719],
        });

        errors.insert(1602, ErrorInfo {
            code: 1602,
            description: "User cancelled installation".to_string(),
            explanation: "The user clicked Cancel during the installation process.".to_string(),
            causes: vec![
                "User clicked Cancel button".to_string(),
                "User closed the installer window".to_string(),
                "Script or automation cancelled the install".to_string(),
            ],
            fixes: vec![
                "This is expected behavior when user cancels".to_string(),
                "For silent installs, check REBOOT property settings".to_string(),
            ],
            related: vec![],
        });

        errors.insert(1603, ErrorInfo {
            code: 1603,
            description: "Fatal error during installation".to_string(),
            explanation: "A fatal error occurred during installation. This is a generic error that can have many causes.".to_string(),
            causes: vec![
                "Insufficient disk space".to_string(),
                "File in use by another process".to_string(),
                "Permissions issue on target directory".to_string(),
                "Custom action failed".to_string(),
                "Invalid MSI package".to_string(),
                "Prerequisite not installed".to_string(),
            ],
            fixes: vec![
                "Check the verbose log (msiexec /i package.msi /l*v log.txt)".to_string(),
                "Free up disk space".to_string(),
                "Close applications that might lock files".to_string(),
                "Run as Administrator".to_string(),
                "Check custom action return codes".to_string(),
                "Verify all prerequisites are installed".to_string(),
            ],
            related: vec![1618, 1619, 1620],
        });

        errors.insert(1605, ErrorInfo {
            code: 1605,
            description: "This action is only valid for products that are currently installed".to_string(),
            explanation: "Attempted to repair or uninstall a product that is not installed.".to_string(),
            causes: vec![
                "Product is not installed".to_string(),
                "Product was installed per-user but running as different user".to_string(),
                "Registry entries are corrupted".to_string(),
            ],
            fixes: vec![
                "Install the product first before repair/uninstall".to_string(),
                "Run as the same user who installed".to_string(),
                "Use Windows Installer CleanUp utility".to_string(),
            ],
            related: vec![1608],
        });

        errors.insert(1608, ErrorInfo {
            code: 1608,
            description: "An installation package for this product could not be found".to_string(),
            explanation: "Windows Installer cannot find the original MSI package.".to_string(),
            causes: vec![
                "Original MSI file was deleted or moved".to_string(),
                "Network path to MSI is unavailable".to_string(),
                "Cached MSI in Windows\\Installer is corrupted".to_string(),
            ],
            fixes: vec![
                "Provide the original MSI package location".to_string(),
                "Reinstall the product".to_string(),
                "Use MsiZap or Windows Installer CleanUp".to_string(),
            ],
            related: vec![1605],
        });

        errors.insert(1618, ErrorInfo {
            code: 1618,
            description: "Another installation is already in progress".to_string(),
            explanation: "Only one Windows Installer operation can run at a time.".to_string(),
            causes: vec![
                "Another MSI installation is running".to_string(),
                "Windows Update is installing updates".to_string(),
                "Previous installation hung".to_string(),
            ],
            fixes: vec![
                "Wait for the other installation to complete".to_string(),
                "Check Task Manager for msiexec.exe processes".to_string(),
                "Restart the Windows Installer service".to_string(),
                "Reboot the computer".to_string(),
            ],
            related: vec![1603],
        });

        errors.insert(1619, ErrorInfo {
            code: 1619,
            description: "This installation package could not be opened".to_string(),
            explanation: "The MSI file is invalid, corrupted, or inaccessible.".to_string(),
            causes: vec![
                "MSI file is corrupted".to_string(),
                "MSI file is incomplete (download issue)".to_string(),
                "No read permission on MSI file".to_string(),
                "MSI was built with incompatible version".to_string(),
            ],
            fixes: vec![
                "Re-download the MSI package".to_string(),
                "Check file permissions".to_string(),
                "Try copying MSI to local drive".to_string(),
                "Verify MSI with Orca or msiinfo".to_string(),
            ],
            related: vec![1620, 1603],
        });

        errors.insert(1620, ErrorInfo {
            code: 1620,
            description: "This installation package could not be opened".to_string(),
            explanation: "The MSI file could not be read or is not a valid Windows Installer package.".to_string(),
            causes: vec![
                "File is not an MSI".to_string(),
                "MSI header is corrupted".to_string(),
                "File has wrong extension".to_string(),
            ],
            fixes: vec![
                "Verify the file is a valid MSI".to_string(),
                "Re-download or obtain a new copy".to_string(),
                "Check if file is a different format (MSP, MSM, etc.)".to_string(),
            ],
            related: vec![1619],
        });

        errors.insert(1624, ErrorInfo {
            code: 1624,
            description: "Error applying transforms".to_string(),
            explanation: "The transform file (MST) could not be applied to the MSI.".to_string(),
            causes: vec![
                "Transform is for different product".to_string(),
                "Transform is for different version".to_string(),
                "Transform file is corrupted".to_string(),
            ],
            fixes: vec![
                "Verify transform matches the MSI".to_string(),
                "Rebuild the transform".to_string(),
                "Check transform validation flags".to_string(),
            ],
            related: vec![],
        });

        errors.insert(1625, ErrorInfo {
            code: 1625,
            description: "This installation is forbidden by system policy".to_string(),
            explanation: "Group Policy or system settings prevent this installation.".to_string(),
            causes: vec![
                "Software Restriction Policy blocks install".to_string(),
                "AppLocker policy blocks install".to_string(),
                "DisableMSI policy is set".to_string(),
            ],
            fixes: vec![
                "Check Group Policy settings".to_string(),
                "Contact your system administrator".to_string(),
                "Run as Administrator".to_string(),
            ],
            related: vec![],
        });

        errors.insert(1638, ErrorInfo {
            code: 1638,
            description: "Another version of this product is already installed".to_string(),
            explanation: "A different version of the same product is installed and must be removed first.".to_string(),
            causes: vec![
                "Older version is installed".to_string(),
                "Newer version is installed".to_string(),
                "Major upgrade not configured correctly".to_string(),
            ],
            fixes: vec![
                "Uninstall the existing version first".to_string(),
                "Configure major upgrade in your MSI".to_string(),
                "Use REINSTALLMODE=vomus".to_string(),
            ],
            related: vec![],
        });

        errors.insert(1639, ErrorInfo {
            code: 1639,
            description: "Invalid command line argument".to_string(),
            explanation: "The command line passed to msiexec contains invalid arguments.".to_string(),
            causes: vec![
                "Typo in property name".to_string(),
                "Missing quotes around value with spaces".to_string(),
                "Invalid option flag".to_string(),
            ],
            fixes: vec![
                "Check command line syntax".to_string(),
                "Quote values with spaces: PROPERTY=\"value with spaces\"".to_string(),
                "Run msiexec /? for help".to_string(),
            ],
            related: vec![],
        });

        errors.insert(1706, ErrorInfo {
            code: 1706,
            description: "No valid source could be found".to_string(),
            explanation: "Windows Installer cannot find a source for the installation.".to_string(),
            causes: vec![
                "Network path is unavailable".to_string(),
                "Removable media is not inserted".to_string(),
                "Source files were deleted".to_string(),
            ],
            fixes: vec![
                "Insert original installation media".to_string(),
                "Connect to network location".to_string(),
                "Provide SOURCELIST property".to_string(),
            ],
            related: vec![1608],
        });

        errors.insert(1715, ErrorInfo {
            code: 1715,
            description: "Installation is disabled by policy".to_string(),
            explanation: "Windows Installer has been disabled through Group Policy.".to_string(),
            causes: vec![
                "DisableMSI policy is enabled".to_string(),
                "AlwaysInstallElevated is disabled".to_string(),
            ],
            fixes: vec![
                "Contact your system administrator".to_string(),
                "Check Group Policy settings".to_string(),
            ],
            related: vec![1625],
        });

        errors.insert(1719, ErrorInfo {
            code: 1719,
            description: "Windows Installer service could not be accessed".to_string(),
            explanation: "The Windows Installer service is unavailable.".to_string(),
            causes: vec![
                "Service is not registered".to_string(),
                "Service is corrupted".to_string(),
                "RPC service issue".to_string(),
            ],
            fixes: vec![
                "Re-register: msiexec /unregister && msiexec /regserver".to_string(),
                "Repair Windows Installer via System File Checker".to_string(),
                "Check RPC service is running".to_string(),
            ],
            related: vec![1601],
        });

        errors.insert(2869, ErrorInfo {
            code: 2869,
            description: "Custom action did not close MSIHANDLE".to_string(),
            explanation: "A custom action failed to properly close handles it opened.".to_string(),
            causes: vec![
                "Custom action bug".to_string(),
                "Custom action crashed".to_string(),
            ],
            fixes: vec![
                "Review custom action code for handle leaks".to_string(),
                "Use proper cleanup in custom actions".to_string(),
                "Check custom action logs".to_string(),
            ],
            related: vec![1603],
        });

        Self { errors }
    }

    /// Decode an error code
    pub fn decode(&self, code: u32) -> Option<&ErrorInfo> {
        self.errors.get(&code)
    }

    /// Get all known error codes
    pub fn all_codes(&self) -> Vec<u32> {
        let mut codes: Vec<_> = self.errors.keys().copied().collect();
        codes.sort();
        codes
    }

    /// Search errors by keyword
    pub fn search(&self, query: &str) -> Vec<&ErrorInfo> {
        let query_lower = query.to_lowercase();
        self.errors
            .values()
            .filter(|info| {
                info.description.to_lowercase().contains(&query_lower)
                    || info.explanation.to_lowercase().contains(&query_lower)
                    || info.causes.iter().any(|c| c.to_lowercase().contains(&query_lower))
            })
            .collect()
    }
}

/// An issue found in a log file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogIssue {
    /// Line number in log
    pub line: usize,
    /// The issue text
    pub text: String,
    /// Issue severity
    pub severity: IssueSeverity,
    /// Issue category
    pub category: IssueCategory,
    /// Additional context
    pub context: Option<String>,
    /// Related error code if any
    pub error_code: Option<u32>,
}

/// Issue severity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IssueSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

/// Issue category
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IssueCategory {
    CustomAction,
    FileOperation,
    RegistryOperation,
    ServiceOperation,
    Permissions,
    Rollback,
    Configuration,
    General,
}

/// Log analyzer for MSI verbose logs
pub struct LogAnalyzer {
    patterns: Vec<LogPattern>,
}

struct LogPattern {
    regex: Regex,
    severity: IssueSeverity,
    category: IssueCategory,
    description: &'static str,
}

impl Default for LogAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl LogAnalyzer {
    pub fn new() -> Self {
        let patterns = vec![
            // Custom action failures
            LogPattern {
                regex: Regex::new(r"(?i)return value\s+3").unwrap(),
                severity: IssueSeverity::Error,
                category: IssueCategory::CustomAction,
                description: "Custom action failed with return code 3",
            },
            LogPattern {
                regex: Regex::new(r"(?i)custom action.*returned actual error code").unwrap(),
                severity: IssueSeverity::Error,
                category: IssueCategory::CustomAction,
                description: "Custom action returned error",
            },
            LogPattern {
                regex: Regex::new(r"(?i)Action ended.*Return value 3").unwrap(),
                severity: IssueSeverity::Error,
                category: IssueCategory::CustomAction,
                description: "Action ended with failure",
            },
            // Fatal errors
            LogPattern {
                regex: Regex::new(r"(?i)return value\s+1603").unwrap(),
                severity: IssueSeverity::Critical,
                category: IssueCategory::General,
                description: "Fatal error occurred",
            },
            LogPattern {
                regex: Regex::new(r"(?i)Installation failed").unwrap(),
                severity: IssueSeverity::Critical,
                category: IssueCategory::General,
                description: "Installation failed",
            },
            // Error codes
            LogPattern {
                regex: Regex::new(r"(?i)error\s+(\d+)").unwrap(),
                severity: IssueSeverity::Error,
                category: IssueCategory::General,
                description: "Error code detected",
            },
            LogPattern {
                regex: Regex::new(r"(?i)GetLastError:\s*(\d+)").unwrap(),
                severity: IssueSeverity::Error,
                category: IssueCategory::General,
                description: "Windows API error",
            },
            // File operations
            LogPattern {
                regex: Regex::new(r"(?i)access\s+(denied|is denied)").unwrap(),
                severity: IssueSeverity::Error,
                category: IssueCategory::Permissions,
                description: "Access denied - permissions issue",
            },
            LogPattern {
                regex: Regex::new(r"(?i)file\s+in\s+use").unwrap(),
                severity: IssueSeverity::Error,
                category: IssueCategory::FileOperation,
                description: "File is locked by another process",
            },
            LogPattern {
                regex: Regex::new(r"(?i)cannot\s+create\s+(file|directory)").unwrap(),
                severity: IssueSeverity::Error,
                category: IssueCategory::FileOperation,
                description: "Cannot create file or directory",
            },
            LogPattern {
                regex: Regex::new(r"(?i)failed\s+to\s+copy").unwrap(),
                severity: IssueSeverity::Error,
                category: IssueCategory::FileOperation,
                description: "File copy failed",
            },
            LogPattern {
                regex: Regex::new(r"(?i)file\s+was\s+rejected").unwrap(),
                severity: IssueSeverity::Error,
                category: IssueCategory::FileOperation,
                description: "File was rejected (hash mismatch or validation)",
            },
            LogPattern {
                regex: Regex::new(r"(?i)Disallowing installation of component").unwrap(),
                severity: IssueSeverity::Error,
                category: IssueCategory::FileOperation,
                description: "Component installation blocked",
            },
            // Rollback
            LogPattern {
                regex: Regex::new(r"(?i)rollback").unwrap(),
                severity: IssueSeverity::Warning,
                category: IssueCategory::Rollback,
                description: "Rollback detected - installation was reverted",
            },
            LogPattern {
                regex: Regex::new(r"(?i)Executing op: ActionStart.*Rollback").unwrap(),
                severity: IssueSeverity::Warning,
                category: IssueCategory::Rollback,
                description: "Rollback action started",
            },
            // Service operations
            LogPattern {
                regex: Regex::new(r"(?i)service\s+'[^']+'\s+could\s+not\s+be").unwrap(),
                severity: IssueSeverity::Error,
                category: IssueCategory::ServiceOperation,
                description: "Service operation failed",
            },
            LogPattern {
                regex: Regex::new(r"(?i)failed\s+to\s+(start|stop|delete)\s+service").unwrap(),
                severity: IssueSeverity::Error,
                category: IssueCategory::ServiceOperation,
                description: "Service control failed",
            },
            // Registry operations
            LogPattern {
                regex: Regex::new(r"(?i)failed\s+to\s+create\s+registry\s+key").unwrap(),
                severity: IssueSeverity::Error,
                category: IssueCategory::RegistryOperation,
                description: "Failed to create registry key",
            },
            LogPattern {
                regex: Regex::new(r"(?i)failed\s+to\s+write\s+registry").unwrap(),
                severity: IssueSeverity::Error,
                category: IssueCategory::RegistryOperation,
                description: "Failed to write registry value",
            },
            // Transform/config issues
            LogPattern {
                regex: Regex::new(r"(?i)transform\s+.*?\s+invalid").unwrap(),
                severity: IssueSeverity::Error,
                category: IssueCategory::Configuration,
                description: "Invalid transform file",
            },
            LogPattern {
                regex: Regex::new(r"(?i)Rejected invalid patch").unwrap(),
                severity: IssueSeverity::Error,
                category: IssueCategory::Configuration,
                description: "Invalid patch rejected",
            },
            // Disk space
            LogPattern {
                regex: Regex::new(r"(?i)disk\s+space").unwrap(),
                severity: IssueSeverity::Warning,
                category: IssueCategory::FileOperation,
                description: "Disk space issue detected",
            },
            LogPattern {
                regex: Regex::new(r"(?i)not\s+enough\s+space").unwrap(),
                severity: IssueSeverity::Error,
                category: IssueCategory::FileOperation,
                description: "Insufficient disk space",
            },
            // Informational
            LogPattern {
                regex: Regex::new(r"(?i)property\s+change").unwrap(),
                severity: IssueSeverity::Info,
                category: IssueCategory::Configuration,
                description: "Property value changed",
            },
            // Source issues
            LogPattern {
                regex: Regex::new(r"(?i)source\s+(not\s+available|unavailable|missing)").unwrap(),
                severity: IssueSeverity::Error,
                category: IssueCategory::Configuration,
                description: "Installation source not available",
            },
            // Reboot
            LogPattern {
                regex: Regex::new(r"(?i)Reboot\s+required").unwrap(),
                severity: IssueSeverity::Warning,
                category: IssueCategory::General,
                description: "Reboot is required",
            },
        ];

        Self { patterns }
    }

    /// Analyze log content for issues
    pub fn analyze(&self, content: &str) -> Vec<LogIssue> {
        let mut issues = Vec::new();

        for (line_num, line) in content.lines().enumerate() {
            for pattern in &self.patterns {
                if pattern.regex.is_match(line) {
                    let error_code = self.extract_error_code(line);

                    issues.push(LogIssue {
                        line: line_num + 1,
                        text: line.trim().to_string(),
                        severity: pattern.severity,
                        category: pattern.category,
                        context: Some(pattern.description.to_string()),
                        error_code,
                    });
                }
            }
        }

        issues
    }

    /// Extract error code from log line
    fn extract_error_code(&self, line: &str) -> Option<u32> {
        let re = Regex::new(r"(?i)(?:error|return value)\s+(\d+)").unwrap();
        re.captures(line)
            .and_then(|caps| caps.get(1))
            .and_then(|m| m.as_str().parse().ok())
    }

    /// Find the root cause of a failure by analyzing context
    pub fn find_root_cause(&self, content: &str) -> Option<RootCauseAnalysis> {
        let lines: Vec<&str> = content.lines().collect();

        // Find "return value 3" lines (the standard failure indicator)
        let failure_lines: Vec<usize> = lines
            .iter()
            .enumerate()
            .filter(|(_, line)| line.to_lowercase().contains("return value 3"))
            .map(|(i, _)| i)
            .collect();

        if failure_lines.is_empty() {
            // Check for 1603 directly
            if content.to_lowercase().contains("return value 1603") ||
               content.to_lowercase().contains("error 1603") {
                return Some(RootCauseAnalysis {
                    summary: "Fatal error 1603 occurred".to_string(),
                    failing_action: None,
                    root_cause: "Check the log for the first 'return value 3' or error message above this line".to_string(),
                    context_lines: Vec::new(),
                    suggestion: "Enable verbose logging and search for the first failure: msiexec /i package.msi /l*v log.txt".to_string(),
                });
            }
            return None;
        }

        // Get the first failure (usually the root cause)
        let first_failure = failure_lines[0];

        // Extract action name from nearby lines
        let action_re = Regex::new(r"Action\s+\d+:\d+:\d+:\s+(\w+)").unwrap();
        let mut failing_action = None;

        // Look backward for the action that failed
        for i in (0..=first_failure).rev() {
            if let Some(caps) = action_re.captures(lines[i]) {
                failing_action = Some(caps.get(1).unwrap().as_str().to_string());
                break;
            }
        }

        // Collect context (lines before the failure)
        let start = first_failure.saturating_sub(10);
        let context_lines: Vec<String> = lines[start..=first_failure]
            .iter()
            .map(|s| s.to_string())
            .collect();

        // Determine root cause from context
        let root_cause = self.determine_root_cause(&context_lines);

        let summary = if let Some(ref action) = failing_action {
            format!("Action '{}' failed", action)
        } else {
            "Installation action failed".to_string()
        };

        let suggestion = self.get_suggestion_for_cause(&root_cause);

        Some(RootCauseAnalysis {
            summary,
            failing_action,
            root_cause,
            context_lines,
            suggestion,
        })
    }

    fn determine_root_cause(&self, context: &[String]) -> String {
        let context_str = context.join("\n").to_lowercase();

        if context_str.contains("access denied") || context_str.contains("permission denied") {
            return "Permission denied - likely need to run as Administrator".to_string();
        }

        if context_str.contains("file in use") || context_str.contains("being used by another process") {
            return "File is locked by another process".to_string();
        }

        if context_str.contains("disk space") || context_str.contains("not enough space") {
            return "Insufficient disk space".to_string();
        }

        if context_str.contains("service") && (context_str.contains("start") || context_str.contains("stop")) {
            return "Service control operation failed".to_string();
        }

        if context_str.contains("registry") {
            return "Registry operation failed".to_string();
        }

        if context_str.contains("custom action") {
            return "Custom action failed - check custom action logs".to_string();
        }

        if context_str.contains("prerequisite") || context_str.contains(".net") || context_str.contains("vcredist") {
            return "Missing prerequisite or runtime".to_string();
        }

        "Unknown cause - check the context lines for details".to_string()
    }

    fn get_suggestion_for_cause(&self, cause: &str) -> String {
        if cause.contains("Permission") {
            return "Run the installer as Administrator or check folder permissions".to_string();
        }
        if cause.contains("locked") {
            return "Close applications that might be using the files, or restart the computer".to_string();
        }
        if cause.contains("disk space") {
            return "Free up disk space and try again".to_string();
        }
        if cause.contains("Service") {
            return "Check Windows Services console for service state and dependencies".to_string();
        }
        if cause.contains("Registry") {
            return "Check registry permissions and ensure no registry keys are locked".to_string();
        }
        if cause.contains("Custom action") {
            return "Review the custom action code and check for logged errors".to_string();
        }
        if cause.contains("prerequisite") {
            return "Install required prerequisites before running the installer".to_string();
        }
        "Review the verbose log for more details".to_string()
    }

    /// Extract installation timeline from log
    pub fn extract_timeline(&self, content: &str) -> Vec<TimelineEntry> {
        let mut timeline = Vec::new();
        let action_re = Regex::new(r"Action\s+(\d+:\d+:\d+):\s+(\w+)").unwrap();
        let end_re = Regex::new(r"Action ended\s+(\d+:\d+:\d+):\s+(\w+)\.\s+Return value\s+(\d+)").unwrap();

        for (line_num, line) in content.lines().enumerate() {
            if let Some(caps) = action_re.captures(line) {
                let timestamp = caps.get(1).unwrap().as_str().to_string();
                let action = caps.get(2).unwrap().as_str().to_string();

                timeline.push(TimelineEntry {
                    line: line_num + 1,
                    timestamp,
                    action,
                    result: None,
                });
            } else if let Some(caps) = end_re.captures(line) {
                let action = caps.get(2).unwrap().as_str().to_string();
                let result: u32 = caps.get(3).unwrap().as_str().parse().unwrap_or(0);

                // Find the matching start entry and update it
                if let Some(entry) = timeline.iter_mut().rev().find(|e| e.action == action && e.result.is_none()) {
                    entry.result = Some(result);
                }
            }
        }

        timeline
    }

    /// Get summary of issues
    pub fn summarize(&self, issues: &[LogIssue]) -> LogSummary {
        let mut summary = LogSummary::default();

        for issue in issues {
            match issue.severity {
                IssueSeverity::Critical => summary.critical_count += 1,
                IssueSeverity::Error => summary.error_count += 1,
                IssueSeverity::Warning => summary.warning_count += 1,
                IssueSeverity::Info => summary.info_count += 1,
            }

            *summary
                .by_category
                .entry(format!("{:?}", issue.category))
                .or_insert(0) += 1;

            if let Some(code) = issue.error_code {
                *summary.error_codes.entry(code).or_insert(0) += 1;
            }
        }

        summary
    }
}

/// Summary of log analysis
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LogSummary {
    pub critical_count: usize,
    pub error_count: usize,
    pub warning_count: usize,
    pub info_count: usize,
    pub by_category: HashMap<String, usize>,
    pub error_codes: HashMap<u32, usize>,
}

impl LogSummary {
    pub fn total_issues(&self) -> usize {
        self.critical_count + self.error_count + self.warning_count + self.info_count
    }

    pub fn has_critical(&self) -> bool {
        self.critical_count > 0
    }
}

/// Root cause analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RootCauseAnalysis {
    /// Brief summary of the failure
    pub summary: String,
    /// The action that failed (if identified)
    pub failing_action: Option<String>,
    /// Identified root cause
    pub root_cause: String,
    /// Relevant log lines for context
    pub context_lines: Vec<String>,
    /// Suggested fix
    pub suggestion: String,
}

/// Entry in installation timeline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineEntry {
    /// Line number in the log
    pub line: usize,
    /// Timestamp from log
    pub timestamp: String,
    /// Action name
    pub action: String,
    /// Return value (None if action still in progress)
    pub result: Option<u32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_known_error() {
        let decoder = ErrorDecoder::new();

        let info = decoder.decode(1603).unwrap();
        assert_eq!(info.code, 1603);
        assert!(info.description.contains("Fatal"));
        assert!(!info.causes.is_empty());
        assert!(!info.fixes.is_empty());
    }

    #[test]
    fn test_decode_unknown_error() {
        let decoder = ErrorDecoder::new();
        assert!(decoder.decode(99999).is_none());
    }

    #[test]
    fn test_search_errors() {
        let decoder = ErrorDecoder::new();

        let results = decoder.search("custom action");
        assert!(!results.is_empty());
    }

    #[test]
    fn test_all_codes() {
        let decoder = ErrorDecoder::new();
        let codes = decoder.all_codes();

        assert!(codes.contains(&1603));
        assert!(codes.contains(&1602));
        // Should be sorted
        assert!(codes.windows(2).all(|w| w[0] <= w[1]));
    }

    #[test]
    fn test_analyze_empty_log() {
        let analyzer = LogAnalyzer::new();
        let issues = analyzer.analyze("");
        assert!(issues.is_empty());
    }

    #[test]
    fn test_analyze_custom_action_failure() {
        let analyzer = LogAnalyzer::new();
        let log = "MSI (s) (A8:6C): Custom Action MyAction returned actual return value 3";

        let issues = analyzer.analyze(log);
        assert!(!issues.is_empty());
        assert_eq!(issues[0].category, IssueCategory::CustomAction);
        assert_eq!(issues[0].severity, IssueSeverity::Error);
    }

    #[test]
    fn test_analyze_access_denied() {
        let analyzer = LogAnalyzer::new();
        let log = "Error: Access denied to file C:\\Program Files\\MyApp";

        let issues = analyzer.analyze(log);
        assert!(!issues.is_empty());
        assert_eq!(issues[0].category, IssueCategory::Permissions);
    }

    #[test]
    fn test_analyze_rollback() {
        let analyzer = LogAnalyzer::new();
        let log = "MSI (s): Executing Rollback script";

        let issues = analyzer.analyze(log);
        assert!(!issues.is_empty());
        assert_eq!(issues[0].category, IssueCategory::Rollback);
    }

    #[test]
    fn test_analyze_error_code_extraction() {
        let analyzer = LogAnalyzer::new();
        let log = "Error 1603: Fatal error during installation";

        let issues = analyzer.analyze(log);
        assert!(!issues.is_empty());
        assert_eq!(issues[0].error_code, Some(1603));
    }

    #[test]
    fn test_log_summary() {
        let analyzer = LogAnalyzer::new();
        let log = r#"
Error 1603: Fatal error
Custom action returned return value 3
Access denied to file
Rollback started
"#;

        let issues = analyzer.analyze(log);
        let summary = analyzer.summarize(&issues);

        assert!(summary.total_issues() > 0);
        assert!(summary.error_count > 0);
    }

    #[test]
    fn test_multiple_issues_one_log() {
        let analyzer = LogAnalyzer::new();
        let log = r#"
Line 1: Starting installation
Line 2: Error 1603 occurred
Line 3: Access denied
Line 4: Rollback initiated
Line 5: Installation complete
"#;

        let issues = analyzer.analyze(log);
        assert!(issues.len() >= 3);
    }

    #[test]
    fn test_error_info_related() {
        let decoder = ErrorDecoder::new();
        let info = decoder.decode(1603).unwrap();

        // 1603 should have related errors
        assert!(!info.related.is_empty());
    }

    #[test]
    fn test_various_error_codes() {
        let decoder = ErrorDecoder::new();

        // Test several common codes exist
        assert!(decoder.decode(1601).is_some());
        assert!(decoder.decode(1602).is_some());
        assert!(decoder.decode(1618).is_some());
        assert!(decoder.decode(1619).is_some());
        assert!(decoder.decode(1638).is_some());
    }

    #[test]
    fn test_issue_severity_levels() {
        let analyzer = LogAnalyzer::new();

        // Critical
        let issues = analyzer.analyze("return value 1603");
        assert!(issues.iter().any(|i| i.severity == IssueSeverity::Critical));

        // Warning
        let issues = analyzer.analyze("Rollback");
        assert!(issues.iter().any(|i| i.severity == IssueSeverity::Warning));
    }

    #[test]
    fn test_log_summary_by_category() {
        let analyzer = LogAnalyzer::new();
        let log = r#"
Access denied error
Cannot create file
"#;

        let issues = analyzer.analyze(log);
        let summary = analyzer.summarize(&issues);

        assert!(summary.by_category.contains_key("Permissions") ||
                summary.by_category.contains_key("FileOperation"));
    }

    #[test]
    fn test_find_root_cause_access_denied() {
        let analyzer = LogAnalyzer::new();
        let log = r#"
Action 10:30:45: InstallFiles
Copying file C:\Program Files\MyApp\app.exe
Access denied to C:\Program Files\MyApp
Custom action returned return value 3
"#;

        let result = analyzer.find_root_cause(log);
        assert!(result.is_some());
        let analysis = result.unwrap();
        assert!(analysis.root_cause.contains("Permission"));
    }

    #[test]
    fn test_find_root_cause_file_in_use() {
        let analyzer = LogAnalyzer::new();
        let log = r#"
Action 10:30:45: InstallFiles
Cannot copy: file in use by another process
Return value 3
"#;

        let result = analyzer.find_root_cause(log);
        assert!(result.is_some());
        let analysis = result.unwrap();
        assert!(analysis.root_cause.contains("locked"));
    }

    #[test]
    fn test_find_root_cause_no_failure() {
        let analyzer = LogAnalyzer::new();
        let log = r#"
Action 10:30:45: InstallFiles
Return value 1
Installation complete
"#;

        let result = analyzer.find_root_cause(log);
        assert!(result.is_none());
    }

    #[test]
    fn test_extract_timeline() {
        let analyzer = LogAnalyzer::new();
        let log = r#"
Action 10:30:00: InstallInitialize
Action ended 10:30:05: InstallInitialize. Return value 1
Action 10:30:05: InstallFiles
Action ended 10:30:10: InstallFiles. Return value 1
Action 10:30:10: InstallFinalize
Action ended 10:30:15: InstallFinalize. Return value 1
"#;

        let timeline = analyzer.extract_timeline(log);
        assert_eq!(timeline.len(), 3);
        assert_eq!(timeline[0].action, "InstallInitialize");
        assert_eq!(timeline[0].result, Some(1));
        assert_eq!(timeline[1].action, "InstallFiles");
        assert_eq!(timeline[2].action, "InstallFinalize");
    }

    #[test]
    fn test_extract_timeline_with_failure() {
        let analyzer = LogAnalyzer::new();
        let log = r#"
Action 10:30:00: InstallInitialize
Action ended 10:30:05: InstallInitialize. Return value 1
Action 10:30:05: CustomAction
Action ended 10:30:10: CustomAction. Return value 3
"#;

        let timeline = analyzer.extract_timeline(log);
        assert_eq!(timeline.len(), 2);
        assert_eq!(timeline[1].action, "CustomAction");
        assert_eq!(timeline[1].result, Some(3));
    }

    #[test]
    fn test_root_cause_analysis_fields() {
        let analysis = RootCauseAnalysis {
            summary: "Test failure".to_string(),
            failing_action: Some("TestAction".to_string()),
            root_cause: "Test cause".to_string(),
            context_lines: vec!["line 1".to_string(), "line 2".to_string()],
            suggestion: "Test suggestion".to_string(),
        };

        assert_eq!(analysis.summary, "Test failure");
        assert_eq!(analysis.failing_action, Some("TestAction".to_string()));
        assert!(!analysis.context_lines.is_empty());
    }

    #[test]
    fn test_timeline_entry_fields() {
        let entry = TimelineEntry {
            line: 42,
            timestamp: "10:30:00".to_string(),
            action: "InstallFiles".to_string(),
            result: Some(1),
        };

        assert_eq!(entry.line, 42);
        assert_eq!(entry.action, "InstallFiles");
        assert_eq!(entry.result, Some(1));
    }
}

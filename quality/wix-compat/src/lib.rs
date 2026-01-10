//! wix-compat - Windows version compatibility checker
//!
//! Checks WiX installers for compatibility issues with:
//! - Windows Server 2025 (MSI hang issues, VersionNT64 detection)
//! - Windows 11 24H2 (August 2025 UAC changes, Error 1730)
//! - ARM64 Windows (driver installation, ProcessorArchitecture)
//! - Older Windows versions (deprecated APIs)

use serde::{Deserialize, Serialize};

/// Windows version
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WindowsVersion {
    pub name: &'static str,
    pub version: &'static str,
    pub build: u32,
    pub version_nt: u32,
}

/// Known Windows versions
pub const WINDOWS_VERSIONS: &[WindowsVersion] = &[
    WindowsVersion { name: "Windows 10 1809", version: "10.0.17763", build: 17763, version_nt: 1000 },
    WindowsVersion { name: "Windows 10 21H2", version: "10.0.19044", build: 19044, version_nt: 1000 },
    WindowsVersion { name: "Windows 11 21H2", version: "10.0.22000", build: 22000, version_nt: 1100 },
    WindowsVersion { name: "Windows 11 23H2", version: "10.0.22631", build: 22631, version_nt: 1100 },
    WindowsVersion { name: "Windows 11 24H2", version: "10.0.26100", build: 26100, version_nt: 1100 },
    WindowsVersion { name: "Windows Server 2019", version: "10.0.17763", build: 17763, version_nt: 1000 },
    WindowsVersion { name: "Windows Server 2022", version: "10.0.20348", build: 20348, version_nt: 1000 },
    WindowsVersion { name: "Windows Server 2025", version: "10.0.26100", build: 26100, version_nt: 1000 },
];

/// Compatibility issue severity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity {
    /// Will cause installation failure
    Critical,
    /// May cause issues in certain scenarios
    Warning,
    /// Informational, best practice
    Info,
}

/// Compatibility issue type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum IssueType {
    /// VersionNT64 detection issue on Server 2025
    VersionNT64Detection,
    /// Per-user MSI UAC issue (August 2025 update)
    PerUserUacIssue,
    /// ARM64 compatibility issue
    Arm64Compatibility,
    /// Deprecated Windows Installer feature
    DeprecatedFeature,
    /// Missing platform support
    MissingPlatformSupport,
    /// Launch condition issue
    LaunchConditionIssue,
    /// Service account issue on DC
    ServiceAccountIssue,
    /// DifxApp driver installation (deprecated)
    DifxAppDeprecation,
}

/// A compatibility issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompatIssue {
    pub issue_type: IssueType,
    pub severity: Severity,
    pub affected_versions: Vec<String>,
    pub element: Option<String>,
    pub line: Option<usize>,
    pub message: String,
    pub workaround: String,
    pub reference: Option<String>,
}

/// Compatibility analysis result
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CompatAnalysis {
    pub issues: Vec<CompatIssue>,
    pub target_platforms: Vec<String>,
    pub launch_conditions: Vec<LaunchCondition>,
    pub has_per_user_install: bool,
    pub has_arm64_support: bool,
    pub has_driver_install: bool,
    pub critical_count: usize,
    pub warning_count: usize,
}

/// Launch condition information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaunchCondition {
    pub condition: String,
    pub message: Option<String>,
    pub potential_issues: Vec<String>,
}

/// Compatibility checker
pub struct CompatChecker;

impl CompatChecker {
    /// Analyze WiX source for compatibility issues
    pub fn analyze(content: &str) -> CompatAnalysis {
        let mut analysis = CompatAnalysis::default();

        if let Ok(doc) = roxmltree::Document::parse(content) {
            for node in doc.descendants() {
                match node.tag_name().name() {
                    "Package" | "Product" => {
                        // Check for per-user installation scope
                        if let Some(scope) = node.attribute("Scope") {
                            if scope == "perUser" {
                                analysis.has_per_user_install = true;
                                analysis.issues.push(CompatIssue {
                                    issue_type: IssueType::PerUserUacIssue,
                                    severity: Severity::Warning,
                                    affected_versions: vec!["Windows 11 24H2".to_string()],
                                    element: Some("Package".to_string()),
                                    line: None,
                                    message: "Per-user installations may trigger UAC prompts after August 2025 update".to_string(),
                                    workaround: "Consider per-machine installation or test with KB5063878".to_string(),
                                    reference: Some("https://support.microsoft.com/kb/5063878".to_string()),
                                });
                            }
                        }

                        // Check Platform attribute
                        if let Some(platform) = node.attribute("Platform") {
                            analysis.target_platforms.push(platform.to_string());
                            if platform.to_lowercase() == "arm64" {
                                analysis.has_arm64_support = true;
                            }
                        }
                    }

                    "Condition" => {
                        if let Some(text) = node.text() {
                            let condition = text.to_string();
                            let mut potential_issues = Vec::new();

                            // Check for VersionNT64 issues
                            if condition.contains("VersionNT64") {
                                potential_issues.push(
                                    "VersionNT64 may not detect Windows Server 2025 correctly with older tools".to_string()
                                );
                                analysis.issues.push(CompatIssue {
                                    issue_type: IssueType::VersionNT64Detection,
                                    severity: Severity::Warning,
                                    affected_versions: vec!["Windows Server 2025".to_string()],
                                    element: Some("Condition".to_string()),
                                    line: None,
                                    message: format!("Condition '{}' uses VersionNT64 which may have detection issues on Server 2025", condition),
                                    workaround: "Use VersionNT >= 1000 or specific build number checks instead".to_string(),
                                    reference: Some("https://github.com/wixtoolset/issues".to_string()),
                                });
                            }

                            // Check for old version checks
                            if condition.contains("VersionNT < 600") || condition.contains("VersionNT < 601") {
                                potential_issues.push(
                                    "Checking for very old Windows versions (pre-Vista/7)".to_string()
                                );
                            }

                            if !potential_issues.is_empty() {
                                analysis.launch_conditions.push(LaunchCondition {
                                    condition,
                                    message: node.parent()
                                        .and_then(|p| p.attribute("Message"))
                                        .map(String::from),
                                    potential_issues,
                                });
                            }
                        }
                    }

                    "Launch" => {
                        // WiX v4+ Launch element
                        if let Some(condition) = node.attribute("Condition") {
                            analysis.launch_conditions.push(LaunchCondition {
                                condition: condition.to_string(),
                                message: node.attribute("Message").map(String::from),
                                potential_issues: Vec::new(),
                            });
                        }
                    }

                    "difx:Driver" | "Driver" => {
                        // DifxApp is deprecated and doesn't support ARM64
                        analysis.has_driver_install = true;
                        analysis.issues.push(CompatIssue {
                            issue_type: IssueType::DifxAppDeprecation,
                            severity: Severity::Critical,
                            affected_versions: vec!["ARM64".to_string(), "Windows Server 2025".to_string()],
                            element: node.attribute("Id").map(String::from),
                            line: None,
                            message: "DifxApp driver installation is deprecated and not supported on ARM64".to_string(),
                            workaround: "Use DIFxAPI directly, or pnputil.exe for driver installation".to_string(),
                            reference: Some("https://github.com/wixtoolset/issues/issues/6251".to_string()),
                        });
                    }

                    "ServiceInstall" => {
                        // Check for LocalSystem account on potential DC
                        if let Some(account) = node.attribute("Account") {
                            if account.to_lowercase().contains("localsystem") || account == "LocalSystem" {
                                analysis.issues.push(CompatIssue {
                                    issue_type: IssueType::ServiceAccountIssue,
                                    severity: Severity::Info,
                                    affected_versions: vec!["Windows Server 2025 DC".to_string()],
                                    element: node.attribute("Id").map(String::from),
                                    line: None,
                                    message: "Services using LocalSystem may have issues during DC promotion on Server 2025".to_string(),
                                    workaround: "Consider using a managed service account or test installation on DC".to_string(),
                                    reference: Some("https://techcommunity.microsoft.com/discussions/windowsserver".to_string()),
                                });
                            }
                        }
                    }

                    _ => {}
                }
            }

            // Check for missing ARM64 support
            if !analysis.has_arm64_support && !analysis.target_platforms.is_empty() {
                let has_x64 = analysis.target_platforms.iter().any(|p| p.to_lowercase() == "x64");
                if has_x64 {
                    analysis.issues.push(CompatIssue {
                        issue_type: IssueType::MissingPlatformSupport,
                        severity: Severity::Info,
                        affected_versions: vec!["ARM64 Windows".to_string()],
                        element: None,
                        line: None,
                        message: "Package targets x64 but not ARM64 - ARM64 devices are growing in market share".to_string(),
                        workaround: "Consider adding ARM64 platform support for broader compatibility".to_string(),
                        reference: Some("https://github.com/wixtoolset/issues/issues/5558".to_string()),
                    });
                }
            }
        }

        // Calculate counts
        analysis.critical_count = analysis.issues.iter()
            .filter(|i| i.severity == Severity::Critical)
            .count();
        analysis.warning_count = analysis.issues.iter()
            .filter(|i| i.severity == Severity::Warning)
            .count();

        analysis
    }

    /// Generate compatibility report
    pub fn generate_report(analysis: &CompatAnalysis) -> String {
        let mut report = String::new();

        report.push_str("Windows Compatibility Report\n");
        report.push_str(&"=".repeat(50));
        report.push('\n');

        // Summary
        report.push_str("\nSummary:\n");
        report.push_str(&format!("  Critical Issues: {}\n", analysis.critical_count));
        report.push_str(&format!("  Warnings: {}\n", analysis.warning_count));
        report.push_str(&format!("  Target Platforms: {}\n",
            if analysis.target_platforms.is_empty() {
                "Not specified".to_string()
            } else {
                analysis.target_platforms.join(", ")
            }
        ));
        report.push_str(&format!("  Per-User Install: {}\n", analysis.has_per_user_install));
        report.push_str(&format!("  ARM64 Support: {}\n", analysis.has_arm64_support));
        report.push_str(&format!("  Driver Install: {}\n", analysis.has_driver_install));

        // Issues
        if !analysis.issues.is_empty() {
            report.push_str("\nIssues:\n");
            report.push_str(&"-".repeat(50));
            report.push('\n');

            for (i, issue) in analysis.issues.iter().enumerate() {
                report.push_str(&format!(
                    "\n{}. [{}] {:?}\n",
                    i + 1,
                    match issue.severity {
                        Severity::Critical => "CRITICAL",
                        Severity::Warning => "WARNING",
                        Severity::Info => "INFO",
                    },
                    issue.issue_type
                ));
                report.push_str(&format!("   {}\n", issue.message));
                report.push_str(&format!("   Affected: {}\n", issue.affected_versions.join(", ")));
                report.push_str(&format!("   Workaround: {}\n", issue.workaround));
                if let Some(ref reference) = issue.reference {
                    report.push_str(&format!("   Reference: {}\n", reference));
                }
            }
        }

        // Launch conditions
        if !analysis.launch_conditions.is_empty() {
            report.push_str("\nLaunch Conditions:\n");
            report.push_str(&"-".repeat(50));
            report.push('\n');

            for lc in &analysis.launch_conditions {
                report.push_str(&format!("\n  Condition: {}\n", lc.condition));
                if let Some(ref msg) = lc.message {
                    report.push_str(&format!("  Message: {}\n", msg));
                }
                for issue in &lc.potential_issues {
                    report.push_str(&format!("  Potential Issue: {}\n", issue));
                }
            }
        }

        report
    }

    /// Generate compatibility matrix
    pub fn generate_matrix(analysis: &CompatAnalysis) -> String {
        let mut matrix = String::new();

        matrix.push_str("Compatibility Matrix\n");
        matrix.push_str(&"=".repeat(70));
        matrix.push('\n');

        matrix.push_str("\n| Windows Version         | Compatible | Notes                           |\n");
        matrix.push_str("|-------------------------|------------|----------------------------------|\n");

        let versions = [
            ("Windows 10 21H2", true, ""),
            ("Windows 11 23H2", true, ""),
            ("Windows 11 24H2", !analysis.has_per_user_install, "UAC changes if per-user"),
            ("Windows Server 2022", true, ""),
            ("Windows Server 2025", !analysis.has_driver_install, "Driver/DC issues possible"),
            ("ARM64 Windows", analysis.has_arm64_support, "Requires ARM64 build"),
        ];

        for (version, compatible, notes) in versions {
            let status = if compatible { "Yes" } else { "Check" };
            matrix.push_str(&format!(
                "| {:<23} | {:<10} | {:<32} |\n",
                version, status, notes
            ));
        }

        matrix
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyze_per_user() {
        let content = r#"
        <Wix xmlns="http://wixtoolset.org/schemas/v4/wxs">
            <Package Name="Test" Scope="perUser" />
        </Wix>
        "#;

        let analysis = CompatChecker::analyze(content);
        assert!(analysis.has_per_user_install);
        assert!(analysis.issues.iter().any(|i| i.issue_type == IssueType::PerUserUacIssue));
    }

    #[test]
    fn test_analyze_version_nt64() {
        let content = r#"
        <Wix xmlns="http://wixtoolset.org/schemas/v4/wxs">
            <Package Name="Test">
                <Condition>VersionNT64</Condition>
            </Package>
        </Wix>
        "#;

        let analysis = CompatChecker::analyze(content);
        assert!(analysis.issues.iter().any(|i| i.issue_type == IssueType::VersionNT64Detection));
    }

    #[test]
    fn test_analyze_difxapp() {
        let content = r#"
        <Wix xmlns="http://wixtoolset.org/schemas/v4/wxs"
             xmlns:difx="http://schemas.microsoft.com/wix/DifxAppExtension">
            <Package Name="Test">
                <Component>
                    <Driver Id="MyDriver" />
                </Component>
            </Package>
        </Wix>
        "#;

        let analysis = CompatChecker::analyze(content);
        assert!(analysis.has_driver_install);
        assert!(analysis.issues.iter().any(|i| i.issue_type == IssueType::DifxAppDeprecation));
    }

    #[test]
    fn test_analyze_clean_package() {
        let content = r#"
        <Wix xmlns="http://wixtoolset.org/schemas/v4/wxs">
            <Package Name="Test" Version="1.0.0" Manufacturer="Test" Platform="x64">
                <Feature Id="Main" />
            </Package>
        </Wix>
        "#;

        let analysis = CompatChecker::analyze(content);
        assert_eq!(analysis.critical_count, 0);
        assert!(analysis.target_platforms.contains(&"x64".to_string()));
    }

    #[test]
    fn test_generate_report() {
        let content = r#"
        <Wix xmlns="http://wixtoolset.org/schemas/v4/wxs">
            <Package Name="Test" Scope="perUser" />
        </Wix>
        "#;

        let analysis = CompatChecker::analyze(content);
        let report = CompatChecker::generate_report(&analysis);
        assert!(report.contains("Per-User Install: true"));
        assert!(report.contains("WARNING"));
    }

    #[test]
    fn test_generate_matrix() {
        let analysis = CompatAnalysis {
            has_arm64_support: true,
            has_per_user_install: false,
            ..Default::default()
        };

        let matrix = CompatChecker::generate_matrix(&analysis);
        assert!(matrix.contains("ARM64 Windows"));
        assert!(matrix.contains("Yes"));
    }
}

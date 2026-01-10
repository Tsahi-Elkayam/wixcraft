//! Security analyzer - identifies potential security issues in WiX files

use crate::core::{
    AnalysisResult, Category, Diagnostic, Location, SymbolIndex, WixDocument,
};
use regex::Regex;
use std::sync::LazyLock;
use super::Analyzer;

/// Sensitive property name patterns
static SENSITIVE_PATTERNS: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)(password|secret|key|token|credential|apikey|api_key|auth)").unwrap()
});

/// Security analyzer
pub struct SecurityAnalyzer;

impl SecurityAnalyzer {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SecurityAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl Analyzer for SecurityAnalyzer {
    fn analyze(&self, doc: &WixDocument, _index: &SymbolIndex) -> AnalysisResult {
        let mut result = AnalysisResult::new();

        // Check for services running as LocalSystem
        self.check_service_accounts(doc, &mut result);

        // Check for overly permissive permissions
        self.check_permissions(doc, &mut result);

        // Check for sensitive data in properties
        self.check_sensitive_properties(doc, &mut result);

        // Check for CustomActions with elevation
        self.check_elevated_custom_actions(doc, &mut result);

        // Check for world-writable directories
        self.check_directory_permissions(doc, &mut result);

        result
    }
}

impl SecurityAnalyzer {
    fn check_service_accounts(&self, doc: &WixDocument, result: &mut AnalysisResult) {
        for node in doc.root().descendants() {
            if node.tag_name().name() == "ServiceInstall" {
                let account = node.attribute("Account").unwrap_or("LocalSystem");

                // LocalSystem is overly privileged for most services
                if account == "LocalSystem" || account.is_empty() {
                    let name = node.attribute("Name").unwrap_or("unknown");
                    let range = doc.node_range(&node);
                    let location = Location::new(doc.file().to_path_buf(), range);

                    // Check if it's a driver or truly needs LocalSystem
                    let service_type = node.attribute("Type").unwrap_or("");
                    if !service_type.contains("kernel") && !service_type.contains("filesystemDriver") {
                        result.add(
                            Diagnostic::warning(
                                "SEC-001",
                                Category::Security,
                                format!(
                                    "Service '{}' runs as LocalSystem. Consider using LocalService or NetworkService",
                                    name
                                ),
                                location,
                            )
                            .with_help("LocalSystem has full system access. Use the least privileged account")
                            .with_tags(["CWE-250", "OWASP-A04:2021"]), // Execution with Unnecessary Privileges
                        );
                    }
                }
            }
        }
    }

    fn check_permissions(&self, doc: &WixDocument, result: &mut AnalysisResult) {
        for node in doc.root().descendants() {
            let tag_name = node.tag_name().name();

            if tag_name == "Permission" || tag_name == "PermissionEx" {
                // Check for Everyone with write access
                if let Some(user) = node.attribute("User") {
                    let is_everyone = user.eq_ignore_ascii_case("Everyone")
                        || user.eq_ignore_ascii_case("*S-1-1-0");

                    if is_everyone {
                        // Check for write permissions
                        let has_write = node.attribute("GenericWrite").map(|v| v == "yes").unwrap_or(false)
                            || node.attribute("Write").map(|v| v == "yes").unwrap_or(false)
                            || node.attribute("Modify").map(|v| v == "yes").unwrap_or(false)
                            || node.attribute("FullControl").map(|v| v == "yes").unwrap_or(false)
                            || node.attribute("GenericAll").map(|v| v == "yes").unwrap_or(false);

                        if has_write {
                            let range = doc.node_range(&node);
                            let location = Location::new(doc.file().to_path_buf(), range);
                            result.add(
                                Diagnostic::warning(
                                    "SEC-002",
                                    Category::Security,
                                    "Granting write permissions to 'Everyone' is a security risk",
                                    location,
                                )
                                .with_help("Restrict permissions to specific users or groups")
                                .with_tags(["CWE-732", "CWE-276", "OWASP-A01:2021"]), // Incorrect Permission Assignment
                            );
                        }
                    }
                }
            }

            // Check registry permissions
            if tag_name == "RegistryKey" || tag_name == "RegistryValue" {
                // Check for HKLM with weak permissions
                if let Some(root) = node.attribute("Root") {
                    if root == "HKLM" {
                        // Check child Permission elements
                        for child in node.children() {
                            if child.is_element() && child.tag_name().name() == "Permission" {
                                if let Some(user) = child.attribute("User") {
                                    if user.eq_ignore_ascii_case("Users") || user.eq_ignore_ascii_case("Everyone") {
                                        let has_write = child.attribute("Write").map(|v| v == "yes").unwrap_or(false)
                                            || child.attribute("FullControl").map(|v| v == "yes").unwrap_or(false);

                                        if has_write {
                                            let range = doc.node_range(&node);
                                            let location = Location::new(doc.file().to_path_buf(), range);
                                            result.add(
                                                Diagnostic::warning(
                                                    "SEC-003",
                                                    Category::Security,
                                                    "Granting write access to HKLM registry keys is a security risk",
                                                    location,
                                                )
                                                .with_tags(["CWE-732", "CWE-276", "OWASP-A01:2021"]), // Incorrect Permission Assignment
                                            );
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    fn check_sensitive_properties(&self, doc: &WixDocument, result: &mut AnalysisResult) {
        for node in doc.root().descendants() {
            if node.tag_name().name() == "Property" {
                if let Some(id) = node.attribute("Id") {
                    // Check if property name suggests sensitive data
                    if SENSITIVE_PATTERNS.is_match(id) {
                        // Check if it has a hardcoded value
                        if let Some(value) = node.attribute("Value") {
                            if !value.is_empty() && !value.starts_with("[") && !value.starts_with("!(") {
                                let range = doc.node_range(&node);
                                let location = Location::new(doc.file().to_path_buf(), range);
                                result.add(
                                    Diagnostic::error(
                                        "SEC-005",
                                        Category::Security,
                                        format!(
                                            "Property '{}' appears to contain sensitive data with a hardcoded value",
                                            id
                                        ),
                                        location,
                                    )
                                    .with_help("Never hardcode sensitive values. Use secure properties or prompt at install time")
                                    .with_tags(["CWE-798", "CWE-259", "OWASP-A07:2021"]), // Hardcoded Credentials
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    fn check_elevated_custom_actions(&self, doc: &WixDocument, result: &mut AnalysisResult) {
        for node in doc.root().descendants() {
            if node.tag_name().name() == "CustomAction" {
                let execute = node.attribute("Execute").unwrap_or("immediate");
                let impersonate = node.attribute("Impersonate").unwrap_or("yes");

                // Deferred actions with Impersonate="no" run elevated
                if execute == "deferred" && impersonate == "no" {
                    let id = node.attribute("Id").unwrap_or("unknown");

                    // Check if it's a script action (higher risk)
                    let is_script = node.attribute("Script").is_some()
                        || node.attribute("VBScriptCall").is_some()
                        || node.attribute("JScriptCall").is_some();

                    if is_script {
                        let range = doc.node_range(&node);
                        let location = Location::new(doc.file().to_path_buf(), range);
                        result.add(
                            Diagnostic::warning(
                                "SEC-004",
                                Category::Security,
                                format!(
                                    "CustomAction '{}' runs elevated script code. Ensure input is validated",
                                    id
                                ),
                                location,
                            )
                            .with_help("Elevated scripts can be a security risk if they process user input")
                            .with_tags(["CWE-94", "CWE-250", "OWASP-A03:2021"]), // Code Injection, Unnecessary Privileges
                        );
                    }
                }
            }
        }
    }

    fn check_directory_permissions(&self, doc: &WixDocument, result: &mut AnalysisResult) {
        for node in doc.root().descendants() {
            if node.tag_name().name() == "Directory" || node.tag_name().name() == "StandardDirectory" {
                // Check for permissions granting write to everyone
                for child in node.children() {
                    if child.is_element() && child.tag_name().name() == "Permission" {
                        if let Some(user) = child.attribute("User") {
                            if user.eq_ignore_ascii_case("Everyone") || user.eq_ignore_ascii_case("Users") {
                                let has_write = child.attribute("GenericWrite").map(|v| v == "yes").unwrap_or(false)
                                    || child.attribute("Write").map(|v| v == "yes").unwrap_or(false)
                                    || child.attribute("Modify").map(|v| v == "yes").unwrap_or(false);

                                if has_write {
                                    let id = node.attribute("Id").unwrap_or("unknown");
                                    let range = doc.node_range(&node);
                                    let location = Location::new(doc.file().to_path_buf(), range);
                                    result.add(
                                        Diagnostic::error(
                                            "SEC-007",
                                            Category::Security,
                                            format!(
                                                "Directory '{}' is world-writable, which is a security vulnerability",
                                                id
                                            ),
                                            location,
                                        )
                                        .with_help("Avoid granting write permissions to Everyone or Users on install directories")
                                        .with_tags(["CWE-732", "CWE-276", "CWE-427", "OWASP-A01:2021"]), // Insecure Permissions, Untrusted Search Path
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn analyze(source: &str) -> AnalysisResult {
        let doc = WixDocument::parse(source, Path::new("test.wxs")).unwrap();
        let index = SymbolIndex::new();
        let analyzer = SecurityAnalyzer::new();
        analyzer.analyze(&doc, &index)
    }

    #[test]
    fn test_default_impl() {
        let analyzer = SecurityAnalyzer::default();
        let doc = WixDocument::parse("<Wix />", Path::new("test.wxs")).unwrap();
        let index = SymbolIndex::new();
        let result = analyzer.analyze(&doc, &index);
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_localsystem_service() {
        let result = analyze(r#"<Wix><ServiceInstall Name="MySvc" Account="LocalSystem" /></Wix>"#);
        assert!(result.diagnostics.iter().any(|d| d.rule_id == "SEC-001"));
    }

    #[test]
    fn test_service_empty_account() {
        // Empty Account defaults to LocalSystem
        let result = analyze(r#"<Wix><ServiceInstall Name="MySvc" Account="" /></Wix>"#);
        assert!(result.diagnostics.iter().any(|d| d.rule_id == "SEC-001"));
    }

    #[test]
    fn test_service_no_account() {
        // No Account attribute defaults to LocalSystem
        let result = analyze(r#"<Wix><ServiceInstall Name="MySvc" /></Wix>"#);
        assert!(result.diagnostics.iter().any(|d| d.rule_id == "SEC-001"));
    }

    #[test]
    fn test_kernel_driver_localsystem_ok() {
        // Kernel drivers legitimately need LocalSystem
        let result = analyze(r#"<Wix><ServiceInstall Name="MyDriver" Account="LocalSystem" Type="kernelDriver" /></Wix>"#);
        assert!(result.diagnostics.iter().all(|d| d.rule_id != "SEC-001"));
    }

    #[test]
    fn test_filesystem_driver_localsystem_ok() {
        // File system drivers legitimately need LocalSystem
        let result = analyze(r#"<Wix><ServiceInstall Name="MyDriver" Account="LocalSystem" Type="filesystemDriver" /></Wix>"#);
        assert!(result.diagnostics.iter().all(|d| d.rule_id != "SEC-001"));
    }

    #[test]
    fn test_localservice_ok() {
        let result = analyze(r#"<Wix><ServiceInstall Name="MySvc" Account="LocalService" /></Wix>"#);
        assert!(result.diagnostics.iter().all(|d| d.rule_id != "SEC-001"));
    }

    #[test]
    fn test_everyone_write_permission() {
        let result = analyze(r#"<Wix><Permission User="Everyone" GenericWrite="yes" /></Wix>"#);
        assert!(result.diagnostics.iter().any(|d| d.rule_id == "SEC-002"));
    }

    #[test]
    fn test_everyone_write_permission_direct() {
        let result = analyze(r#"<Wix><Permission User="Everyone" Write="yes" /></Wix>"#);
        assert!(result.diagnostics.iter().any(|d| d.rule_id == "SEC-002"));
    }

    #[test]
    fn test_everyone_modify_permission() {
        let result = analyze(r#"<Wix><Permission User="Everyone" Modify="yes" /></Wix>"#);
        assert!(result.diagnostics.iter().any(|d| d.rule_id == "SEC-002"));
    }

    #[test]
    fn test_everyone_fullcontrol_permission() {
        let result = analyze(r#"<Wix><Permission User="Everyone" FullControl="yes" /></Wix>"#);
        assert!(result.diagnostics.iter().any(|d| d.rule_id == "SEC-002"));
    }

    #[test]
    fn test_everyone_genericall_permission() {
        let result = analyze(r#"<Wix><Permission User="Everyone" GenericAll="yes" /></Wix>"#);
        assert!(result.diagnostics.iter().any(|d| d.rule_id == "SEC-002"));
    }

    #[test]
    fn test_everyone_sid_permission() {
        // S-1-1-0 is the SID for Everyone
        let result = analyze(r#"<Wix><Permission User="*S-1-1-0" GenericWrite="yes" /></Wix>"#);
        assert!(result.diagnostics.iter().any(|d| d.rule_id == "SEC-002"));
    }

    #[test]
    fn test_permissionex_everyone_write() {
        let result = analyze(r#"<Wix><PermissionEx User="Everyone" GenericWrite="yes" /></Wix>"#);
        assert!(result.diagnostics.iter().any(|d| d.rule_id == "SEC-002"));
    }

    #[test]
    fn test_everyone_read_only_ok() {
        let result = analyze(r#"<Wix><Permission User="Everyone" Read="yes" /></Wix>"#);
        assert!(result.diagnostics.iter().all(|d| d.rule_id != "SEC-002"));
    }

    #[test]
    fn test_registry_hklm_users_write() {
        let result = analyze(r#"<Wix>
            <RegistryKey Root="HKLM" Key="Software\Test">
                <Permission User="Users" Write="yes" />
            </RegistryKey>
        </Wix>"#);
        assert!(result.diagnostics.iter().any(|d| d.rule_id == "SEC-003"));
    }

    #[test]
    fn test_registry_hklm_everyone_fullcontrol() {
        let result = analyze(r#"<Wix>
            <RegistryKey Root="HKLM" Key="Software\Test">
                <Permission User="Everyone" FullControl="yes" />
            </RegistryKey>
        </Wix>"#);
        assert!(result.diagnostics.iter().any(|d| d.rule_id == "SEC-003"));
    }

    #[test]
    fn test_registry_value_hklm_weak_permission() {
        let result = analyze(r#"<Wix>
            <RegistryValue Root="HKLM" Key="Software\Test" Name="Val">
                <Permission User="Users" Write="yes" />
            </RegistryValue>
        </Wix>"#);
        assert!(result.diagnostics.iter().any(|d| d.rule_id == "SEC-003"));
    }

    #[test]
    fn test_registry_hkcu_ok() {
        // HKCU doesn't trigger SEC-003
        let result = analyze(r#"<Wix>
            <RegistryKey Root="HKCU" Key="Software\Test">
                <Permission User="Users" Write="yes" />
            </RegistryKey>
        </Wix>"#);
        assert!(result.diagnostics.iter().all(|d| d.rule_id != "SEC-003"));
    }

    #[test]
    fn test_sensitive_property() {
        let result = analyze(r#"<Wix><Property Id="DB_PASSWORD" Value="secret123" /></Wix>"#);
        assert!(result.diagnostics.iter().any(|d| d.rule_id == "SEC-005"));
    }

    #[test]
    fn test_sensitive_property_various_patterns() {
        // Test various sensitive patterns
        let patterns = ["SECRET", "API_KEY", "TOKEN", "CREDENTIAL", "APIKEY", "AUTH_TOKEN"];
        for pattern in patterns {
            let xml = format!(r#"<Wix><Property Id="{}" Value="hardcoded" /></Wix>"#, pattern);
            let result = analyze(&xml);
            assert!(
                result.diagnostics.iter().any(|d| d.rule_id == "SEC-005"),
                "Pattern {} should trigger SEC-005",
                pattern
            );
        }
    }

    #[test]
    fn test_sensitive_property_no_value() {
        let result = analyze(r#"<Wix><Property Id="DB_PASSWORD" /></Wix>"#);
        assert!(result.diagnostics.iter().all(|d| d.rule_id != "SEC-005"));
    }

    #[test]
    fn test_sensitive_property_empty_value() {
        let result = analyze(r#"<Wix><Property Id="DB_PASSWORD" Value="" /></Wix>"#);
        assert!(result.diagnostics.iter().all(|d| d.rule_id != "SEC-005"));
    }

    #[test]
    fn test_sensitive_property_reference_ok() {
        // Property references are OK
        let result = analyze(r#"<Wix><Property Id="DB_PASSWORD" Value="[ACTUAL_PASSWORD]" /></Wix>"#);
        assert!(result.diagnostics.iter().all(|d| d.rule_id != "SEC-005"));
    }

    #[test]
    fn test_sensitive_property_format_specifier_ok() {
        // Format specifiers are OK
        let result = analyze(r#"<Wix><Property Id="DB_PASSWORD" Value="!(wix.SomeVar)" /></Wix>"#);
        assert!(result.diagnostics.iter().all(|d| d.rule_id != "SEC-005"));
    }

    #[test]
    fn test_normal_property_ok() {
        let result = analyze(r#"<Wix><Property Id="INSTALLDIR" Value="C:\Program Files\App" /></Wix>"#);
        assert!(result.diagnostics.iter().all(|d| d.rule_id != "SEC-005"));
    }

    #[test]
    fn test_elevated_script_action() {
        let result = analyze(r#"<Wix><CustomAction Id="CA1" Execute="deferred" Impersonate="no" Script="vbscript" /></Wix>"#);
        assert!(result.diagnostics.iter().any(|d| d.rule_id == "SEC-004"));
    }

    #[test]
    fn test_elevated_vbscriptcall_action() {
        let result = analyze(r#"<Wix><CustomAction Id="CA1" Execute="deferred" Impersonate="no" VBScriptCall="MyFunc" /></Wix>"#);
        assert!(result.diagnostics.iter().any(|d| d.rule_id == "SEC-004"));
    }

    #[test]
    fn test_elevated_jscriptcall_action() {
        let result = analyze(r#"<Wix><CustomAction Id="CA1" Execute="deferred" Impersonate="no" JScriptCall="MyFunc" /></Wix>"#);
        assert!(result.diagnostics.iter().any(|d| d.rule_id == "SEC-004"));
    }

    #[test]
    fn test_elevated_exe_action_ok() {
        // Non-script elevated actions don't trigger SEC-004
        let result = analyze(r#"<Wix><CustomAction Id="CA1" Execute="deferred" Impersonate="no" ExeCommand="cmd" /></Wix>"#);
        assert!(result.diagnostics.iter().all(|d| d.rule_id != "SEC-004"));
    }

    #[test]
    fn test_immediate_script_ok() {
        // Immediate actions don't trigger SEC-004
        let result = analyze(r#"<Wix><CustomAction Id="CA1" Execute="immediate" Script="vbscript" /></Wix>"#);
        assert!(result.diagnostics.iter().all(|d| d.rule_id != "SEC-004"));
    }

    #[test]
    fn test_deferred_impersonate_yes_ok() {
        // Impersonate="yes" doesn't run elevated
        let result = analyze(r#"<Wix><CustomAction Id="CA1" Execute="deferred" Impersonate="yes" Script="vbscript" /></Wix>"#);
        assert!(result.diagnostics.iter().all(|d| d.rule_id != "SEC-004"));
    }

    #[test]
    fn test_world_writable_directory() {
        let result = analyze(r#"<Wix>
            <Directory Id="D1">
                <Permission User="Everyone" GenericWrite="yes" />
            </Directory>
        </Wix>"#);
        assert!(result.diagnostics.iter().any(|d| d.rule_id == "SEC-007"));
    }

    #[test]
    fn test_world_writable_directory_write() {
        let result = analyze(r#"<Wix>
            <Directory Id="D1">
                <Permission User="Everyone" Write="yes" />
            </Directory>
        </Wix>"#);
        assert!(result.diagnostics.iter().any(|d| d.rule_id == "SEC-007"));
    }

    #[test]
    fn test_world_writable_directory_modify() {
        let result = analyze(r#"<Wix>
            <Directory Id="D1">
                <Permission User="Everyone" Modify="yes" />
            </Directory>
        </Wix>"#);
        assert!(result.diagnostics.iter().any(|d| d.rule_id == "SEC-007"));
    }

    #[test]
    fn test_users_writable_directory() {
        let result = analyze(r#"<Wix>
            <Directory Id="D1">
                <Permission User="Users" GenericWrite="yes" />
            </Directory>
        </Wix>"#);
        assert!(result.diagnostics.iter().any(|d| d.rule_id == "SEC-007"));
    }

    #[test]
    fn test_standard_directory_world_writable() {
        let result = analyze(r#"<Wix>
            <StandardDirectory Id="ProgramFilesFolder">
                <Permission User="Everyone" GenericWrite="yes" />
            </StandardDirectory>
        </Wix>"#);
        assert!(result.diagnostics.iter().any(|d| d.rule_id == "SEC-007"));
    }

    #[test]
    fn test_directory_read_only_ok() {
        let result = analyze(r#"<Wix>
            <Directory Id="D1">
                <Permission User="Everyone" Read="yes" />
            </Directory>
        </Wix>"#);
        assert!(result.diagnostics.iter().all(|d| d.rule_id != "SEC-007"));
    }
}

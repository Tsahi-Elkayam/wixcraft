//! Secrets detection for WiX files
//!
//! Finds hardcoded credentials, API keys, tokens, and other sensitive data.

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Type of secret detected
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SecretType {
    /// API key (generic)
    ApiKey,
    /// AWS access key
    AwsAccessKey,
    /// AWS secret key
    AwsSecretKey,
    /// Azure storage key
    AzureKey,
    /// GitHub token
    GitHubToken,
    /// Private key (RSA, SSH, etc.)
    PrivateKey,
    /// Password in plain text
    Password,
    /// Connection string
    ConnectionString,
    /// JWT token
    JwtToken,
    /// Generic secret/token
    GenericSecret,
    /// Slack token
    SlackToken,
    /// Google API key
    GoogleApiKey,
}

impl SecretType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ApiKey => "API Key",
            Self::AwsAccessKey => "AWS Access Key",
            Self::AwsSecretKey => "AWS Secret Key",
            Self::AzureKey => "Azure Key",
            Self::GitHubToken => "GitHub Token",
            Self::PrivateKey => "Private Key",
            Self::Password => "Password",
            Self::ConnectionString => "Connection String",
            Self::JwtToken => "JWT Token",
            Self::GenericSecret => "Generic Secret",
            Self::SlackToken => "Slack Token",
            Self::GoogleApiKey => "Google API Key",
        }
    }

    pub fn severity(&self) -> SecretSeverity {
        match self {
            Self::AwsAccessKey | Self::AwsSecretKey | Self::PrivateKey => SecretSeverity::Critical,
            Self::Password | Self::ConnectionString | Self::AzureKey => SecretSeverity::High,
            Self::GitHubToken | Self::SlackToken | Self::JwtToken => SecretSeverity::High,
            Self::ApiKey | Self::GoogleApiKey => SecretSeverity::Medium,
            Self::GenericSecret => SecretSeverity::Low,
        }
    }
}

/// Severity of the secret exposure
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum SecretSeverity {
    Low = 1,
    Medium = 2,
    High = 3,
    Critical = 4,
}

impl SecretSeverity {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
            Self::Critical => "critical",
        }
    }
}

/// A detected secret
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedSecret {
    /// Type of secret
    pub secret_type: SecretType,
    /// File where found
    pub file: PathBuf,
    /// Line number (1-indexed)
    pub line: usize,
    /// Column where secret starts
    pub column: usize,
    /// The matched text (partially redacted)
    pub match_text: String,
    /// Context (surrounding line, redacted)
    pub context: String,
    /// Severity
    pub severity: SecretSeverity,
    /// Rule ID that matched
    pub rule_id: String,
}

impl DetectedSecret {
    /// Get a redacted version of the match
    pub fn redacted_match(&self) -> String {
        if self.match_text.len() <= 8 {
            "*".repeat(self.match_text.len())
        } else {
            let visible = 4;
            format!(
                "{}{}",
                &self.match_text[..visible],
                "*".repeat(self.match_text.len() - visible)
            )
        }
    }
}

/// Secret detection rule
struct SecretRule {
    id: &'static str,
    secret_type: SecretType,
    pattern: Regex,
    description: &'static str,
}

/// Secrets detector
pub struct SecretsDetector {
    rules: Vec<SecretRule>,
    /// Patterns to exclude (false positives)
    exclusions: Vec<Regex>,
}

impl Default for SecretsDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl SecretsDetector {
    pub fn new() -> Self {
        let rules = vec![
            // AWS
            SecretRule {
                id: "SEC-AWS-001",
                secret_type: SecretType::AwsAccessKey,
                pattern: Regex::new(r"AKIA[0-9A-Z]{16}").unwrap(),
                description: "AWS Access Key ID",
            },
            SecretRule {
                id: "SEC-AWS-002",
                secret_type: SecretType::AwsSecretKey,
                pattern: Regex::new(r#"(?i)(aws_secret|aws_key|secret_key)\s*[=:]\s*["']?[A-Za-z0-9/+=]{40}"#).unwrap(),
                description: "AWS Secret Access Key",
            },
            // Azure
            SecretRule {
                id: "SEC-AZ-001",
                secret_type: SecretType::AzureKey,
                pattern: Regex::new(r#"(?i)(azure|storage)[_-]?(key|secret|account)\s*[=:]\s*["']?[A-Za-z0-9+/=]{44,88}"#).unwrap(),
                description: "Azure Storage Key",
            },
            // GitHub
            SecretRule {
                id: "SEC-GH-001",
                secret_type: SecretType::GitHubToken,
                pattern: Regex::new(r"gh[pousr]_[A-Za-z0-9_]{36,}").unwrap(),
                description: "GitHub Personal Access Token",
            },
            SecretRule {
                id: "SEC-GH-002",
                secret_type: SecretType::GitHubToken,
                pattern: Regex::new(r"github_pat_[A-Za-z0-9_]{22,}").unwrap(),
                description: "GitHub Fine-Grained Token",
            },
            // Slack
            SecretRule {
                id: "SEC-SLACK-001",
                secret_type: SecretType::SlackToken,
                pattern: Regex::new(r"xox[baprs]-[0-9]{10,}-[0-9]{10,}-[a-zA-Z0-9]{24,}").unwrap(),
                description: "Slack Token",
            },
            // Google
            SecretRule {
                id: "SEC-GOOG-001",
                secret_type: SecretType::GoogleApiKey,
                pattern: Regex::new(r"AIza[0-9A-Za-z_-]{35}").unwrap(),
                description: "Google API Key",
            },
            // JWT
            SecretRule {
                id: "SEC-JWT-001",
                secret_type: SecretType::JwtToken,
                pattern: Regex::new(r"eyJ[A-Za-z0-9_-]+\.eyJ[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+").unwrap(),
                description: "JWT Token",
            },
            // Private Keys
            SecretRule {
                id: "SEC-KEY-001",
                secret_type: SecretType::PrivateKey,
                pattern: Regex::new(r"-----BEGIN\s+(RSA\s+)?PRIVATE\s+KEY-----").unwrap(),
                description: "Private Key Header",
            },
            SecretRule {
                id: "SEC-KEY-002",
                secret_type: SecretType::PrivateKey,
                pattern: Regex::new(r"-----BEGIN\s+OPENSSH\s+PRIVATE\s+KEY-----").unwrap(),
                description: "OpenSSH Private Key",
            },
            // Passwords
            SecretRule {
                id: "SEC-PWD-001",
                secret_type: SecretType::Password,
                pattern: Regex::new(r#"(?i)(password|passwd|pwd)\s*[=:]\s*["'][^"']{8,}["']"#).unwrap(),
                description: "Hardcoded Password",
            },
            SecretRule {
                id: "SEC-PWD-002",
                secret_type: SecretType::Password,
                pattern: Regex::new(r#"(?i)Password\s*=\s*["'][^"']{4,}["']"#).unwrap(),
                description: "Password in attribute",
            },
            // Connection Strings
            SecretRule {
                id: "SEC-CONN-001",
                secret_type: SecretType::ConnectionString,
                pattern: Regex::new(r#"(?i)(connection_?string|connstr)\s*[=:]\s*["'][^"']+password[^"']+["']"#).unwrap(),
                description: "Connection String with Password",
            },
            SecretRule {
                id: "SEC-CONN-002",
                secret_type: SecretType::ConnectionString,
                pattern: Regex::new(r"Server=.+;.*Password=.+;").unwrap(),
                description: "SQL Connection String",
            },
            // Generic API Keys
            SecretRule {
                id: "SEC-API-001",
                secret_type: SecretType::ApiKey,
                pattern: Regex::new(r#"(?i)(api[_-]?key|apikey)\s*[=:]\s*["'][A-Za-z0-9_-]{20,}["']"#).unwrap(),
                description: "Generic API Key",
            },
            SecretRule {
                id: "SEC-API-002",
                secret_type: SecretType::ApiKey,
                pattern: Regex::new(r#"(?i)(secret|token)\s*[=:]\s*["'][A-Za-z0-9_-]{20,}["']"#).unwrap(),
                description: "Generic Secret/Token",
            },
        ];

        let exclusions = vec![
            // Example/placeholder values
            Regex::new(r"(?i)(example|sample|test|dummy|placeholder|your[_-]?)").unwrap(),
            // Variable references
            Regex::new(r"\$\([^)]+\)").unwrap(),
            Regex::new(r"\$\{[^}]+\}").unwrap(),
            // XML entity references
            Regex::new(r"&[a-zA-Z]+;").unwrap(),
        ];

        Self { rules, exclusions }
    }

    /// Scan a file for secrets
    pub fn scan_file(&self, path: &str, content: &str) -> Vec<DetectedSecret> {
        let mut secrets = Vec::new();

        for (line_num, line) in content.lines().enumerate() {
            for rule in &self.rules {
                for mat in rule.pattern.find_iter(line) {
                    let match_text = mat.as_str();

                    // Check exclusions
                    if self.is_excluded(match_text, line) {
                        continue;
                    }

                    secrets.push(DetectedSecret {
                        secret_type: rule.secret_type,
                        file: PathBuf::from(path),
                        line: line_num + 1,
                        column: mat.start() + 1,
                        match_text: match_text.to_string(),
                        context: self.redact_line(line, mat.start(), mat.end()),
                        severity: rule.secret_type.severity(),
                        rule_id: rule.id.to_string(),
                    });
                }
            }
        }

        secrets
    }

    /// Check if a match should be excluded (likely false positive)
    fn is_excluded(&self, match_text: &str, line: &str) -> bool {
        for exclusion in &self.exclusions {
            if exclusion.is_match(match_text) || exclusion.is_match(line) {
                return true;
            }
        }
        false
    }

    /// Redact a line, showing context but hiding the secret
    fn redact_line(&self, line: &str, start: usize, end: usize) -> String {
        let before = &line[..start];
        let secret = &line[start..end];
        let after = &line[end..];

        let redacted_secret = if secret.len() <= 8 {
            "*".repeat(secret.len())
        } else {
            format!("{}****", &secret[..4])
        };

        format!("{}{}{}", before, redacted_secret, after)
    }
}

/// Result of secrets scan
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SecretsResult {
    /// All detected secrets
    pub secrets: Vec<DetectedSecret>,
    /// Count by type
    pub by_type: std::collections::HashMap<String, usize>,
    /// Count by severity
    pub critical_count: usize,
    pub high_count: usize,
    pub medium_count: usize,
    pub low_count: usize,
}

impl SecretsResult {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, secret: DetectedSecret) {
        *self
            .by_type
            .entry(secret.secret_type.as_str().to_string())
            .or_insert(0) += 1;

        match secret.severity {
            SecretSeverity::Critical => self.critical_count += 1,
            SecretSeverity::High => self.high_count += 1,
            SecretSeverity::Medium => self.medium_count += 1,
            SecretSeverity::Low => self.low_count += 1,
        }

        self.secrets.push(secret);
    }

    pub fn total(&self) -> usize {
        self.secrets.len()
    }

    pub fn has_critical(&self) -> bool {
        self.critical_count > 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_secrets() {
        let detector = SecretsDetector::new();
        let secrets = detector.scan_file(
            "test.wxs",
            r#"<Wix>
    <Package Name="Test" />
</Wix>"#,
        );

        assert!(secrets.is_empty());
    }

    #[test]
    fn test_detect_aws_key() {
        let detector = SecretsDetector::new();
        // AWS Access Key IDs are exactly AKIA + 16 characters
        let secrets = detector.scan_file(
            "test.wxs",
            r#"<Property Id="AWS_KEY" Value="AKIAIOSFODNN7EXAMPAA" />"#,
        );

        assert!(!secrets.is_empty());
        assert_eq!(secrets[0].secret_type, SecretType::AwsAccessKey);
        assert_eq!(secrets[0].severity, SecretSeverity::Critical);
    }

    #[test]
    fn test_detect_github_token() {
        let detector = SecretsDetector::new();
        let secrets = detector.scan_file(
            "test.wxs",
            r#"<Property Id="GH_TOKEN" Value="ghp_xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx" />"#,
        );

        assert!(!secrets.is_empty());
        assert_eq!(secrets[0].secret_type, SecretType::GitHubToken);
    }

    #[test]
    fn test_detect_password() {
        let detector = SecretsDetector::new();
        let secrets = detector.scan_file(
            "test.wxs",
            r#"<Property Id="DB_CONN" Value="Password='SuperSecret123'" />"#,
        );

        assert!(!secrets.is_empty());
        assert_eq!(secrets[0].secret_type, SecretType::Password);
    }

    #[test]
    fn test_detect_jwt() {
        let detector = SecretsDetector::new();
        let secrets = detector.scan_file(
            "test.wxs",
            r#"<Property Id="TOKEN" Value="eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.dozjgNryP4J3jVmNHl0w5N_XgL0n3I9PlFUP0THsR8U" />"#,
        );

        assert!(!secrets.is_empty());
        assert_eq!(secrets[0].secret_type, SecretType::JwtToken);
    }

    #[test]
    fn test_detect_private_key() {
        let detector = SecretsDetector::new();
        let secrets = detector.scan_file("test.wxs", r#"-----BEGIN RSA PRIVATE KEY-----"#);

        assert!(!secrets.is_empty());
        assert_eq!(secrets[0].secret_type, SecretType::PrivateKey);
        assert_eq!(secrets[0].severity, SecretSeverity::Critical);
    }

    #[test]
    fn test_detect_api_key() {
        let detector = SecretsDetector::new();
        let secrets = detector.scan_file(
            "test.wxs",
            r#"<Property Id="KEY" Value="api_key='abcdefghij1234567890klmnop'" />"#,
        );

        assert!(!secrets.is_empty());
        assert_eq!(secrets[0].secret_type, SecretType::ApiKey);
    }

    #[test]
    fn test_exclude_placeholders() {
        let detector = SecretsDetector::new();
        let secrets = detector.scan_file(
            "test.wxs",
            r#"<Property Id="KEY" Value="api_key='your_api_key_here'" />"#,
        );

        // Should be excluded as placeholder
        assert!(secrets.is_empty());
    }

    #[test]
    fn test_exclude_variables() {
        let detector = SecretsDetector::new();
        let secrets =
            detector.scan_file("test.wxs", r#"<Property Id="KEY" Value="$(var.ApiKey)" />"#);

        assert!(secrets.is_empty());
    }

    #[test]
    fn test_redacted_match() {
        let secret = DetectedSecret {
            secret_type: SecretType::ApiKey,
            file: PathBuf::from("test.wxs"),
            line: 1,
            column: 1,
            match_text: "AKIAIOSFODNN7EXAMPLE".to_string(),
            context: "...".to_string(),
            severity: SecretSeverity::High,
            rule_id: "SEC-001".to_string(),
        };

        let redacted = secret.redacted_match();
        assert!(redacted.starts_with("AKIA"));
        assert!(redacted.contains("*"));
    }

    #[test]
    fn test_secrets_result() {
        let mut result = SecretsResult::new();

        result.add(DetectedSecret {
            secret_type: SecretType::AwsAccessKey,
            file: PathBuf::from("test.wxs"),
            line: 1,
            column: 1,
            match_text: "AKIATEST".to_string(),
            context: "...".to_string(),
            severity: SecretSeverity::Critical,
            rule_id: "SEC-001".to_string(),
        });

        result.add(DetectedSecret {
            secret_type: SecretType::Password,
            file: PathBuf::from("test.wxs"),
            line: 2,
            column: 1,
            match_text: "secret".to_string(),
            context: "...".to_string(),
            severity: SecretSeverity::High,
            rule_id: "SEC-002".to_string(),
        });

        assert_eq!(result.total(), 2);
        assert_eq!(result.critical_count, 1);
        assert_eq!(result.high_count, 1);
        assert!(result.has_critical());
    }

    #[test]
    fn test_secret_type_severity() {
        assert_eq!(
            SecretType::AwsAccessKey.severity(),
            SecretSeverity::Critical
        );
        assert_eq!(SecretType::Password.severity(), SecretSeverity::High);
        assert_eq!(SecretType::ApiKey.severity(), SecretSeverity::Medium);
        assert_eq!(SecretType::GenericSecret.severity(), SecretSeverity::Low);
    }

    #[test]
    fn test_google_api_key() {
        let detector = SecretsDetector::new();
        let secrets = detector.scan_file(
            "test.wxs",
            r#"<Property Id="GAPI" Value="AIzaSyDaGmWKa4JsXZ-HjGw7ISLn_3namBGewQe" />"#,
        );

        assert!(!secrets.is_empty());
        assert_eq!(secrets[0].secret_type, SecretType::GoogleApiKey);
    }

    // Note: Slack token test removed - any pattern-matching token triggers GitHub's secret scanner

    #[test]
    fn test_connection_string() {
        let detector = SecretsDetector::new();
        let secrets = detector.scan_file(
            "test.wxs",
            r#"<Property Id="CONN" Value="Server=localhost;Database=mydb;User=sa;Password=secret123;" />"#,
        );

        // Should detect either connection string or password pattern
        assert!(!secrets.is_empty());
        let has_relevant = secrets.iter().any(|s| {
            s.secret_type == SecretType::ConnectionString || s.secret_type == SecretType::Password
        });
        assert!(has_relevant);
    }

    #[test]
    fn test_multiple_secrets_one_file() {
        let detector = SecretsDetector::new();
        let secrets = detector.scan_file(
            "test.wxs",
            r#"<Wix>
    <Property Id="AWS" Value="AKIAIOSFODNN7EXAMPAA" />
    <Property Id="GH" Value="ghp_xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx" />
</Wix>"#,
        );

        assert!(secrets.len() >= 2);
    }
}

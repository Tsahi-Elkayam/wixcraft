//! Code signing helper for MSI packages
//!
//! Provides a unified interface for signing MSI files using SignTool, Azure SignTool,
//! or other signing providers.
//!
//! # Example
//!
//! ```
//! use wix_sign::{SignConfig, SignProvider, TimestampServer};
//!
//! let config = SignConfig {
//!     provider: SignProvider::SignTool,
//!     certificate_path: Some("cert.pfx".into()),
//!     password: Some("secret".into()),
//!     timestamp_server: TimestampServer::DigiCert,
//!     ..Default::default()
//! };
//!
//! let command = config.build_command("installer.msi");
//! assert!(command.contains("signtool"));
//! ```

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;

/// Signing errors
#[derive(Error, Debug)]
pub enum SignError {
    #[error("Certificate not found: {0}")]
    CertificateNotFound(String),
    #[error("Signing failed: {0}")]
    SigningFailed(String),
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
    #[error("Provider not available: {0}")]
    ProviderNotAvailable(String),
}

/// Code signing provider
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum SignProvider {
    #[default]
    SignTool,
    AzureSignTool,
    Osslsigncode,
}

impl SignProvider {
    pub fn as_str(&self) -> &'static str {
        match self {
            SignProvider::SignTool => "signtool",
            SignProvider::AzureSignTool => "azuresigntool",
            SignProvider::Osslsigncode => "osslsigncode",
        }
    }

    pub fn executable(&self) -> &'static str {
        match self {
            SignProvider::SignTool => "signtool",
            SignProvider::AzureSignTool => "AzureSignTool",
            SignProvider::Osslsigncode => "osslsigncode",
        }
    }

    pub fn is_windows_only(&self) -> bool {
        matches!(self, SignProvider::SignTool)
    }
}

/// Timestamp server
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum TimestampServer {
    #[default]
    DigiCert,
    Sectigo,
    GlobalSign,
    Custom(String),
    None,
}

impl TimestampServer {
    pub fn url(&self) -> Option<&str> {
        match self {
            TimestampServer::DigiCert => Some("http://timestamp.digicert.com"),
            TimestampServer::Sectigo => Some("http://timestamp.sectigo.com"),
            TimestampServer::GlobalSign => Some("http://timestamp.globalsign.com/tsa/r6advanced1"),
            TimestampServer::Custom(url) => Some(url),
            TimestampServer::None => None,
        }
    }
}

/// Hash algorithm for signing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum HashAlgorithm {
    SHA1,
    #[default]
    SHA256,
    SHA384,
    SHA512,
}

impl HashAlgorithm {
    pub fn as_str(&self) -> &'static str {
        match self {
            HashAlgorithm::SHA1 => "SHA1",
            HashAlgorithm::SHA256 => "SHA256",
            HashAlgorithm::SHA384 => "SHA384",
            HashAlgorithm::SHA512 => "SHA512",
        }
    }
}

/// Certificate source type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CertificateSource {
    /// PFX file with password
    PfxFile { path: PathBuf, password: String },
    /// Windows certificate store
    CertStore { store: String, subject: String },
    /// Azure Key Vault
    AzureKeyVault {
        vault_url: String,
        client_id: String,
        client_secret: String,
        tenant_id: String,
        certificate_name: String,
    },
    /// EV certificate via hardware token
    HardwareToken { thumbprint: String },
}

/// Signing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignConfig {
    /// Signing provider
    pub provider: SignProvider,
    /// Certificate file path (for PFX)
    pub certificate_path: Option<PathBuf>,
    /// Certificate password
    pub password: Option<String>,
    /// Certificate thumbprint (for store-based)
    pub thumbprint: Option<String>,
    /// Certificate store name
    pub store_name: Option<String>,
    /// Certificate subject (for store search)
    pub subject: Option<String>,
    /// Timestamp server
    pub timestamp_server: TimestampServer,
    /// Hash algorithm
    pub hash_algorithm: HashAlgorithm,
    /// Description for signing
    pub description: Option<String>,
    /// Description URL
    pub description_url: Option<String>,
    /// Append signature (for dual signing)
    pub append_signature: bool,
    /// Verbose output
    pub verbose: bool,
    /// Azure Key Vault URL (for Azure SignTool)
    pub azure_vault_url: Option<String>,
    /// Azure client ID
    pub azure_client_id: Option<String>,
    /// Azure tenant ID
    pub azure_tenant_id: Option<String>,
}

impl Default for SignConfig {
    fn default() -> Self {
        Self {
            provider: SignProvider::SignTool,
            certificate_path: None,
            password: None,
            thumbprint: None,
            store_name: None,
            subject: None,
            timestamp_server: TimestampServer::DigiCert,
            hash_algorithm: HashAlgorithm::SHA256,
            description: None,
            description_url: None,
            append_signature: false,
            verbose: false,
            azure_vault_url: None,
            azure_client_id: None,
            azure_tenant_id: None,
        }
    }
}

impl SignConfig {
    /// Create a new signing config with PFX file
    pub fn with_pfx(path: impl Into<PathBuf>, password: impl Into<String>) -> Self {
        Self {
            certificate_path: Some(path.into()),
            password: Some(password.into()),
            ..Default::default()
        }
    }

    /// Create a new signing config with certificate store
    pub fn with_store(store: impl Into<String>, thumbprint: impl Into<String>) -> Self {
        Self {
            store_name: Some(store.into()),
            thumbprint: Some(thumbprint.into()),
            ..Default::default()
        }
    }

    /// Create a new signing config for Azure SignTool
    pub fn with_azure(
        vault_url: impl Into<String>,
        certificate_name: impl Into<String>,
        client_id: impl Into<String>,
        tenant_id: impl Into<String>,
    ) -> Self {
        Self {
            provider: SignProvider::AzureSignTool,
            azure_vault_url: Some(vault_url.into()),
            azure_client_id: Some(client_id.into()),
            azure_tenant_id: Some(tenant_id.into()),
            description: Some(certificate_name.into()),
            ..Default::default()
        }
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<(), SignError> {
        match self.provider {
            SignProvider::SignTool => {
                if self.certificate_path.is_none()
                    && self.thumbprint.is_none()
                    && self.subject.is_none()
                {
                    return Err(SignError::InvalidConfig(
                        "SignTool requires certificate_path, thumbprint, or subject".to_string(),
                    ));
                }
            }
            SignProvider::AzureSignTool => {
                if self.azure_vault_url.is_none() {
                    return Err(SignError::InvalidConfig(
                        "AzureSignTool requires azure_vault_url".to_string(),
                    ));
                }
            }
            SignProvider::Osslsigncode => {
                if self.certificate_path.is_none() {
                    return Err(SignError::InvalidConfig(
                        "osslsigncode requires certificate_path".to_string(),
                    ));
                }
            }
        }
        Ok(())
    }

    /// Build the signing command
    pub fn build_command(&self, file: &str) -> String {
        match self.provider {
            SignProvider::SignTool => self.build_signtool_command(file),
            SignProvider::AzureSignTool => self.build_azure_signtool_command(file),
            SignProvider::Osslsigncode => self.build_osslsigncode_command(file),
        }
    }

    /// Build command arguments as a vector
    pub fn build_args(&self, file: &str) -> Vec<String> {
        match self.provider {
            SignProvider::SignTool => self.signtool_args(file),
            SignProvider::AzureSignTool => self.azure_signtool_args(file),
            SignProvider::Osslsigncode => self.osslsigncode_args(file),
        }
    }

    fn build_signtool_command(&self, file: &str) -> String {
        let mut cmd = String::from("signtool sign");

        if let Some(ref path) = self.certificate_path {
            cmd.push_str(&format!(" /f \"{}\"", path.display()));
        }

        if let Some(ref pwd) = self.password {
            cmd.push_str(&format!(" /p \"{}\"", pwd));
        }

        if let Some(ref thumb) = self.thumbprint {
            cmd.push_str(&format!(" /sha1 {}", thumb));
        }

        if let Some(ref store) = self.store_name {
            cmd.push_str(&format!(" /s {}", store));
        }

        if let Some(ref subj) = self.subject {
            cmd.push_str(&format!(" /n \"{}\"", subj));
        }

        cmd.push_str(&format!(" /fd {}", self.hash_algorithm.as_str()));

        if let Some(url) = self.timestamp_server.url() {
            cmd.push_str(&format!(" /tr {} /td {}", url, self.hash_algorithm.as_str()));
        }

        if let Some(ref desc) = self.description {
            cmd.push_str(&format!(" /d \"{}\"", desc));
        }

        if let Some(ref url) = self.description_url {
            cmd.push_str(&format!(" /du \"{}\"", url));
        }

        if self.append_signature {
            cmd.push_str(" /as");
        }

        if self.verbose {
            cmd.push_str(" /v");
        }

        cmd.push_str(&format!(" \"{}\"", file));

        cmd
    }

    fn signtool_args(&self, file: &str) -> Vec<String> {
        let mut args = vec!["sign".to_string()];

        if let Some(ref path) = self.certificate_path {
            args.push("/f".to_string());
            args.push(path.display().to_string());
        }

        if let Some(ref pwd) = self.password {
            args.push("/p".to_string());
            args.push(pwd.clone());
        }

        if let Some(ref thumb) = self.thumbprint {
            args.push("/sha1".to_string());
            args.push(thumb.clone());
        }

        if let Some(ref store) = self.store_name {
            args.push("/s".to_string());
            args.push(store.clone());
        }

        if let Some(ref subj) = self.subject {
            args.push("/n".to_string());
            args.push(subj.clone());
        }

        args.push("/fd".to_string());
        args.push(self.hash_algorithm.as_str().to_string());

        if let Some(url) = self.timestamp_server.url() {
            args.push("/tr".to_string());
            args.push(url.to_string());
            args.push("/td".to_string());
            args.push(self.hash_algorithm.as_str().to_string());
        }

        if let Some(ref desc) = self.description {
            args.push("/d".to_string());
            args.push(desc.clone());
        }

        if let Some(ref url) = self.description_url {
            args.push("/du".to_string());
            args.push(url.clone());
        }

        if self.append_signature {
            args.push("/as".to_string());
        }

        if self.verbose {
            args.push("/v".to_string());
        }

        args.push(file.to_string());

        args
    }

    fn build_azure_signtool_command(&self, file: &str) -> String {
        let mut cmd = String::from("AzureSignTool sign");

        if let Some(ref vault) = self.azure_vault_url {
            cmd.push_str(&format!(" -kvu {}", vault));
        }

        if let Some(ref client) = self.azure_client_id {
            cmd.push_str(&format!(" -kvi {}", client));
        }

        if let Some(ref tenant) = self.azure_tenant_id {
            cmd.push_str(&format!(" -kvt {}", tenant));
        }

        cmd.push_str(&format!(" -fd {}", self.hash_algorithm.as_str().to_lowercase()));

        if let Some(url) = self.timestamp_server.url() {
            cmd.push_str(&format!(" -tr {}", url));
        }

        if let Some(ref desc) = self.description {
            cmd.push_str(&format!(" -d \"{}\"", desc));
        }

        if self.verbose {
            cmd.push_str(" -v");
        }

        cmd.push_str(&format!(" \"{}\"", file));

        cmd
    }

    fn azure_signtool_args(&self, file: &str) -> Vec<String> {
        let mut args = vec!["sign".to_string()];

        if let Some(ref vault) = self.azure_vault_url {
            args.push("-kvu".to_string());
            args.push(vault.clone());
        }

        if let Some(ref client) = self.azure_client_id {
            args.push("-kvi".to_string());
            args.push(client.clone());
        }

        if let Some(ref tenant) = self.azure_tenant_id {
            args.push("-kvt".to_string());
            args.push(tenant.clone());
        }

        args.push("-fd".to_string());
        args.push(self.hash_algorithm.as_str().to_lowercase());

        if let Some(url) = self.timestamp_server.url() {
            args.push("-tr".to_string());
            args.push(url.to_string());
        }

        if let Some(ref desc) = self.description {
            args.push("-d".to_string());
            args.push(desc.clone());
        }

        if self.verbose {
            args.push("-v".to_string());
        }

        args.push(file.to_string());

        args
    }

    fn build_osslsigncode_command(&self, file: &str) -> String {
        let mut cmd = String::from("osslsigncode sign");

        if let Some(ref path) = self.certificate_path {
            cmd.push_str(&format!(" -pkcs12 \"{}\"", path.display()));
        }

        if let Some(ref pwd) = self.password {
            cmd.push_str(&format!(" -pass \"{}\"", pwd));
        }

        cmd.push_str(&format!(
            " -h {}",
            self.hash_algorithm.as_str().to_lowercase()
        ));

        if let Some(url) = self.timestamp_server.url() {
            cmd.push_str(&format!(" -ts {}", url));
        }

        if let Some(ref desc) = self.description {
            cmd.push_str(&format!(" -n \"{}\"", desc));
        }

        if let Some(ref url) = self.description_url {
            cmd.push_str(&format!(" -i \"{}\"", url));
        }

        cmd.push_str(&format!(" -in \"{}\" -out \"{}.signed\"", file, file));

        cmd
    }

    fn osslsigncode_args(&self, file: &str) -> Vec<String> {
        let mut args = vec!["sign".to_string()];

        if let Some(ref path) = self.certificate_path {
            args.push("-pkcs12".to_string());
            args.push(path.display().to_string());
        }

        if let Some(ref pwd) = self.password {
            args.push("-pass".to_string());
            args.push(pwd.clone());
        }

        args.push("-h".to_string());
        args.push(self.hash_algorithm.as_str().to_lowercase());

        if let Some(url) = self.timestamp_server.url() {
            args.push("-ts".to_string());
            args.push(url.to_string());
        }

        if let Some(ref desc) = self.description {
            args.push("-n".to_string());
            args.push(desc.clone());
        }

        if let Some(ref url) = self.description_url {
            args.push("-i".to_string());
            args.push(url.clone());
        }

        args.push("-in".to_string());
        args.push(file.to_string());
        args.push("-out".to_string());
        args.push(format!("{}.signed", file));

        args
    }
}

/// Result of signing operation
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SignResult {
    pub success: bool,
    pub file: String,
    pub provider: String,
    pub output: String,
    pub error: Option<String>,
}

/// Verify if a file is signed
pub fn is_signed(_file: &str) -> bool {
    // Placeholder - actual implementation would check PE signature
    false
}

/// Get signing information from a file
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SignatureInfo {
    pub is_signed: bool,
    pub subject: Option<String>,
    pub issuer: Option<String>,
    pub timestamp: Option<String>,
    pub algorithm: Option<String>,
    pub valid: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = SignConfig::default();

        assert_eq!(config.provider, SignProvider::SignTool);
        assert_eq!(config.hash_algorithm, HashAlgorithm::SHA256);
        assert_eq!(config.timestamp_server, TimestampServer::DigiCert);
        assert!(!config.append_signature);
    }

    #[test]
    fn test_with_pfx() {
        let config = SignConfig::with_pfx("cert.pfx", "password123");

        assert_eq!(
            config.certificate_path,
            Some(PathBuf::from("cert.pfx"))
        );
        assert_eq!(config.password, Some("password123".to_string()));
    }

    #[test]
    fn test_with_store() {
        let config = SignConfig::with_store("My", "ABC123");

        assert_eq!(config.store_name, Some("My".to_string()));
        assert_eq!(config.thumbprint, Some("ABC123".to_string()));
    }

    #[test]
    fn test_build_signtool_command_pfx() {
        let config = SignConfig {
            certificate_path: Some(PathBuf::from("cert.pfx")),
            password: Some("secret".to_string()),
            ..Default::default()
        };

        let cmd = config.build_command("installer.msi");

        assert!(cmd.contains("signtool sign"));
        assert!(cmd.contains("/f \"cert.pfx\""));
        assert!(cmd.contains("/p \"secret\""));
        assert!(cmd.contains("/fd SHA256"));
        assert!(cmd.contains("installer.msi"));
    }

    #[test]
    fn test_build_signtool_command_store() {
        let config = SignConfig {
            store_name: Some("My".to_string()),
            thumbprint: Some("ABC123".to_string()),
            ..Default::default()
        };

        let cmd = config.build_command("installer.msi");

        assert!(cmd.contains("/s My"));
        assert!(cmd.contains("/sha1 ABC123"));
    }

    #[test]
    fn test_timestamp_servers() {
        assert_eq!(
            TimestampServer::DigiCert.url(),
            Some("http://timestamp.digicert.com")
        );
        assert_eq!(
            TimestampServer::Sectigo.url(),
            Some("http://timestamp.sectigo.com")
        );
        assert_eq!(
            TimestampServer::Custom("http://custom.ts".to_string()).url(),
            Some("http://custom.ts")
        );
        assert_eq!(TimestampServer::None.url(), None);
    }

    #[test]
    fn test_hash_algorithms() {
        assert_eq!(HashAlgorithm::SHA256.as_str(), "SHA256");
        assert_eq!(HashAlgorithm::SHA1.as_str(), "SHA1");
        assert_eq!(HashAlgorithm::SHA384.as_str(), "SHA384");
        assert_eq!(HashAlgorithm::SHA512.as_str(), "SHA512");
    }

    #[test]
    fn test_append_signature() {
        let config = SignConfig {
            certificate_path: Some(PathBuf::from("cert.pfx")),
            append_signature: true,
            ..Default::default()
        };

        let cmd = config.build_command("installer.msi");

        assert!(cmd.contains("/as"));
    }

    #[test]
    fn test_description_and_url() {
        let config = SignConfig {
            certificate_path: Some(PathBuf::from("cert.pfx")),
            description: Some("My App".to_string()),
            description_url: Some("https://example.com".to_string()),
            ..Default::default()
        };

        let cmd = config.build_command("installer.msi");

        assert!(cmd.contains("/d \"My App\""));
        assert!(cmd.contains("/du \"https://example.com\""));
    }

    #[test]
    fn test_azure_signtool_command() {
        let config = SignConfig {
            provider: SignProvider::AzureSignTool,
            azure_vault_url: Some("https://myvault.vault.azure.net".to_string()),
            azure_client_id: Some("client-id".to_string()),
            azure_tenant_id: Some("tenant-id".to_string()),
            ..Default::default()
        };

        let cmd = config.build_command("installer.msi");

        assert!(cmd.contains("AzureSignTool sign"));
        assert!(cmd.contains("-kvu https://myvault.vault.azure.net"));
        assert!(cmd.contains("-kvi client-id"));
        assert!(cmd.contains("-kvt tenant-id"));
    }

    #[test]
    fn test_osslsigncode_command() {
        let config = SignConfig {
            provider: SignProvider::Osslsigncode,
            certificate_path: Some(PathBuf::from("cert.pfx")),
            password: Some("secret".to_string()),
            ..Default::default()
        };

        let cmd = config.build_command("installer.msi");

        assert!(cmd.contains("osslsigncode sign"));
        assert!(cmd.contains("-pkcs12 \"cert.pfx\""));
        assert!(cmd.contains("-pass \"secret\""));
        assert!(cmd.contains("-in \"installer.msi\""));
    }

    #[test]
    fn test_validate_signtool_valid() {
        let config = SignConfig::with_pfx("cert.pfx", "password");
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validate_signtool_invalid() {
        let config = SignConfig::default();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_azure_signtool_invalid() {
        let config = SignConfig {
            provider: SignProvider::AzureSignTool,
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_provider_names() {
        assert_eq!(SignProvider::SignTool.as_str(), "signtool");
        assert_eq!(SignProvider::AzureSignTool.as_str(), "azuresigntool");
        assert_eq!(SignProvider::Osslsigncode.as_str(), "osslsigncode");
    }

    #[test]
    fn test_provider_executables() {
        assert_eq!(SignProvider::SignTool.executable(), "signtool");
        assert_eq!(SignProvider::AzureSignTool.executable(), "AzureSignTool");
        assert_eq!(SignProvider::Osslsigncode.executable(), "osslsigncode");
    }

    #[test]
    fn test_windows_only_provider() {
        assert!(SignProvider::SignTool.is_windows_only());
        assert!(!SignProvider::AzureSignTool.is_windows_only());
        assert!(!SignProvider::Osslsigncode.is_windows_only());
    }

    #[test]
    fn test_build_args_signtool() {
        let config = SignConfig::with_pfx("cert.pfx", "secret");
        let args = config.build_args("installer.msi");

        assert!(args.contains(&"sign".to_string()));
        assert!(args.contains(&"/f".to_string()));
        assert!(args.contains(&"cert.pfx".to_string()));
    }

    #[test]
    fn test_verbose_mode() {
        let config = SignConfig {
            certificate_path: Some(PathBuf::from("cert.pfx")),
            verbose: true,
            ..Default::default()
        };

        let cmd = config.build_command("installer.msi");

        assert!(cmd.contains("/v"));
    }

    #[test]
    fn test_with_azure() {
        let config = SignConfig::with_azure(
            "https://vault.azure.net",
            "cert-name",
            "client-id",
            "tenant-id",
        );

        assert_eq!(config.provider, SignProvider::AzureSignTool);
        assert_eq!(
            config.azure_vault_url,
            Some("https://vault.azure.net".to_string())
        );
    }

    #[test]
    fn test_sign_result() {
        let result = SignResult {
            success: true,
            file: "test.msi".to_string(),
            provider: "signtool".to_string(),
            output: "Successfully signed".to_string(),
            error: None,
        };

        assert!(result.success);
        assert_eq!(result.file, "test.msi");
    }

    #[test]
    fn test_signature_info_default() {
        let info = SignatureInfo::default();

        assert!(!info.is_signed);
        assert!(!info.valid);
        assert!(info.subject.is_none());
    }

    #[test]
    fn test_subject_signing() {
        let config = SignConfig {
            subject: Some("My Company".to_string()),
            store_name: Some("My".to_string()),
            ..Default::default()
        };

        let cmd = config.build_command("installer.msi");

        assert!(cmd.contains("/n \"My Company\""));
    }
}

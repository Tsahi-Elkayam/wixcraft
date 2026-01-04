//! Helper functions for rule condition evaluation

use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::HashSet;

/// Standard Windows Installer directory IDs
static STANDARD_DIRECTORIES: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    [
        "TARGETDIR",
        "ProgramFilesFolder",
        "ProgramFiles64Folder",
        "ProgramFiles6432Folder",
        "CommonFilesFolder",
        "CommonFiles64Folder",
        "CommonFiles6432Folder",
        "SystemFolder",
        "System64Folder",
        "System6432Folder",
        "WindowsFolder",
        "TempFolder",
        "AppDataFolder",
        "LocalAppDataFolder",
        "ProgramMenuFolder",
        "DesktopFolder",
        "StartMenuFolder",
        "StartupFolder",
        "FontsFolder",
        "PersonalFolder",
        "CommonAppDataFolder",
        "AdminToolsFolder",
        "FavoritesFolder",
        "NetHoodFolder",
        "PrintHoodFolder",
        "RecentFolder",
        "SendToFolder",
        "TemplateFolder",
    ]
    .into_iter()
    .collect()
});

/// Property names that typically contain sensitive data
static SENSITIVE_PROPERTY_PATTERNS: Lazy<Vec<Regex>> = Lazy::new(|| {
    vec![
        Regex::new(r"(?i)password").unwrap(),
        Regex::new(r"(?i)secret").unwrap(),
        Regex::new(r"(?i)key").unwrap(),
        Regex::new(r"(?i)token").unwrap(),
        Regex::new(r"(?i)credential").unwrap(),
        Regex::new(r"(?i)apikey").unwrap(),
        Regex::new(r"(?i)api_key").unwrap(),
    ]
});

/// GUID validation regex
static GUID_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^[{(]?[0-9A-Fa-f]{8}[-]?([0-9A-Fa-f]{4}[-]?){3}[0-9A-Fa-f]{12}[)}]?$").unwrap()
});

/// Windows hardcoded path regex (e.g., C:\, D:\)
static HARDCODED_PATH_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^[A-Za-z]:\\").unwrap()
});

/// Helper functions for rule evaluation
pub struct Helpers;

impl Helpers {
    /// Check if a string is a valid GUID
    pub fn is_valid_guid(s: &str) -> bool {
        if s == "*" {
            // Auto-generated GUID marker is valid
            return true;
        }
        GUID_REGEX.is_match(s)
    }

    /// Check if a directory ID is a standard Windows Installer directory
    pub fn is_standard_directory(id: &str) -> bool {
        STANDARD_DIRECTORIES.contains(id)
    }

    /// Check if a property name suggests sensitive data
    pub fn is_sensitive_property_name(name: &str) -> bool {
        SENSITIVE_PROPERTY_PATTERNS.iter().any(|re| re.is_match(name))
    }

    /// Check if a string looks like a hardcoded path
    pub fn is_hardcoded_path(s: &str) -> bool {
        HARDCODED_PATH_REGEX.is_match(s)
    }

    /// Check if a filename appears to be a .NET assembly
    pub fn looks_like_dotnet_assembly(filename: &str) -> bool {
        let lower = filename.to_lowercase();
        lower.ends_with(".dll") || lower.ends_with(".exe")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_valid_guid() {
        assert!(Helpers::is_valid_guid("*"));
        assert!(Helpers::is_valid_guid("12345678-1234-1234-1234-123456789012"));
        assert!(Helpers::is_valid_guid("{12345678-1234-1234-1234-123456789012}"));
        assert!(Helpers::is_valid_guid("(12345678-1234-1234-1234-123456789012)"));
        assert!(!Helpers::is_valid_guid("not-a-guid"));
        assert!(!Helpers::is_valid_guid(""));
    }

    #[test]
    fn test_is_valid_guid_uppercase() {
        assert!(Helpers::is_valid_guid("ABCDEF01-2345-6789-ABCD-EF0123456789"));
        assert!(Helpers::is_valid_guid("{ABCDEF01-2345-6789-ABCD-EF0123456789}"));
    }

    #[test]
    fn test_is_valid_guid_lowercase() {
        assert!(Helpers::is_valid_guid("abcdef01-2345-6789-abcd-ef0123456789"));
        assert!(Helpers::is_valid_guid("{abcdef01-2345-6789-abcd-ef0123456789}"));
    }

    #[test]
    fn test_is_valid_guid_mixed_case() {
        assert!(Helpers::is_valid_guid("AbCdEf01-2345-6789-AbCd-Ef0123456789"));
    }

    #[test]
    fn test_is_valid_guid_no_dashes() {
        assert!(Helpers::is_valid_guid("12345678123412341234123456789012"));
    }

    #[test]
    fn test_is_valid_guid_invalid() {
        assert!(!Helpers::is_valid_guid("12345678-1234-1234-1234")); // Too short
        assert!(!Helpers::is_valid_guid("GHIJKLMN-1234-1234-1234-123456789012")); // Invalid chars
        assert!(!Helpers::is_valid_guid("not-a-guid-at-all"));
        assert!(!Helpers::is_valid_guid("1234"));
    }

    #[test]
    fn test_is_standard_directory() {
        assert!(Helpers::is_standard_directory("ProgramFilesFolder"));
        assert!(Helpers::is_standard_directory("TARGETDIR"));
        assert!(!Helpers::is_standard_directory("INSTALLFOLDER"));
        assert!(!Helpers::is_standard_directory("MyCustomDir"));
    }

    #[test]
    fn test_is_standard_directory_all() {
        // Test all standard directories
        assert!(Helpers::is_standard_directory("ProgramFilesFolder"));
        assert!(Helpers::is_standard_directory("ProgramFiles64Folder"));
        assert!(Helpers::is_standard_directory("ProgramFiles6432Folder"));
        assert!(Helpers::is_standard_directory("CommonFilesFolder"));
        assert!(Helpers::is_standard_directory("CommonFiles64Folder"));
        assert!(Helpers::is_standard_directory("CommonFiles6432Folder"));
        assert!(Helpers::is_standard_directory("SystemFolder"));
        assert!(Helpers::is_standard_directory("System64Folder"));
        assert!(Helpers::is_standard_directory("System6432Folder"));
        assert!(Helpers::is_standard_directory("WindowsFolder"));
        assert!(Helpers::is_standard_directory("TempFolder"));
        assert!(Helpers::is_standard_directory("AppDataFolder"));
        assert!(Helpers::is_standard_directory("LocalAppDataFolder"));
        assert!(Helpers::is_standard_directory("ProgramMenuFolder"));
        assert!(Helpers::is_standard_directory("DesktopFolder"));
        assert!(Helpers::is_standard_directory("StartMenuFolder"));
        assert!(Helpers::is_standard_directory("StartupFolder"));
        assert!(Helpers::is_standard_directory("FontsFolder"));
        assert!(Helpers::is_standard_directory("PersonalFolder"));
        assert!(Helpers::is_standard_directory("CommonAppDataFolder"));
        assert!(Helpers::is_standard_directory("AdminToolsFolder"));
        assert!(Helpers::is_standard_directory("FavoritesFolder"));
        assert!(Helpers::is_standard_directory("NetHoodFolder"));
        assert!(Helpers::is_standard_directory("PrintHoodFolder"));
        assert!(Helpers::is_standard_directory("RecentFolder"));
        assert!(Helpers::is_standard_directory("SendToFolder"));
        assert!(Helpers::is_standard_directory("TemplateFolder"));
    }

    #[test]
    fn test_is_sensitive_property_name() {
        assert!(Helpers::is_sensitive_property_name("DATABASE_PASSWORD"));
        assert!(Helpers::is_sensitive_property_name("ApiKey"));
        assert!(Helpers::is_sensitive_property_name("SECRET_TOKEN"));
        assert!(!Helpers::is_sensitive_property_name("INSTALLFOLDER"));
        assert!(!Helpers::is_sensitive_property_name("ProductName"));
    }

    #[test]
    fn test_is_sensitive_property_name_variations() {
        // password variations
        assert!(Helpers::is_sensitive_property_name("PASSWORD"));
        assert!(Helpers::is_sensitive_property_name("UserPassword"));
        assert!(Helpers::is_sensitive_property_name("DB_PASSWORD"));

        // secret variations
        assert!(Helpers::is_sensitive_property_name("SECRET"));
        assert!(Helpers::is_sensitive_property_name("ClientSecret"));
        assert!(Helpers::is_sensitive_property_name("SECRET_KEY"));

        // key variations
        assert!(Helpers::is_sensitive_property_name("KEY"));
        assert!(Helpers::is_sensitive_property_name("PrivateKey"));
        assert!(Helpers::is_sensitive_property_name("ENCRYPTION_KEY"));

        // token variations
        assert!(Helpers::is_sensitive_property_name("TOKEN"));
        assert!(Helpers::is_sensitive_property_name("AccessToken"));
        assert!(Helpers::is_sensitive_property_name("AUTH_TOKEN"));

        // credential variations
        assert!(Helpers::is_sensitive_property_name("CREDENTIAL"));
        assert!(Helpers::is_sensitive_property_name("UserCredentials"));

        // apikey variations
        assert!(Helpers::is_sensitive_property_name("APIKEY"));
        assert!(Helpers::is_sensitive_property_name("ApiKey"));
        assert!(Helpers::is_sensitive_property_name("API_KEY"));
    }

    #[test]
    fn test_is_sensitive_property_name_safe() {
        assert!(!Helpers::is_sensitive_property_name("ProductName"));
        assert!(!Helpers::is_sensitive_property_name("InstallDir"));
        assert!(!Helpers::is_sensitive_property_name("Version"));
        assert!(!Helpers::is_sensitive_property_name("Manufacturer"));
        assert!(!Helpers::is_sensitive_property_name("ARPCONTACT"));
    }

    #[test]
    fn test_is_hardcoded_path() {
        assert!(Helpers::is_hardcoded_path("C:\\Program Files\\MyApp"));
        assert!(Helpers::is_hardcoded_path("D:\\Data"));
        assert!(!Helpers::is_hardcoded_path("[INSTALLFOLDER]"));
        assert!(!Helpers::is_hardcoded_path("relative/path"));
    }

    #[test]
    fn test_is_hardcoded_path_all_drives() {
        assert!(Helpers::is_hardcoded_path("A:\\"));
        assert!(Helpers::is_hardcoded_path("B:\\data"));
        assert!(Helpers::is_hardcoded_path("C:\\Windows"));
        assert!(Helpers::is_hardcoded_path("D:\\Program Files"));
        assert!(Helpers::is_hardcoded_path("E:\\Users"));
        assert!(Helpers::is_hardcoded_path("Z:\\Network"));
    }

    #[test]
    fn test_is_hardcoded_path_lowercase() {
        assert!(Helpers::is_hardcoded_path("c:\\Windows"));
        assert!(Helpers::is_hardcoded_path("d:\\data"));
    }

    #[test]
    fn test_is_hardcoded_path_not_hardcoded() {
        assert!(!Helpers::is_hardcoded_path("[ProgramFilesFolder]"));
        assert!(!Helpers::is_hardcoded_path("[INSTALLFOLDER]MyApp"));
        assert!(!Helpers::is_hardcoded_path("./relative/path"));
        assert!(!Helpers::is_hardcoded_path("../parent/path"));
        assert!(!Helpers::is_hardcoded_path("filename.exe"));
        assert!(!Helpers::is_hardcoded_path(""));
    }

    #[test]
    fn test_looks_like_dotnet_assembly() {
        assert!(Helpers::looks_like_dotnet_assembly("MyApp.dll"));
        assert!(Helpers::looks_like_dotnet_assembly("MyApp.exe"));
        assert!(Helpers::looks_like_dotnet_assembly("System.Core.dll"));
        assert!(Helpers::looks_like_dotnet_assembly("APP.DLL")); // uppercase
        assert!(Helpers::looks_like_dotnet_assembly("APP.EXE")); // uppercase
        assert!(Helpers::looks_like_dotnet_assembly("MixedCase.Dll"));
    }

    #[test]
    fn test_looks_like_dotnet_assembly_not_assembly() {
        assert!(!Helpers::looks_like_dotnet_assembly("config.xml"));
        assert!(!Helpers::looks_like_dotnet_assembly("readme.txt"));
        assert!(!Helpers::looks_like_dotnet_assembly("icon.ico"));
        assert!(!Helpers::looks_like_dotnet_assembly("data.json"));
        assert!(!Helpers::looks_like_dotnet_assembly("script.ps1"));
        assert!(!Helpers::looks_like_dotnet_assembly("")); // empty
    }
}

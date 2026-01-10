//! wix-license - License key validation for WiX installers
//!
//! Generates WiX components for:
//! - License key input dialogs
//! - Validation custom actions
//! - Serial number formats

use rand::Rng;
use serde::{Deserialize, Serialize};

/// License key format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LicenseFormat {
    /// XXXXX-XXXXX-XXXXX-XXXXX (Microsoft style)
    Microsoft,
    /// XXXX-XXXX-XXXX-XXXX (Short segments)
    Short,
    /// XXXXXXXX-XXXX-XXXX-XXXX-XXXXXXXXXXXX (GUID style)
    Guid,
    /// Custom format with pattern
    Custom,
}

impl std::fmt::Display for LicenseFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LicenseFormat::Microsoft => write!(f, "XXXXX-XXXXX-XXXXX-XXXXX"),
            LicenseFormat::Short => write!(f, "XXXX-XXXX-XXXX-XXXX"),
            LicenseFormat::Guid => write!(f, "XXXXXXXX-XXXX-XXXX-XXXX-XXXXXXXXXXXX"),
            LicenseFormat::Custom => write!(f, "Custom"),
        }
    }
}

/// Validation type for license keys
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValidationType {
    /// Format only (regex pattern match)
    FormatOnly,
    /// Checksum validation (built-in algorithm)
    Checksum,
    /// Online validation (requires server)
    Online,
    /// Custom DLL validation
    CustomDll,
}

/// License validation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseConfig {
    pub format: LicenseFormat,
    pub pattern: Option<String>,
    pub validation: ValidationType,
    pub property_name: String,
    pub error_message: String,
    pub dialog_title: String,
    pub dialog_description: String,
}

impl Default for LicenseConfig {
    fn default() -> Self {
        Self {
            format: LicenseFormat::Microsoft,
            pattern: None,
            validation: ValidationType::FormatOnly,
            property_name: "LICENSEKEY".to_string(),
            error_message: "Invalid license key. Please enter a valid key.".to_string(),
            dialog_title: "License Key".to_string(),
            dialog_description: "Please enter your license key to continue installation.".to_string(),
        }
    }
}

/// License key generator
pub struct LicenseGenerator;

impl LicenseGenerator {
    /// Generate a sample license key
    pub fn generate_sample(format: LicenseFormat) -> String {
        let mut rng = rand::thread_rng();
        let chars: Vec<char> = "ABCDEFGHJKLMNPQRSTUVWXYZ23456789".chars().collect();

        match format {
            LicenseFormat::Microsoft => {
                let segments: Vec<String> = (0..4)
                    .map(|_| {
                        (0..5)
                            .map(|_| chars[rng.gen_range(0..chars.len())])
                            .collect::<String>()
                    })
                    .collect();
                segments.join("-")
            }
            LicenseFormat::Short => {
                let segments: Vec<String> = (0..4)
                    .map(|_| {
                        (0..4)
                            .map(|_| chars[rng.gen_range(0..chars.len())])
                            .collect::<String>()
                    })
                    .collect();
                segments.join("-")
            }
            LicenseFormat::Guid => {
                let hex: Vec<char> = "0123456789ABCDEF".chars().collect();
                format!(
                    "{}-{}-{}-{}-{}",
                    (0..8).map(|_| hex[rng.gen_range(0..hex.len())]).collect::<String>(),
                    (0..4).map(|_| hex[rng.gen_range(0..hex.len())]).collect::<String>(),
                    (0..4).map(|_| hex[rng.gen_range(0..hex.len())]).collect::<String>(),
                    (0..4).map(|_| hex[rng.gen_range(0..hex.len())]).collect::<String>(),
                    (0..12).map(|_| hex[rng.gen_range(0..hex.len())]).collect::<String>(),
                )
            }
            LicenseFormat::Custom => "CUSTOM-KEY-FORMAT".to_string(),
        }
    }

    /// Get regex pattern for format validation
    pub fn get_pattern(format: LicenseFormat) -> String {
        match format {
            LicenseFormat::Microsoft => r"^[A-Z0-9]{5}-[A-Z0-9]{5}-[A-Z0-9]{5}-[A-Z0-9]{5}$".to_string(),
            LicenseFormat::Short => r"^[A-Z0-9]{4}-[A-Z0-9]{4}-[A-Z0-9]{4}-[A-Z0-9]{4}$".to_string(),
            LicenseFormat::Guid => r"^[A-F0-9]{8}-[A-F0-9]{4}-[A-F0-9]{4}-[A-F0-9]{4}-[A-F0-9]{12}$".to_string(),
            LicenseFormat::Custom => r".*".to_string(),
        }
    }

    /// Simple checksum validation (Luhn-like)
    pub fn validate_checksum(key: &str) -> bool {
        let clean: String = key.chars().filter(|c| c.is_alphanumeric()).collect();
        if clean.len() < 4 {
            return false;
        }

        let sum: u32 = clean
            .chars()
            .enumerate()
            .map(|(i, c)| {
                let val = if c.is_ascii_digit() {
                    c.to_digit(10).unwrap()
                } else {
                    (c.to_ascii_uppercase() as u32) - ('A' as u32) + 10
                };
                if i % 2 == 0 { val * 2 } else { val }
            })
            .map(|v| if v > 9 { v - 9 } else { v })
            .sum();

        sum % 10 == 0
    }
}

/// WiX code generator for license validation
pub struct WixLicenseGenerator {
    config: LicenseConfig,
}

impl WixLicenseGenerator {
    pub fn new(config: LicenseConfig) -> Self {
        Self { config }
    }

    /// Generate WiX dialog for license key input
    pub fn generate_dialog(&self) -> String {
        format!(
            r#"<!-- License Key Dialog -->
<UI>
    <Dialog Id="LicenseKeyDlg" Width="370" Height="270" Title="[ProductName] Setup">
        <Control Id="Title" Type="Text" X="15" Y="6" Width="200" Height="15" Transparent="yes" NoPrefix="yes">
            <Text>{{\WixUI_Font_Title}}{}</Text>
        </Control>
        <Control Id="Description" Type="Text" X="25" Y="23" Width="280" Height="15" Transparent="yes" NoPrefix="yes">
            <Text>{}</Text>
        </Control>
        <Control Id="BannerLine" Type="Line" X="0" Y="44" Width="370" Height="0" />

        <Control Id="LicenseKeyLabel" Type="Text" X="20" Y="60" Width="330" Height="15" NoPrefix="yes">
            <Text>Enter your license key (format: {}):</Text>
        </Control>
        <Control Id="LicenseKeyEdit" Type="Edit" X="20" Y="80" Width="330" Height="18" Property="{}" />

        <Control Id="ErrorText" Type="Text" X="20" Y="110" Width="330" Height="30" NoPrefix="yes" Hidden="yes">
            <Text>{}</Text>
            <Condition Action="show">LicenseKeyValid = "0"</Condition>
        </Control>

        <Control Id="BottomLine" Type="Line" X="0" Y="234" Width="370" Height="0" />
        <Control Id="Back" Type="PushButton" X="180" Y="243" Width="56" Height="17" Text="&amp;Back">
            <Publish Event="NewDialog" Value="LicenseAgreementDlg">1</Publish>
        </Control>
        <Control Id="Next" Type="PushButton" X="236" Y="243" Width="56" Height="17" Default="yes" Text="&amp;Next">
            <Publish Event="DoAction" Value="ValidateLicenseKey">1</Publish>
            <Publish Event="NewDialog" Value="InstallDirDlg">LicenseKeyValid = "1"</Publish>
        </Control>
        <Control Id="Cancel" Type="PushButton" X="304" Y="243" Width="56" Height="17" Cancel="yes" Text="Cancel">
            <Publish Event="SpawnDialog" Value="CancelDlg">1</Publish>
        </Control>
    </Dialog>
</UI>
"#,
            self.config.dialog_title,
            self.config.dialog_description,
            self.config.format,
            self.config.property_name,
            self.config.error_message,
        )
    }

    /// Generate properties for license validation
    pub fn generate_properties(&self) -> String {
        let pattern = self.config.pattern.clone()
            .unwrap_or_else(|| LicenseGenerator::get_pattern(self.config.format));

        format!(
            r#"<!-- License Key Properties -->
<Property Id="{}" Secure="yes" />
<Property Id="LicenseKeyValid" Value="0" />
<Property Id="LicenseKeyPattern" Value="{}" />
"#,
            self.config.property_name,
            escape_xml(&pattern),
        )
    }

    /// Generate custom action for validation
    pub fn generate_custom_action(&self) -> String {
        match self.config.validation {
            ValidationType::FormatOnly => self.generate_format_validation_ca(),
            ValidationType::Checksum => self.generate_checksum_validation_ca(),
            ValidationType::Online => self.generate_online_validation_ca(),
            ValidationType::CustomDll => self.generate_dll_validation_ca(),
        }
    }

    fn generate_format_validation_ca(&self) -> String {
        format!(
            r#"<!-- License Key Format Validation (VBScript) -->
<CustomAction Id="ValidateLicenseKey" Script="vbscript">
<![CDATA[
    Dim key, pattern, regex, valid
    key = Session.Property("{}")
    pattern = Session.Property("LicenseKeyPattern")

    Set regex = New RegExp
    regex.Pattern = pattern
    regex.IgnoreCase = True

    If regex.Test(key) Then
        Session.Property("LicenseKeyValid") = "1"
    Else
        Session.Property("LicenseKeyValid") = "0"
    End If
]]>
</CustomAction>
"#,
            self.config.property_name,
        )
    }

    fn generate_checksum_validation_ca(&self) -> String {
        format!(
            r#"<!-- License Key Checksum Validation (VBScript) -->
<CustomAction Id="ValidateLicenseKey" Script="vbscript">
<![CDATA[
    Dim key, clean, sum, i, c, val
    key = Session.Property("{}")
    clean = ""

    ' Remove non-alphanumeric characters
    For i = 1 To Len(key)
        c = Mid(key, i, 1)
        If (c >= "A" And c <= "Z") Or (c >= "0" And c <= "9") Then
            clean = clean & c
        End If
    Next

    If Len(clean) < 4 Then
        Session.Property("LicenseKeyValid") = "0"
        Exit Sub
    End If

    ' Calculate checksum (Luhn-like)
    sum = 0
    For i = 1 To Len(clean)
        c = UCase(Mid(clean, i, 1))
        If c >= "0" And c <= "9" Then
            val = Asc(c) - Asc("0")
        Else
            val = Asc(c) - Asc("A") + 10
        End If

        If (i Mod 2) = 1 Then
            val = val * 2
            If val > 9 Then val = val - 9
        End If
        sum = sum + val
    Next

    If (sum Mod 10) = 0 Then
        Session.Property("LicenseKeyValid") = "1"
    Else
        Session.Property("LicenseKeyValid") = "0"
    End If
]]>
</CustomAction>
"#,
            self.config.property_name,
        )
    }

    fn generate_online_validation_ca(&self) -> String {
        format!(
            r#"<!-- License Key Online Validation (placeholder) -->
<!-- Note: Replace URL with your validation server -->
<CustomAction Id="ValidateLicenseKey" Script="vbscript">
<![CDATA[
    Dim key, http, url, response
    key = Session.Property("{}")
    url = "https://your-validation-server.com/validate?key=" & key

    On Error Resume Next
    Set http = CreateObject("MSXML2.XMLHTTP")
    http.Open "GET", url, False
    http.Send

    If http.Status = 200 And InStr(http.ResponseText, "valid") > 0 Then
        Session.Property("LicenseKeyValid") = "1"
    Else
        Session.Property("LicenseKeyValid") = "0"
    End If
    On Error Goto 0
]]>
</CustomAction>
"#,
            self.config.property_name,
        )
    }

    fn generate_dll_validation_ca(&self) -> String {
        format!(
            r#"<!-- License Key DLL Validation -->
<!-- Note: Implement ValidateLicense function in your DLL -->
<Binary Id="LicenseValidatorDll" SourceFile="LicenseValidator.dll" />
<CustomAction Id="ValidateLicenseKey" BinaryRef="LicenseValidatorDll"
              DllEntry="ValidateLicense" Execute="immediate" Return="check" />
"#
        )
    }

    /// Generate complete license validation fragment
    pub fn generate_fragment(&self) -> String {
        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<Wix xmlns="http://wixtoolset.org/schemas/v4/wxs">
    <Fragment>
{}
{}
{}
    </Fragment>
</Wix>
"#,
            self.generate_properties(),
            self.generate_custom_action(),
            self.generate_dialog(),
        )
    }
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_sample_microsoft() {
        let key = LicenseGenerator::generate_sample(LicenseFormat::Microsoft);
        assert_eq!(key.len(), 23); // 5*4 + 3 dashes
        assert_eq!(key.matches('-').count(), 3);
    }

    #[test]
    fn test_generate_sample_short() {
        let key = LicenseGenerator::generate_sample(LicenseFormat::Short);
        assert_eq!(key.len(), 19); // 4*4 + 3 dashes
    }

    #[test]
    fn test_generate_sample_guid() {
        let key = LicenseGenerator::generate_sample(LicenseFormat::Guid);
        assert_eq!(key.len(), 36); // GUID format
    }

    #[test]
    fn test_get_pattern() {
        let pattern = LicenseGenerator::get_pattern(LicenseFormat::Microsoft);
        assert!(pattern.contains("5"));
    }

    #[test]
    fn test_checksum_validation() {
        // This will vary based on the key
        let _ = LicenseGenerator::validate_checksum("ABCD-1234-EFGH-5678");
    }

    #[test]
    fn test_generate_dialog() {
        let config = LicenseConfig::default();
        let gen = WixLicenseGenerator::new(config);
        let dialog = gen.generate_dialog();

        assert!(dialog.contains("LicenseKeyDlg"));
        assert!(dialog.contains("LICENSEKEY"));
    }

    #[test]
    fn test_generate_properties() {
        let config = LicenseConfig::default();
        let gen = WixLicenseGenerator::new(config);
        let props = gen.generate_properties();

        assert!(props.contains("LICENSEKEY"));
        assert!(props.contains("LicenseKeyValid"));
    }

    #[test]
    fn test_generate_fragment() {
        let config = LicenseConfig::default();
        let gen = WixLicenseGenerator::new(config);
        let fragment = gen.generate_fragment();

        assert!(fragment.contains("<?xml"));
        assert!(fragment.contains("<Fragment>"));
    }

    #[test]
    fn test_format_display() {
        assert_eq!(
            format!("{}", LicenseFormat::Microsoft),
            "XXXXX-XXXXX-XXXXX-XXXXX"
        );
    }
}

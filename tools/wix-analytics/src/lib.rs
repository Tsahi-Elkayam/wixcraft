//! wix-analytics - Installation telemetry for MSI packages
//!
//! Generates WiX code for:
//! - Installation success/failure tracking
//! - System information collection
//! - Anonymous usage statistics
//! - Error reporting

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Telemetry event type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventType {
    InstallStart,
    InstallSuccess,
    InstallFailure,
    UninstallStart,
    UninstallSuccess,
    UninstallFailure,
    RepairStart,
    RepairSuccess,
    RepairFailure,
    FeatureChange,
}

impl std::fmt::Display for EventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EventType::InstallStart => write!(f, "install_start"),
            EventType::InstallSuccess => write!(f, "install_success"),
            EventType::InstallFailure => write!(f, "install_failure"),
            EventType::UninstallStart => write!(f, "uninstall_start"),
            EventType::UninstallSuccess => write!(f, "uninstall_success"),
            EventType::UninstallFailure => write!(f, "uninstall_failure"),
            EventType::RepairStart => write!(f, "repair_start"),
            EventType::RepairSuccess => write!(f, "repair_success"),
            EventType::RepairFailure => write!(f, "repair_failure"),
            EventType::FeatureChange => write!(f, "feature_change"),
        }
    }
}

/// Telemetry data to collect
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TelemetryField {
    ProductVersion,
    OsVersion,
    OsArchitecture,
    InstallPath,
    InstallMode,
    Features,
    Duration,
    ErrorCode,
    Locale,
    TimeZone,
}

impl std::fmt::Display for TelemetryField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TelemetryField::ProductVersion => write!(f, "product_version"),
            TelemetryField::OsVersion => write!(f, "os_version"),
            TelemetryField::OsArchitecture => write!(f, "os_arch"),
            TelemetryField::InstallPath => write!(f, "install_path"),
            TelemetryField::InstallMode => write!(f, "install_mode"),
            TelemetryField::Features => write!(f, "features"),
            TelemetryField::Duration => write!(f, "duration"),
            TelemetryField::ErrorCode => write!(f, "error_code"),
            TelemetryField::Locale => write!(f, "locale"),
            TelemetryField::TimeZone => write!(f, "timezone"),
        }
    }
}

/// Analytics configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsConfig {
    pub endpoint_url: String,
    pub product_id: String,
    pub fields: Vec<TelemetryField>,
    pub events: Vec<EventType>,
    pub anonymous: bool,
    pub opt_in: bool,
}

impl Default for AnalyticsConfig {
    fn default() -> Self {
        Self {
            endpoint_url: "https://analytics.example.com/v1/events".to_string(),
            product_id: "my-product".to_string(),
            fields: vec![
                TelemetryField::ProductVersion,
                TelemetryField::OsVersion,
                TelemetryField::OsArchitecture,
                TelemetryField::InstallMode,
                TelemetryField::Duration,
                TelemetryField::ErrorCode,
            ],
            events: vec![
                EventType::InstallStart,
                EventType::InstallSuccess,
                EventType::InstallFailure,
            ],
            anonymous: true,
            opt_in: true,
        }
    }
}

/// Telemetry event record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryEvent {
    pub event_type: EventType,
    pub timestamp: DateTime<Utc>,
    pub product_id: String,
    pub product_version: Option<String>,
    pub os_version: Option<String>,
    pub os_arch: Option<String>,
    pub install_mode: Option<String>,
    pub duration_ms: Option<u64>,
    pub error_code: Option<i32>,
    pub features: Option<Vec<String>>,
}

/// WiX analytics code generator
pub struct AnalyticsGenerator {
    config: AnalyticsConfig,
}

impl AnalyticsGenerator {
    pub fn new(config: AnalyticsConfig) -> Self {
        Self { config }
    }

    /// Generate WiX properties for telemetry
    pub fn generate_properties(&self) -> String {
        let mut xml = String::new();

        xml.push_str("<!-- Analytics Properties -->\n");

        if self.config.opt_in {
            xml.push_str("<Property Id=\"TELEMETRY_ENABLED\" Value=\"0\" Secure=\"yes\" />\n");
        } else {
            xml.push_str("<Property Id=\"TELEMETRY_ENABLED\" Value=\"1\" Secure=\"yes\" />\n");
        }

        xml.push_str(&format!(
            "<Property Id=\"ANALYTICS_ENDPOINT\" Value=\"{}\" />\n",
            self.config.endpoint_url
        ));
        xml.push_str(&format!(
            "<Property Id=\"ANALYTICS_PRODUCT_ID\" Value=\"{}\" />\n",
            self.config.product_id
        ));
        xml.push_str("<Property Id=\"INSTALL_START_TIME\" Value=\"\" />\n");
        xml.push_str("<Property Id=\"INSTALL_DURATION\" Value=\"\" />\n");

        xml
    }

    /// Generate custom actions for telemetry
    pub fn generate_custom_actions(&self) -> String {
        let mut xml = String::new();

        xml.push_str("<!-- Analytics Custom Actions -->\n\n");

        // Record start time
        xml.push_str(r#"<CustomAction Id="CA_RecordStartTime" Script="vbscript">
<![CDATA[
    Session.Property("INSTALL_START_TIME") = Timer
]]>
</CustomAction>

"#);

        // Send telemetry event
        xml.push_str(&format!(r#"<CustomAction Id="CA_SendTelemetry" Script="vbscript">
<![CDATA[
    If Session.Property("TELEMETRY_ENABLED") = "1" Then
        Dim http, endpoint, data, startTime, duration
        endpoint = Session.Property("ANALYTICS_ENDPOINT")
        startTime = Session.Property("INSTALL_START_TIME")

        If startTime <> "" Then
            duration = Int((Timer - CDbl(startTime)) * 1000)
        Else
            duration = 0
        End If

        data = "{{"
        data = data & """event"": ""install_complete"","
        data = data & """product_id"": """ & Session.Property("ANALYTICS_PRODUCT_ID") & ""","
        data = data & """product_version"": """ & Session.Property("ProductVersion") & ""","
        data = data & """duration_ms"": " & duration
        data = data & "}}"

        On Error Resume Next
        Set http = CreateObject("MSXML2.XMLHTTP")
        http.Open "POST", endpoint, False
        http.setRequestHeader "Content-Type", "application/json"
        http.Send data
        On Error Goto 0
    End If
]]>
</CustomAction>

"#));

        // Send error telemetry
        xml.push_str(r#"<CustomAction Id="CA_SendErrorTelemetry" Script="vbscript">
<![CDATA[
    If Session.Property("TELEMETRY_ENABLED") = "1" Then
        Dim http, endpoint, data
        endpoint = Session.Property("ANALYTICS_ENDPOINT")

        data = "{"
        data = data & """event"": ""install_error"","
        data = data & """product_id"": """ & Session.Property("ANALYTICS_PRODUCT_ID") & ""","
        data = data & """product_version"": """ & Session.Property("ProductVersion") & """"
        data = data & "}"

        On Error Resume Next
        Set http = CreateObject("MSXML2.XMLHTTP")
        http.Open "POST", endpoint, False
        http.setRequestHeader "Content-Type", "application/json"
        http.Send data
        On Error Goto 0
    End If
]]>
</CustomAction>

"#);

        xml
    }

    /// Generate install execute sequence
    pub fn generate_sequence(&self) -> String {
        r#"<!-- Analytics Sequence -->
<InstallExecuteSequence>
    <Custom Action="CA_RecordStartTime" Before="InstallInitialize">
        TELEMETRY_ENABLED = "1"
    </Custom>
    <Custom Action="CA_SendTelemetry" After="InstallFinalize">
        TELEMETRY_ENABLED = "1" AND NOT REMOVE
    </Custom>
</InstallExecuteSequence>
"#.to_string()
    }

    /// Generate opt-in checkbox for UI
    pub fn generate_opt_in_control(&self) -> String {
        r#"<!-- Telemetry Opt-In Checkbox -->
<Control Id="TelemetryCheckbox" Type="CheckBox" X="20" Y="200" Width="300" Height="17"
         Property="TELEMETRY_ENABLED" CheckBoxValue="1">
    <Text>Help improve this product by sending anonymous usage statistics</Text>
</Control>
"#.to_string()
    }

    /// Generate complete analytics fragment
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
            self.generate_custom_actions(),
            self.generate_sequence(),
        )
    }
}

/// Parse log file to extract telemetry data
pub fn parse_msi_log(content: &str) -> TelemetryEvent {
    let mut event = TelemetryEvent {
        event_type: EventType::InstallSuccess,
        timestamp: Utc::now(),
        product_id: String::new(),
        product_version: None,
        os_version: None,
        os_arch: None,
        install_mode: None,
        duration_ms: None,
        error_code: None,
        features: None,
    };

    // Extract product version
    if let Some(line) = content.lines().find(|l| l.contains("ProductVersion")) {
        if let Some(ver) = line.split('=').nth(1) {
            event.product_version = Some(ver.trim().to_string());
        }
    }

    // Check for errors
    if content.contains("Installation failed") || content.contains("Error ") {
        event.event_type = EventType::InstallFailure;

        // Try to extract error code
        for line in content.lines() {
            if line.contains("Error ") {
                if let Some(code) = extract_error_code(line) {
                    event.error_code = Some(code);
                    break;
                }
            }
        }
    }

    event
}

fn extract_error_code(line: &str) -> Option<i32> {
    // Look for patterns like "Error 1234" or "error code: 1234"
    let patterns = ["Error ", "error code: ", "code "];
    for pattern in &patterns {
        if let Some(pos) = line.find(pattern) {
            let after = &line[pos + pattern.len()..];
            let code_str: String = after.chars().take_while(|c| c.is_ascii_digit()).collect();
            if let Ok(code) = code_str.parse() {
                return Some(code);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_type_display() {
        assert_eq!(format!("{}", EventType::InstallStart), "install_start");
        assert_eq!(format!("{}", EventType::InstallSuccess), "install_success");
    }

    #[test]
    fn test_field_display() {
        assert_eq!(format!("{}", TelemetryField::ProductVersion), "product_version");
        assert_eq!(format!("{}", TelemetryField::OsVersion), "os_version");
    }

    #[test]
    fn test_generate_properties() {
        let config = AnalyticsConfig::default();
        let gen = AnalyticsGenerator::new(config);
        let props = gen.generate_properties();

        assert!(props.contains("TELEMETRY_ENABLED"));
        assert!(props.contains("ANALYTICS_ENDPOINT"));
    }

    #[test]
    fn test_generate_custom_actions() {
        let config = AnalyticsConfig::default();
        let gen = AnalyticsGenerator::new(config);
        let actions = gen.generate_custom_actions();

        assert!(actions.contains("CA_RecordStartTime"));
        assert!(actions.contains("CA_SendTelemetry"));
    }

    #[test]
    fn test_parse_msi_log_success() {
        let log = "ProductVersion = 1.0.0\nInstallation completed successfully";
        let event = parse_msi_log(log);

        assert_eq!(event.event_type, EventType::InstallSuccess);
    }

    #[test]
    fn test_parse_msi_log_failure() {
        let log = "ProductVersion = 1.0.0\nInstallation failed\nError 1603";
        let event = parse_msi_log(log);

        assert_eq!(event.event_type, EventType::InstallFailure);
        assert_eq!(event.error_code, Some(1603));
    }

    #[test]
    fn test_extract_error_code() {
        assert_eq!(extract_error_code("Error 1234 occurred"), Some(1234));
        assert_eq!(extract_error_code("error code: 5678"), Some(5678));
    }
}

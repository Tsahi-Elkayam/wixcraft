//! wix-ui - WiX UI sequence generator and customizer
//!
//! Helps create and customize WiX installer UI sequences.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Standard WiX UI types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UIType {
    /// Minimal UI - just progress bar and completion
    Minimal,
    /// InstallDir UI - allows directory selection
    InstallDir,
    /// Feature Tree - allows feature selection
    FeatureTree,
    /// Mondo - full featured with repair/remove
    Mondo,
    /// Advanced - InstallDir with feature tree
    Advanced,
    /// Custom - user-defined dialogs
    Custom,
}

impl UIType {
    pub fn ui_ref_id(&self) -> &'static str {
        match self {
            UIType::Minimal => "WixUI_Minimal",
            UIType::InstallDir => "WixUI_InstallDir",
            UIType::FeatureTree => "WixUI_FeatureTree",
            UIType::Mondo => "WixUI_Mondo",
            UIType::Advanced => "WixUI_Advanced",
            UIType::Custom => "",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            UIType::Minimal => "Simple progress bar and completion dialog",
            UIType::InstallDir => "Allows user to select installation directory",
            UIType::FeatureTree => "Allows user to select which features to install",
            UIType::Mondo => "Full UI with maintenance mode (repair/change/remove)",
            UIType::Advanced => "InstallDir with feature tree selection",
            UIType::Custom => "Custom dialog sequence",
        }
    }
}

/// UI Dialog definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dialog {
    pub id: String,
    pub title: String,
    pub width: u32,
    pub height: u32,
    pub controls: Vec<Control>,
}

/// UI Control types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ControlType {
    PushButton,
    Text,
    EditText,
    CheckBox,
    RadioButtonGroup,
    ComboBox,
    ListBox,
    ListView,
    DirectoryCombo,
    DirectoryList,
    VolumeCostList,
    ScrollableText,
    Bitmap,
    Icon,
    ProgressBar,
    Line,
    GroupBox,
}

/// UI Control definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Control {
    pub id: String,
    pub control_type: ControlType,
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub text: Option<String>,
    pub property: Option<String>,
    pub attributes: HashMap<String, String>,
}

impl Control {
    pub fn push_button(id: &str, x: u32, y: u32, width: u32, height: u32, text: &str) -> Self {
        Self {
            id: id.to_string(),
            control_type: ControlType::PushButton,
            x,
            y,
            width,
            height,
            text: Some(text.to_string()),
            property: None,
            attributes: HashMap::new(),
        }
    }

    pub fn text(id: &str, x: u32, y: u32, width: u32, height: u32, text: &str) -> Self {
        Self {
            id: id.to_string(),
            control_type: ControlType::Text,
            x,
            y,
            width,
            height,
            text: Some(text.to_string()),
            property: None,
            attributes: HashMap::new(),
        }
    }

    pub fn edit_text(id: &str, x: u32, y: u32, width: u32, height: u32, property: &str) -> Self {
        Self {
            id: id.to_string(),
            control_type: ControlType::EditText,
            x,
            y,
            width,
            height,
            text: None,
            property: Some(property.to_string()),
            attributes: HashMap::new(),
        }
    }
}

/// UI Sequence event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UIEvent {
    pub dialog: String,
    pub control: String,
    pub event: String,
    pub argument: String,
    pub condition: Option<String>,
    pub order: i32,
}

/// UI Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UIConfig {
    pub ui_type: UIType,
    pub license_dialog: bool,
    pub install_dir_dialog: bool,
    pub feature_dialog: bool,
    pub banner_bitmap: Option<String>,
    pub dialog_bitmap: Option<String>,
    pub icon: Option<String>,
    pub custom_dialogs: Vec<Dialog>,
    pub variables: HashMap<String, String>,
}

impl Default for UIConfig {
    fn default() -> Self {
        Self {
            ui_type: UIType::Minimal,
            license_dialog: false,
            install_dir_dialog: false,
            feature_dialog: false,
            banner_bitmap: None,
            dialog_bitmap: None,
            icon: None,
            custom_dialogs: Vec::new(),
            variables: HashMap::new(),
        }
    }
}

impl UIConfig {
    pub fn minimal() -> Self {
        Self {
            ui_type: UIType::Minimal,
            ..Default::default()
        }
    }

    pub fn install_dir() -> Self {
        Self {
            ui_type: UIType::InstallDir,
            install_dir_dialog: true,
            ..Default::default()
        }
    }

    pub fn feature_tree() -> Self {
        Self {
            ui_type: UIType::FeatureTree,
            feature_dialog: true,
            ..Default::default()
        }
    }

    pub fn mondo() -> Self {
        Self {
            ui_type: UIType::Mondo,
            license_dialog: true,
            install_dir_dialog: true,
            feature_dialog: true,
            ..Default::default()
        }
    }

    pub fn with_license(mut self) -> Self {
        self.license_dialog = true;
        self
    }

    pub fn with_banner(mut self, path: &str) -> Self {
        self.banner_bitmap = Some(path.to_string());
        self
    }

    pub fn with_dialog_bitmap(mut self, path: &str) -> Self {
        self.dialog_bitmap = Some(path.to_string());
        self
    }

    pub fn with_icon(mut self, path: &str) -> Self {
        self.icon = Some(path.to_string());
        self
    }
}

/// UI Generator
pub struct UIGenerator;

impl UIGenerator {
    /// Generate UI reference XML
    pub fn generate_ui_ref(config: &UIConfig) -> String {
        let mut xml = String::new();
        xml.push_str("<UI>\n");

        if config.ui_type != UIType::Custom {
            xml.push_str(&format!("    <UIRef Id=\"{}\" />\n", config.ui_type.ui_ref_id()));
        }

        // Add property for InstallDir UI
        if config.ui_type == UIType::InstallDir || config.ui_type == UIType::Advanced {
            xml.push_str("    <Property Id=\"WIXUI_INSTALLDIR\" Value=\"INSTALLFOLDER\" />\n");
        }

        // Custom bitmaps
        if let Some(ref banner) = config.banner_bitmap {
            xml.push_str(&format!("    <WixVariable Id=\"WixUIBannerBmp\" Value=\"{}\" />\n", banner));
        }
        if let Some(ref dialog) = config.dialog_bitmap {
            xml.push_str(&format!("    <WixVariable Id=\"WixUIDialogBmp\" Value=\"{}\" />\n", dialog));
        }

        // License customization
        if config.license_dialog {
            xml.push_str("    <WixVariable Id=\"WixUILicenseRtf\" Value=\"License.rtf\" />\n");
        }

        // Custom variables
        for (key, value) in &config.variables {
            xml.push_str(&format!("    <Property Id=\"{}\" Value=\"{}\" />\n", key, value));
        }

        xml.push_str("</UI>\n");
        xml
    }

    /// Generate a custom dialog XML
    pub fn generate_dialog(dialog: &Dialog) -> String {
        let mut xml = String::new();
        xml.push_str(&format!(
            "<Dialog Id=\"{}\" Width=\"{}\" Height=\"{}\" Title=\"{}\">\n",
            dialog.id, dialog.width, dialog.height, dialog.title
        ));

        for control in &dialog.controls {
            xml.push_str(&Self::generate_control(control));
        }

        xml.push_str("</Dialog>\n");
        xml
    }

    fn generate_control(control: &Control) -> String {
        let type_name = match control.control_type {
            ControlType::PushButton => "PushButton",
            ControlType::Text => "Text",
            ControlType::EditText => "Edit",
            ControlType::CheckBox => "CheckBox",
            ControlType::RadioButtonGroup => "RadioButtonGroup",
            ControlType::ComboBox => "ComboBox",
            ControlType::ListBox => "ListBox",
            ControlType::ListView => "ListView",
            ControlType::DirectoryCombo => "DirectoryCombo",
            ControlType::DirectoryList => "DirectoryList",
            ControlType::VolumeCostList => "VolumeCostList",
            ControlType::ScrollableText => "ScrollableText",
            ControlType::Bitmap => "Bitmap",
            ControlType::Icon => "Icon",
            ControlType::ProgressBar => "ProgressBar",
            ControlType::Line => "Line",
            ControlType::GroupBox => "GroupBox",
        };

        let mut xml = format!(
            "    <Control Id=\"{}\" Type=\"{}\" X=\"{}\" Y=\"{}\" Width=\"{}\" Height=\"{}\"",
            control.id, type_name, control.x, control.y, control.width, control.height
        );

        if let Some(ref text) = control.text {
            xml.push_str(&format!(" Text=\"{}\"", text));
        }
        if let Some(ref property) = control.property {
            xml.push_str(&format!(" Property=\"{}\"", property));
        }
        for (key, value) in &control.attributes {
            xml.push_str(&format!(" {}=\"{}\"", key, value));
        }

        xml.push_str(" />\n");
        xml
    }

    /// Generate welcome dialog
    pub fn generate_welcome_dialog(product_name: &str) -> Dialog {
        Dialog {
            id: "WelcomeDlg".to_string(),
            title: format!("[ProductName] Setup"),
            width: 370,
            height: 270,
            controls: vec![
                Control::text(
                    "Title",
                    135, 20, 220, 60,
                    &format!("Welcome to the {} Setup Wizard", product_name),
                ),
                Control::text(
                    "Description",
                    135, 70, 220, 80,
                    "This wizard will guide you through the installation.",
                ),
                Control::push_button("Back", 180, 243, 56, 17, "&Back"),
                Control::push_button("Next", 236, 243, 56, 17, "&Next"),
                Control::push_button("Cancel", 304, 243, 56, 17, "Cancel"),
            ],
        }
    }

    /// Generate finish dialog
    pub fn generate_finish_dialog(product_name: &str) -> Dialog {
        Dialog {
            id: "FinishDlg".to_string(),
            title: "[ProductName] Setup".to_string(),
            width: 370,
            height: 270,
            controls: vec![
                Control::text(
                    "Title",
                    135, 20, 220, 60,
                    &format!("Completing the {} Setup Wizard", product_name),
                ),
                Control::text(
                    "Description",
                    135, 70, 220, 40,
                    "Click Finish to exit the Setup Wizard.",
                ),
                Control::push_button("Back", 180, 243, 56, 17, "&Back"),
                Control::push_button("Finish", 236, 243, 56, 17, "&Finish"),
                Control::push_button("Cancel", 304, 243, 56, 17, "Cancel"),
            ],
        }
    }
}

/// UI Template presets
pub struct UITemplates;

impl UITemplates {
    /// Simple progress-only UI
    pub fn progress_only() -> UIConfig {
        UIConfig::minimal()
    }

    /// Standard application installer
    pub fn standard_app() -> UIConfig {
        UIConfig::install_dir().with_license()
    }

    /// Feature-selectable installer
    pub fn feature_installer() -> UIConfig {
        UIConfig::feature_tree().with_license()
    }

    /// Full enterprise installer
    pub fn enterprise() -> UIConfig {
        UIConfig::mondo()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ui_type_ref_id() {
        assert_eq!(UIType::Minimal.ui_ref_id(), "WixUI_Minimal");
        assert_eq!(UIType::InstallDir.ui_ref_id(), "WixUI_InstallDir");
        assert_eq!(UIType::Mondo.ui_ref_id(), "WixUI_Mondo");
    }

    #[test]
    fn test_minimal_config() {
        let config = UIConfig::minimal();
        assert_eq!(config.ui_type, UIType::Minimal);
        assert!(!config.license_dialog);
    }

    #[test]
    fn test_install_dir_config() {
        let config = UIConfig::install_dir();
        assert_eq!(config.ui_type, UIType::InstallDir);
        assert!(config.install_dir_dialog);
    }

    #[test]
    fn test_mondo_config() {
        let config = UIConfig::mondo();
        assert_eq!(config.ui_type, UIType::Mondo);
        assert!(config.license_dialog);
        assert!(config.install_dir_dialog);
        assert!(config.feature_dialog);
    }

    #[test]
    fn test_with_license() {
        let config = UIConfig::minimal().with_license();
        assert!(config.license_dialog);
    }

    #[test]
    fn test_with_banner() {
        let config = UIConfig::minimal().with_banner("banner.bmp");
        assert_eq!(config.banner_bitmap, Some("banner.bmp".to_string()));
    }

    #[test]
    fn test_generate_ui_ref_minimal() {
        let config = UIConfig::minimal();
        let xml = UIGenerator::generate_ui_ref(&config);
        assert!(xml.contains("WixUI_Minimal"));
    }

    #[test]
    fn test_generate_ui_ref_install_dir() {
        let config = UIConfig::install_dir();
        let xml = UIGenerator::generate_ui_ref(&config);
        assert!(xml.contains("WixUI_InstallDir"));
        assert!(xml.contains("WIXUI_INSTALLDIR"));
    }

    #[test]
    fn test_generate_ui_ref_with_banner() {
        let config = UIConfig::minimal().with_banner("my_banner.bmp");
        let xml = UIGenerator::generate_ui_ref(&config);
        assert!(xml.contains("WixUIBannerBmp"));
        assert!(xml.contains("my_banner.bmp"));
    }

    #[test]
    fn test_control_push_button() {
        let button = Control::push_button("Next", 100, 200, 50, 20, "&Next");
        assert_eq!(button.id, "Next");
        assert_eq!(button.text, Some("&Next".to_string()));
    }

    #[test]
    fn test_control_text() {
        let text = Control::text("Title", 10, 20, 200, 30, "Welcome");
        assert_eq!(text.id, "Title");
        assert!(matches!(text.control_type, ControlType::Text));
    }

    #[test]
    fn test_control_edit_text() {
        let edit = Control::edit_text("PathEdit", 10, 50, 200, 20, "INSTALLFOLDER");
        assert_eq!(edit.property, Some("INSTALLFOLDER".to_string()));
    }

    #[test]
    fn test_generate_dialog() {
        let dialog = UIGenerator::generate_welcome_dialog("MyApp");
        let xml = UIGenerator::generate_dialog(&dialog);
        assert!(xml.contains("WelcomeDlg"));
        assert!(xml.contains("370"));
        assert!(xml.contains("270"));
    }

    #[test]
    fn test_welcome_dialog() {
        let dialog = UIGenerator::generate_welcome_dialog("TestApp");
        assert_eq!(dialog.id, "WelcomeDlg");
        assert!(!dialog.controls.is_empty());
    }

    #[test]
    fn test_finish_dialog() {
        let dialog = UIGenerator::generate_finish_dialog("TestApp");
        assert_eq!(dialog.id, "FinishDlg");
        assert!(dialog.controls.iter().any(|c| c.id == "Finish"));
    }

    #[test]
    fn test_ui_templates() {
        let progress = UITemplates::progress_only();
        assert_eq!(progress.ui_type, UIType::Minimal);

        let standard = UITemplates::standard_app();
        assert!(standard.license_dialog);

        let feature = UITemplates::feature_installer();
        assert_eq!(feature.ui_type, UIType::FeatureTree);

        let enterprise = UITemplates::enterprise();
        assert_eq!(enterprise.ui_type, UIType::Mondo);
    }

    #[test]
    fn test_ui_type_description() {
        assert!(UIType::Minimal.description().contains("progress"));
        assert!(UIType::Mondo.description().contains("maintenance"));
    }
}

//! Localization helper for WiX installers
//!
//! Provides tools for creating multi-language installers with WiX.
//!
//! # Example
//!
//! ```
//! use wix_i18n::{LocalizationManager, Language, StringEntry};
//!
//! let mut mgr = LocalizationManager::new();
//! mgr.add_string("WelcomeTitle", "Welcome", Language::English);
//! mgr.add_string("WelcomeTitle", "Bienvenue", Language::French);
//!
//! let wxl = mgr.generate_wxl(Language::French);
//! assert!(wxl.contains("Bienvenue"));
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// Localization errors
#[derive(Error, Debug)]
pub enum I18nError {
    #[error("Language not found: {0}")]
    LanguageNotFound(String),
    #[error("String not found: {0}")]
    StringNotFound(String),
    #[error("Parse error: {0}")]
    ParseError(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Supported languages with LCID codes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Language {
    English,
    French,
    German,
    Spanish,
    Italian,
    Portuguese,
    Dutch,
    Polish,
    Russian,
    Japanese,
    Chinese,
    Korean,
    Arabic,
    Hebrew,
    Swedish,
    Norwegian,
    Danish,
    Finnish,
    Czech,
    Hungarian,
    Turkish,
    Greek,
    Thai,
    Vietnamese,
    Indonesian,
}

impl Language {
    /// Get the Windows LCID for this language
    pub fn lcid(&self) -> u32 {
        match self {
            Language::English => 1033,
            Language::French => 1036,
            Language::German => 1031,
            Language::Spanish => 1034,
            Language::Italian => 1040,
            Language::Portuguese => 1046,
            Language::Dutch => 1043,
            Language::Polish => 1045,
            Language::Russian => 1049,
            Language::Japanese => 1041,
            Language::Chinese => 2052,
            Language::Korean => 1042,
            Language::Arabic => 1025,
            Language::Hebrew => 1037,
            Language::Swedish => 1053,
            Language::Norwegian => 1044,
            Language::Danish => 1030,
            Language::Finnish => 1035,
            Language::Czech => 1029,
            Language::Hungarian => 1038,
            Language::Turkish => 1055,
            Language::Greek => 1032,
            Language::Thai => 1054,
            Language::Vietnamese => 1066,
            Language::Indonesian => 1057,
        }
    }

    /// Get the ISO 639-1 code
    pub fn code(&self) -> &'static str {
        match self {
            Language::English => "en",
            Language::French => "fr",
            Language::German => "de",
            Language::Spanish => "es",
            Language::Italian => "it",
            Language::Portuguese => "pt",
            Language::Dutch => "nl",
            Language::Polish => "pl",
            Language::Russian => "ru",
            Language::Japanese => "ja",
            Language::Chinese => "zh",
            Language::Korean => "ko",
            Language::Arabic => "ar",
            Language::Hebrew => "he",
            Language::Swedish => "sv",
            Language::Norwegian => "no",
            Language::Danish => "da",
            Language::Finnish => "fi",
            Language::Czech => "cs",
            Language::Hungarian => "hu",
            Language::Turkish => "tr",
            Language::Greek => "el",
            Language::Thai => "th",
            Language::Vietnamese => "vi",
            Language::Indonesian => "id",
        }
    }

    /// Get the display name
    pub fn name(&self) -> &'static str {
        match self {
            Language::English => "English",
            Language::French => "French",
            Language::German => "German",
            Language::Spanish => "Spanish",
            Language::Italian => "Italian",
            Language::Portuguese => "Portuguese",
            Language::Dutch => "Dutch",
            Language::Polish => "Polish",
            Language::Russian => "Russian",
            Language::Japanese => "Japanese",
            Language::Chinese => "Chinese (Simplified)",
            Language::Korean => "Korean",
            Language::Arabic => "Arabic",
            Language::Hebrew => "Hebrew",
            Language::Swedish => "Swedish",
            Language::Norwegian => "Norwegian",
            Language::Danish => "Danish",
            Language::Finnish => "Finnish",
            Language::Czech => "Czech",
            Language::Hungarian => "Hungarian",
            Language::Turkish => "Turkish",
            Language::Greek => "Greek",
            Language::Thai => "Thai",
            Language::Vietnamese => "Vietnamese",
            Language::Indonesian => "Indonesian",
        }
    }

    /// Get the culture name (e.g., "en-US")
    pub fn culture(&self) -> &'static str {
        match self {
            Language::English => "en-US",
            Language::French => "fr-FR",
            Language::German => "de-DE",
            Language::Spanish => "es-ES",
            Language::Italian => "it-IT",
            Language::Portuguese => "pt-BR",
            Language::Dutch => "nl-NL",
            Language::Polish => "pl-PL",
            Language::Russian => "ru-RU",
            Language::Japanese => "ja-JP",
            Language::Chinese => "zh-CN",
            Language::Korean => "ko-KR",
            Language::Arabic => "ar-SA",
            Language::Hebrew => "he-IL",
            Language::Swedish => "sv-SE",
            Language::Norwegian => "nb-NO",
            Language::Danish => "da-DK",
            Language::Finnish => "fi-FI",
            Language::Czech => "cs-CZ",
            Language::Hungarian => "hu-HU",
            Language::Turkish => "tr-TR",
            Language::Greek => "el-GR",
            Language::Thai => "th-TH",
            Language::Vietnamese => "vi-VN",
            Language::Indonesian => "id-ID",
        }
    }

    /// Get all supported languages
    pub fn all() -> Vec<Language> {
        vec![
            Language::English,
            Language::French,
            Language::German,
            Language::Spanish,
            Language::Italian,
            Language::Portuguese,
            Language::Dutch,
            Language::Polish,
            Language::Russian,
            Language::Japanese,
            Language::Chinese,
            Language::Korean,
            Language::Arabic,
            Language::Hebrew,
            Language::Swedish,
            Language::Norwegian,
            Language::Danish,
            Language::Finnish,
            Language::Czech,
            Language::Hungarian,
            Language::Turkish,
            Language::Greek,
            Language::Thai,
            Language::Vietnamese,
            Language::Indonesian,
        ]
    }

    /// Parse from string (code, name, or culture)
    pub fn from_str(s: &str) -> Option<Language> {
        let s_lower = s.to_lowercase();
        Language::all().into_iter().find(|l| {
            l.code() == s_lower
                || l.name().to_lowercase() == s_lower
                || l.culture().to_lowercase() == s_lower
        })
    }
}

/// A localized string entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StringEntry {
    /// String ID
    pub id: String,
    /// The localized value
    pub value: String,
    /// Optional comment/description
    pub comment: Option<String>,
    /// Whether this is overridable
    pub overridable: bool,
}

impl StringEntry {
    pub fn new(id: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            value: value.into(),
            comment: None,
            overridable: false,
        }
    }

    pub fn with_comment(mut self, comment: impl Into<String>) -> Self {
        self.comment = Some(comment.into());
        self
    }

    pub fn overridable(mut self) -> Self {
        self.overridable = true;
        self
    }
}

/// Localization manager
#[derive(Debug, Clone, Default)]
pub struct LocalizationManager {
    /// Strings by language then by ID
    strings: HashMap<Language, HashMap<String, StringEntry>>,
    /// Base language for fallback
    base_language: Option<Language>,
}

impl LocalizationManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the base language for fallback
    pub fn set_base_language(&mut self, lang: Language) {
        self.base_language = Some(lang);
    }

    /// Add a localized string
    pub fn add_string(&mut self, id: &str, value: &str, language: Language) {
        self.strings
            .entry(language)
            .or_default()
            .insert(id.to_string(), StringEntry::new(id, value));
    }

    /// Add a string entry
    pub fn add_entry(&mut self, entry: StringEntry, language: Language) {
        self.strings
            .entry(language)
            .or_default()
            .insert(entry.id.clone(), entry);
    }

    /// Get a string for a language
    pub fn get_string(&self, id: &str, language: Language) -> Option<&str> {
        self.strings
            .get(&language)
            .and_then(|m| m.get(id))
            .map(|e| e.value.as_str())
            .or_else(|| {
                // Fallback to base language
                self.base_language.and_then(|base| {
                    self.strings
                        .get(&base)
                        .and_then(|m| m.get(id))
                        .map(|e| e.value.as_str())
                })
            })
    }

    /// Get all languages that have strings
    pub fn languages(&self) -> Vec<Language> {
        self.strings.keys().copied().collect()
    }

    /// Get all string IDs
    pub fn string_ids(&self) -> Vec<String> {
        let mut ids: Vec<_> = self
            .strings
            .values()
            .flat_map(|m| m.keys())
            .cloned()
            .collect();
        ids.sort();
        ids.dedup();
        ids
    }

    /// Check for missing translations
    pub fn find_missing(&self, target: Language) -> Vec<String> {
        let all_ids = self.string_ids();
        let target_ids: std::collections::HashSet<_> = self
            .strings
            .get(&target)
            .map(|m| m.keys().cloned().collect())
            .unwrap_or_default();

        all_ids
            .into_iter()
            .filter(|id| !target_ids.contains(id))
            .collect()
    }

    /// Generate WXL (WiX Localization) file content
    pub fn generate_wxl(&self, language: Language) -> String {
        let mut wxl = String::new();

        wxl.push_str("<?xml version=\"1.0\" encoding=\"utf-8\"?>\n");
        wxl.push_str(&format!(
            "<WixLocalization Culture=\"{}\" Language=\"{}\" xmlns=\"http://wixtoolset.org/schemas/v4/wxl\">\n",
            language.culture(),
            language.lcid()
        ));

        if let Some(strings) = self.strings.get(&language) {
            let mut entries: Vec<_> = strings.values().collect();
            entries.sort_by(|a, b| a.id.cmp(&b.id));

            for entry in entries {
                if let Some(ref comment) = entry.comment {
                    wxl.push_str(&format!("  <!-- {} -->\n", comment));
                }

                let overridable = if entry.overridable {
                    " Overridable=\"yes\""
                } else {
                    ""
                };

                wxl.push_str(&format!(
                    "  <String Id=\"{}\"{}>{}</String>\n",
                    entry.id,
                    overridable,
                    escape_xml(&entry.value)
                ));
            }
        }

        wxl.push_str("</WixLocalization>\n");
        wxl
    }

    /// Generate a template with common installer strings
    pub fn with_common_strings(mut self, language: Language) -> Self {
        let strings = get_common_strings(language);
        for (id, value) in strings {
            self.add_string(&id, &value, language);
        }
        self
    }

    /// Import strings from WXL content
    pub fn import_wxl(&mut self, content: &str) -> Result<Language, I18nError> {
        let doc = roxmltree::Document::parse(content)
            .map_err(|e| I18nError::ParseError(e.to_string()))?;

        let root = doc.root_element();

        // Get language from Culture attribute
        let culture = root
            .attribute("Culture")
            .ok_or_else(|| I18nError::ParseError("Missing Culture attribute".to_string()))?;

        let language = Language::from_str(culture)
            .ok_or_else(|| I18nError::LanguageNotFound(culture.to_string()))?;

        // Parse String elements
        for node in root.children().filter(|n| n.has_tag_name("String")) {
            if let Some(id) = node.attribute("Id") {
                let value = node.text().unwrap_or("");
                let overridable = node.attribute("Overridable") == Some("yes");

                let mut entry = StringEntry::new(id, value);
                if overridable {
                    entry = entry.overridable();
                }
                self.add_entry(entry, language);
            }
        }

        Ok(language)
    }

    /// Export strings to JSON
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(&self.strings).unwrap_or_default()
    }

    /// Get translation coverage percentage
    pub fn coverage(&self, language: Language) -> f64 {
        let all_ids = self.string_ids();
        if all_ids.is_empty() {
            return 100.0;
        }

        let translated = self
            .strings
            .get(&language)
            .map(|m| m.len())
            .unwrap_or(0);

        (translated as f64 / all_ids.len() as f64) * 100.0
    }
}

/// Get common installer strings for a language
pub fn get_common_strings(language: Language) -> Vec<(String, String)> {
    match language {
        Language::English => vec![
            ("WelcomeTitle".into(), "Welcome to [ProductName] Setup".into()),
            ("WelcomeText".into(), "This wizard will guide you through the installation of [ProductName].".into()),
            ("LicenseTitle".into(), "License Agreement".into()),
            ("LicenseText".into(), "Please read the following license agreement carefully.".into()),
            ("LicenseAccept".into(), "I accept the terms in the License Agreement".into()),
            ("InstallDirTitle".into(), "Installation Folder".into()),
            ("InstallDirText".into(), "Choose the folder where you want to install [ProductName].".into()),
            ("ReadyTitle".into(), "Ready to Install".into()),
            ("ReadyText".into(), "Click Install to begin the installation.".into()),
            ("ProgressTitle".into(), "Installing [ProductName]".into()),
            ("ProgressText".into(), "Please wait while [ProductName] is being installed.".into()),
            ("FinishTitle".into(), "Installation Complete".into()),
            ("FinishText".into(), "[ProductName] has been successfully installed.".into()),
            ("CancelMessage".into(), "Are you sure you want to cancel the installation?".into()),
            ("BackButton".into(), "< Back".into()),
            ("NextButton".into(), "Next >".into()),
            ("InstallButton".into(), "Install".into()),
            ("CancelButton".into(), "Cancel".into()),
            ("FinishButton".into(), "Finish".into()),
        ],
        Language::French => vec![
            ("WelcomeTitle".into(), "Bienvenue dans l'installation de [ProductName]".into()),
            ("WelcomeText".into(), "Cet assistant va vous guider dans l'installation de [ProductName].".into()),
            ("LicenseTitle".into(), "Contrat de licence".into()),
            ("LicenseText".into(), "Veuillez lire attentivement le contrat de licence suivant.".into()),
            ("LicenseAccept".into(), "J'accepte les termes du contrat de licence".into()),
            ("InstallDirTitle".into(), "Dossier d'installation".into()),
            ("InstallDirText".into(), "Choisissez le dossier où vous souhaitez installer [ProductName].".into()),
            ("ReadyTitle".into(), "Prêt à installer".into()),
            ("ReadyText".into(), "Cliquez sur Installer pour commencer l'installation.".into()),
            ("ProgressTitle".into(), "Installation de [ProductName]".into()),
            ("ProgressText".into(), "Veuillez patienter pendant l'installation de [ProductName].".into()),
            ("FinishTitle".into(), "Installation terminée".into()),
            ("FinishText".into(), "[ProductName] a été installé avec succès.".into()),
            ("CancelMessage".into(), "Êtes-vous sûr de vouloir annuler l'installation ?".into()),
            ("BackButton".into(), "< Précédent".into()),
            ("NextButton".into(), "Suivant >".into()),
            ("InstallButton".into(), "Installer".into()),
            ("CancelButton".into(), "Annuler".into()),
            ("FinishButton".into(), "Terminer".into()),
        ],
        Language::German => vec![
            ("WelcomeTitle".into(), "Willkommen bei der Installation von [ProductName]".into()),
            ("WelcomeText".into(), "Dieser Assistent führt Sie durch die Installation von [ProductName].".into()),
            ("LicenseTitle".into(), "Lizenzvereinbarung".into()),
            ("LicenseText".into(), "Bitte lesen Sie die folgende Lizenzvereinbarung sorgfältig durch.".into()),
            ("LicenseAccept".into(), "Ich akzeptiere die Bedingungen der Lizenzvereinbarung".into()),
            ("InstallDirTitle".into(), "Installationsordner".into()),
            ("InstallDirText".into(), "Wählen Sie den Ordner, in dem [ProductName] installiert werden soll.".into()),
            ("ReadyTitle".into(), "Bereit zur Installation".into()),
            ("ReadyText".into(), "Klicken Sie auf Installieren, um die Installation zu starten.".into()),
            ("ProgressTitle".into(), "[ProductName] wird installiert".into()),
            ("ProgressText".into(), "Bitte warten Sie, während [ProductName] installiert wird.".into()),
            ("FinishTitle".into(), "Installation abgeschlossen".into()),
            ("FinishText".into(), "[ProductName] wurde erfolgreich installiert.".into()),
            ("CancelMessage".into(), "Möchten Sie die Installation wirklich abbrechen?".into()),
            ("BackButton".into(), "< Zurück".into()),
            ("NextButton".into(), "Weiter >".into()),
            ("InstallButton".into(), "Installieren".into()),
            ("CancelButton".into(), "Abbrechen".into()),
            ("FinishButton".into(), "Fertig stellen".into()),
        ],
        Language::Spanish => vec![
            ("WelcomeTitle".into(), "Bienvenido a la instalación de [ProductName]".into()),
            ("WelcomeText".into(), "Este asistente le guiará en la instalación de [ProductName].".into()),
            ("LicenseTitle".into(), "Acuerdo de licencia".into()),
            ("LicenseText".into(), "Por favor lea detenidamente el siguiente acuerdo de licencia.".into()),
            ("LicenseAccept".into(), "Acepto los términos del acuerdo de licencia".into()),
            ("InstallDirTitle".into(), "Carpeta de instalación".into()),
            ("InstallDirText".into(), "Elija la carpeta donde desea instalar [ProductName].".into()),
            ("ReadyTitle".into(), "Listo para instalar".into()),
            ("ReadyText".into(), "Haga clic en Instalar para comenzar la instalación.".into()),
            ("ProgressTitle".into(), "Instalando [ProductName]".into()),
            ("ProgressText".into(), "Por favor espere mientras se instala [ProductName].".into()),
            ("FinishTitle".into(), "Instalación completa".into()),
            ("FinishText".into(), "[ProductName] se ha instalado correctamente.".into()),
            ("CancelMessage".into(), "¿Está seguro de que desea cancelar la instalación?".into()),
            ("BackButton".into(), "< Atrás".into()),
            ("NextButton".into(), "Siguiente >".into()),
            ("InstallButton".into(), "Instalar".into()),
            ("CancelButton".into(), "Cancelar".into()),
            ("FinishButton".into(), "Finalizar".into()),
        ],
        _ => vec![
            ("WelcomeTitle".into(), "Welcome to [ProductName] Setup".into()),
            ("WelcomeText".into(), "This wizard will guide you through the installation of [ProductName].".into()),
            ("LicenseTitle".into(), "License Agreement".into()),
            ("LicenseAccept".into(), "I accept the terms in the License Agreement".into()),
            ("InstallButton".into(), "Install".into()),
            ("CancelButton".into(), "Cancel".into()),
            ("FinishButton".into(), "Finish".into()),
        ],
    }
}

/// Escape XML special characters
fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_lcid() {
        assert_eq!(Language::English.lcid(), 1033);
        assert_eq!(Language::French.lcid(), 1036);
        assert_eq!(Language::German.lcid(), 1031);
    }

    #[test]
    fn test_language_code() {
        assert_eq!(Language::English.code(), "en");
        assert_eq!(Language::French.code(), "fr");
        assert_eq!(Language::Japanese.code(), "ja");
    }

    #[test]
    fn test_language_culture() {
        assert_eq!(Language::English.culture(), "en-US");
        assert_eq!(Language::French.culture(), "fr-FR");
        assert_eq!(Language::Chinese.culture(), "zh-CN");
    }

    #[test]
    fn test_language_from_str() {
        assert_eq!(Language::from_str("en"), Some(Language::English));
        assert_eq!(Language::from_str("French"), Some(Language::French));
        assert_eq!(Language::from_str("de-DE"), Some(Language::German));
        assert_eq!(Language::from_str("invalid"), None);
    }

    #[test]
    fn test_add_and_get_string() {
        let mut mgr = LocalizationManager::new();
        mgr.add_string("Hello", "Hello World", Language::English);
        mgr.add_string("Hello", "Bonjour le monde", Language::French);

        assert_eq!(mgr.get_string("Hello", Language::English), Some("Hello World"));
        assert_eq!(mgr.get_string("Hello", Language::French), Some("Bonjour le monde"));
    }

    #[test]
    fn test_base_language_fallback() {
        let mut mgr = LocalizationManager::new();
        mgr.set_base_language(Language::English);
        mgr.add_string("Hello", "Hello", Language::English);

        // French doesn't have this string, should fall back to English
        assert_eq!(mgr.get_string("Hello", Language::French), Some("Hello"));
    }

    #[test]
    fn test_find_missing() {
        let mut mgr = LocalizationManager::new();
        mgr.add_string("A", "A-en", Language::English);
        mgr.add_string("B", "B-en", Language::English);
        mgr.add_string("A", "A-fr", Language::French);

        let missing = mgr.find_missing(Language::French);
        assert_eq!(missing, vec!["B".to_string()]);
    }

    #[test]
    fn test_generate_wxl() {
        let mut mgr = LocalizationManager::new();
        mgr.add_string("Title", "My App", Language::English);

        let wxl = mgr.generate_wxl(Language::English);

        assert!(wxl.contains("Culture=\"en-US\""));
        assert!(wxl.contains("Language=\"1033\""));
        assert!(wxl.contains("<String Id=\"Title\">My App</String>"));
    }

    #[test]
    fn test_string_entry_with_comment() {
        let entry = StringEntry::new("Id", "Value").with_comment("This is a comment");

        assert_eq!(entry.comment, Some("This is a comment".to_string()));
    }

    #[test]
    fn test_string_entry_overridable() {
        let entry = StringEntry::new("Id", "Value").overridable();

        assert!(entry.overridable);
    }

    #[test]
    fn test_generate_wxl_with_overridable() {
        let mut mgr = LocalizationManager::new();
        mgr.add_entry(
            StringEntry::new("Title", "My App").overridable(),
            Language::English,
        );

        let wxl = mgr.generate_wxl(Language::English);

        assert!(wxl.contains("Overridable=\"yes\""));
    }

    #[test]
    fn test_common_strings_english() {
        let strings = get_common_strings(Language::English);

        assert!(!strings.is_empty());
        assert!(strings.iter().any(|(id, _)| id == "WelcomeTitle"));
        assert!(strings.iter().any(|(id, _)| id == "InstallButton"));
    }

    #[test]
    fn test_common_strings_french() {
        let strings = get_common_strings(Language::French);

        assert!(!strings.is_empty());
        let welcome = strings.iter().find(|(id, _)| id == "WelcomeTitle");
        assert!(welcome.is_some());
        assert!(welcome.unwrap().1.contains("Bienvenue"));
    }

    #[test]
    fn test_with_common_strings() {
        let mgr = LocalizationManager::new().with_common_strings(Language::English);

        assert!(mgr.get_string("WelcomeTitle", Language::English).is_some());
        assert!(mgr.get_string("InstallButton", Language::English).is_some());
    }

    #[test]
    fn test_escape_xml() {
        assert_eq!(escape_xml("a & b"), "a &amp; b");
        assert_eq!(escape_xml("<tag>"), "&lt;tag&gt;");
        assert_eq!(escape_xml("\"quoted\""), "&quot;quoted&quot;");
    }

    #[test]
    fn test_import_wxl() {
        let wxl = r#"<?xml version="1.0" encoding="utf-8"?>
<WixLocalization Culture="en-US" Language="1033" xmlns="http://wixtoolset.org/schemas/v4/wxl">
  <String Id="Title">My App</String>
  <String Id="Version" Overridable="yes">1.0</String>
</WixLocalization>"#;

        let mut mgr = LocalizationManager::new();
        let lang = mgr.import_wxl(wxl).unwrap();

        assert_eq!(lang, Language::English);
        assert_eq!(mgr.get_string("Title", Language::English), Some("My App"));
        assert_eq!(mgr.get_string("Version", Language::English), Some("1.0"));
    }

    #[test]
    fn test_coverage() {
        let mut mgr = LocalizationManager::new();
        mgr.add_string("A", "A-en", Language::English);
        mgr.add_string("B", "B-en", Language::English);
        mgr.add_string("A", "A-fr", Language::French);

        assert_eq!(mgr.coverage(Language::English), 100.0);
        assert_eq!(mgr.coverage(Language::French), 50.0);
    }

    #[test]
    fn test_all_languages() {
        let languages = Language::all();

        assert!(languages.contains(&Language::English));
        assert!(languages.contains(&Language::French));
        assert!(languages.contains(&Language::Japanese));
        assert!(languages.len() >= 20);
    }

    #[test]
    fn test_languages_list() {
        let mut mgr = LocalizationManager::new();
        mgr.add_string("A", "A", Language::English);
        mgr.add_string("A", "A", Language::French);

        let langs = mgr.languages();
        assert!(langs.contains(&Language::English));
        assert!(langs.contains(&Language::French));
    }

    #[test]
    fn test_string_ids() {
        let mut mgr = LocalizationManager::new();
        mgr.add_string("B", "B", Language::English);
        mgr.add_string("A", "A", Language::English);
        mgr.add_string("C", "C", Language::French);

        let ids = mgr.string_ids();
        assert_eq!(ids, vec!["A", "B", "C"]);
    }

    #[test]
    fn test_to_json() {
        let mut mgr = LocalizationManager::new();
        mgr.add_string("Title", "Test", Language::English);

        let json = mgr.to_json();
        assert!(json.contains("Title"));
        assert!(json.contains("Test"));
    }

    #[test]
    fn test_generate_wxl_with_comment() {
        let mut mgr = LocalizationManager::new();
        mgr.add_entry(
            StringEntry::new("Title", "My App").with_comment("App title"),
            Language::English,
        );

        let wxl = mgr.generate_wxl(Language::English);
        assert!(wxl.contains("<!-- App title -->"));
    }
}

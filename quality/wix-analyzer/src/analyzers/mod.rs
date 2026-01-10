//! Analysis modules

pub mod best_practices;
pub mod dead_code;
pub mod references;
pub mod security;
pub mod validation;

pub use best_practices::BestPracticesAnalyzer;
pub use dead_code::DeadCodeAnalyzer;
pub use references::ReferencesAnalyzer;
pub use security::SecurityAnalyzer;
pub use validation::ValidationAnalyzer;

use crate::core::{AnalysisResult, SymbolIndex, WixDocument};
use std::path::Path;

/// Trait for analyzers
pub trait Analyzer {
    /// Analyze a document and return diagnostics
    fn analyze(&self, doc: &WixDocument, index: &SymbolIndex) -> AnalysisResult;
}

/// Run all enabled analyzers on a source file
pub fn analyze_source(
    source: &str,
    file: &Path,
    index: &SymbolIndex,
    config: &AnalyzerConfig,
) -> Result<AnalysisResult, String> {
    let doc = WixDocument::parse(source, file)?;
    let mut result = AnalysisResult::new();
    result.add_file(file.to_path_buf());

    if config.validation {
        let analyzer = ValidationAnalyzer::new();
        result.merge(analyzer.analyze(&doc, index));
    }

    if config.best_practices {
        let analyzer = BestPracticesAnalyzer::new();
        result.merge(analyzer.analyze(&doc, index));
    }

    if config.security {
        let analyzer = SecurityAnalyzer::new();
        result.merge(analyzer.analyze(&doc, index));
    }

    if config.dead_code {
        let analyzer = DeadCodeAnalyzer::new();
        result.merge(analyzer.analyze(&doc, index));
    }

    Ok(result)
}

/// Configuration for which analyzers to run
#[derive(Debug, Clone)]
pub struct AnalyzerConfig {
    pub validation: bool,
    pub best_practices: bool,
    pub security: bool,
    pub dead_code: bool,
}

impl Default for AnalyzerConfig {
    fn default() -> Self {
        Self {
            validation: true,
            best_practices: true,
            security: true,
            dead_code: true,
        }
    }
}

impl AnalyzerConfig {
    pub fn all() -> Self {
        Self::default()
    }

    pub fn none() -> Self {
        Self {
            validation: false,
            best_practices: false,
            security: false,
            dead_code: false,
        }
    }

    pub fn validation_only() -> Self {
        Self {
            validation: true,
            ..Self::none()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyzer_config_default() {
        let config = AnalyzerConfig::default();
        assert!(config.validation);
        assert!(config.best_practices);
        assert!(config.security);
        assert!(config.dead_code);
    }

    #[test]
    fn test_analyzer_config_all() {
        let config = AnalyzerConfig::all();
        assert!(config.validation);
        assert!(config.best_practices);
        assert!(config.security);
        assert!(config.dead_code);
    }

    #[test]
    fn test_analyzer_config_none() {
        let config = AnalyzerConfig::none();
        assert!(!config.validation);
        assert!(!config.best_practices);
        assert!(!config.security);
        assert!(!config.dead_code);
    }

    #[test]
    fn test_analyzer_config_validation_only() {
        let config = AnalyzerConfig::validation_only();
        assert!(config.validation);
        assert!(!config.best_practices);
        assert!(!config.security);
        assert!(!config.dead_code);
    }

    #[test]
    fn test_analyze_source_all() {
        let source = r#"<Wix><Package Name="Test" Version="1.0" /></Wix>"#;
        let index = SymbolIndex::new();
        let config = AnalyzerConfig::all();

        let result = analyze_source(source, Path::new("test.wxs"), &index, &config).unwrap();
        assert!(!result.diagnostics.is_empty());
    }

    #[test]
    fn test_analyze_source_none() {
        let source = r#"<Wix><Package Name="Test" Version="1.0" /></Wix>"#;
        let index = SymbolIndex::new();
        let config = AnalyzerConfig::none();

        let result = analyze_source(source, Path::new("test.wxs"), &index, &config).unwrap();
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_analyze_source_validation_only() {
        let source = r#"<Wix><ComponentRef Id="Missing" /></Wix>"#;
        let index = SymbolIndex::new();
        let config = AnalyzerConfig::validation_only();

        let result = analyze_source(source, Path::new("test.wxs"), &index, &config).unwrap();
        assert!(result.diagnostics.iter().any(|d| d.rule_id.starts_with("VAL-")));
    }

    #[test]
    fn test_analyze_source_invalid_xml() {
        let source = r#"<Wix><Invalid"#;
        let index = SymbolIndex::new();
        let config = AnalyzerConfig::all();

        let result = analyze_source(source, Path::new("test.wxs"), &index, &config);
        assert!(result.is_err());
    }

    #[test]
    fn test_analyze_source_security() {
        let source = r#"<Wix><Property Id="PASSWORD" Value="secret" /></Wix>"#;
        let mut index = SymbolIndex::new();
        let _ = index.index_source(source, Path::new("test.wxs"));
        let config = AnalyzerConfig {
            validation: false,
            best_practices: false,
            security: true,
            dead_code: false,
        };

        let result = analyze_source(source, Path::new("test.wxs"), &index, &config).unwrap();
        assert!(result.diagnostics.iter().any(|d| d.rule_id.starts_with("SEC-")));
    }

    #[test]
    fn test_analyze_source_dead_code() {
        let source = r#"<Wix><Property Id="unusedProp" Value="test" /></Wix>"#;
        let mut index = SymbolIndex::new();
        let _ = index.index_source(source, Path::new("test.wxs"));
        let config = AnalyzerConfig {
            validation: false,
            best_practices: false,
            security: false,
            dead_code: true,
        };

        let result = analyze_source(source, Path::new("test.wxs"), &index, &config).unwrap();
        assert!(result.diagnostics.iter().any(|d| d.rule_id.starts_with("DEAD-")));
    }
}

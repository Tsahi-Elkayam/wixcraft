//! WiX Analyzer - Unified analysis tool for WiX installation files
//!
//! This library provides comprehensive analysis capabilities for WiX XML files:
//! - Validation: Reference checking, relationship validation, attribute validation
//! - Best Practices: Efficiency, idiom, performance, and maintainability checks
//! - Security: Security vulnerability detection
//! - Dead Code: Unused element detection
//! - Auto-fix: Automatic fixes for common issues
//!
//! # Example
//!
//! ```no_run
//! use wix_analyzer::{Analyzer, Config, WixDocument, SymbolIndex};
//! use wix_analyzer::analyzers::{ValidationAnalyzer, BestPracticesAnalyzer};
//! use std::path::Path;
//!
//! let source = r#"<Wix><Package Name="Test" /></Wix>"#;
//! let doc = WixDocument::parse(source, Path::new("test.wxs")).unwrap();
//! let index = SymbolIndex::new();
//!
//! let validator = ValidationAnalyzer::new();
//! let result = validator.analyze(&doc, &index);
//!
//! for diag in result.diagnostics {
//!     println!("{}: {}", diag.rule_id, diag.message);
//! }
//! ```

pub mod core;
pub mod analyzers;
pub mod fixes;
pub mod output;
pub mod config;
pub mod lsp;
pub mod engine;
pub mod plugins;
pub mod deps;
pub mod analytics;
pub mod licenses;

// Re-export main types
pub use crate::core::{
    AnalysisResult, Category, Diagnostic, Fix, FixAction, InsertPosition,
    Location, Position, Range, Severity, SymbolDefinition, SymbolIndex,
    SymbolReference, WixDocument, SuppressionContext, IssueType, SecurityStandard,
    RelatedInfo,
    Baseline, BaselineEntry, BaselineError, BaselineStats, BASELINE_FILE_NAME, filter_baseline,
    // Cache / Incremental analysis
    AnalysisCache, CacheError, CacheStats, CACHE_DIR_NAME,
    // Diff-aware analysis
    DiffDetector, DiffError, DiffResult, DiffSource, filter_to_changed,
    // New code period
    NewCodeDetector, NewCodeError, NewCodePeriod, NewCodeResult, filter_to_new_code,
    // Plugin system
    PluginRegistry, PluginError, PluginManifest, PluginRule, RuleCondition,
    DeprecatedRuleInfo, PluginCategory, PluginSeverity,
    // Quality profiles
    ProfileName, QualityProfile, available_profiles, profile_descriptions,
    // Quality Gate
    GateCondition, GateFailure, GateResult, QualityGate, RatingType,
    // Watch mode
    FileWatcher, WatchConfig, WatchError, WatchEvent,
};
pub use crate::lsp::{WixLanguageServer, run_server, CodeActionProvider};
pub use crate::analyzers::Analyzer;
pub use crate::config::Config;
pub use crate::fixes::FixEngine;
pub use crate::output::{
    Formatter, OutputFormat, get_formatter, HtmlFormatter,
    MetricsFormatter, MetricsSummary, SeverityCounts, TypeCounts, CategoryCounts, RuleCount,
};

use std::path::Path;

/// Run all enabled analyzers on a document
pub fn analyze(doc: &WixDocument, index: &SymbolIndex, config: &Config) -> AnalysisResult {
    analyze_with_source(doc, index, config, None)
}

/// Run all enabled analyzers on a document with optional source for suppression parsing
pub fn analyze_with_source(
    doc: &WixDocument,
    index: &SymbolIndex,
    config: &Config,
    source: Option<&str>,
) -> AnalysisResult {
    use analyzers::*;

    let mut result = AnalysisResult::new();

    if config.analyzers.validation {
        let analyzer = ValidationAnalyzer::new();
        result.merge(analyzer.analyze(doc, index));
    }

    if config.analyzers.best_practices {
        let analyzer = BestPracticesAnalyzer::new();
        result.merge(analyzer.analyze(doc, index));
    }

    if config.analyzers.security {
        let analyzer = SecurityAnalyzer::new();
        result.merge(analyzer.analyze(doc, index));
    }

    if config.analyzers.dead_code {
        let analyzer = DeadCodeAnalyzer::new();
        result.merge(analyzer.analyze(doc, index));
    }

    // Filter by enabled rules
    result.diagnostics.retain(|d| config.is_rule_enabled(&d.rule_id));

    // Filter by minimum severity
    result.diagnostics.retain(|d| {
        match config.min_severity {
            config::MinSeverity::Error => d.severity >= Severity::High,
            config::MinSeverity::Warning => d.severity >= Severity::Medium,
            config::MinSeverity::Info => true,
        }
    });

    // Apply inline suppressions if source is provided
    if let Some(src) = source {
        let suppression_ctx = SuppressionContext::parse(src);
        if suppression_ctx.has_suppressions() {
            result.diagnostics.retain(|d| {
                !suppression_ctx.is_suppressed(&d.rule_id, d.location.range.start.line)
            });
        }
    }

    result
}

/// Analyze multiple files with cross-file reference resolution
pub fn analyze_project(
    files: &[&Path],
    config: &Config,
) -> Result<Vec<AnalysisResult>, String> {
    // Build cross-file index
    let mut index = SymbolIndex::new();

    for file in files {
        if config.is_excluded(file) {
            continue;
        }

        let source = std::fs::read_to_string(file)
            .map_err(|e| format!("Failed to read {}: {}", file.display(), e))?;

        index.index_source(&source, file)
            .map_err(|e| format!("Failed to index {}: {}", file.display(), e))?;
    }

    // Analyze each file
    let mut results = Vec::new();

    for file in files {
        if config.is_excluded(file) {
            continue;
        }

        let source = std::fs::read_to_string(file)
            .map_err(|e| format!("Failed to read {}: {}", file.display(), e))?;

        let doc = WixDocument::parse(&source, file)
            .map_err(|e| format!("Failed to parse {}: {}", file.display(), e))?;

        // Use analyze_with_source to support inline suppression comments
        let result = analyze_with_source(&doc, &index, config, Some(&source));
        if !result.diagnostics.is_empty() {
            results.push(result);
        }
    }

    Ok(results)
}

/// Analyze multiple files in parallel with cross-file reference resolution
///
/// This uses rayon for parallel processing and is faster for large projects.
/// Note: Index building is still sequential to ensure consistency.
pub fn analyze_project_parallel(
    files: &[&Path],
    config: &Config,
) -> Result<Vec<AnalysisResult>, String> {
    use rayon::prelude::*;
    use std::sync::Arc;

    // Build cross-file index (sequential for consistency)
    let mut index = SymbolIndex::new();

    for file in files {
        if config.is_excluded(file) {
            continue;
        }

        let source = std::fs::read_to_string(file)
            .map_err(|e| format!("Failed to read {}: {}", file.display(), e))?;

        index.index_source(&source, file)
            .map_err(|e| format!("Failed to index {}: {}", file.display(), e))?;
    }

    let index = Arc::new(index);

    // Filter out excluded files
    let files_to_analyze: Vec<_> = files
        .iter()
        .filter(|f| !config.is_excluded(f))
        .collect();

    // Analyze files in parallel
    let results: Result<Vec<_>, String> = files_to_analyze
        .par_iter()
        .map(|file| {
            let source = std::fs::read_to_string(file)
                .map_err(|e| format!("Failed to read {}: {}", file.display(), e))?;

            let doc = WixDocument::parse(&source, file)
                .map_err(|e| format!("Failed to parse {}: {}", file.display(), e))?;

            let result = analyze_with_source(&doc, &index, config, Some(&source));
            Ok(result)
        })
        .collect();

    // Filter out empty results
    Ok(results?
        .into_iter()
        .filter(|r| !r.diagnostics.is_empty())
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs::File;
    use std::io::Write;

    #[test]
    fn test_analyze_simple() {
        let source = r#"<Wix><Package Name="Test" Version="1.0" /></Wix>"#;
        let doc = WixDocument::parse(source, Path::new("test.wxs")).unwrap();
        let index = SymbolIndex::new();
        let config = Config::default();

        let result = analyze(&doc, &index, &config);
        // Should detect missing MajorUpgrade and missing UpgradeCode
        assert!(!result.diagnostics.is_empty());
    }

    #[test]
    fn test_analyzer_filtering() {
        let source = r#"<Wix><Package Name="Test" Version="1.0" /></Wix>"#;
        let doc = WixDocument::parse(source, Path::new("test.wxs")).unwrap();
        let index = SymbolIndex::new();

        let mut config = Config::default();
        config.analyzers.best_practices = false;

        let result = analyze(&doc, &index, &config);
        // Should not have best practice warnings
        assert!(result.diagnostics.iter().all(|d| !d.rule_id.starts_with("BP-")));
    }

    #[test]
    fn test_rule_filtering() {
        let source = r#"<Wix><Package Name="Test" Version="1.0" /></Wix>"#;
        let doc = WixDocument::parse(source, Path::new("test.wxs")).unwrap();
        let index = SymbolIndex::new();

        let mut config = Config::default();
        config.rules.disable.push("BP-IDIOM-001".to_string());

        let result = analyze(&doc, &index, &config);
        // Should not have the disabled rule
        assert!(result.diagnostics.iter().all(|d| d.rule_id != "BP-IDIOM-001"));
    }

    #[test]
    fn test_severity_filter_error() {
        let source = r#"<Wix><Package Name="Test" Version="1.0" /></Wix>"#;
        let doc = WixDocument::parse(source, Path::new("test.wxs")).unwrap();
        let index = SymbolIndex::new();

        let mut config = Config::default();
        config.min_severity = config::MinSeverity::Error;

        let result = analyze(&doc, &index, &config);
        // Should only have High or Blocker severity
        assert!(result.diagnostics.iter().all(|d| d.severity >= Severity::High));
    }

    #[test]
    fn test_severity_filter_warning() {
        let source = r#"<Wix><Package Name="Test" Version="1.0" /></Wix>"#;
        let doc = WixDocument::parse(source, Path::new("test.wxs")).unwrap();
        let index = SymbolIndex::new();

        let mut config = Config::default();
        config.min_severity = config::MinSeverity::Warning;

        let result = analyze(&doc, &index, &config);
        // Should only have Medium, High, or Blocker severity
        assert!(result.diagnostics.iter().all(|d| d.severity >= Severity::Medium));
    }

    #[test]
    fn test_analyze_project() {
        let temp_dir = TempDir::new().unwrap();

        // Create two files
        let file1 = temp_dir.path().join("file1.wxs");
        {
            let mut f = File::create(&file1).unwrap();
            writeln!(f, r#"<Wix><Component Id="C1" /></Wix>"#).unwrap();
        }

        let file2 = temp_dir.path().join("file2.wxs");
        {
            let mut f = File::create(&file2).unwrap();
            writeln!(f, r#"<Wix><ComponentRef Id="C1" /></Wix>"#).unwrap();
        }

        let files: Vec<&Path> = vec![file1.as_path(), file2.as_path()];
        let config = Config::default();

        let results = analyze_project(&files, &config).unwrap();
        // Should have analyzed both files
        assert!(!results.is_empty());
    }

    #[test]
    fn test_analyze_project_with_exclusion() {
        let temp_dir = TempDir::new().unwrap();

        let file1 = temp_dir.path().join("file1.wxs");
        {
            let mut f = File::create(&file1).unwrap();
            writeln!(f, r#"<Wix><Component Id="C1" /></Wix>"#).unwrap();
        }

        let file2 = temp_dir.path().join("excluded.test.wxs");
        {
            let mut f = File::create(&file2).unwrap();
            writeln!(f, r#"<Wix><Component Id="C2" /></Wix>"#).unwrap();
        }

        let files: Vec<&Path> = vec![file1.as_path(), file2.as_path()];
        let mut config = Config::default();
        config.exclude.push("*.test.wxs".to_string());

        let results = analyze_project(&files, &config).unwrap();
        // Only file1 should be analyzed
        for result in &results {
            for file in &result.files {
                assert!(!file.to_str().unwrap().contains("excluded"));
            }
        }
    }

    #[test]
    fn test_analyze_project_file_not_found() {
        let files: Vec<&Path> = vec![Path::new("/nonexistent/file.wxs")];
        let config = Config::default();

        let result = analyze_project(&files, &config);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Failed to read"));
    }

    #[test]
    fn test_analyze_project_invalid_xml() {
        let temp_dir = TempDir::new().unwrap();

        let file1 = temp_dir.path().join("invalid.wxs");
        {
            let mut f = File::create(&file1).unwrap();
            writeln!(f, r#"<Wix><Invalid"#).unwrap();
        }

        let files: Vec<&Path> = vec![file1.as_path()];
        let config = Config::default();

        let result = analyze_project(&files, &config);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("Failed to parse") || err.contains("Failed to index"));
    }

    #[test]
    fn test_analyze_all_analyzers_disabled() {
        let source = r#"<Wix><Package Name="Test" Version="1.0" /></Wix>"#;
        let doc = WixDocument::parse(source, Path::new("test.wxs")).unwrap();
        let index = SymbolIndex::new();

        let mut config = Config::default();
        config.analyzers.validation = false;
        config.analyzers.best_practices = false;
        config.analyzers.security = false;
        config.analyzers.dead_code = false;

        let result = analyze(&doc, &index, &config);
        // Should have no diagnostics
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_analyze_security_only() {
        let source = r#"<Wix><Property Id="PASSWORD" Value="secret" /></Wix>"#;
        let doc = WixDocument::parse(source, Path::new("test.wxs")).unwrap();
        let mut index = SymbolIndex::new();
        index.index_source(source, Path::new("test.wxs")).unwrap();

        let mut config = Config::default();
        config.analyzers.validation = false;
        config.analyzers.best_practices = false;
        config.analyzers.dead_code = false;
        // security stays enabled

        let result = analyze(&doc, &index, &config);
        // Should have security warnings
        assert!(result.diagnostics.iter().any(|d| d.rule_id.starts_with("SEC-")));
    }

    #[test]
    fn test_analyze_dead_code_only() {
        let source = r#"<Wix><Property Id="unusedProp" Value="test" /></Wix>"#;
        let doc = WixDocument::parse(source, Path::new("test.wxs")).unwrap();
        let mut index = SymbolIndex::new();
        index.index_source(source, Path::new("test.wxs")).unwrap();

        let mut config = Config::default();
        config.analyzers.validation = false;
        config.analyzers.best_practices = false;
        config.analyzers.security = false;
        // dead_code stays enabled

        let result = analyze(&doc, &index, &config);
        // Should have dead code warnings
        assert!(result.diagnostics.iter().any(|d| d.rule_id.starts_with("DEAD-")));
    }
}

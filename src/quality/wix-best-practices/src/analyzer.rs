//! Main best practices analyzer

use crate::practices::{
    EfficiencyAnalyzer, IdiomsAnalyzer, MaintainabilityAnalyzer, PerformanceAnalyzer,
};
use crate::types::{AnalysisResult, Impact, PracticeCategory};
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

/// Configuration for the analyzer
#[derive(Debug, Clone)]
pub struct AnalyzerConfig {
    /// Categories to analyze
    pub categories: Vec<PracticeCategory>,
    /// Minimum impact level to report
    pub min_impact: Impact,
    /// Maximum files per component (for performance checks)
    pub max_files_per_component: usize,
    /// Maximum directory depth (for performance checks)
    pub max_directory_depth: usize,
}

impl Default for AnalyzerConfig {
    fn default() -> Self {
        Self {
            categories: vec![
                PracticeCategory::Efficiency,
                PracticeCategory::Idiom,
                PracticeCategory::Performance,
                PracticeCategory::Maintainability,
            ],
            min_impact: Impact::Low,
            max_files_per_component: 1,
            max_directory_depth: 10,
        }
    }
}

/// Main best practices analyzer
pub struct BestPracticesAnalyzer {
    config: AnalyzerConfig,
    efficiency: EfficiencyAnalyzer,
    idioms: IdiomsAnalyzer,
    performance: PerformanceAnalyzer,
    maintainability: MaintainabilityAnalyzer,
}

impl BestPracticesAnalyzer {
    pub fn new() -> Self {
        Self::with_config(AnalyzerConfig::default())
    }

    pub fn with_config(config: AnalyzerConfig) -> Self {
        Self {
            efficiency: EfficiencyAnalyzer::new(),
            idioms: IdiomsAnalyzer::new(),
            performance: PerformanceAnalyzer::with_thresholds(
                config.max_files_per_component,
                config.max_directory_depth,
            ),
            maintainability: MaintainabilityAnalyzer::new(),
            config,
        }
    }

    /// Analyze a single file
    pub fn analyze_file(&self, path: &Path) -> Result<AnalysisResult, String> {
        let source = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;

        self.analyze_source(&source, path)
    }

    /// Analyze source content
    pub fn analyze_source(&self, source: &str, file: &Path) -> Result<AnalysisResult, String> {
        let mut result = AnalysisResult::new();
        result.add_file(file.to_path_buf());

        // Run enabled analyzers
        if self.config.categories.contains(&PracticeCategory::Efficiency) {
            result.extend(self.efficiency.analyze(source, file)?);
        }

        if self.config.categories.contains(&PracticeCategory::Idiom) {
            result.extend(self.idioms.analyze(source, file)?);
        }

        if self.config.categories.contains(&PracticeCategory::Performance) {
            result.extend(self.performance.analyze(source, file)?);
        }

        if self.config.categories.contains(&PracticeCategory::Maintainability) {
            result.extend(self.maintainability.analyze(source, file)?);
        }

        // Filter by minimum impact
        result.filter_by_impact(self.config.min_impact);

        Ok(result)
    }

    /// Analyze a directory recursively
    pub fn analyze_directory(&self, path: &Path) -> Result<AnalysisResult, String> {
        let mut result = AnalysisResult::new();

        for entry in WalkDir::new(path)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let entry_path = entry.path();

            // Only analyze .wxs files
            if entry_path.extension().map(|e| e == "wxs").unwrap_or(false) {
                match self.analyze_file(entry_path) {
                    Ok(file_result) => {
                        result.files.extend(file_result.files);
                        result.extend(file_result.suggestions);
                    }
                    Err(e) => {
                        eprintln!("Warning: Failed to analyze {}: {}", entry_path.display(), e);
                    }
                }
            }
        }

        Ok(result)
    }
}

impl Default for BestPracticesAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_analyzer_creation() {
        let analyzer = BestPracticesAnalyzer::new();
        let source = "<Wix />";
        let result = analyzer
            .analyze_source(source, Path::new("test.wxs"))
            .unwrap();

        assert!(result.files.contains(&PathBuf::from("test.wxs")));
    }

    #[test]
    fn test_config_categories_filter() {
        let config = AnalyzerConfig {
            categories: vec![PracticeCategory::Idiom],
            ..Default::default()
        };

        let analyzer = BestPracticesAnalyzer::with_config(config);

        // This should trigger idiom issues but not efficiency
        let source = r#"<Wix>
            <Package Name="Test" Version="1.0" />
            <Component Id="UnusedComp" />
        </Wix>"#;

        let result = analyzer
            .analyze_source(source, Path::new("test.wxs"))
            .unwrap();

        // Should have idiom issues (missing MajorUpgrade, UpgradeCode)
        assert!(result.suggestions.iter().any(|s| s.category == PracticeCategory::Idiom));

        // Should not have efficiency issues (unused component)
        assert!(result.suggestions.iter().all(|s| s.category != PracticeCategory::Efficiency));
    }

    #[test]
    fn test_config_min_impact_filter() {
        let config = AnalyzerConfig {
            min_impact: Impact::High,
            ..Default::default()
        };

        let analyzer = BestPracticesAnalyzer::with_config(config);

        let source = r#"<Wix>
            <Package Name="Test" Version="1.0" />
            <Component Id="MyComp" />
        </Wix>"#;

        let result = analyzer
            .analyze_source(source, Path::new("test.wxs"))
            .unwrap();

        // All suggestions should be High impact
        assert!(result.suggestions.iter().all(|s| s.impact == Impact::High));
    }

    #[test]
    fn test_full_analysis() {
        let analyzer = BestPracticesAnalyzer::new();

        let source = r#"<Wix>
            <Package Name="Test" Version="1.0">
                <MajorUpgrade DowngradeErrorMessage="A newer version is installed." />
            </Package>
            <Directory Id="TARGETDIR">
                <Component Id="C_Main" Guid="*">
                    <File Id="F1" Source="app.exe" />
                </Component>
            </Directory>
            <Feature Id="F_Main">
                <ComponentRef Id="C_Main" />
            </Feature>
        </Wix>"#;

        let result = analyzer
            .analyze_source(source, Path::new("test.wxs"))
            .unwrap();

        // This is a well-structured file, should have minimal issues
        // Only issue should be missing UpgradeCode on Package
        let high_impact: Vec<_> = result
            .suggestions
            .iter()
            .filter(|s| s.impact == Impact::High)
            .collect();

        // Missing UpgradeCode is the only high impact issue
        assert!(high_impact.iter().any(|s| s.rule_id == "BP-IDIOM-004"));
    }
}

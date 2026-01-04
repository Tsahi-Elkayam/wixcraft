//! WiX best practices analyzer
//!
//! Analyzes WiX XML files for efficiency, idioms, performance, and maintainability issues.
//!
//! # Example
//!
//! ```
//! use wix_best_practices::{BestPracticesAnalyzer, PracticeCategory};
//! use std::path::Path;
//!
//! let analyzer = BestPracticesAnalyzer::new();
//! let source = r#"<Wix><Package Name="Test" Version="1.0" /></Wix>"#;
//! let result = analyzer.analyze_source(source, Path::new("test.wxs")).unwrap();
//!
//! // Check for high impact issues
//! let high_impact = result.count_by_impact(wix_best_practices::Impact::High);
//! println!("Found {} high impact issues", high_impact);
//! ```

mod analyzer;
pub mod practices;
mod types;

pub use analyzer::{AnalyzerConfig, BestPracticesAnalyzer};
pub use types::{
    AnalysisResult, Impact, Location, Position, PracticeCategory, Range, SuggestedFix, Suggestion,
};

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_library_api() {
        let analyzer = BestPracticesAnalyzer::new();
        let source = "<Wix />";
        let result = analyzer.analyze_source(source, Path::new("test.wxs"));
        assert!(result.is_ok());
    }

    #[test]
    fn test_impact_levels() {
        assert!(Impact::Low < Impact::Medium);
        assert!(Impact::Medium < Impact::High);
    }

    #[test]
    fn test_category_strings() {
        assert_eq!(PracticeCategory::Efficiency.as_str(), "efficiency");
        assert_eq!(PracticeCategory::Idiom.as_str(), "idiom");
        assert_eq!(PracticeCategory::Performance.as_str(), "performance");
        assert_eq!(PracticeCategory::Maintainability.as_str(), "maintainability");
    }
}

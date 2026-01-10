//! Complexity metrics for WiX files
//!
//! Provides cyclomatic and cognitive complexity calculations for WiX documents.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Complexity metrics for a single file
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FileComplexity {
    /// File path
    pub file: PathBuf,
    /// Total element count
    pub element_count: usize,
    /// Total attribute count
    pub attribute_count: usize,
    /// Maximum nesting depth
    pub max_depth: usize,
    /// Cyclomatic complexity (decision points + 1)
    pub cyclomatic: usize,
    /// Cognitive complexity (weighted complexity considering nesting)
    pub cognitive: usize,
    /// Number of components
    pub component_count: usize,
    /// Number of features
    pub feature_count: usize,
    /// Number of custom actions
    pub custom_action_count: usize,
    /// Number of conditions
    pub condition_count: usize,
    /// Lines of code (non-empty)
    pub lines_of_code: usize,
}

impl FileComplexity {
    pub fn new(file: impl Into<PathBuf>) -> Self {
        Self {
            file: file.into(),
            cyclomatic: 1, // Base complexity is 1
            ..Default::default()
        }
    }

    /// Get complexity rating (A-E scale like SonarQube)
    pub fn rating(&self) -> ComplexityRating {
        // Based on cognitive complexity thresholds
        if self.cognitive <= 10 {
            ComplexityRating::A
        } else if self.cognitive <= 20 {
            ComplexityRating::B
        } else if self.cognitive <= 30 {
            ComplexityRating::C
        } else if self.cognitive <= 50 {
            ComplexityRating::D
        } else {
            ComplexityRating::E
        }
    }

    /// Check if complexity exceeds threshold
    pub fn exceeds_threshold(&self, threshold: &ComplexityThreshold) -> bool {
        self.cyclomatic > threshold.max_cyclomatic
            || self.cognitive > threshold.max_cognitive
            || self.max_depth > threshold.max_depth
            || self.element_count > threshold.max_elements
    }
}

/// Complexity rating (A = best, E = worst)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
pub enum ComplexityRating {
    #[default]
    A = 1,
    B = 2,
    C = 3,
    D = 4,
    E = 5,
}

impl ComplexityRating {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::A => "A",
            Self::B => "B",
            Self::C => "C",
            Self::D => "D",
            Self::E => "E",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::A => "Excellent - easy to maintain",
            Self::B => "Good - minor complexity",
            Self::C => "Moderate - some refactoring recommended",
            Self::D => "Poor - significant complexity, refactor soon",
            Self::E => "Critical - very high complexity, refactor now",
        }
    }
}

/// Thresholds for complexity warnings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityThreshold {
    pub max_cyclomatic: usize,
    pub max_cognitive: usize,
    pub max_depth: usize,
    pub max_elements: usize,
}

impl Default for ComplexityThreshold {
    fn default() -> Self {
        Self {
            max_cyclomatic: 15,
            max_cognitive: 25,
            max_depth: 10,
            max_elements: 500,
        }
    }
}

/// Complexity calculator for WiX XML documents
pub struct ComplexityCalculator {
    /// Current nesting depth during traversal
    current_depth: usize,
    /// Result being built
    result: FileComplexity,
}

impl ComplexityCalculator {
    pub fn new(file: impl Into<PathBuf>) -> Self {
        Self {
            current_depth: 0,
            result: FileComplexity::new(file),
        }
    }

    /// Calculate complexity from XML source
    pub fn calculate(mut self, source: &str) -> FileComplexity {
        // Count lines of code
        self.result.lines_of_code = source.lines().filter(|l| !l.trim().is_empty()).count();

        // Parse and traverse
        if let Ok(doc) = roxmltree::Document::parse(source) {
            self.traverse_node(doc.root_element());
        }

        self.result
    }

    fn traverse_node(&mut self, node: roxmltree::Node) {
        if !node.is_element() {
            return;
        }

        self.result.element_count += 1;
        self.current_depth += 1;

        if self.current_depth > self.result.max_depth {
            self.result.max_depth = self.current_depth;
        }

        // Count attributes
        self.result.attribute_count += node.attributes().count();

        let tag_name = node.tag_name().name();

        // Count specific element types
        match tag_name {
            "Component" => self.result.component_count += 1,
            "Feature" => self.result.feature_count += 1,
            "CustomAction" => self.result.custom_action_count += 1,
            _ => {}
        }

        // Cyclomatic complexity: count decision points
        // In WiX, these are conditions, launch conditions, and control flow
        if self.is_decision_point(tag_name, &node) {
            self.result.cyclomatic += 1;
            // Cognitive complexity adds weight for nesting
            self.result.cognitive += 1 + (self.current_depth.saturating_sub(1));
            self.result.condition_count += 1;
        }

        // Some elements add cognitive complexity due to their nature
        match tag_name {
            // These elements increase complexity due to their behavioral impact
            "CustomAction" | "SetProperty" | "SetDirectory" => {
                self.result.cognitive += 1;
            }
            // Sequences add complexity
            "InstallExecuteSequence" | "InstallUISequence" | "AdminExecuteSequence"
            | "AdminUISequence" | "AdvertiseExecuteSequence" => {
                self.result.cognitive += 2;
            }
            // Nested structures add cognitive load
            "Directory" | "DirectoryRef" if self.current_depth > 3 => {
                self.result.cognitive += 1;
            }
            _ => {}
        }

        // Recurse into children
        for child in node.children().filter(|n| n.is_element()) {
            self.traverse_node(child);
        }

        self.current_depth -= 1;
    }

    /// Check if an element is a decision point (adds to cyclomatic complexity)
    fn is_decision_point(&self, tag_name: &str, node: &roxmltree::Node) -> bool {
        match tag_name {
            // Explicit conditions
            "Condition" | "LaunchCondition" => true,
            // Conditional expressions in attributes
            _ => {
                node.attribute("Condition").is_some()
                    || node.attribute("InstallCondition").is_some()
                    || node.attribute("LaunchCondition").is_some()
            }
        }
    }
}

/// Project-wide complexity summary
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProjectComplexity {
    /// Per-file complexity
    pub files: Vec<FileComplexity>,
    /// Total element count
    pub total_elements: usize,
    /// Total attribute count
    pub total_attributes: usize,
    /// Total lines of code
    pub total_lines: usize,
    /// Total cyclomatic complexity
    pub total_cyclomatic: usize,
    /// Total cognitive complexity
    pub total_cognitive: usize,
    /// Average cognitive complexity per file
    pub avg_cognitive: f64,
    /// Maximum file complexity
    pub max_file_cognitive: usize,
    /// Overall project rating
    pub rating: ComplexityRating,
}

impl ProjectComplexity {
    pub fn new() -> Self {
        Self {
            rating: ComplexityRating::A,
            ..Default::default()
        }
    }

    pub fn add_file(&mut self, file_complexity: FileComplexity) {
        self.total_elements += file_complexity.element_count;
        self.total_attributes += file_complexity.attribute_count;
        self.total_lines += file_complexity.lines_of_code;
        self.total_cyclomatic += file_complexity.cyclomatic;
        self.total_cognitive += file_complexity.cognitive;

        if file_complexity.cognitive > self.max_file_cognitive {
            self.max_file_cognitive = file_complexity.cognitive;
        }

        self.files.push(file_complexity);
        self.recalculate();
    }

    fn recalculate(&mut self) {
        if !self.files.is_empty() {
            self.avg_cognitive = self.total_cognitive as f64 / self.files.len() as f64;
        }

        // Project rating based on average and max
        self.rating = if self.avg_cognitive <= 10.0 && self.max_file_cognitive <= 20 {
            ComplexityRating::A
        } else if self.avg_cognitive <= 15.0 && self.max_file_cognitive <= 30 {
            ComplexityRating::B
        } else if self.avg_cognitive <= 20.0 && self.max_file_cognitive <= 40 {
            ComplexityRating::C
        } else if self.avg_cognitive <= 30.0 && self.max_file_cognitive <= 60 {
            ComplexityRating::D
        } else {
            ComplexityRating::E
        };
    }

    /// Get files sorted by complexity (highest first)
    pub fn files_by_complexity(&self) -> Vec<&FileComplexity> {
        let mut files: Vec<_> = self.files.iter().collect();
        files.sort_by(|a, b| b.cognitive.cmp(&a.cognitive));
        files
    }

    /// Get files exceeding threshold
    pub fn files_exceeding_threshold(&self, threshold: &ComplexityThreshold) -> Vec<&FileComplexity> {
        self.files
            .iter()
            .filter(|f| f.exceeds_threshold(threshold))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_complexity() {
        let source = r#"<?xml version="1.0"?>
<Wix xmlns="http://wixtoolset.org/schemas/v4/wxs">
    <Package Name="Test" />
</Wix>"#;

        let calc = ComplexityCalculator::new("test.wxs");
        let result = calc.calculate(source);

        assert_eq!(result.element_count, 2); // Wix + Package
        assert!(result.cyclomatic >= 1);
        assert_eq!(result.lines_of_code, 4);
    }

    #[test]
    fn test_nested_complexity() {
        let source = r#"<?xml version="1.0"?>
<Wix>
    <Fragment>
        <Directory Id="TARGETDIR">
            <Directory Id="ProgramFilesFolder">
                <Directory Id="INSTALLFOLDER">
                    <Component Id="Comp1">
                        <File Source="file.txt" />
                    </Component>
                </Directory>
            </Directory>
        </Directory>
    </Fragment>
</Wix>"#;

        let calc = ComplexityCalculator::new("test.wxs");
        let result = calc.calculate(source);

        assert_eq!(result.max_depth, 7); // Wix > Fragment > Dir > Dir > Dir > Component > File
        assert_eq!(result.component_count, 1);
    }

    #[test]
    fn test_condition_complexity() {
        let source = r#"<?xml version="1.0"?>
<Wix>
    <Fragment>
        <Component Id="Comp1">
            <Condition>SOME_PROPERTY</Condition>
        </Component>
        <Component Id="Comp2" Condition="OTHER_PROP" />
        <LaunchCondition Message="Error">VersionNT &gt;= 600</LaunchCondition>
    </Fragment>
</Wix>"#;

        let calc = ComplexityCalculator::new("test.wxs");
        let result = calc.calculate(source);

        assert_eq!(result.condition_count, 3);
        assert!(result.cyclomatic >= 4); // 1 base + 3 conditions
        assert!(result.cognitive > 0);
    }

    #[test]
    fn test_custom_action_complexity() {
        let source = r#"<?xml version="1.0"?>
<Wix>
    <Fragment>
        <CustomAction Id="CA1" Execute="deferred" />
        <CustomAction Id="CA2" Execute="immediate" />
        <InstallExecuteSequence>
            <Custom Action="CA1" After="InstallFiles" />
        </InstallExecuteSequence>
    </Fragment>
</Wix>"#;

        let calc = ComplexityCalculator::new("test.wxs");
        let result = calc.calculate(source);

        assert_eq!(result.custom_action_count, 2);
        assert!(result.cognitive >= 4); // 2 custom actions + sequence
    }

    #[test]
    fn test_complexity_rating() {
        let mut complexity = FileComplexity::new("test.wxs");

        complexity.cognitive = 5;
        assert_eq!(complexity.rating(), ComplexityRating::A);

        complexity.cognitive = 15;
        assert_eq!(complexity.rating(), ComplexityRating::B);

        complexity.cognitive = 25;
        assert_eq!(complexity.rating(), ComplexityRating::C);

        complexity.cognitive = 40;
        assert_eq!(complexity.rating(), ComplexityRating::D);

        complexity.cognitive = 60;
        assert_eq!(complexity.rating(), ComplexityRating::E);
    }

    #[test]
    fn test_complexity_threshold() {
        let threshold = ComplexityThreshold::default();
        let mut complexity = FileComplexity::new("test.wxs");

        complexity.cyclomatic = 10;
        complexity.cognitive = 20;
        complexity.max_depth = 5;
        complexity.element_count = 100;
        assert!(!complexity.exceeds_threshold(&threshold));

        complexity.cyclomatic = 20; // Exceeds
        assert!(complexity.exceeds_threshold(&threshold));
    }

    #[test]
    fn test_project_complexity() {
        let mut project = ProjectComplexity::new();

        let mut file1 = FileComplexity::new("file1.wxs");
        file1.cognitive = 10;
        file1.element_count = 50;
        file1.lines_of_code = 100;
        project.add_file(file1);

        let mut file2 = FileComplexity::new("file2.wxs");
        file2.cognitive = 20;
        file2.element_count = 100;
        file2.lines_of_code = 200;
        project.add_file(file2);

        assert_eq!(project.files.len(), 2);
        assert_eq!(project.total_elements, 150);
        assert_eq!(project.total_lines, 300);
        assert_eq!(project.total_cognitive, 30);
        assert_eq!(project.avg_cognitive, 15.0);
        assert_eq!(project.max_file_cognitive, 20);
    }

    #[test]
    fn test_project_rating() {
        let mut project = ProjectComplexity::new();

        // Add low complexity file -> A rating
        let mut file1 = FileComplexity::new("file1.wxs");
        file1.cognitive = 5;
        project.add_file(file1);
        assert_eq!(project.rating, ComplexityRating::A);

        // Add higher complexity file -> B rating
        let mut file2 = FileComplexity::new("file2.wxs");
        file2.cognitive = 25;
        project.add_file(file2);
        assert_eq!(project.rating, ComplexityRating::B);
    }

    #[test]
    fn test_files_by_complexity() {
        let mut project = ProjectComplexity::new();

        let mut file1 = FileComplexity::new("low.wxs");
        file1.cognitive = 5;
        project.add_file(file1);

        let mut file2 = FileComplexity::new("high.wxs");
        file2.cognitive = 30;
        project.add_file(file2);

        let mut file3 = FileComplexity::new("medium.wxs");
        file3.cognitive = 15;
        project.add_file(file3);

        let sorted = project.files_by_complexity();
        assert_eq!(sorted[0].cognitive, 30);
        assert_eq!(sorted[1].cognitive, 15);
        assert_eq!(sorted[2].cognitive, 5);
    }

    #[test]
    fn test_rating_descriptions() {
        assert_eq!(ComplexityRating::A.as_str(), "A");
        assert!(ComplexityRating::A.description().contains("Excellent"));
        assert!(ComplexityRating::E.description().contains("Critical"));
    }

    #[test]
    fn test_feature_count() {
        let source = r#"<?xml version="1.0"?>
<Wix>
    <Fragment>
        <Feature Id="MainFeature">
            <Feature Id="SubFeature" />
        </Feature>
    </Fragment>
</Wix>"#;

        let calc = ComplexityCalculator::new("test.wxs");
        let result = calc.calculate(source);

        assert_eq!(result.feature_count, 2);
    }

    #[test]
    fn test_empty_document() {
        let source = "";
        let calc = ComplexityCalculator::new("test.wxs");
        let result = calc.calculate(source);

        assert_eq!(result.element_count, 0);
        assert_eq!(result.cyclomatic, 1); // Base
        assert_eq!(result.lines_of_code, 0);
    }

    #[test]
    fn test_invalid_xml() {
        let source = "not valid xml <>";
        let calc = ComplexityCalculator::new("test.wxs");
        let result = calc.calculate(source);

        // Should handle gracefully
        assert_eq!(result.element_count, 0);
        assert_eq!(result.lines_of_code, 1);
    }
}

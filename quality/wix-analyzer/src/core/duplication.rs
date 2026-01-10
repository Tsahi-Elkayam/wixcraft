//! Duplication detection for WiX files
//!
//! Finds duplicated code blocks, similar structures, and copy-paste issues.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// A duplicated code block
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Duplicate {
    /// Hash/fingerprint of the duplicated content
    pub fingerprint: u64,
    /// All locations where this duplicate appears
    pub locations: Vec<DuplicateLocation>,
    /// Number of lines duplicated
    pub lines: usize,
    /// Number of tokens/elements duplicated
    pub tokens: usize,
    /// The duplicated content (truncated for display)
    pub preview: String,
}

impl Duplicate {
    /// Get the number of duplicate instances
    pub fn count(&self) -> usize {
        self.locations.len()
    }

    /// Calculate duplicated lines (total - original)
    pub fn duplicated_lines(&self) -> usize {
        if self.locations.len() > 1 {
            self.lines * (self.locations.len() - 1)
        } else {
            0
        }
    }
}

/// Location of a duplicate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicateLocation {
    pub file: PathBuf,
    pub start_line: usize,
    pub end_line: usize,
}

impl DuplicateLocation {
    pub fn new(file: impl Into<PathBuf>, start_line: usize, end_line: usize) -> Self {
        Self {
            file: file.into(),
            start_line,
            end_line,
        }
    }
}

/// Configuration for duplication detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicationConfig {
    /// Minimum number of lines for a block to be considered
    pub min_lines: usize,
    /// Minimum number of tokens for a block to be considered
    pub min_tokens: usize,
    /// Ignore whitespace differences
    pub ignore_whitespace: bool,
    /// Ignore attribute value differences (detect structural duplicates)
    pub ignore_values: bool,
}

impl Default for DuplicationConfig {
    fn default() -> Self {
        Self {
            min_lines: 3,
            min_tokens: 10,
            ignore_whitespace: true,
            ignore_values: false,
        }
    }
}

/// Result of duplication analysis
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DuplicationResult {
    /// All detected duplicates
    pub duplicates: Vec<Duplicate>,
    /// Total lines analyzed
    pub total_lines: usize,
    /// Total duplicated lines
    pub duplicated_lines: usize,
    /// Duplication percentage
    pub duplication_percentage: f64,
}

impl DuplicationResult {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a duplicate
    pub fn add(&mut self, duplicate: Duplicate) {
        self.duplicated_lines += duplicate.duplicated_lines();
        self.duplicates.push(duplicate);
        self.recalculate();
    }

    fn recalculate(&mut self) {
        if self.total_lines > 0 {
            self.duplication_percentage =
                (self.duplicated_lines as f64 / self.total_lines as f64) * 100.0;
        }
    }

    /// Get duplication rating
    pub fn rating(&self) -> DuplicationRating {
        if self.duplication_percentage <= 3.0 {
            DuplicationRating::A
        } else if self.duplication_percentage <= 5.0 {
            DuplicationRating::B
        } else if self.duplication_percentage <= 10.0 {
            DuplicationRating::C
        } else if self.duplication_percentage <= 20.0 {
            DuplicationRating::D
        } else {
            DuplicationRating::E
        }
    }
}

/// Duplication rating
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
pub enum DuplicationRating {
    #[default]
    A = 1,
    B = 2,
    C = 3,
    D = 4,
    E = 5,
}

impl DuplicationRating {
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
            Self::A => "Excellent - minimal duplication",
            Self::B => "Good - low duplication",
            Self::C => "Moderate - some duplication, consider refactoring",
            Self::D => "Poor - high duplication, refactor recommended",
            Self::E => "Critical - very high duplication, immediate refactoring needed",
        }
    }
}

/// Duplication detector
pub struct DuplicationDetector {
    config: DuplicationConfig,
    /// Fingerprint -> locations map
    fingerprints: HashMap<u64, Vec<(PathBuf, usize, usize, String)>>,
    total_lines: usize,
}

impl DuplicationDetector {
    pub fn new(config: DuplicationConfig) -> Self {
        Self {
            config,
            fingerprints: HashMap::new(),
            total_lines: 0,
        }
    }

    pub fn with_default_config() -> Self {
        Self::new(DuplicationConfig::default())
    }

    /// Add a file to analyze
    pub fn add_file(&mut self, path: &str, content: &str) {
        let lines: Vec<&str> = content.lines().collect();
        self.total_lines += lines.len();

        // Extract blocks and compute fingerprints
        let blocks = self.extract_blocks(&lines);

        for block in blocks {
            if block.lines >= self.config.min_lines && block.tokens >= self.config.min_tokens {
                self.fingerprints
                    .entry(block.fingerprint)
                    .or_default()
                    .push((
                        PathBuf::from(path),
                        block.start_line,
                        block.end_line,
                        block.preview,
                    ));
            }
        }
    }

    /// Get duplication results
    pub fn results(&self) -> DuplicationResult {
        let mut result = DuplicationResult {
            total_lines: self.total_lines,
            ..Default::default()
        };

        for (fingerprint, locations) in &self.fingerprints {
            if locations.len() > 1 {
                let first = &locations[0];
                let lines = first.2 - first.1 + 1;

                let duplicate = Duplicate {
                    fingerprint: *fingerprint,
                    locations: locations
                        .iter()
                        .map(|(file, start, end, _)| DuplicateLocation::new(file, *start, *end))
                        .collect(),
                    lines,
                    tokens: self.count_tokens(&first.3),
                    preview: first.3.clone(),
                };

                result.add(duplicate);
            }
        }

        result
    }

    /// Extract code blocks from lines
    fn extract_blocks(&self, lines: &[&str]) -> Vec<CodeBlock> {
        let mut blocks = Vec::new();

        // Use sliding window to find blocks
        let window_sizes = [3, 5, 7, 10, 15, 20];

        for &window_size in &window_sizes {
            if window_size > lines.len() {
                continue;
            }

            for start in 0..=(lines.len() - window_size) {
                let end = start + window_size - 1;
                let block_lines = &lines[start..=end];
                let content = self.normalize_block(block_lines);

                if !content.trim().is_empty() {
                    let fingerprint = self.hash_content(&content);
                    let tokens = self.count_tokens(&content);

                    blocks.push(CodeBlock {
                        fingerprint,
                        start_line: start + 1, // 1-indexed
                        end_line: end + 1,
                        lines: window_size,
                        tokens,
                        preview: self.create_preview(block_lines),
                    });
                }
            }
        }

        blocks
    }

    /// Normalize a block for comparison
    fn normalize_block(&self, lines: &[&str]) -> String {
        let mut result = String::new();

        for line in lines {
            let mut normalized = line.to_string();

            if self.config.ignore_whitespace {
                // Normalize whitespace
                normalized = normalized.split_whitespace().collect::<Vec<_>>().join(" ");
            }

            if self.config.ignore_values {
                // Remove attribute values for structural comparison
                normalized = self.remove_attribute_values(&normalized);
            }

            if !normalized.trim().is_empty() {
                result.push_str(&normalized);
                result.push('\n');
            }
        }

        result
    }

    /// Remove attribute values (keep just structure)
    fn remove_attribute_values(&self, line: &str) -> String {
        // Simple regex-like replacement: replace ="..." with =""
        let mut result = String::new();
        let mut in_value = false;
        let mut quote_char = '"';

        for ch in line.chars() {
            if !in_value {
                result.push(ch);
                if ch == '=' {
                    // Check for quote after =
                    continue;
                }
                if ch == '"' || ch == '\'' {
                    quote_char = ch;
                    in_value = true;
                }
            } else if ch == quote_char {
                result.push(ch);
                in_value = false;
            }
            // Skip characters inside quotes when in_value is true
        }

        result
    }

    /// Hash content for fingerprinting
    fn hash_content(&self, content: &str) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);
        hasher.finish()
    }

    /// Count tokens in content
    fn count_tokens(&self, content: &str) -> usize {
        content
            .split(|c: char| c.is_whitespace() || c == '<' || c == '>' || c == '=' || c == '"')
            .filter(|s| !s.is_empty())
            .count()
    }

    /// Create a preview of the block
    fn create_preview(&self, lines: &[&str]) -> String {
        let max_preview_lines = 5;
        let preview_lines: Vec<_> = lines.iter().take(max_preview_lines).collect();
        let mut preview = preview_lines
            .iter()
            .map(|l| l.trim())
            .collect::<Vec<_>>()
            .join("\n");

        if lines.len() > max_preview_lines {
            preview.push_str("\n...");
        }

        // Truncate if too long
        if preview.len() > 200 {
            preview.truncate(197);
            preview.push_str("...");
        }

        preview
    }
}

/// Internal representation of a code block
struct CodeBlock {
    fingerprint: u64,
    start_line: usize,
    end_line: usize,
    lines: usize,
    tokens: usize,
    preview: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_duplicates() {
        let mut detector = DuplicationDetector::with_default_config();
        detector.add_file(
            "test.wxs",
            r#"<Wix>
    <Package Name="Test" />
    <Feature Id="Main" />
</Wix>"#,
        );

        let result = detector.results();
        assert!(result.duplicates.is_empty());
    }

    #[test]
    fn test_detect_duplicate_blocks() {
        let config = DuplicationConfig {
            min_lines: 2,
            min_tokens: 3,
            ..Default::default()
        };
        let mut detector = DuplicationDetector::new(config);

        // Create clearly duplicated content
        let content = r#"<Component Id="Comp1" Guid="*">
<File Source="file1.exe" />
<File Source="file2.dll" />
</Component>
<Component Id="Comp1" Guid="*">
<File Source="file1.exe" />
<File Source="file2.dll" />
</Component>"#;

        detector.add_file("test.wxs", content);
        let result = detector.results();

        // Should detect some duplication
        assert!(result.duplicated_lines > 0 || !result.duplicates.is_empty());
    }

    #[test]
    fn test_cross_file_duplicates() {
        let mut detector = DuplicationDetector::with_default_config();

        let block = r#"<Component Id="SharedComp" Guid="*">
    <File Source="shared.dll" />
    <RegistryKey Root="HKLM" Key="Software\Test" />
</Component>"#;

        detector.add_file("file1.wxs", &format!("<Wix>\n{}\n</Wix>", block));
        detector.add_file("file2.wxs", &format!("<Wix>\n{}\n</Wix>", block));

        let result = detector.results();
        assert!(!result.duplicates.is_empty());

        // Should have 2 locations for the duplicate
        let dup = result.duplicates.iter().find(|d| d.count() >= 2);
        assert!(dup.is_some());
    }

    #[test]
    fn test_duplication_rating() {
        let mut result = DuplicationResult::new();
        result.total_lines = 100;

        result.duplicated_lines = 2;
        result.recalculate();
        assert_eq!(result.rating(), DuplicationRating::A);

        result.duplicated_lines = 8;
        result.recalculate();
        assert_eq!(result.rating(), DuplicationRating::C);

        result.duplicated_lines = 25;
        result.recalculate();
        assert_eq!(result.rating(), DuplicationRating::E);
    }

    #[test]
    fn test_config_min_lines() {
        let config = DuplicationConfig {
            min_lines: 5,
            ..Default::default()
        };
        let mut detector = DuplicationDetector::new(config);

        // 3-line duplicate should not be detected with min_lines=5
        let content = r#"<Wix>
<A />
<B />
<A />
<B />
</Wix>"#;

        detector.add_file("test.wxs", content);
        let result = detector.results();

        // Small blocks should be ignored
        let small_dups = result.duplicates.iter().filter(|d| d.lines < 5).count();
        assert_eq!(small_dups, 0);
    }

    #[test]
    fn test_ignore_whitespace() {
        let config = DuplicationConfig {
            ignore_whitespace: true,
            min_lines: 2,
            min_tokens: 5,
            ..Default::default()
        };
        let mut detector = DuplicationDetector::new(config);

        let content = r#"<Component Id="A">
    <File Source="test.exe" />
</Component>
<Component  Id="A">
  <File  Source="test.exe" />
</Component>"#;

        detector.add_file("test.wxs", content);
        let result = detector.results();

        // Should detect as duplicates despite whitespace differences
        assert!(!result.duplicates.is_empty() || result.duplicated_lines > 0);
    }

    #[test]
    fn test_duplicate_location() {
        let loc = DuplicateLocation::new("test.wxs", 5, 10);
        assert_eq!(loc.file, PathBuf::from("test.wxs"));
        assert_eq!(loc.start_line, 5);
        assert_eq!(loc.end_line, 10);
    }

    #[test]
    fn test_duplicate_count() {
        let dup = Duplicate {
            fingerprint: 12345,
            locations: vec![
                DuplicateLocation::new("a.wxs", 1, 5),
                DuplicateLocation::new("b.wxs", 1, 5),
                DuplicateLocation::new("c.wxs", 1, 5),
            ],
            lines: 5,
            tokens: 20,
            preview: "...".to_string(),
        };

        assert_eq!(dup.count(), 3);
        assert_eq!(dup.duplicated_lines(), 10); // 5 lines * (3-1) instances
    }

    #[test]
    fn test_rating_descriptions() {
        assert!(DuplicationRating::A.description().contains("Excellent"));
        assert!(DuplicationRating::C.description().contains("Moderate"));
        assert!(DuplicationRating::E.description().contains("Critical"));
    }

    #[test]
    fn test_empty_file() {
        let mut detector = DuplicationDetector::with_default_config();
        detector.add_file("empty.wxs", "");

        let result = detector.results();
        assert!(result.duplicates.is_empty());
        assert_eq!(result.duplication_percentage, 0.0);
    }

    #[test]
    fn test_single_line_file() {
        let mut detector = DuplicationDetector::with_default_config();
        detector.add_file("single.wxs", "<Wix />");

        let result = detector.results();
        assert!(result.duplicates.is_empty());
    }
}

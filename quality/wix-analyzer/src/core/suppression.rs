//! Inline suppression support for wix-analyzer
//!
//! Supports suppression comments in WiX XML files:
//! - `<!-- wix-analyzer-disable RULE-001 -->` - Disable specific rule for next line
//! - `<!-- wix-analyzer-disable RULE-001, RULE-002 -->` - Disable multiple rules
//! - `<!-- wix-analyzer-disable-next-line RULE-001 -->` - Disable for next line only
//! - `<!-- wix-analyzer-disable -->` - Disable all rules until enabled
//! - `<!-- wix-analyzer-enable -->` - Re-enable all rules
//!
//! Also supports inline suppression:
//! - `<Element /> <!-- wix-analyzer-disable-line RULE-001 -->`

use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

/// Pattern to match suppression comments
static DISABLE_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"<!--\s*wix-analyzer-disable(?:-next-line|-line)?\s*([\w\-,\s]*)\s*-->").unwrap()
});

/// Pattern to match enable comments
static ENABLE_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"<!--\s*wix-analyzer-enable\s*-->").unwrap());

/// Type of suppression
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SuppressionKind {
    /// Disable for the current line (inline comment)
    Line,
    /// Disable for the next line only
    NextLine,
    /// Disable until re-enabled (block)
    Block,
}

/// A suppression directive
#[derive(Debug, Clone)]
pub struct Suppression {
    /// Line number where the suppression appears (1-based)
    pub line: usize,
    /// Rules to suppress (empty means all rules)
    pub rules: HashSet<String>,
    /// Type of suppression
    pub kind: SuppressionKind,
}

/// Suppression context for filtering diagnostics
#[derive(Debug, Default)]
pub struct SuppressionContext {
    /// Suppressions by affected line number
    line_suppressions: HashMap<usize, HashSet<String>>,
    /// Block suppressions: (start_line, end_line, rules)
    block_suppressions: Vec<(usize, usize, HashSet<String>)>,
}

impl SuppressionContext {
    /// Parse suppression comments from source
    pub fn parse(source: &str) -> Self {
        let mut ctx = Self::default();
        let lines: Vec<&str> = source.lines().collect();

        let mut block_start: Option<(usize, HashSet<String>)> = None;

        for (idx, line) in lines.iter().enumerate() {
            let line_num = idx + 1; // 1-based

            // Check for enable comment (ends block)
            if ENABLE_PATTERN.is_match(line) {
                if let Some((start, rules)) = block_start.take() {
                    ctx.block_suppressions.push((start, line_num, rules));
                }
                continue;
            }

            // Check for disable comment
            if let Some(caps) = DISABLE_PATTERN.captures(line) {
                let rules_str = caps.get(1).map(|m| m.as_str()).unwrap_or("");
                let rules = parse_rules(rules_str);
                let is_inline = line.contains("-line");
                let is_next_line = line.contains("-next-line");

                if is_inline && !is_next_line {
                    // -line: suppress on this line only
                    ctx.add_line_suppression(line_num, rules);
                } else if is_next_line {
                    // -next-line: suppress on the next line
                    ctx.add_line_suppression(line_num + 1, rules);
                } else if rules.is_empty() {
                    // Block disable all rules
                    block_start = Some((line_num + 1, HashSet::new()));
                } else {
                    // Block disable specific rules (or just next line if rules specified)
                    // Convention: with specific rules, it's next-line behavior
                    ctx.add_line_suppression(line_num + 1, rules);
                }
            }
        }

        // Handle unclosed block (extends to end of file)
        if let Some((start, rules)) = block_start {
            ctx.block_suppressions.push((start, lines.len() + 1, rules));
        }

        ctx
    }

    fn add_line_suppression(&mut self, line: usize, rules: HashSet<String>) {
        self.line_suppressions
            .entry(line)
            .or_default()
            .extend(rules);
    }

    /// Check if a rule is suppressed at the given line
    pub fn is_suppressed(&self, rule_id: &str, line: usize) -> bool {
        // Check line-specific suppression
        if let Some(rules) = self.line_suppressions.get(&line) {
            if rules.is_empty() || rules.contains(rule_id) {
                return true;
            }
        }

        // Check block suppressions
        for (start, end, rules) in &self.block_suppressions {
            if line >= *start && line <= *end
                && (rules.is_empty() || rules.contains(rule_id)) {
                    return true;
                }
        }

        false
    }

    /// Get all suppressed rule IDs at a line
    pub fn suppressed_rules(&self, line: usize) -> HashSet<String> {
        let mut rules = HashSet::new();

        if let Some(line_rules) = self.line_suppressions.get(&line) {
            rules.extend(line_rules.iter().cloned());
        }

        for (start, end, block_rules) in &self.block_suppressions {
            if line >= *start && line <= *end {
                rules.extend(block_rules.iter().cloned());
            }
        }

        rules
    }

    /// Check if any suppressions exist
    pub fn has_suppressions(&self) -> bool {
        !self.line_suppressions.is_empty() || !self.block_suppressions.is_empty()
    }
}

/// Parse comma-separated rule IDs from a string
fn parse_rules(s: &str) -> HashSet<String> {
    s.split(',')
        .map(|r| r.trim().to_uppercase())
        .filter(|r| !r.is_empty())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_rule_next_line() {
        let source = r#"
<!-- wix-analyzer-disable SEC-001 -->
<Property Id="PASSWORD" Value="secret" />
<Property Id="OTHER" Value="test" />
"#;
        let ctx = SuppressionContext::parse(source);

        assert!(ctx.is_suppressed("SEC-001", 3));
        assert!(!ctx.is_suppressed("SEC-001", 4));
        assert!(!ctx.is_suppressed("SEC-002", 3));
    }

    #[test]
    fn test_explicit_next_line() {
        let source = r#"
<!-- wix-analyzer-disable-next-line SEC-001 -->
<Property Id="PASSWORD" Value="secret" />
"#;
        let ctx = SuppressionContext::parse(source);

        assert!(ctx.is_suppressed("SEC-001", 3));
        assert!(!ctx.is_suppressed("SEC-001", 2));
    }

    #[test]
    fn test_inline_disable_line() {
        let source = r#"
<Property Id="PASSWORD" Value="secret" /> <!-- wix-analyzer-disable-line SEC-001 -->
<Property Id="OTHER" Value="test" />
"#;
        let ctx = SuppressionContext::parse(source);

        assert!(ctx.is_suppressed("SEC-001", 2));
        assert!(!ctx.is_suppressed("SEC-001", 3));
    }

    #[test]
    fn test_multiple_rules() {
        let source = r#"
<!-- wix-analyzer-disable SEC-001, SEC-002 -->
<Element />
"#;
        let ctx = SuppressionContext::parse(source);

        assert!(ctx.is_suppressed("SEC-001", 3));
        assert!(ctx.is_suppressed("SEC-002", 3));
        assert!(!ctx.is_suppressed("SEC-003", 3));
    }

    #[test]
    fn test_block_disable_all() {
        let source = r#"
<!-- wix-analyzer-disable -->
<Element1 />
<Element2 />
<Element3 />
<!-- wix-analyzer-enable -->
<Element4 />
"#;
        let ctx = SuppressionContext::parse(source);

        assert!(ctx.is_suppressed("ANY-RULE", 3));
        assert!(ctx.is_suppressed("ANY-RULE", 4));
        assert!(ctx.is_suppressed("ANY-RULE", 5));
        assert!(!ctx.is_suppressed("ANY-RULE", 7));
    }

    #[test]
    fn test_unclosed_block() {
        let source = r#"
<!-- wix-analyzer-disable -->
<Element1 />
<Element2 />
"#;
        let ctx = SuppressionContext::parse(source);

        // Block extends to end of file
        assert!(ctx.is_suppressed("ANY-RULE", 3));
        assert!(ctx.is_suppressed("ANY-RULE", 4));
    }

    #[test]
    fn test_case_insensitive_rules() {
        let source = r#"
<!-- wix-analyzer-disable sec-001 -->
<Element />
"#;
        let ctx = SuppressionContext::parse(source);

        assert!(ctx.is_suppressed("SEC-001", 3));
    }

    #[test]
    fn test_no_suppressions() {
        let source = r#"
<Wix>
    <Package Name="Test" />
</Wix>
"#;
        let ctx = SuppressionContext::parse(source);

        assert!(!ctx.has_suppressions());
        assert!(!ctx.is_suppressed("SEC-001", 2));
    }

    #[test]
    fn test_suppressed_rules_at_line() {
        let source = r#"
<!-- wix-analyzer-disable SEC-001, BP-002 -->
<Element />
"#;
        let ctx = SuppressionContext::parse(source);

        let rules = ctx.suppressed_rules(3);
        assert!(rules.contains("SEC-001"));
        assert!(rules.contains("BP-002"));
        assert_eq!(rules.len(), 2);
    }

    #[test]
    fn test_whitespace_handling() {
        let source = r#"
<!--   wix-analyzer-disable   SEC-001  ,  SEC-002   -->
<Element />
"#;
        let ctx = SuppressionContext::parse(source);

        assert!(ctx.is_suppressed("SEC-001", 3));
        assert!(ctx.is_suppressed("SEC-002", 3));
    }

    #[test]
    fn test_has_suppressions() {
        let source = r#"
<!-- wix-analyzer-disable SEC-001 -->
<Element />
"#;
        let ctx = SuppressionContext::parse(source);
        assert!(ctx.has_suppressions());
    }
}

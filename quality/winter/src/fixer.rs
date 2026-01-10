//! Auto-fix system for applying rule fixes to files
//!
//! Supports various fix actions:
//! - Add/remove/set attributes
//! - Replace/remove elements
//! - Custom text replacements
//!
//! Fixes are classified as safe or unsafe:
//! - Safe fixes preserve code meaning and can be applied automatically
//! - Unsafe fixes may change runtime behavior and require explicit opt-in

use crate::diagnostic::{Diagnostic, FixSafety, Location};
use crate::rule::{FixAction, FixSuggestion};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// A fix to be applied to a file
#[derive(Debug, Clone)]
pub struct Fix {
    /// File path
    pub file: PathBuf,
    /// Location in file
    pub location: Location,
    /// The fix suggestion
    pub suggestion: FixSuggestion,
    /// Original text to replace (if known)
    pub original: Option<String>,
    /// Rule ID that generated this fix
    pub rule_id: String,
    /// Safety classification
    pub safety: FixSafety,
}

/// Result of applying fixes
#[derive(Debug, Default)]
pub struct FixResult {
    /// Number of files modified
    pub files_modified: usize,
    /// Number of fixes applied
    pub fixes_applied: usize,
    /// Number of safe fixes applied
    pub safe_fixes_applied: usize,
    /// Number of unsafe fixes applied
    pub unsafe_fixes_applied: usize,
    /// Number of fixes that failed
    pub fixes_failed: usize,
    /// Number of fixes skipped (unsafe when not allowed)
    pub fixes_skipped: usize,
    /// Errors encountered
    pub errors: Vec<String>,
    /// Diff output (if diff mode enabled)
    pub diffs: HashMap<PathBuf, String>,
}

/// Fix mode options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FixMode {
    /// Apply only safe fixes (default)
    #[default]
    SafeOnly,
    /// Apply all fixes including unsafe
    All,
    /// Diff mode - show changes without applying
    Diff,
    /// Show fixes without applying
    ShowOnly,
}

/// Auto-fixer that applies fixes to files
pub struct Fixer {
    /// Dry run mode (don't write changes)
    dry_run: bool,
    /// Fixes grouped by file
    fixes_by_file: HashMap<PathBuf, Vec<Fix>>,
    /// Fix mode
    mode: FixMode,
    /// Include unsafe fixes
    include_unsafe: bool,
}

impl Fixer {
    /// Create a new fixer
    pub fn new(dry_run: bool) -> Self {
        Self {
            dry_run,
            fixes_by_file: HashMap::new(),
            mode: FixMode::SafeOnly,
            include_unsafe: false,
        }
    }

    /// Set the fix mode
    pub fn with_mode(mut self, mode: FixMode) -> Self {
        self.mode = mode;
        self
    }

    /// Include unsafe fixes
    pub fn with_unsafe_fixes(mut self, include: bool) -> Self {
        self.include_unsafe = include;
        if include {
            self.mode = FixMode::All;
        }
        self
    }

    /// Set diff mode
    pub fn with_diff_mode(mut self) -> Self {
        self.mode = FixMode::Diff;
        self
    }

    /// Set show-only mode
    pub fn with_show_only(mut self) -> Self {
        self.mode = FixMode::ShowOnly;
        self
    }

    /// Collect fixes from diagnostics
    pub fn collect_from_diagnostics(&mut self, diagnostics: &[Diagnostic]) {
        for diag in diagnostics {
            if let Some(fix) = &diag.fix {
                let fix_entry = Fix {
                    file: diag.location.file.clone(),
                    location: diag.location.clone(),
                    suggestion: FixSuggestion {
                        action: FixAction::Custom,
                        attribute: None,
                        value: Some(fix.replacement.clone()),
                        description: Some(fix.description.clone()),
                    },
                    original: diag.source_line.clone(),
                    rule_id: diag.rule_id.clone(),
                    safety: fix.safety,
                };

                self.fixes_by_file
                    .entry(diag.location.file.clone())
                    .or_default()
                    .push(fix_entry);
            }
        }
    }

    /// Add a fix manually
    pub fn add_fix(&mut self, fix: Fix) {
        self.fixes_by_file
            .entry(fix.file.clone())
            .or_default()
            .push(fix);
    }

    /// Check if a fix should be applied based on mode and safety
    fn should_apply_fix(&self, fix: &Fix) -> bool {
        match self.mode {
            FixMode::All => true,
            FixMode::SafeOnly => fix.safety == FixSafety::Safe,
            FixMode::Diff | FixMode::ShowOnly => {
                if self.include_unsafe {
                    true
                } else {
                    fix.safety == FixSafety::Safe
                }
            }
        }
    }

    /// Apply all collected fixes
    pub fn apply_all(&self) -> FixResult {
        let mut result = FixResult::default();

        for (file, fixes) in &self.fixes_by_file {
            // Filter fixes based on mode
            let applicable_fixes: Vec<_> = fixes
                .iter()
                .filter(|f| self.should_apply_fix(f))
                .cloned()
                .collect();

            let skipped = fixes.len() - applicable_fixes.len();
            result.fixes_skipped += skipped;

            if self.mode == FixMode::ShowOnly {
                // Just count fixes without applying
                for fix in &applicable_fixes {
                    if fix.safety == FixSafety::Safe {
                        result.safe_fixes_applied += 1;
                    } else {
                        result.unsafe_fixes_applied += 1;
                    }
                    result.fixes_applied += 1;
                }
                continue;
            }

            match self.apply_fixes_to_file(file, &applicable_fixes, &mut result) {
                Ok(count) => {
                    if count > 0 {
                        result.files_modified += 1;
                        result.fixes_applied += count;
                    }
                }
                Err(e) => {
                    result.fixes_failed += applicable_fixes.len();
                    result.errors.push(format!("{}: {}", file.display(), e));
                }
            }
        }

        result
    }

    /// Get all fixes that would be applied (for --show-fixes)
    pub fn get_pending_fixes(&self) -> Vec<&Fix> {
        let mut all_fixes = Vec::new();
        for fixes in self.fixes_by_file.values() {
            for fix in fixes {
                if self.should_apply_fix(fix) {
                    all_fixes.push(fix);
                }
            }
        }
        all_fixes.sort_by(|a, b| {
            a.file
                .cmp(&b.file)
                .then(a.location.line.cmp(&b.location.line))
        });
        all_fixes
    }

    /// Format fixes for display (--show-fixes)
    pub fn format_fixes(&self) -> String {
        let fixes = self.get_pending_fixes();
        let mut output = String::new();

        if fixes.is_empty() {
            return "No fixes available.\n".to_string();
        }

        output.push_str(&format!("Found {} fix(es):\n\n", fixes.len()));

        let mut current_file: Option<&PathBuf> = None;
        for fix in fixes {
            if current_file != Some(&fix.file) {
                current_file = Some(&fix.file);
                output.push_str(&format!("{}:\n", fix.file.display()));
            }

            let safety_marker = match fix.safety {
                FixSafety::Safe => "[safe]",
                FixSafety::Unsafe => "[unsafe]",
                FixSafety::Display => "[display]",
            };

            output.push_str(&format!(
                "  Line {}: {} {} - {}\n",
                fix.location.line,
                safety_marker,
                fix.rule_id,
                fix.suggestion
                    .description
                    .as_deref()
                    .unwrap_or("No description")
            ));
        }

        output
    }

    /// Apply fixes to a single file
    fn apply_fixes_to_file(
        &self,
        file: &Path,
        fixes: &[Fix],
        result: &mut FixResult,
    ) -> Result<usize, std::io::Error> {
        if fixes.is_empty() {
            return Ok(0);
        }

        let content = std::fs::read_to_string(file)?;
        let lines: Vec<&str> = content.lines().collect();

        // Sort fixes by line number in reverse order (apply from bottom to top)
        let mut sorted_fixes = fixes.to_vec();
        sorted_fixes.sort_by(|a, b| b.location.line.cmp(&a.location.line));

        let mut modified_lines: Vec<String> = lines.iter().map(|s| s.to_string()).collect();
        let mut applied = 0;

        for fix in &sorted_fixes {
            if fix.location.line == 0 || fix.location.line > modified_lines.len() {
                continue;
            }

            let line_idx = fix.location.line - 1;
            let line = &modified_lines[line_idx];

            if let Some(new_line) = self.apply_fix_to_line(line, fix) {
                modified_lines[line_idx] = new_line;
                applied += 1;

                // Track safe vs unsafe fixes
                if fix.safety == FixSafety::Safe {
                    result.safe_fixes_applied += 1;
                } else {
                    result.unsafe_fixes_applied += 1;
                }
            }
        }

        if applied > 0 {
            let new_content = modified_lines.join("\n");
            // Preserve trailing newline if original had one
            let new_content = if content.ends_with('\n') {
                format!("{}\n", new_content)
            } else {
                new_content
            };

            if self.mode == FixMode::Diff {
                // Generate unified diff
                let diff = generate_unified_diff(file, &content, &new_content);
                result.diffs.insert(file.to_path_buf(), diff);
            } else if !self.dry_run {
                std::fs::write(file, new_content)?;
            }
        }

        Ok(applied)
    }

    /// Apply a single fix to a line
    fn apply_fix_to_line(&self, line: &str, fix: &Fix) -> Option<String> {
        match &fix.suggestion.action {
            FixAction::AddAttribute => {
                // Add attribute to element
                if let (Some(attr), Some(value)) =
                    (&fix.suggestion.attribute, &fix.suggestion.value)
                {
                    // Find the element closing > or />
                    if let Some(pos) = line.rfind("/>") {
                        let new_line = format!(
                            "{} {}=\"{}\"{}",
                            &line[..pos].trim_end(),
                            attr,
                            value,
                            &line[pos..]
                        );
                        return Some(new_line);
                    } else if let Some(pos) = line.rfind('>') {
                        let new_line = format!(
                            "{} {}=\"{}\"{}",
                            &line[..pos].trim_end(),
                            attr,
                            value,
                            &line[pos..]
                        );
                        return Some(new_line);
                    }
                }
            }
            FixAction::RemoveAttribute => {
                // Remove attribute from element
                if let Some(attr) = &fix.suggestion.attribute {
                    let pattern = format!(r#"{}="[^"]*""#, attr);
                    if let Ok(re) = regex::Regex::new(&pattern) {
                        let new_line = re.replace(line, "").to_string();
                        // Clean up extra spaces
                        let new_line = new_line.replace("  ", " ");
                        return Some(new_line);
                    }
                }
            }
            FixAction::SetAttribute => {
                // Change attribute value
                if let (Some(attr), Some(value)) =
                    (&fix.suggestion.attribute, &fix.suggestion.value)
                {
                    let pattern = format!(r#"{}="[^"]*""#, attr);
                    if let Ok(re) = regex::Regex::new(&pattern) {
                        let replacement = format!("{}=\"{}\"", attr, value);
                        let new_line = re.replace(line, replacement.as_str()).to_string();
                        return Some(new_line);
                    }
                }
            }
            FixAction::ReplaceElement => {
                // Replace entire element
                if let Some(value) = &fix.suggestion.value {
                    return Some(value.clone());
                }
            }
            FixAction::RemoveElement => {
                // Remove the entire line
                return Some(String::new());
            }
            FixAction::RenameElement => {
                // Rename element tag
                if let Some(new_name) = &fix.suggestion.value {
                    // Match opening tag
                    let open_pattern = r"<([a-zA-Z][a-zA-Z0-9]*)";
                    if let Ok(re) = regex::Regex::new(open_pattern) {
                        let new_line = re
                            .replace(line, format!("<{}", new_name).as_str())
                            .to_string();
                        // Also rename closing tag if present
                        let close_pattern = r"</([a-zA-Z][a-zA-Z0-9]*)>";
                        if let Ok(re_close) = regex::Regex::new(close_pattern) {
                            let new_line = re_close
                                .replace(&new_line, format!("</{}>", new_name).as_str())
                                .to_string();
                            return Some(new_line);
                        }
                        return Some(new_line);
                    }
                }
            }
            FixAction::Custom => {
                // Custom fix - use the value as replacement hint
                // For now, just return the suggestion value if it looks like a replacement
                if let Some(value) = &fix.suggestion.value {
                    if !value.is_empty() && !value.contains('\n') {
                        // Simple single-line replacement
                        return Some(value.clone());
                    }
                }
            }
        }

        None
    }

    /// Get count of fixes pending
    pub fn pending_count(&self) -> usize {
        self.fixes_by_file.values().map(|v| v.len()).sum()
    }

    /// Get fixes grouped by file
    pub fn fixes_by_file(&self) -> &HashMap<PathBuf, Vec<Fix>> {
        &self.fixes_by_file
    }

    /// Check if running in dry-run mode
    pub fn is_dry_run(&self) -> bool {
        self.dry_run
    }

    /// Get the current fix mode
    pub fn mode(&self) -> FixMode {
        self.mode
    }

    /// Format diff output for display
    pub fn format_diffs(&self, result: &FixResult) -> String {
        let mut output = String::new();

        for (file, diff) in &result.diffs {
            output.push_str(&format!(
                "diff --winter a/{} b/{}\n",
                file.display(),
                file.display()
            ));
            output.push_str(diff);
            output.push('\n');
        }

        output
    }
}

/// Generate a unified diff between two strings
fn generate_unified_diff(file: &Path, original: &str, modified: &str) -> String {
    let mut diff = String::new();

    let original_lines: Vec<&str> = original.lines().collect();
    let modified_lines: Vec<&str> = modified.lines().collect();

    diff.push_str(&format!("--- a/{}\n", file.display()));
    diff.push_str(&format!("+++ b/{}\n", file.display()));

    // Simple line-by-line diff (not optimal, but functional)
    let max_len = original_lines.len().max(modified_lines.len());
    let mut in_hunk = false;
    let mut hunk_start = 0;
    let mut hunk_lines: Vec<String> = Vec::new();

    for i in 0..max_len {
        let orig = original_lines.get(i);
        let modif = modified_lines.get(i);

        match (orig, modif) {
            (Some(o), Some(m)) if o == m => {
                if in_hunk {
                    // Context line in hunk
                    hunk_lines.push(format!(" {}", o));
                }
            }
            (Some(o), Some(m)) => {
                if !in_hunk {
                    in_hunk = true;
                    hunk_start = i + 1;
                    // Add context before
                    if i > 0 {
                        if let Some(ctx) = original_lines.get(i - 1) {
                            hunk_lines.push(format!(" {}", ctx));
                        }
                    }
                }
                hunk_lines.push(format!("-{}", o));
                hunk_lines.push(format!("+{}", m));
            }
            (Some(o), None) => {
                if !in_hunk {
                    in_hunk = true;
                    hunk_start = i + 1;
                }
                hunk_lines.push(format!("-{}", o));
            }
            (None, Some(m)) => {
                if !in_hunk {
                    in_hunk = true;
                    hunk_start = i + 1;
                }
                hunk_lines.push(format!("+{}", m));
            }
            (None, None) => {}
        }
    }

    if !hunk_lines.is_empty() {
        diff.push_str(&format!(
            "@@ -{},{} +{},{} @@\n",
            hunk_start,
            original_lines.len(),
            hunk_start,
            modified_lines.len()
        ));
        for line in hunk_lines {
            diff.push_str(&line);
            diff.push('\n');
        }
    }

    diff
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fixer_new() {
        let fixer = Fixer::new(true);
        assert!(fixer.is_dry_run());
        assert_eq!(fixer.pending_count(), 0);
    }

    #[test]
    fn test_add_attribute_fix() {
        let fixer = Fixer::new(true);
        let line = r#"<Component Id="MyComp">"#;
        let fix = Fix {
            file: PathBuf::from("test.wxs"),
            location: Location::new(PathBuf::from("test.wxs"), 1, 0),
            suggestion: FixSuggestion {
                action: FixAction::AddAttribute,
                attribute: Some("Guid".to_string()),
                value: Some("*".to_string()),
                description: None,
            },
            original: None,
            rule_id: "test".to_string(),
            safety: FixSafety::Safe,
        };

        let result = fixer.apply_fix_to_line(line, &fix);
        assert!(result.is_some());
        assert!(result.unwrap().contains(r#"Guid="*""#));
    }

    #[test]
    fn test_set_attribute_fix() {
        let fixer = Fixer::new(true);
        let line = r#"<Component Id="MyComp" Guid="old-guid">"#;
        let fix = Fix {
            file: PathBuf::from("test.wxs"),
            location: Location::new(PathBuf::from("test.wxs"), 1, 0),
            suggestion: FixSuggestion {
                action: FixAction::SetAttribute,
                attribute: Some("Guid".to_string()),
                value: Some("*".to_string()),
                description: None,
            },
            original: None,
            rule_id: "test".to_string(),
            safety: FixSafety::Safe,
        };

        let result = fixer.apply_fix_to_line(line, &fix);
        assert!(result.is_some());
        let new_line = result.unwrap();
        assert!(new_line.contains(r#"Guid="*""#));
        assert!(!new_line.contains("old-guid"));
    }

    #[test]
    fn test_remove_attribute_fix() {
        let fixer = Fixer::new(true);
        let line = r#"<Component Id="MyComp" Obsolete="yes">"#;
        let fix = Fix {
            file: PathBuf::from("test.wxs"),
            location: Location::new(PathBuf::from("test.wxs"), 1, 0),
            suggestion: FixSuggestion {
                action: FixAction::RemoveAttribute,
                attribute: Some("Obsolete".to_string()),
                value: None,
                description: None,
            },
            original: None,
            rule_id: "test".to_string(),
            safety: FixSafety::Safe,
        };

        let result = fixer.apply_fix_to_line(line, &fix);
        assert!(result.is_some());
        assert!(!result.unwrap().contains("Obsolete"));
    }

    #[test]
    fn test_remove_element_fix() {
        let fixer = Fixer::new(true);
        let line = r#"<DeprecatedElement />"#;
        let fix = Fix {
            file: PathBuf::from("test.wxs"),
            location: Location::new(PathBuf::from("test.wxs"), 1, 0),
            suggestion: FixSuggestion {
                action: FixAction::RemoveElement,
                attribute: None,
                value: None,
                description: None,
            },
            original: None,
            rule_id: "test".to_string(),
            safety: FixSafety::Unsafe, // Removing elements is potentially unsafe
        };

        let result = fixer.apply_fix_to_line(line, &fix);
        assert!(result.is_some());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_fix_result_default() {
        let result = FixResult::default();
        assert_eq!(result.files_modified, 0);
        assert_eq!(result.fixes_applied, 0);
        assert_eq!(result.fixes_failed, 0);
        assert_eq!(result.safe_fixes_applied, 0);
        assert_eq!(result.unsafe_fixes_applied, 0);
        assert_eq!(result.fixes_skipped, 0);
    }

    #[test]
    fn test_fix_modes() {
        let fixer = Fixer::new(true).with_mode(FixMode::SafeOnly);
        assert_eq!(fixer.mode(), FixMode::SafeOnly);

        let fixer = Fixer::new(true).with_unsafe_fixes(true);
        assert_eq!(fixer.mode(), FixMode::All);

        let fixer = Fixer::new(true).with_diff_mode();
        assert_eq!(fixer.mode(), FixMode::Diff);
    }

    #[test]
    fn test_generate_diff() {
        let original = "line1\nline2\nline3\n";
        let modified = "line1\nmodified\nline3\n";
        let diff = generate_unified_diff(Path::new("test.wxs"), original, modified);

        assert!(diff.contains("--- a/test.wxs"));
        assert!(diff.contains("+++ b/test.wxs"));
        assert!(diff.contains("-line2"));
        assert!(diff.contains("+modified"));
    }

    #[test]
    fn test_safe_fix_filtering() {
        let mut fixer = Fixer::new(true);

        let safe_fix = Fix {
            file: PathBuf::from("test.wxs"),
            location: Location::new(PathBuf::from("test.wxs"), 1, 0),
            suggestion: FixSuggestion {
                action: FixAction::Custom,
                attribute: None,
                value: Some("safe".to_string()),
                description: Some("Safe fix".to_string()),
            },
            original: None,
            rule_id: "safe-rule".to_string(),
            safety: FixSafety::Safe,
        };

        let unsafe_fix = Fix {
            file: PathBuf::from("test.wxs"),
            location: Location::new(PathBuf::from("test.wxs"), 2, 0),
            suggestion: FixSuggestion {
                action: FixAction::Custom,
                attribute: None,
                value: Some("unsafe".to_string()),
                description: Some("Unsafe fix".to_string()),
            },
            original: None,
            rule_id: "unsafe-rule".to_string(),
            safety: FixSafety::Unsafe,
        };

        fixer.add_fix(safe_fix);
        fixer.add_fix(unsafe_fix);

        // In SafeOnly mode, only safe fixes should be pending
        let pending = fixer.get_pending_fixes();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].rule_id, "safe-rule");

        // In All mode, both fixes should be pending
        let fixer = fixer.with_unsafe_fixes(true);
        let pending = fixer.get_pending_fixes();
        assert_eq!(pending.len(), 2);
    }
}

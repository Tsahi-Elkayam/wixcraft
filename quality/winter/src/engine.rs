//! Core linter engine

use crate::config::Config;
use crate::cross_file::CrossFileValidator;
use crate::diagnostic::{Diagnostic, Location, Severity};
use crate::plugin::{Document, Node, Plugin};
use crate::rule::Rule;
use rayon::prelude::*;
use regex::Regex;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Per-rule timing statistics
#[derive(Debug, Clone, Default)]
pub struct RuleTiming {
    /// Rule ID
    pub rule_id: String,
    /// Total time spent on this rule
    pub total_time: Duration,
    /// Number of times the rule was evaluated
    pub evaluation_count: usize,
    /// Number of matches found
    pub match_count: usize,
}

impl RuleTiming {
    /// Create a new timing entry
    pub fn new(rule_id: &str) -> Self {
        Self {
            rule_id: rule_id.to_string(),
            ..Default::default()
        }
    }

    /// Average time per evaluation
    pub fn avg_time(&self) -> Duration {
        if self.evaluation_count > 0 {
            self.total_time / self.evaluation_count as u32
        } else {
            Duration::ZERO
        }
    }
}

/// Result of linting operation
#[derive(Debug, Default)]
pub struct LintResult {
    /// All diagnostics
    pub diagnostics: Vec<Diagnostic>,

    /// Files processed
    pub files_processed: usize,

    /// Files with errors
    pub files_with_errors: usize,

    /// Files with warnings
    pub files_with_warnings: usize,

    /// Total errors
    pub error_count: usize,

    /// Total warnings
    pub warning_count: usize,

    /// Total info messages
    pub info_count: usize,

    /// Processing duration
    pub duration: Duration,

    /// Per-rule timing statistics (rule_id -> timing)
    pub rule_timings: HashMap<String, RuleTiming>,
}

impl LintResult {
    /// Check if there are any errors
    pub fn has_errors(&self) -> bool {
        self.error_count > 0
    }

    /// Check if there are any warnings
    pub fn has_warnings(&self) -> bool {
        self.warning_count > 0
    }

    /// Check if result is clean (no errors or warnings)
    pub fn is_clean(&self) -> bool {
        self.error_count == 0 && self.warning_count == 0
    }

    /// Get exit code (0 = success, 1 = warnings, 2 = errors)
    pub fn exit_code(&self) -> i32 {
        if self.error_count > 0 {
            2
        } else if self.warning_count > 0 {
            1
        } else {
            0
        }
    }

    /// Merge another result into this one
    pub fn merge(&mut self, other: LintResult) {
        self.diagnostics.extend(other.diagnostics);
        self.files_processed += other.files_processed;
        self.files_with_errors += other.files_with_errors;
        self.files_with_warnings += other.files_with_warnings;
        self.error_count += other.error_count;
        self.warning_count += other.warning_count;
        self.info_count += other.info_count;

        // Merge rule timings
        for (rule_id, timing) in other.rule_timings {
            let entry = self
                .rule_timings
                .entry(rule_id)
                .or_insert_with(|| RuleTiming::new(&timing.rule_id));
            entry.total_time += timing.total_time;
            entry.evaluation_count += timing.evaluation_count;
            entry.match_count += timing.match_count;
        }
    }

    /// Get rule timings sorted by total time (descending)
    pub fn sorted_timings(&self) -> Vec<&RuleTiming> {
        let mut timings: Vec<_> = self.rule_timings.values().collect();
        timings.sort_by(|a, b| b.total_time.cmp(&a.total_time));
        timings
    }

    /// Format timing statistics as a string
    pub fn format_timings(&self) -> String {
        let mut output = String::new();
        let timings = self.sorted_timings();

        if timings.is_empty() {
            return "No timing data available".to_string();
        }

        output.push_str("Rule Timing Statistics:\n");
        output.push_str(&format!(
            "{:<40} {:>12} {:>12} {:>10} {:>12}\n",
            "Rule ID", "Total", "Avg", "Evals", "Matches"
        ));
        output.push_str(&"-".repeat(90));
        output.push('\n');

        for timing in timings {
            let total_ms = timing.total_time.as_secs_f64() * 1000.0;
            let avg_us = timing.avg_time().as_secs_f64() * 1_000_000.0;

            output.push_str(&format!(
                "{:<40} {:>10.2}ms {:>10.2}µs {:>10} {:>12}\n",
                timing.rule_id, total_ms, avg_us, timing.evaluation_count, timing.match_count
            ));
        }

        output
    }
}

/// The main linter engine
pub struct Engine {
    /// Configuration
    config: Config,

    /// Registered plugins (keyed by extension)
    plugins: HashMap<String, Arc<dyn Plugin>>,

    /// Number of context lines to include
    context_lines: usize,
}

impl Engine {
    /// Create a new engine with configuration
    pub fn new(config: Config) -> Self {
        Self {
            config,
            plugins: HashMap::new(),
            context_lines: 0,
        }
    }

    /// Set the number of context lines to include
    pub fn with_context_lines(mut self, lines: usize) -> Self {
        self.context_lines = lines;
        self
    }

    /// Set context lines (mutable reference)
    pub fn set_context_lines(&mut self, lines: usize) {
        self.context_lines = lines;
    }

    /// Register a plugin
    pub fn register_plugin(&mut self, plugin: Arc<dyn Plugin>) {
        for ext in plugin.extensions() {
            self.plugins.insert(ext.to_string(), Arc::clone(&plugin));
        }
    }

    /// Get plugin for a file
    fn get_plugin(&self, path: &Path) -> Option<Arc<dyn Plugin>> {
        let ext = path.extension()?.to_str()?;
        self.plugins.get(ext).cloned()
    }

    /// Lint multiple files
    pub fn lint(&self, files: &[PathBuf]) -> LintResult {
        let start = Instant::now();

        let results: Vec<LintResult> = if self.config.engine.parallel {
            let pool = rayon::ThreadPoolBuilder::new()
                .num_threads(if self.config.engine.jobs > 0 {
                    self.config.engine.jobs
                } else {
                    num_cpus::get()
                })
                .build()
                .unwrap_or_else(|_| rayon::ThreadPoolBuilder::new().build().unwrap());

            pool.install(|| files.par_iter().map(|f| self.lint_file(f)).collect())
        } else {
            files.iter().map(|f| self.lint_file(f)).collect()
        };

        let mut combined = LintResult::default();
        for result in results {
            combined.merge(result);
        }

        combined.duration = start.elapsed();
        combined
    }

    /// Lint multiple files with cross-file validation
    ///
    /// This performs:
    /// 1. Normal single-file linting for each file
    /// 2. Cross-file validation (reference checking, duplicate detection)
    pub fn lint_with_cross_file(&self, files: &[PathBuf]) -> LintResult {
        let start = Instant::now();

        // First, do normal single-file linting
        let mut result = self.lint(files);

        // Then do cross-file validation
        let mut validator = CrossFileValidator::new();

        // Collect all definitions and references
        for file in files {
            let plugin = match self.get_plugin(file) {
                Some(p) => p,
                None => continue,
            };

            let content = match std::fs::read_to_string(file) {
                Ok(c) => c,
                Err(_) => continue,
            };

            let document = match plugin.parse(&content, file) {
                Ok(d) => d,
                Err(_) => continue,
            };

            validator.collect_definitions(document.as_ref(), file);
            validator.collect_references(document.as_ref(), file);
        }

        // Validate and collect diagnostics
        let cross_file_diagnostics = validator.validate();

        for diag in cross_file_diagnostics {
            match diag.severity {
                Severity::Error => result.error_count += 1,
                Severity::Warning => result.warning_count += 1,
                Severity::Info => result.info_count += 1,
            }
            result.diagnostics.push(diag);
        }

        result.duration = start.elapsed();
        result
    }

    /// Lint a single file
    pub fn lint_file(&self, path: &Path) -> LintResult {
        let mut result = LintResult {
            files_processed: 1,
            ..LintResult::default()
        };

        // Get plugin for this file type
        let plugin = match self.get_plugin(path) {
            Some(p) => p,
            None => return result,
        };

        // Read file content
        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(e) => {
                result.diagnostics.push(Diagnostic::new(
                    "file-read-error",
                    Severity::Error,
                    &format!("Failed to read file: {}", e),
                    Location::new(path.to_path_buf(), 0, 0),
                ));
                result.error_count = 1;
                result.files_with_errors = 1;
                return result;
            }
        };

        // Parse the document
        let document = match plugin.parse(&content, path) {
            Ok(d) => d,
            Err(e) => {
                result.diagnostics.push(Diagnostic::new(
                    "parse-error",
                    Severity::Error,
                    &format!("Parse error: {}", e),
                    Location::new(path.to_path_buf(), 0, 0),
                ));
                result.error_count = 1;
                result.files_with_errors = 1;
                return result;
            }
        };

        // Run rules
        let (diagnostics, timings) =
            self.evaluate_rules(plugin.rules(), document.as_ref(), path, &content);

        // Count by severity
        for diag in &diagnostics {
            match diag.severity {
                Severity::Error => result.error_count += 1,
                Severity::Warning => result.warning_count += 1,
                Severity::Info => result.info_count += 1,
            }
        }

        if result.error_count > 0 {
            result.files_with_errors = 1;
        }
        if result.warning_count > 0 {
            result.files_with_warnings = 1;
        }

        result.diagnostics = diagnostics;
        result.rule_timings = timings;
        result
    }

    /// Evaluate rules against a document
    fn evaluate_rules(
        &self,
        rules: &[Rule],
        document: &dyn Document,
        file_path: &Path,
        content: &str,
    ) -> (Vec<Diagnostic>, HashMap<String, RuleTiming>) {
        let mut diagnostics = Vec::new();
        let mut timings: HashMap<String, RuleTiming> = HashMap::new();
        let source_lines: Vec<&str> = content.lines().collect();

        for node in document.iter() {
            for rule in rules {
                // Check if rule is enabled
                if !rule.enabled || !self.config.is_rule_enabled(&rule.id) {
                    continue;
                }

                // Check per-file ignore
                if self.config.should_ignore_rule_for_file(&rule.id, file_path) {
                    continue;
                }

                // Check inline disable
                let location = node.location();
                if document.is_rule_disabled(&rule.id, location.line)
                    || document.is_rule_disabled_for_file(&rule.id)
                {
                    continue;
                }

                // Check target match
                if !self.matches_target(node, &rule.target) {
                    continue;
                }

                // Time the condition evaluation
                let start = Instant::now();
                let matched = self.evaluate_condition(&rule.condition, node);
                let elapsed = start.elapsed();

                // Update timing stats
                let timing = timings
                    .entry(rule.id.clone())
                    .or_insert_with(|| RuleTiming::new(&rule.id));
                timing.total_time += elapsed;
                timing.evaluation_count += 1;

                // Evaluate condition
                if matched {
                    timing.match_count += 1;
                    let severity = self
                        .config
                        .get_severity_override(&rule.id)
                        .unwrap_or(rule.severity);

                    let message = self.format_message(&rule.message, node);
                    let line_num = location.line;
                    let mut diag = Diagnostic::new(&rule.id, severity, &message, location);

                    // Add source line
                    if line_num > 0 && line_num <= source_lines.len() {
                        diag = diag.with_source_line(source_lines[line_num - 1]);

                        // Add context lines
                        if self.context_lines > 0 {
                            diag = diag.with_context(&source_lines, self.context_lines);
                        }
                    }

                    // Add help text
                    if let Some(desc) = &rule.description {
                        diag = diag.with_help(desc);
                    }

                    // Add fix suggestion from rule definition
                    if let Some(fix) = &rule.fix {
                        if let Some(desc) = &fix.description {
                            if let Some(value) = &fix.value {
                                diag = diag.with_fix(desc, value);
                            }
                        }
                    }

                    // Add auto-generated fix for well-known rules
                    if diag.fix.is_none() {
                        if let Some(fix) = self.generate_fix(&rule.id, node, &source_lines) {
                            diag.fix = Some(fix);
                        }
                    }

                    diagnostics.push(diag);
                }
            }
        }

        (diagnostics, timings)
    }

    /// Check if a node matches a target specification
    fn matches_target(&self, node: &dyn Node, target: &crate::rule::Target) -> bool {
        // Check kind
        if let Some(kind) = &target.kind {
            if node.kind() != kind {
                return false;
            }
        }

        // Check name (supports wildcards)
        if let Some(name_pattern) = &target.name {
            let name = node.name();
            if name_pattern.contains('*') {
                let regex_pattern =
                    format!("^{}$", name_pattern.replace("*", ".*").replace("?", "."));
                if let Ok(re) = Regex::new(&regex_pattern) {
                    if !re.is_match(name) {
                        return false;
                    }
                }
            } else if name != name_pattern {
                return false;
            }
        }

        // Check parent
        if let Some(parent_name) = &target.parent {
            if let Some(parent) = node.parent() {
                if parent.name() != parent_name {
                    return false;
                }
            } else {
                return false;
            }
        }

        true
    }

    /// Evaluate a condition expression against a node
    fn evaluate_condition(&self, condition: &str, node: &dyn Node) -> bool {
        let condition = condition.trim();

        // Handle logical operators
        if let Some(idx) = find_logical_operator(condition, "||") {
            let left = &condition[..idx];
            let right = &condition[idx + 2..];
            return self.evaluate_condition(left, node) || self.evaluate_condition(right, node);
        }

        if let Some(idx) = find_logical_operator(condition, "&&") {
            let left = &condition[..idx];
            let right = &condition[idx + 2..];
            return self.evaluate_condition(left, node) && self.evaluate_condition(right, node);
        }

        // Handle negation
        if let Some(rest) = condition.strip_prefix('!') {
            return !self.evaluate_condition(rest, node);
        }

        // Handle parentheses
        if condition.starts_with('(') && condition.ends_with(')') {
            return self.evaluate_condition(&condition[1..condition.len() - 1], node);
        }

        // Handle comparisons
        if let Some(idx) = condition.find("==") {
            let left = condition[..idx].trim();
            let right = condition[idx + 2..]
                .trim()
                .trim_matches('"')
                .trim_matches('\'');
            return self.get_value(left, node).is_some_and(|v| v == right);
        }

        if let Some(idx) = condition.find("!=") {
            let left = condition[..idx].trim();
            let right = condition[idx + 2..]
                .trim()
                .trim_matches('"')
                .trim_matches('\'');
            return self.get_value(left, node).is_none_or(|v| v != right);
        }

        if let Some(idx) = condition.find("=~") {
            let left = condition[..idx].trim();
            let right = condition[idx + 2..].trim().trim_matches('/');
            if let (Some(value), Ok(re)) = (self.get_value(left, node), Regex::new(right)) {
                return re.is_match(&value);
            }
            return false;
        }

        // Handle functions
        if condition.starts_with("countChildren(") && condition.ends_with(')') {
            let arg = &condition[14..condition.len() - 1]
                .trim_matches('\'')
                .trim_matches('"');
            let count = if *arg == "*" {
                node.children().len()
            } else {
                node.children().iter().filter(|c| c.name() == *arg).count()
            };
            // Check if there's a comparison
            return count > 0;
        }

        if condition.starts_with("hasChild(") && condition.ends_with(')') {
            let arg = &condition[9..condition.len() - 1]
                .trim_matches('\'')
                .trim_matches('"');
            return node.children().iter().any(|c| c.name() == *arg);
        }

        if condition.starts_with("isEmpty(") && condition.ends_with(')') {
            let arg = &condition[8..condition.len() - 1];
            return self.get_value(arg, node).is_none_or(|v| v.is_empty());
        }

        if condition.starts_with("isGuid(") && condition.ends_with(')') {
            let arg = &condition[7..condition.len() - 1];
            if let Some(value) = self.get_value(arg, node) {
                return is_valid_guid(&value);
            }
            return false;
        }

        // Handle attribute existence check
        if let Some(attr_name) = condition.strip_prefix("attributes.") {
            return node.get(attr_name).is_some();
        }

        // Simple truthiness
        self.get_value(condition, node).is_some()
    }

    /// Get a value from a node based on path
    fn get_value(&self, path: &str, node: &dyn Node) -> Option<String> {
        if let Some(attr_name) = path.strip_prefix("attributes.") {
            return node.get(attr_name).map(String::from);
        }

        if path == "name" {
            return Some(node.name().to_string());
        }

        if path == "kind" {
            return Some(node.kind().to_string());
        }

        None
    }

    /// Format a message template with node values
    fn format_message(&self, template: &str, node: &dyn Node) -> String {
        let mut result = template.to_string();

        // Replace {attributes.X} placeholders
        let re = Regex::new(r"\{attributes\.([^}]+)\}").unwrap();
        result = re
            .replace_all(&result, |caps: &regex::Captures| {
                let attr_name = &caps[1];
                node.get(attr_name).unwrap_or("(unknown)")
            })
            .to_string();

        // Replace {name}
        result = result.replace("{name}", node.name());

        // Replace {kind}
        result = result.replace("{kind}", node.kind());

        result
    }

    /// Generate auto-fix for well-known rules
    fn generate_fix(
        &self,
        rule_id: &str,
        node: &dyn Node,
        source_lines: &[&str],
    ) -> Option<crate::diagnostic::Fix> {
        use crate::diagnostic::Fix;

        match rule_id {
            // Fix 1: Add Guid="*" to Component
            "component-requires-guid" => {
                let line_idx = node.location().line.saturating_sub(1);
                if line_idx < source_lines.len() {
                    let line = source_lines[line_idx];
                    // Find the closing > of the element
                    if let Some(pos) = line.rfind('>') {
                        let before_close = &line[..pos];
                        // Check if it's self-closing or not
                        if let Some(stripped) = before_close.strip_suffix('/') {
                            let replacement = format!("{} Guid=\"*\"/>", stripped);
                            return Some(Fix::safe("Add Guid=\"*\" attribute", &replacement));
                        } else {
                            let replacement = format!("{} Guid=\"*\">", before_close);
                            return Some(Fix::safe("Add Guid=\"*\" attribute", &replacement));
                        }
                    }
                }
                None
            }

            // Fix 2: Add Version="1.0.0.0" to Package
            "package-requires-version" => {
                let line_idx = node.location().line.saturating_sub(1);
                if line_idx < source_lines.len() {
                    let line = source_lines[line_idx];
                    if let Some(pos) = line.rfind('>') {
                        let before_close = &line[..pos];
                        if let Some(stripped) = before_close.strip_suffix('/') {
                            let replacement = format!("{} Version=\"1.0.0.0\"/>", stripped);
                            return Some(Fix::safe(
                                "Add Version=\"1.0.0.0\" attribute",
                                &replacement,
                            ));
                        } else {
                            let replacement = format!("{} Version=\"1.0.0.0\">", before_close);
                            return Some(Fix::safe(
                                "Add Version=\"1.0.0.0\" attribute",
                                &replacement,
                            ));
                        }
                    }
                }
                None
            }

            // Fix 3: Add UpgradeCode to Package (generates new GUID)
            "package-requires-upgradecode" => {
                let line_idx = node.location().line.saturating_sub(1);
                if line_idx < source_lines.len() {
                    let line = source_lines[line_idx];
                    if let Some(pos) = line.rfind('>') {
                        let before_close = &line[..pos];
                        // Generate a placeholder GUID (user should replace)
                        let guid = "PUT-YOUR-GUID-HERE";
                        if let Some(stripped) = before_close.strip_suffix('/') {
                            let replacement = format!("{} UpgradeCode=\"{}\"/>", stripped, guid);
                            return Some(Fix::unsafe_fix(
                                "Add UpgradeCode attribute (replace GUID)",
                                &replacement,
                            ));
                        } else {
                            let replacement = format!("{} UpgradeCode=\"{}\">", before_close, guid);
                            return Some(Fix::unsafe_fix(
                                "Add UpgradeCode attribute (replace GUID)",
                                &replacement,
                            ));
                        }
                    }
                }
                None
            }

            // Fix 4: Replace hardcoded path with variable
            "file-hardcoded-path" => {
                if let Some(source) = node.get("Source") {
                    // Replace C:\...\filename with $(var.SourceDir)\filename
                    if let Some(filename_start) = source.rfind('\\') {
                        let filename = &source[filename_start + 1..];
                        let replacement = format!("$(var.SourceDir)\\{}", filename);
                        return Some(Fix::unsafe_fix(
                            &format!("Replace hardcoded path with $(var.SourceDir)\\{}", filename),
                            &replacement,
                        ));
                    }
                }
                None
            }

            // Fix 5: Add Type attribute to RegistryValue
            "registryvalue-requires-type" => {
                let line_idx = node.location().line.saturating_sub(1);
                if line_idx < source_lines.len() {
                    let line = source_lines[line_idx];
                    if let Some(pos) = line.rfind('>') {
                        let before_close = &line[..pos];
                        if let Some(stripped) = before_close.strip_suffix('/') {
                            let replacement = format!("{} Type=\"string\"/>", stripped);
                            return Some(Fix::safe("Add Type=\"string\" attribute", &replacement));
                        } else {
                            let replacement = format!("{} Type=\"string\">", before_close);
                            return Some(Fix::safe("Add Type=\"string\" attribute", &replacement));
                        }
                    }
                }
                None
            }

            _ => None,
        }
    }
}

/// Find logical operator position (handles nested parentheses)
fn find_logical_operator(s: &str, op: &str) -> Option<usize> {
    let mut depth = 0;
    let chars: Vec<char> = s.chars().collect();
    let op_chars: Vec<char> = op.chars().collect();

    for i in 0..chars.len() {
        match chars[i] {
            '(' => depth += 1,
            ')' => depth -= 1,
            _ => {}
        }

        if depth == 0 && i + op_chars.len() <= chars.len() {
            let slice: String = chars[i..i + op_chars.len()].iter().collect();
            if slice == op {
                return Some(i);
            }
        }
    }

    None
}

/// Validate GUID format
fn is_valid_guid(s: &str) -> bool {
    let guid_re =
        Regex::new(r"(?i)^\{?[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}\}?$")
            .unwrap();
    guid_re.is_match(s) || s == "*"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lint_result_exit_code() {
        let mut result = LintResult::default();
        assert_eq!(result.exit_code(), 0);

        result.warning_count = 1;
        assert_eq!(result.exit_code(), 1);

        result.error_count = 1;
        assert_eq!(result.exit_code(), 2);
    }

    #[test]
    fn test_lint_result_is_clean() {
        let mut result = LintResult::default();
        assert!(result.is_clean());

        result.warning_count = 1;
        assert!(!result.is_clean());
    }

    #[test]
    fn test_lint_result_merge() {
        let mut result1 = LintResult::default();
        result1.files_processed = 1;
        result1.error_count = 2;

        let mut result2 = LintResult::default();
        result2.files_processed = 1;
        result2.warning_count = 3;

        result1.merge(result2);
        assert_eq!(result1.files_processed, 2);
        assert_eq!(result1.error_count, 2);
        assert_eq!(result1.warning_count, 3);
    }

    #[test]
    fn test_is_valid_guid() {
        assert!(is_valid_guid("12345678-1234-1234-1234-123456789012"));
        assert!(is_valid_guid("{12345678-1234-1234-1234-123456789012}"));
        assert!(is_valid_guid("*"));
        assert!(!is_valid_guid("not-a-guid"));
        assert!(!is_valid_guid("12345678-1234-1234-1234"));
    }

    #[test]
    fn test_find_logical_operator() {
        assert_eq!(find_logical_operator("a && b", "&&"), Some(2));
        assert_eq!(find_logical_operator("a || b", "||"), Some(2));
        assert_eq!(find_logical_operator("(a && b) || c", "||"), Some(9));
        assert_eq!(find_logical_operator("(a || b) && c", "&&"), Some(9));
        assert_eq!(find_logical_operator("(a || b)", "||"), None); // inside parens
    }
}

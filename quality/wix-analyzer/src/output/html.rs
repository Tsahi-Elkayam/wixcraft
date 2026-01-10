//! HTML output formatter for standalone reports
//!
//! Generates self-contained HTML reports with styling and interactivity.

use super::Formatter;
use crate::core::{AnalysisResult, Diagnostic, IssueType, Severity};

/// HTML formatter
pub struct HtmlFormatter {
    /// Report title
    title: String,
    /// Whether to include inline CSS
    inline_css: bool,
}

impl HtmlFormatter {
    pub fn new() -> Self {
        Self {
            title: "WiX Analyzer Report".to_string(),
            inline_css: true,
        }
    }

    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }
}

impl Default for HtmlFormatter {
    fn default() -> Self {
        Self::new()
    }
}

impl Formatter for HtmlFormatter {
    fn format(&self, results: &[AnalysisResult]) -> String {
        let mut all_diagnostics: Vec<&Diagnostic> =
            results.iter().flat_map(|r| &r.diagnostics).collect();

        // Sort by severity (highest first), then by file, then by line
        all_diagnostics.sort_by(|a, b| {
            b.severity
                .cmp(&a.severity)
                .then_with(|| a.location.file.cmp(&b.location.file))
                .then_with(|| {
                    a.location
                        .range
                        .start
                        .line
                        .cmp(&b.location.range.start.line)
                })
        });

        let total = all_diagnostics.len();
        let blockers = all_diagnostics
            .iter()
            .filter(|d| d.severity == Severity::Blocker)
            .count();
        let high = all_diagnostics
            .iter()
            .filter(|d| d.severity == Severity::High)
            .count();
        let medium = all_diagnostics
            .iter()
            .filter(|d| d.severity == Severity::Medium)
            .count();
        let low = all_diagnostics
            .iter()
            .filter(|d| d.severity == Severity::Low)
            .count();
        let info = all_diagnostics
            .iter()
            .filter(|d| d.severity == Severity::Info)
            .count();

        let bugs = all_diagnostics
            .iter()
            .filter(|d| d.issue_type == IssueType::Bug)
            .count();
        let vulns = all_diagnostics
            .iter()
            .filter(|d| d.issue_type == IssueType::Vulnerability)
            .count();
        let smells = all_diagnostics
            .iter()
            .filter(|d| d.issue_type == IssueType::CodeSmell)
            .count();
        let hotspots = all_diagnostics
            .iter()
            .filter(|d| d.issue_type == IssueType::SecurityHotspot)
            .count();
        let secrets = all_diagnostics
            .iter()
            .filter(|d| d.issue_type == IssueType::Secret)
            .count();

        let files_count = results.iter().map(|r| r.files.len()).sum::<usize>();

        let rows: String = all_diagnostics
            .iter()
            .map(|d| format_row(d))
            .collect::<Vec<_>>()
            .join("\n");

        format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{title}</title>
    {css}
</head>
<body>
    <header>
        <h1>{title}</h1>
        <p class="timestamp">Generated: {timestamp}</p>
    </header>

    <section class="summary">
        <h2>Summary</h2>
        <div class="stats-grid">
            <div class="stat-card">
                <span class="stat-value">{total}</span>
                <span class="stat-label">Total Issues</span>
            </div>
            <div class="stat-card">
                <span class="stat-value">{files_count}</span>
                <span class="stat-label">Files Analyzed</span>
            </div>
        </div>

        <h3>By Severity</h3>
        <div class="severity-bar">
            <div class="severity-segment blocker" style="flex: {blockers};" title="Blocker: {blockers}"></div>
            <div class="severity-segment high" style="flex: {high};" title="High: {high}"></div>
            <div class="severity-segment medium" style="flex: {medium};" title="Medium: {medium}"></div>
            <div class="severity-segment low" style="flex: {low};" title="Low: {low}"></div>
            <div class="severity-segment info" style="flex: {info};" title="Info: {info}"></div>
        </div>
        <div class="severity-legend">
            <span class="legend-item"><span class="dot blocker"></span> Blocker: {blockers}</span>
            <span class="legend-item"><span class="dot high"></span> High: {high}</span>
            <span class="legend-item"><span class="dot medium"></span> Medium: {medium}</span>
            <span class="legend-item"><span class="dot low"></span> Low: {low}</span>
            <span class="legend-item"><span class="dot info"></span> Info: {info}</span>
        </div>

        <h3>By Type</h3>
        <div class="type-grid">
            <div class="type-card bug"><span class="count">{bugs}</span> Bugs</div>
            <div class="type-card vulnerability"><span class="count">{vulns}</span> Vulnerabilities</div>
            <div class="type-card smell"><span class="count">{smells}</span> Code Smells</div>
            <div class="type-card hotspot"><span class="count">{hotspots}</span> Security Hotspots</div>
            <div class="type-card secret"><span class="count">{secrets}</span> Secrets</div>
        </div>
    </section>

    <section class="issues">
        <h2>Issues</h2>
        <table>
            <thead>
                <tr>
                    <th>Severity</th>
                    <th>Type</th>
                    <th>Rule</th>
                    <th>File</th>
                    <th>Line</th>
                    <th>Message</th>
                </tr>
            </thead>
            <tbody>
                {rows}
            </tbody>
        </table>
    </section>

    <footer>
        <p>Generated by <a href="https://github.com/hyperlight/wixcraft">wix-analyzer</a> v{version}</p>
    </footer>
</body>
</html>"#,
            title = self.title,
            css = if self.inline_css {
                format!("<style>{}</style>", CSS)
            } else {
                String::new()
            },
            timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"),
            total = total,
            files_count = files_count,
            blockers = blockers,
            high = high,
            medium = medium,
            low = low,
            info = info,
            bugs = bugs,
            vulns = vulns,
            smells = smells,
            hotspots = hotspots,
            secrets = secrets,
            rows = rows,
            version = env!("CARGO_PKG_VERSION"),
        )
    }

    fn format_diagnostic(&self, diag: &Diagnostic) -> String {
        let results = vec![AnalysisResult {
            files: Vec::new(),
            diagnostics: vec![diag.clone()],
        }];
        self.format(&results)
    }
}

fn format_row(diag: &Diagnostic) -> String {
    let severity_class = match diag.severity {
        Severity::Blocker => "blocker",
        Severity::High => "high",
        Severity::Medium => "medium",
        Severity::Low => "low",
        Severity::Info => "info",
    };

    let type_class = match diag.issue_type {
        IssueType::Bug => "bug",
        IssueType::Vulnerability => "vulnerability",
        IssueType::CodeSmell => "smell",
        IssueType::SecurityHotspot => "hotspot",
        IssueType::Secret => "secret",
    };

    let doc_link = diag
        .doc_url
        .as_ref()
        .map(|url| {
            format!(
                r#"<a href="{}" target="_blank">{}</a>"#,
                html_escape(url),
                html_escape(&diag.rule_id)
            )
        })
        .unwrap_or_else(|| html_escape(&diag.rule_id));

    let security_tags = diag
        .security
        .as_ref()
        .map(|sec| {
            let mut tags = Vec::new();
            if let Some(ref cwe) = sec.cwe {
                tags.push(format!(
                    r#"<span class="tag cwe">{}</span>"#,
                    html_escape(cwe)
                ));
            }
            if let Some(ref owasp) = sec.owasp {
                tags.push(format!(
                    r#"<span class="tag owasp">{}</span>"#,
                    html_escape(owasp)
                ));
            }
            tags.join(" ")
        })
        .unwrap_or_default();

    format!(
        r#"<tr class="severity-{sev}">
    <td><span class="badge {sev}">{sev_display}</span></td>
    <td><span class="badge type-{type_class}">{type_display}</span></td>
    <td>{doc_link} {security_tags}</td>
    <td class="file">{file}</td>
    <td class="line">{line}</td>
    <td class="message">{message}</td>
</tr>"#,
        sev = severity_class,
        sev_display = diag.severity.as_str(),
        type_class = type_class,
        type_display = diag.issue_type.display_name(),
        doc_link = doc_link,
        security_tags = security_tags,
        file = html_escape(&diag.location.file.display().to_string()),
        line = diag.location.range.start.line,
        message = html_escape(&diag.message),
    )
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

const CSS: &str = r#"
:root {
    --color-blocker: #d32f2f;
    --color-high: #f44336;
    --color-medium: #ff9800;
    --color-low: #2196f3;
    --color-info: #9e9e9e;
    --color-bug: #e91e63;
    --color-vulnerability: #d32f2f;
    --color-smell: #ff9800;
    --color-hotspot: #9c27b0;
    --color-secret: #f44336;
}

* {
    box-sizing: border-box;
    margin: 0;
    padding: 0;
}

body {
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, sans-serif;
    line-height: 1.6;
    color: #333;
    max-width: 1400px;
    margin: 0 auto;
    padding: 20px;
    background: #f5f5f5;
}

header {
    background: linear-gradient(135deg, #1976d2 0%, #0d47a1 100%);
    color: white;
    padding: 30px;
    border-radius: 8px;
    margin-bottom: 20px;
}

header h1 {
    margin-bottom: 5px;
}

.timestamp {
    opacity: 0.8;
    font-size: 0.9em;
}

section {
    background: white;
    padding: 25px;
    border-radius: 8px;
    margin-bottom: 20px;
    box-shadow: 0 2px 4px rgba(0,0,0,0.1);
}

h2 {
    color: #1976d2;
    border-bottom: 2px solid #1976d2;
    padding-bottom: 10px;
    margin-bottom: 20px;
}

h3 {
    color: #555;
    margin: 20px 0 10px;
}

.stats-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
    gap: 15px;
    margin-bottom: 20px;
}

.stat-card {
    background: #f8f9fa;
    padding: 20px;
    border-radius: 8px;
    text-align: center;
}

.stat-value {
    display: block;
    font-size: 2.5em;
    font-weight: bold;
    color: #1976d2;
}

.stat-label {
    color: #666;
}

.severity-bar {
    display: flex;
    height: 24px;
    border-radius: 4px;
    overflow: hidden;
    margin-bottom: 10px;
}

.severity-segment {
    min-width: 2px;
}

.severity-segment.blocker { background: var(--color-blocker); }
.severity-segment.high { background: var(--color-high); }
.severity-segment.medium { background: var(--color-medium); }
.severity-segment.low { background: var(--color-low); }
.severity-segment.info { background: var(--color-info); }

.severity-legend {
    display: flex;
    flex-wrap: wrap;
    gap: 15px;
}

.legend-item {
    display: flex;
    align-items: center;
    gap: 5px;
}

.dot {
    width: 12px;
    height: 12px;
    border-radius: 50%;
}

.dot.blocker { background: var(--color-blocker); }
.dot.high { background: var(--color-high); }
.dot.medium { background: var(--color-medium); }
.dot.low { background: var(--color-low); }
.dot.info { background: var(--color-info); }

.type-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(120px, 1fr));
    gap: 10px;
}

.type-card {
    padding: 15px;
    border-radius: 6px;
    text-align: center;
    color: white;
}

.type-card.bug { background: var(--color-bug); }
.type-card.vulnerability { background: var(--color-vulnerability); }
.type-card.smell { background: var(--color-smell); }
.type-card.hotspot { background: var(--color-hotspot); }
.type-card.secret { background: var(--color-secret); }

.type-card .count {
    display: block;
    font-size: 1.8em;
    font-weight: bold;
}

table {
    width: 100%;
    border-collapse: collapse;
    font-size: 0.9em;
}

th, td {
    padding: 12px;
    text-align: left;
    border-bottom: 1px solid #eee;
}

th {
    background: #f8f9fa;
    font-weight: 600;
    position: sticky;
    top: 0;
}

tr:hover {
    background: #f8f9fa;
}

.badge {
    display: inline-block;
    padding: 3px 8px;
    border-radius: 4px;
    font-size: 0.8em;
    font-weight: 500;
    color: white;
    text-transform: uppercase;
}

.badge.blocker { background: var(--color-blocker); }
.badge.high { background: var(--color-high); }
.badge.medium { background: var(--color-medium); }
.badge.low { background: var(--color-low); }
.badge.info { background: var(--color-info); }

.badge.type-bug { background: var(--color-bug); }
.badge.type-vulnerability { background: var(--color-vulnerability); }
.badge.type-smell { background: var(--color-smell); }
.badge.type-hotspot { background: var(--color-hotspot); }
.badge.type-secret { background: var(--color-secret); }

.tag {
    display: inline-block;
    padding: 2px 6px;
    border-radius: 3px;
    font-size: 0.75em;
    margin-left: 4px;
}

.tag.cwe { background: #e3f2fd; color: #1565c0; }
.tag.owasp { background: #fce4ec; color: #c62828; }

.file {
    font-family: monospace;
    font-size: 0.85em;
    color: #666;
    max-width: 250px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}

.line {
    font-family: monospace;
    color: #1976d2;
}

.message {
    max-width: 400px;
}

footer {
    text-align: center;
    color: #666;
    padding: 20px;
}

footer a {
    color: #1976d2;
}

@media (max-width: 768px) {
    .stats-grid, .type-grid {
        grid-template-columns: repeat(2, 1fr);
    }

    table {
        font-size: 0.8em;
    }

    th, td {
        padding: 8px;
    }
}
"#;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{Category, Location, Position, Range};
    use std::path::PathBuf;

    fn make_location() -> Location {
        Location::new(
            PathBuf::from("test.wxs"),
            Range::new(Position::new(10, 5), Position::new(10, 20)),
        )
    }

    #[test]
    fn test_html_formatter_default() {
        let formatter = HtmlFormatter::default();
        let results: Vec<AnalysisResult> = vec![];
        let output = formatter.format(&results);

        assert!(output.contains("<!DOCTYPE html>"));
        assert!(output.contains("WiX Analyzer Report"));
    }

    #[test]
    fn test_html_with_custom_title() {
        let formatter = HtmlFormatter::new().with_title("My Report");
        let results: Vec<AnalysisResult> = vec![];
        let output = formatter.format(&results);

        assert!(output.contains("<title>My Report</title>"));
    }

    #[test]
    fn test_html_with_diagnostics() {
        let formatter = HtmlFormatter::new();
        let results = vec![AnalysisResult {
            files: vec![PathBuf::from("test.wxs")],
            diagnostics: vec![
                Diagnostic::error(
                    "VAL-001",
                    Category::Validation,
                    "Test error",
                    make_location(),
                ),
                Diagnostic::warning(
                    "BP-001",
                    Category::BestPractice,
                    "Test warning",
                    make_location(),
                ),
            ],
        }];

        let output = formatter.format(&results);
        assert!(output.contains("VAL-001"));
        assert!(output.contains("BP-001"));
        assert!(output.contains("Total Issues"));
    }

    #[test]
    fn test_html_with_cwe_owasp() {
        let formatter = HtmlFormatter::new();
        let diag = Diagnostic::high(
            "SEC-001",
            IssueType::Vulnerability,
            "Injection",
            make_location(),
        )
        .with_cwe("CWE-89")
        .with_owasp("A03:2021");

        let results = vec![AnalysisResult {
            files: Vec::new(),
            diagnostics: vec![diag],
        }];

        let output = formatter.format(&results);
        assert!(output.contains("CWE-89"));
        assert!(output.contains("A03:2021"));
    }

    #[test]
    fn test_html_escape() {
        assert_eq!(html_escape("<script>"), "&lt;script&gt;");
        assert_eq!(html_escape("a & b"), "a &amp; b");
        assert_eq!(html_escape("\"quoted\""), "&quot;quoted&quot;");
    }

    #[test]
    fn test_html_severity_counts() {
        let formatter = HtmlFormatter::new();
        let results = vec![AnalysisResult {
            files: Vec::new(),
            diagnostics: vec![
                Diagnostic::blocker("B1", IssueType::Bug, "Blocker", make_location()),
                Diagnostic::high("H1", IssueType::Bug, "High", make_location()),
                Diagnostic::medium("M1", IssueType::CodeSmell, "Medium", make_location()),
            ],
        }];

        let output = formatter.format(&results);
        assert!(output.contains("Blocker: 1"));
        assert!(output.contains("High: 1"));
        assert!(output.contains("Medium: 1"));
    }
}

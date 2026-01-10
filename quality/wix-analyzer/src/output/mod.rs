//! Output formatters for analysis results

mod text;
mod json;
mod sarif;
mod html;
mod metrics;

pub use text::TextFormatter;
pub use json::JsonFormatter;
pub use sarif::SarifFormatter;
pub use html::HtmlFormatter;
pub use metrics::{MetricsFormatter, MetricsSummary, SeverityCounts, TypeCounts, CategoryCounts, RuleCount};

use crate::core::{AnalysisResult, Diagnostic};

/// Output format options
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Text,
    Json,
    Sarif,
    Html,
    Metrics,
    MetricsJson,
}

impl std::str::FromStr for OutputFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "text" => Ok(Self::Text),
            "json" => Ok(Self::Json),
            "sarif" => Ok(Self::Sarif),
            "html" => Ok(Self::Html),
            "metrics" => Ok(Self::Metrics),
            "metrics-json" | "metricsjson" => Ok(Self::MetricsJson),
            _ => Err(format!("Unknown format: {}", s)),
        }
    }
}

/// Trait for output formatters
pub trait Formatter {
    /// Format analysis results
    fn format(&self, results: &[AnalysisResult]) -> String;

    /// Format a single diagnostic (for streaming output)
    fn format_diagnostic(&self, diagnostic: &Diagnostic) -> String;
}

/// Get a formatter for the specified format
pub fn get_formatter(format: OutputFormat, colored: bool) -> Box<dyn Formatter> {
    match format {
        OutputFormat::Text => Box::new(TextFormatter::new(colored)),
        OutputFormat::Json => Box::new(JsonFormatter::new()),
        OutputFormat::Sarif => Box::new(SarifFormatter::new()),
        OutputFormat::Html => Box::new(HtmlFormatter::new()),
        OutputFormat::Metrics => Box::new(MetricsFormatter::text()),
        OutputFormat::MetricsJson => Box::new(MetricsFormatter::json()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_output_format_from_str() {
        assert_eq!("text".parse::<OutputFormat>().unwrap(), OutputFormat::Text);
        assert_eq!("json".parse::<OutputFormat>().unwrap(), OutputFormat::Json);
        assert_eq!("sarif".parse::<OutputFormat>().unwrap(), OutputFormat::Sarif);
        assert_eq!("html".parse::<OutputFormat>().unwrap(), OutputFormat::Html);
        assert_eq!("metrics".parse::<OutputFormat>().unwrap(), OutputFormat::Metrics);
        assert_eq!("metrics-json".parse::<OutputFormat>().unwrap(), OutputFormat::MetricsJson);
        assert_eq!("TEXT".parse::<OutputFormat>().unwrap(), OutputFormat::Text);
        assert_eq!("JSON".parse::<OutputFormat>().unwrap(), OutputFormat::Json);
        assert_eq!("HTML".parse::<OutputFormat>().unwrap(), OutputFormat::Html);
        assert_eq!("METRICS".parse::<OutputFormat>().unwrap(), OutputFormat::Metrics);
    }

    #[test]
    fn test_output_format_from_str_invalid() {
        let result = "invalid".parse::<OutputFormat>();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unknown format"));
    }

    #[test]
    fn test_get_formatter_text() {
        let formatter = get_formatter(OutputFormat::Text, false);
        let results = vec![];
        let output = formatter.format(&results);
        assert!(output.is_empty() || output.contains("Found"));
    }

    #[test]
    fn test_get_formatter_json() {
        let formatter = get_formatter(OutputFormat::Json, false);
        let results = vec![];
        let output = formatter.format(&results);
        assert!(output.contains("diagnostics"));
    }

    #[test]
    fn test_get_formatter_sarif() {
        let formatter = get_formatter(OutputFormat::Sarif, false);
        let results = vec![];
        let output = formatter.format(&results);
        assert!(output.contains("$schema"));
    }

    #[test]
    fn test_get_formatter_metrics() {
        let formatter = get_formatter(OutputFormat::Metrics, false);
        let results = vec![];
        let output = formatter.format(&results);
        assert!(output.contains("Metrics Summary"));
    }

    #[test]
    fn test_get_formatter_metrics_json() {
        let formatter = get_formatter(OutputFormat::MetricsJson, false);
        let results = vec![];
        let output = formatter.format(&results);
        assert!(output.contains("files_analyzed"));
        assert!(output.contains("total_issues"));
    }
}

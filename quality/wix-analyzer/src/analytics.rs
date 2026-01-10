//! Analytics module
//!
//! Collects and reports installation metrics, usage patterns, and diagnostics.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Analytics event type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventType {
    InstallStart,
    InstallComplete,
    InstallFailed,
    UninstallStart,
    UninstallComplete,
    RepairStart,
    RepairComplete,
    FeatureSelect,
    CustomAction,
    Error,
}

/// Analytics event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsEvent {
    pub event_type: EventType,
    pub timestamp: String,
    pub product_code: String,
    pub version: String,
    pub properties: HashMap<String, String>,
}

impl AnalyticsEvent {
    pub fn new(event_type: EventType, product_code: &str, version: &str) -> Self {
        Self {
            event_type,
            timestamp: "2024-01-01T00:00:00Z".to_string(),
            product_code: product_code.to_string(),
            version: version.to_string(),
            properties: HashMap::new(),
        }
    }

    pub fn with_property(mut self, key: &str, value: &str) -> Self {
        self.properties.insert(key.to_string(), value.to_string());
        self
    }
}

/// Installation metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct InstallMetrics {
    pub total_installs: u64,
    pub successful_installs: u64,
    pub failed_installs: u64,
    pub total_uninstalls: u64,
    pub total_repairs: u64,
    pub average_install_time_secs: f64,
    pub most_common_errors: Vec<String>,
}

impl InstallMetrics {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_install(&mut self, success: bool, duration_secs: f64) {
        self.total_installs += 1;
        if success {
            self.successful_installs += 1;
        } else {
            self.failed_installs += 1;
        }
        // Update average
        let total = self.successful_installs as f64;
        if total > 0.0 {
            self.average_install_time_secs =
                (self.average_install_time_secs * (total - 1.0) + duration_secs) / total;
        }
    }

    pub fn success_rate(&self) -> f64 {
        if self.total_installs == 0 {
            return 0.0;
        }
        (self.successful_installs as f64 / self.total_installs as f64) * 100.0
    }
}

/// Feature usage tracking
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FeatureUsage {
    pub feature_id: String,
    pub times_selected: u64,
    pub times_deselected: u64,
    pub install_percentage: f64,
}

impl FeatureUsage {
    pub fn new(feature_id: &str) -> Self {
        Self {
            feature_id: feature_id.to_string(),
            ..Default::default()
        }
    }

    pub fn record_selection(&mut self, selected: bool) {
        if selected {
            self.times_selected += 1;
        } else {
            self.times_deselected += 1;
        }
        let total = self.times_selected + self.times_deselected;
        if total > 0 {
            self.install_percentage = (self.times_selected as f64 / total as f64) * 100.0;
        }
    }
}

/// Analytics collector
#[derive(Debug, Clone, Default)]
pub struct AnalyticsCollector {
    events: Vec<AnalyticsEvent>,
    metrics: InstallMetrics,
    feature_usage: HashMap<String, FeatureUsage>,
}

impl AnalyticsCollector {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_event(&mut self, event: AnalyticsEvent) {
        self.events.push(event);
    }

    pub fn record_install(&mut self, success: bool, duration_secs: f64) {
        self.metrics.record_install(success, duration_secs);
    }

    pub fn record_feature_selection(&mut self, feature_id: &str, selected: bool) {
        self.feature_usage
            .entry(feature_id.to_string())
            .or_insert_with(|| FeatureUsage::new(feature_id))
            .record_selection(selected);
    }

    pub fn get_metrics(&self) -> &InstallMetrics {
        &self.metrics
    }

    pub fn get_events(&self) -> &[AnalyticsEvent] {
        &self.events
    }

    pub fn get_feature_usage(&self, feature_id: &str) -> Option<&FeatureUsage> {
        self.feature_usage.get(feature_id)
    }

    pub fn clear(&mut self) {
        self.events.clear();
    }
}

/// Analytics report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsReport {
    pub product_code: String,
    pub product_name: String,
    pub report_period: String,
    pub metrics: InstallMetrics,
    pub top_features: Vec<FeatureUsage>,
    pub event_count: usize,
}

impl AnalyticsReport {
    pub fn generate(
        product_code: &str,
        product_name: &str,
        period: &str,
        collector: &AnalyticsCollector,
    ) -> Self {
        let mut top_features: Vec<FeatureUsage> =
            collector.feature_usage.values().cloned().collect();
        top_features.sort_by(|a, b| {
            b.install_percentage
                .partial_cmp(&a.install_percentage)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        top_features.truncate(10);

        Self {
            product_code: product_code.to_string(),
            product_name: product_name.to_string(),
            report_period: period.to_string(),
            metrics: collector.metrics.clone(),
            top_features,
            event_count: collector.events.len(),
        }
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_default()
    }
}

/// Analytics configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsConfig {
    pub enabled: bool,
    pub endpoint: Option<String>,
    pub batch_size: usize,
    pub flush_interval_secs: u64,
    pub include_system_info: bool,
    pub anonymize_data: bool,
}

impl Default for AnalyticsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            endpoint: None,
            batch_size: 100,
            flush_interval_secs: 300,
            include_system_info: true,
            anonymize_data: true,
        }
    }
}

/// Generate analytics custom action code
pub struct AnalyticsGenerator;

impl AnalyticsGenerator {
    /// Generate WiX fragment with analytics custom actions
    pub fn generate_fragment(config: &AnalyticsConfig) -> String {
        let mut output = String::new();
        output.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
        output.push_str("<Wix xmlns=\"http://wixtoolset.org/schemas/v4/wxs\">\n");
        output.push_str("  <Fragment>\n");
        output.push_str("    <PropertyGroup>\n");
        output.push_str(&format!(
            "      <AnalyticsEnabled>{}</AnalyticsEnabled>\n",
            if config.enabled { "1" } else { "0" }
        ));
        if let Some(ref endpoint) = config.endpoint {
            output.push_str(&format!(
                "      <AnalyticsEndpoint>{}</AnalyticsEndpoint>\n",
                endpoint
            ));
        }
        output.push_str("    </PropertyGroup>\n");
        output.push_str("  </Fragment>\n");
        output.push_str("</Wix>\n");
        output
    }

    /// Generate analytics DLL reference
    pub fn generate_dll_reference(dll_path: &PathBuf) -> String {
        format!(
            "<Binary Id=\"AnalyticsDll\" SourceFile=\"{}\" />",
            dll_path.display()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analytics_event_new() {
        let event = AnalyticsEvent::new(EventType::InstallStart, "{CODE}", "1.0.0");
        assert_eq!(event.event_type, EventType::InstallStart);
        assert_eq!(event.product_code, "{CODE}");
    }

    #[test]
    fn test_analytics_event_with_property() {
        let event = AnalyticsEvent::new(EventType::InstallComplete, "{CODE}", "1.0.0")
            .with_property("duration", "120");
        assert_eq!(event.properties.get("duration"), Some(&"120".to_string()));
    }

    #[test]
    fn test_install_metrics_new() {
        let metrics = InstallMetrics::new();
        assert_eq!(metrics.total_installs, 0);
    }

    #[test]
    fn test_install_metrics_record() {
        let mut metrics = InstallMetrics::new();
        metrics.record_install(true, 60.0);
        assert_eq!(metrics.total_installs, 1);
        assert_eq!(metrics.successful_installs, 1);
    }

    #[test]
    fn test_install_metrics_success_rate() {
        let mut metrics = InstallMetrics::new();
        metrics.record_install(true, 60.0);
        metrics.record_install(true, 60.0);
        metrics.record_install(false, 30.0);
        assert!((metrics.success_rate() - 66.66).abs() < 1.0);
    }

    #[test]
    fn test_feature_usage_new() {
        let usage = FeatureUsage::new("Feature1");
        assert_eq!(usage.feature_id, "Feature1");
        assert_eq!(usage.times_selected, 0);
    }

    #[test]
    fn test_feature_usage_record_selection() {
        let mut usage = FeatureUsage::new("Feature1");
        usage.record_selection(true);
        usage.record_selection(true);
        usage.record_selection(false);
        assert_eq!(usage.times_selected, 2);
        assert_eq!(usage.times_deselected, 1);
        assert!((usage.install_percentage - 66.66).abs() < 1.0);
    }

    #[test]
    fn test_analytics_collector_new() {
        let collector = AnalyticsCollector::new();
        assert!(collector.events.is_empty());
    }

    #[test]
    fn test_analytics_collector_record_event() {
        let mut collector = AnalyticsCollector::new();
        let event = AnalyticsEvent::new(EventType::InstallStart, "{CODE}", "1.0");
        collector.record_event(event);
        assert_eq!(collector.get_events().len(), 1);
    }

    #[test]
    fn test_analytics_collector_record_install() {
        let mut collector = AnalyticsCollector::new();
        collector.record_install(true, 60.0);
        assert_eq!(collector.get_metrics().total_installs, 1);
    }

    #[test]
    fn test_analytics_collector_record_feature() {
        let mut collector = AnalyticsCollector::new();
        collector.record_feature_selection("Feature1", true);
        let usage = collector.get_feature_usage("Feature1").unwrap();
        assert_eq!(usage.times_selected, 1);
    }

    #[test]
    fn test_analytics_collector_clear() {
        let mut collector = AnalyticsCollector::new();
        collector.record_event(AnalyticsEvent::new(EventType::InstallStart, "{CODE}", "1.0"));
        collector.clear();
        assert!(collector.get_events().is_empty());
    }

    #[test]
    fn test_analytics_report_generate() {
        let mut collector = AnalyticsCollector::new();
        collector.record_install(true, 60.0);
        collector.record_feature_selection("Feature1", true);

        let report = AnalyticsReport::generate("{CODE}", "MyApp", "2024-01", &collector);
        assert_eq!(report.product_name, "MyApp");
        assert_eq!(report.metrics.total_installs, 1);
    }

    #[test]
    fn test_analytics_report_to_json() {
        let collector = AnalyticsCollector::new();
        let report = AnalyticsReport::generate("{CODE}", "MyApp", "2024-01", &collector);
        let json = report.to_json();
        assert!(json.contains("MyApp"));
    }

    #[test]
    fn test_analytics_config_default() {
        let config = AnalyticsConfig::default();
        assert!(config.enabled);
        assert!(config.anonymize_data);
        assert_eq!(config.batch_size, 100);
    }

    #[test]
    fn test_analytics_generator_fragment() {
        let config = AnalyticsConfig::default();
        let fragment = AnalyticsGenerator::generate_fragment(&config);
        assert!(fragment.contains("<AnalyticsEnabled>1</AnalyticsEnabled>"));
    }

    #[test]
    fn test_analytics_generator_fragment_disabled() {
        let mut config = AnalyticsConfig::default();
        config.enabled = false;
        let fragment = AnalyticsGenerator::generate_fragment(&config);
        assert!(fragment.contains("<AnalyticsEnabled>0</AnalyticsEnabled>"));
    }

    #[test]
    fn test_analytics_generator_fragment_with_endpoint() {
        let mut config = AnalyticsConfig::default();
        config.endpoint = Some("https://analytics.example.com".to_string());
        let fragment = AnalyticsGenerator::generate_fragment(&config);
        assert!(fragment.contains("<AnalyticsEndpoint>https://analytics.example.com</AnalyticsEndpoint>"));
    }

    #[test]
    fn test_analytics_generator_dll_reference() {
        let reference = AnalyticsGenerator::generate_dll_reference(&PathBuf::from("analytics.dll"));
        assert!(reference.contains("AnalyticsDll"));
        assert!(reference.contains("analytics.dll"));
    }
}

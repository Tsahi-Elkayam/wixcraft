//! License detection module
//!
//! Detects and reports licenses used by files bundled in installers.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Known license type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LicenseType {
    MIT,
    Apache2,
    GPL2,
    GPL3,
    LGPL2,
    LGPL3,
    BSD2Clause,
    BSD3Clause,
    ISC,
    MPL2,
    Unlicense,
    CC0,
    Proprietary,
    Commercial,
    EULA,
    Unknown,
}

impl LicenseType {
    pub fn spdx_id(&self) -> &'static str {
        match self {
            LicenseType::MIT => "MIT",
            LicenseType::Apache2 => "Apache-2.0",
            LicenseType::GPL2 => "GPL-2.0-only",
            LicenseType::GPL3 => "GPL-3.0-only",
            LicenseType::LGPL2 => "LGPL-2.1-only",
            LicenseType::LGPL3 => "LGPL-3.0-only",
            LicenseType::BSD2Clause => "BSD-2-Clause",
            LicenseType::BSD3Clause => "BSD-3-Clause",
            LicenseType::ISC => "ISC",
            LicenseType::MPL2 => "MPL-2.0",
            LicenseType::Unlicense => "Unlicense",
            LicenseType::CC0 => "CC0-1.0",
            LicenseType::Proprietary => "Proprietary",
            LicenseType::Commercial => "Commercial",
            LicenseType::EULA => "EULA",
            LicenseType::Unknown => "UNKNOWN",
        }
    }

    pub fn is_open_source(&self) -> bool {
        matches!(
            self,
            LicenseType::MIT
                | LicenseType::Apache2
                | LicenseType::GPL2
                | LicenseType::GPL3
                | LicenseType::LGPL2
                | LicenseType::LGPL3
                | LicenseType::BSD2Clause
                | LicenseType::BSD3Clause
                | LicenseType::ISC
                | LicenseType::MPL2
                | LicenseType::Unlicense
                | LicenseType::CC0
        )
    }

    pub fn is_copyleft(&self) -> bool {
        matches!(
            self,
            LicenseType::GPL2 | LicenseType::GPL3 | LicenseType::LGPL2 | LicenseType::LGPL3
        )
    }

    pub fn requires_attribution(&self) -> bool {
        matches!(
            self,
            LicenseType::MIT
                | LicenseType::Apache2
                | LicenseType::BSD2Clause
                | LicenseType::BSD3Clause
                | LicenseType::ISC
                | LicenseType::MPL2
        )
    }
}

/// Detected license
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedLicense {
    pub license_type: LicenseType,
    pub confidence: f64,
    pub source_file: Option<PathBuf>,
    pub copyright_holder: Option<String>,
    pub year: Option<String>,
}

impl DetectedLicense {
    pub fn new(license_type: LicenseType, confidence: f64) -> Self {
        Self {
            license_type,
            confidence,
            source_file: None,
            copyright_holder: None,
            year: None,
        }
    }

    pub fn with_source(mut self, path: PathBuf) -> Self {
        self.source_file = Some(path);
        self
    }

    pub fn with_copyright(mut self, holder: &str, year: &str) -> Self {
        self.copyright_holder = Some(holder.to_string());
        self.year = Some(year.to_string());
        self
    }
}

/// File license info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileLicenseInfo {
    pub file_path: PathBuf,
    pub file_name: String,
    pub licenses: Vec<DetectedLicense>,
    pub needs_review: bool,
}

impl FileLicenseInfo {
    pub fn new(path: PathBuf) -> Self {
        let file_name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        Self {
            file_path: path,
            file_name,
            licenses: Vec::new(),
            needs_review: false,
        }
    }

    pub fn add_license(&mut self, license: DetectedLicense) {
        if license.confidence < 0.8 {
            self.needs_review = true;
        }
        self.licenses.push(license);
    }

    pub fn primary_license(&self) -> Option<&DetectedLicense> {
        self.licenses.iter().max_by(|a, b| {
            a.confidence
                .partial_cmp(&b.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }
}

/// License detector
#[derive(Debug, Clone, Default)]
pub struct LicenseDetector {
    patterns: HashMap<LicenseType, Vec<&'static str>>,
}

impl LicenseDetector {
    pub fn new() -> Self {
        let mut patterns = HashMap::new();

        patterns.insert(
            LicenseType::MIT,
            vec![
                "MIT License",
                "Permission is hereby granted, free of charge",
                "THE SOFTWARE IS PROVIDED \"AS IS\"",
            ],
        );

        patterns.insert(
            LicenseType::Apache2,
            vec![
                "Apache License",
                "Version 2.0",
                "Licensed under the Apache License",
            ],
        );

        patterns.insert(
            LicenseType::GPL3,
            vec![
                "GNU GENERAL PUBLIC LICENSE",
                "Version 3",
                "either version 3 of the License",
            ],
        );

        patterns.insert(
            LicenseType::GPL2,
            vec![
                "GNU GENERAL PUBLIC LICENSE",
                "Version 2",
                "either version 2 of the License",
            ],
        );

        patterns.insert(
            LicenseType::BSD3Clause,
            vec![
                "BSD 3-Clause",
                "Redistribution and use in source and binary forms",
                "Neither the name",
            ],
        );

        patterns.insert(
            LicenseType::BSD2Clause,
            vec![
                "BSD 2-Clause",
                "Redistribution and use in source and binary forms",
            ],
        );

        patterns.insert(
            LicenseType::ISC,
            vec!["ISC License", "Permission to use, copy, modify"],
        );

        patterns.insert(
            LicenseType::MPL2,
            vec!["Mozilla Public License", "Version 2.0"],
        );

        Self { patterns }
    }

    /// Detect license from text content
    pub fn detect_from_text(&self, text: &str) -> Vec<DetectedLicense> {
        let mut results = Vec::new();
        let text_lower = text.to_lowercase();

        for (license_type, patterns) in &self.patterns {
            let matches: usize = patterns
                .iter()
                .filter(|p| text_lower.contains(&p.to_lowercase()))
                .count();

            if matches > 0 {
                let confidence = matches as f64 / patterns.len() as f64;
                if confidence >= 0.5 {
                    results.push(DetectedLicense::new(*license_type, confidence));
                }
            }
        }

        if results.is_empty() {
            results.push(DetectedLicense::new(LicenseType::Unknown, 0.0));
        }

        results.sort_by(|a, b| {
            b.confidence
                .partial_cmp(&a.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        results
    }

    /// Detect license from file
    pub fn detect_from_file(&self, _path: &PathBuf) -> FileLicenseInfo {
        // In production, would read file and analyze
        let mut info = FileLicenseInfo::new(_path.clone());
        info.add_license(DetectedLicense::new(LicenseType::Unknown, 0.0));
        info
    }

    /// Scan directory for licenses
    pub fn scan_directory(&self, _path: &PathBuf) -> Vec<FileLicenseInfo> {
        // Would recursively scan directory
        Vec::new()
    }
}

/// License compatibility checker
pub struct LicenseCompatibility;

impl LicenseCompatibility {
    /// Check if licenses are compatible
    pub fn are_compatible(primary: LicenseType, dependency: LicenseType) -> bool {
        // GPL is incompatible with many licenses when used as dependency
        if dependency == LicenseType::GPL3 || dependency == LicenseType::GPL2 {
            return matches!(
                primary,
                LicenseType::GPL3 | LicenseType::GPL2 | LicenseType::LGPL3 | LicenseType::LGPL2
            );
        }

        // Most permissive licenses are compatible with everything
        if matches!(
            dependency,
            LicenseType::MIT
                | LicenseType::BSD2Clause
                | LicenseType::BSD3Clause
                | LicenseType::ISC
                | LicenseType::Unlicense
                | LicenseType::CC0
        ) {
            return true;
        }

        // LGPL is compatible with most
        if matches!(dependency, LicenseType::LGPL2 | LicenseType::LGPL3) {
            return true;
        }

        // Apache 2.0 is mostly compatible
        if dependency == LicenseType::Apache2 {
            return primary != LicenseType::GPL2;
        }

        // MPL 2.0 is file-level copyleft, generally compatible
        if dependency == LicenseType::MPL2 {
            return true;
        }

        // Unknown/Proprietary need review
        false
    }

    /// Get compatibility issues
    pub fn get_issues(licenses: &[LicenseType]) -> Vec<String> {
        let mut issues = Vec::new();

        // Check for copyleft contamination
        let has_gpl = licenses.iter().any(|l| matches!(l, LicenseType::GPL2 | LicenseType::GPL3));
        let has_proprietary = licenses.contains(&LicenseType::Proprietary);

        if has_gpl && has_proprietary {
            issues.push("GPL and Proprietary licenses are incompatible".to_string());
        }

        // Check for unknown licenses
        if licenses.contains(&LicenseType::Unknown) {
            issues.push("Unknown licenses detected - manual review required".to_string());
        }

        issues
    }
}

/// License report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseReport {
    pub project_name: String,
    pub total_files: usize,
    pub files_with_licenses: usize,
    pub needs_review: usize,
    pub license_summary: HashMap<String, usize>,
    pub compatibility_issues: Vec<String>,
    pub attribution_required: Vec<String>,
}

impl LicenseReport {
    pub fn generate(project_name: &str, files: &[FileLicenseInfo]) -> Self {
        let mut license_summary: HashMap<String, usize> = HashMap::new();
        let mut needs_review = 0;
        let mut files_with_licenses = 0;
        let mut attribution_required = Vec::new();
        let mut all_licenses = Vec::new();

        for file in files {
            if !file.licenses.is_empty() {
                files_with_licenses += 1;
            }

            if file.needs_review {
                needs_review += 1;
            }

            for license in &file.licenses {
                let key = license.license_type.spdx_id().to_string();
                *license_summary.entry(key).or_insert(0) += 1;
                all_licenses.push(license.license_type);

                if license.license_type.requires_attribution() {
                    if let Some(ref holder) = license.copyright_holder {
                        if !attribution_required.contains(holder) {
                            attribution_required.push(holder.clone());
                        }
                    }
                }
            }
        }

        let compatibility_issues = LicenseCompatibility::get_issues(&all_licenses);

        Self {
            project_name: project_name.to_string(),
            total_files: files.len(),
            files_with_licenses,
            needs_review,
            license_summary,
            compatibility_issues,
            attribution_required,
        }
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_default()
    }

    /// Generate NOTICE file content
    pub fn generate_notice(&self) -> String {
        let mut output = String::new();
        output.push_str(&format!("THIRD-PARTY NOTICES FOR {}\n", self.project_name));
        output.push_str(&"=".repeat(50));
        output.push_str("\n\n");

        output.push_str("This software includes the following third-party components:\n\n");

        for holder in &self.attribution_required {
            output.push_str(&format!("- {}\n", holder));
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_license_type_spdx_id() {
        assert_eq!(LicenseType::MIT.spdx_id(), "MIT");
        assert_eq!(LicenseType::Apache2.spdx_id(), "Apache-2.0");
    }

    #[test]
    fn test_license_type_is_open_source() {
        assert!(LicenseType::MIT.is_open_source());
        assert!(LicenseType::GPL3.is_open_source());
        assert!(!LicenseType::Proprietary.is_open_source());
    }

    #[test]
    fn test_license_type_is_copyleft() {
        assert!(LicenseType::GPL3.is_copyleft());
        assert!(LicenseType::LGPL2.is_copyleft());
        assert!(!LicenseType::MIT.is_copyleft());
    }

    #[test]
    fn test_license_type_requires_attribution() {
        assert!(LicenseType::MIT.requires_attribution());
        assert!(LicenseType::Apache2.requires_attribution());
        assert!(!LicenseType::Unlicense.requires_attribution());
    }

    #[test]
    fn test_detected_license_new() {
        let license = DetectedLicense::new(LicenseType::MIT, 0.95);
        assert_eq!(license.license_type, LicenseType::MIT);
        assert_eq!(license.confidence, 0.95);
    }

    #[test]
    fn test_detected_license_with_source() {
        let license = DetectedLicense::new(LicenseType::MIT, 0.95)
            .with_source(PathBuf::from("LICENSE"));
        assert!(license.source_file.is_some());
    }

    #[test]
    fn test_detected_license_with_copyright() {
        let license = DetectedLicense::new(LicenseType::MIT, 0.95)
            .with_copyright("Acme Inc", "2024");
        assert_eq!(license.copyright_holder, Some("Acme Inc".to_string()));
        assert_eq!(license.year, Some("2024".to_string()));
    }

    #[test]
    fn test_file_license_info_new() {
        let info = FileLicenseInfo::new(PathBuf::from("lib.dll"));
        assert_eq!(info.file_name, "lib.dll");
        assert!(info.licenses.is_empty());
    }

    #[test]
    fn test_file_license_info_add_license() {
        let mut info = FileLicenseInfo::new(PathBuf::from("lib.dll"));
        info.add_license(DetectedLicense::new(LicenseType::MIT, 0.95));
        assert_eq!(info.licenses.len(), 1);
    }

    #[test]
    fn test_file_license_info_needs_review() {
        let mut info = FileLicenseInfo::new(PathBuf::from("lib.dll"));
        info.add_license(DetectedLicense::new(LicenseType::Unknown, 0.5));
        assert!(info.needs_review);
    }

    #[test]
    fn test_file_license_info_primary() {
        let mut info = FileLicenseInfo::new(PathBuf::from("lib.dll"));
        info.add_license(DetectedLicense::new(LicenseType::MIT, 0.95));
        info.add_license(DetectedLicense::new(LicenseType::Apache2, 0.7));
        let primary = info.primary_license().unwrap();
        assert_eq!(primary.license_type, LicenseType::MIT);
    }

    #[test]
    fn test_license_detector_new() {
        let detector = LicenseDetector::new();
        assert!(!detector.patterns.is_empty());
    }

    #[test]
    fn test_license_detector_detect_mit() {
        let detector = LicenseDetector::new();
        let text = "MIT License\nPermission is hereby granted, free of charge";
        let results = detector.detect_from_text(text);
        assert!(!results.is_empty());
        assert_eq!(results[0].license_type, LicenseType::MIT);
    }

    #[test]
    fn test_license_detector_detect_unknown() {
        let detector = LicenseDetector::new();
        let text = "Some random text that doesn't match any license";
        let results = detector.detect_from_text(text);
        assert_eq!(results[0].license_type, LicenseType::Unknown);
    }

    #[test]
    fn test_license_compatibility_permissive() {
        assert!(LicenseCompatibility::are_compatible(
            LicenseType::Apache2,
            LicenseType::MIT
        ));
    }

    #[test]
    fn test_license_compatibility_gpl_restriction() {
        assert!(!LicenseCompatibility::are_compatible(
            LicenseType::MIT,
            LicenseType::GPL3
        ));
    }

    #[test]
    fn test_license_compatibility_gpl_compatible() {
        assert!(LicenseCompatibility::are_compatible(
            LicenseType::GPL3,
            LicenseType::GPL3
        ));
    }

    #[test]
    fn test_license_compatibility_get_issues() {
        let licenses = vec![LicenseType::GPL3, LicenseType::Proprietary];
        let issues = LicenseCompatibility::get_issues(&licenses);
        assert!(!issues.is_empty());
    }

    #[test]
    fn test_license_report_generate() {
        let mut file = FileLicenseInfo::new(PathBuf::from("lib.dll"));
        file.add_license(DetectedLicense::new(LicenseType::MIT, 0.95));
        let files = vec![file];

        let report = LicenseReport::generate("MyProject", &files);
        assert_eq!(report.total_files, 1);
        assert_eq!(report.files_with_licenses, 1);
    }

    #[test]
    fn test_license_report_to_json() {
        let report = LicenseReport::generate("MyProject", &[]);
        let json = report.to_json();
        assert!(json.contains("MyProject"));
    }

    #[test]
    fn test_license_report_generate_notice() {
        let mut file = FileLicenseInfo::new(PathBuf::from("lib.dll"));
        let mut license = DetectedLicense::new(LicenseType::MIT, 0.95);
        license.copyright_holder = Some("Acme Inc".to_string());
        file.add_license(license);
        let files = vec![file];

        let report = LicenseReport::generate("MyProject", &files);
        let notice = report.generate_notice();
        assert!(notice.contains("THIRD-PARTY NOTICES"));
    }
}

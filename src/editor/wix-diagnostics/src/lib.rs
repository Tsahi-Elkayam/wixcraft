//! WiX diagnostics library
//!
//! Real-time validation for WiX XML files including:
//! - Reference validation (ComponentRef points to existing Component)
//! - Parent/child relationship validation
//! - Attribute type validation (GUID format, required attrs, enum values)
//!
//! # Example
//!
//! ```
//! use wix_diagnostics::DiagnosticsEngine;
//! use std::path::Path;
//!
//! let mut engine = DiagnosticsEngine::new();
//! let source = r#"<Wix><ComponentRef Id="Missing" /></Wix>"#;
//! let result = engine.diagnose(source, Path::new("test.wxs")).unwrap();
//!
//! assert!(!result.is_empty());
//! ```

mod types;
pub mod validators;

pub use types::{
    Diagnostic, DiagnosticSeverity, DiagnosticsResult, Location, Position, Range,
    RelatedInformation,
};
pub use validators::{AttributeValidator, ReferenceValidator, RelationshipValidator};

use std::path::Path;

/// Main diagnostics engine
pub struct DiagnosticsEngine {
    reference_validator: ReferenceValidator,
    relationship_validator: RelationshipValidator,
    attribute_validator: AttributeValidator,
}

impl DiagnosticsEngine {
    /// Create a new diagnostics engine
    pub fn new() -> Self {
        Self {
            reference_validator: ReferenceValidator::new(),
            relationship_validator: RelationshipValidator::new(),
            attribute_validator: AttributeValidator::new(),
        }
    }

    /// Index a file's definitions for cross-file reference validation
    pub fn index_file(&mut self, source: &str) -> Result<(), String> {
        self.reference_validator.index_file(source)
    }

    /// Add a known definition for reference validation
    pub fn add_definition(&mut self, element_type: &str, id: &str) {
        self.reference_validator.add_definition(element_type, id);
    }

    /// Run all diagnostics on a source file
    pub fn diagnose(&self, source: &str, file: &Path) -> Result<DiagnosticsResult, String> {
        let mut result = DiagnosticsResult::new(file.to_path_buf());

        // Run reference validation
        let ref_diagnostics = self.reference_validator.validate(source, file)?;
        result.extend(ref_diagnostics);

        // Run relationship validation
        let rel_diagnostics = self.relationship_validator.validate(source, file)?;
        result.extend(rel_diagnostics);

        // Run attribute validation
        let attr_diagnostics = self.attribute_validator.validate(source, file)?;
        result.extend(attr_diagnostics);

        Ok(result)
    }

    /// Run only reference validation
    pub fn diagnose_references(
        &self,
        source: &str,
        file: &Path,
    ) -> Result<DiagnosticsResult, String> {
        let mut result = DiagnosticsResult::new(file.to_path_buf());
        result.extend(self.reference_validator.validate(source, file)?);
        Ok(result)
    }

    /// Run only relationship validation
    pub fn diagnose_relationships(
        &self,
        source: &str,
        file: &Path,
    ) -> Result<DiagnosticsResult, String> {
        let mut result = DiagnosticsResult::new(file.to_path_buf());
        result.extend(self.relationship_validator.validate(source, file)?);
        Ok(result)
    }

    /// Run only attribute validation
    pub fn diagnose_attributes(
        &self,
        source: &str,
        file: &Path,
    ) -> Result<DiagnosticsResult, String> {
        let mut result = DiagnosticsResult::new(file.to_path_buf());
        result.extend(self.attribute_validator.validate(source, file)?);
        Ok(result)
    }
}

impl Default for DiagnosticsEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_creation() {
        let engine = DiagnosticsEngine::new();
        assert!(engine
            .diagnose("<Wix />", Path::new("test.wxs"))
            .unwrap()
            .is_empty());
    }

    #[test]
    fn test_reference_validation() {
        let engine = DiagnosticsEngine::new();
        let source = r#"<Wix><ComponentRef Id="Missing" /></Wix>"#;
        let result = engine.diagnose(source, Path::new("test.wxs")).unwrap();

        assert_eq!(result.error_count(), 1);
    }

    #[test]
    fn test_relationship_validation() {
        let engine = DiagnosticsEngine::new();
        let source = r#"<Wix><Directory Id="D1"><File Id="F1" /></Directory></Wix>"#;
        let result = engine.diagnose(source, Path::new("test.wxs")).unwrap();

        // File not in Component is an error
        assert!(result.error_count() > 0);
    }

    #[test]
    fn test_attribute_validation() {
        let engine = DiagnosticsEngine::new();
        let source = r#"<Wix><Component Guid="invalid" /></Wix>"#;
        let result = engine.diagnose(source, Path::new("test.wxs")).unwrap();

        assert!(result.error_count() > 0);
    }

    #[test]
    fn test_index_and_validate() {
        let mut engine = DiagnosticsEngine::new();

        // Index definitions
        engine
            .index_file(r#"<Wix><Component Id="C1" /></Wix>"#)
            .unwrap();

        // Validate references
        let source = r#"<Wix><ComponentRef Id="C1" /></Wix>"#;
        let result = engine.diagnose(source, Path::new("test.wxs")).unwrap();

        // Reference to C1 should be valid now
        assert_eq!(result.error_count(), 0);
    }

    #[test]
    fn test_add_definition() {
        let mut engine = DiagnosticsEngine::new();
        engine.add_definition("Component", "External");

        let source = r#"<Wix><ComponentRef Id="External" /></Wix>"#;
        let result = engine.diagnose(source, Path::new("test.wxs")).unwrap();

        assert_eq!(result.error_count(), 0);
    }

    #[test]
    fn test_valid_file() {
        let mut engine = DiagnosticsEngine::new();

        let source = r#"
<Wix>
    <Package Name="Test" Version="1.0">
        <Directory Id="TARGETDIR">
            <Component Id="C1" Guid="*">
                <File Id="F1" Source="test.exe" />
            </Component>
        </Directory>
        <Feature Id="Main">
            <ComponentRef Id="C1" />
        </Feature>
    </Package>
</Wix>"#;

        engine.index_file(source).unwrap();
        let result = engine.diagnose(source, Path::new("test.wxs")).unwrap();

        // This should be a valid WiX file
        assert_eq!(result.error_count(), 0, "Errors: {:?}", result.diagnostics);
    }
}

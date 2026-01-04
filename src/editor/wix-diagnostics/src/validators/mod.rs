//! Validators for WiX diagnostics

pub mod attributes;
pub mod references;
pub mod relationships;

pub use attributes::AttributeValidator;
pub use references::ReferenceValidator;
pub use relationships::RelationshipValidator;

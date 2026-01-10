//! wix-test - Unified testing framework for WiX installers
//!
//! This crate provides comprehensive testing capabilities for WiX projects:
//!
//! - **MSI Testing**: Validate MSI structure, files, registry, components, features
//! - **Custom Action Testing**: Unit test custom action DLLs with mock sessions
//!
//! # Subcommands
//!
//! - `msi` - Test MSI package structure and contents
//! - `ca` - Test custom action DLLs
//! - `suite` - Run test suites (MSI or CA)
//! - `sandbox` - Test MSI installation in Windows Sandbox

pub mod msi;
pub mod ca;

// Re-export MSI testing types
pub use msi::{
    TestStatus, TestType, TestCase, Assertion, TestResult, AssertionResult,
    TestSuite, TestSetup, TestTeardown, TestReport, TestBuilder, TestLoader,
};

// Re-export CA testing types
pub use ca::{
    CATest, SessionData, ComponentState, FeatureState, InstallMode, UILevel,
    CAResult, CATestResult, CATestSuite, CATestReport, CAAssert, CATestData,
};

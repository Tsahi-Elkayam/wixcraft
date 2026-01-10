//! wix-init - Unified WiX project lifecycle tool
//!
//! This crate combines project scaffolding, GUID generation, installation management,
//! environment setup, and licensing into a single comprehensive tool.
//!
//! # Subcommands
//!
//! - `new` - Create new WiX project from template
//! - `list` - List available templates
//! - `wizard` - Interactive project configuration
//! - `guid` - GUID generation utilities
//! - `install` - Install MSI package
//! - `uninstall` - Uninstall MSI package
//! - `update` - Update installed MSI
//! - `repair` - Repair MSI installation
//! - `doctor` - Check WiX environment
//! - `setup` - Setup WiX development environment
//! - `license` - Generate license files

pub mod project;
pub mod wizard;
pub mod guid;
pub mod install;
pub mod uninstall;
pub mod update;
pub mod env;
pub mod license;
pub mod repair;
pub mod silent;

// Re-export main types from project
pub use project::{Project, Template, WixVersion, CreatedProject, InitError};

// Re-export from wizard
pub use wizard::{Wizard, WizardStep, Question, ProjectConfig, ProjectType};

// Re-export from guid
pub use guid::{Guid, GuidFormat, GuidGenerator, GuidBatch, GuidError};

// Re-export from install
pub use install::{InstallOptions, InstallResult, InstallMode, UILevel, MsiExecCommand, InstallPresets};

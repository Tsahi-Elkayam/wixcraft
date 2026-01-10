//! Auto-fix system for applying suggested fixes

mod engine;

pub use engine::{FixEngine, FixError, FixPreview, FixResult};

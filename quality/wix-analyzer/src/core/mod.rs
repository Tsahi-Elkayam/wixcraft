//! Core infrastructure for WiX analysis

pub mod baseline;
pub mod cache;
pub mod complexity;
pub mod debt;
pub mod diff;
pub mod document;
pub mod duplication;
pub mod extractor;
pub mod gate;
pub mod index;
pub mod newcode;
pub mod plugin;
pub mod profile;
pub mod secrets;
pub mod suppression;
pub mod types;
pub mod watch;

pub use complexity::{
    ComplexityCalculator, ComplexityRating, ComplexityThreshold, FileComplexity, ProjectComplexity,
};
pub use debt::{
    DebtBreakdown, DebtCategory, DebtQualityGate, DebtRating, QualityGateResult, SeverityCounts,
    TechnicalDebt,
};
pub use document::{NodeExt, WixDocument};
pub use duplication::{
    Duplicate, DuplicateLocation, DuplicationConfig, DuplicationDetector, DuplicationRating,
    DuplicationResult,
};
pub use extractor::{extract_from_source, extract_symbols, symbol_at_position, ExtractionResult, SymbolAtPosition};
pub use index::SymbolIndex;
pub use secrets::{DetectedSecret, SecretSeverity, SecretType, SecretsDetector, SecretsResult};
pub use suppression::SuppressionContext;
pub use baseline::{Baseline, BaselineEntry, BaselineError, BaselineStats, BASELINE_FILE_NAME, filter_baseline};
pub use cache::{AnalysisCache, CacheError, CacheStats, CACHE_DIR_NAME};
pub use diff::{DiffDetector, DiffError, DiffResult, DiffSource, filter_to_changed};
pub use newcode::{NewCodeDetector, NewCodeError, NewCodePeriod, NewCodeResult, filter_to_new_code};
pub use plugin::{
    PluginRegistry, PluginError, PluginManifest, PluginRule, RuleCondition,
    DeprecatedRuleInfo, PluginCategory, PluginSeverity,
};
pub use profile::{ProfileName, QualityProfile, available_profiles, profile_descriptions};
pub use gate::{GateCondition, GateFailure, GateResult, QualityGate, RatingType};
pub use watch::{FileWatcher, WatchConfig, WatchError, WatchEvent};
pub use types::*;

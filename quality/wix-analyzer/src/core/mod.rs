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

pub use baseline::{
    filter_baseline, Baseline, BaselineEntry, BaselineError, BaselineStats, BASELINE_FILE_NAME,
};
pub use cache::{AnalysisCache, CacheError, CacheStats, CACHE_DIR_NAME};
pub use complexity::{
    ComplexityCalculator, ComplexityRating, ComplexityThreshold, FileComplexity, ProjectComplexity,
};
pub use debt::{
    DebtBreakdown, DebtCategory, DebtQualityGate, DebtRating, QualityGateResult, SeverityCounts,
    TechnicalDebt,
};
pub use diff::{filter_to_changed, DiffDetector, DiffError, DiffResult, DiffSource};
pub use document::{NodeExt, WixDocument};
pub use duplication::{
    Duplicate, DuplicateLocation, DuplicationConfig, DuplicationDetector, DuplicationRating,
    DuplicationResult,
};
pub use extractor::{
    extract_from_source, extract_symbols, symbol_at_position, ExtractionResult, SymbolAtPosition,
};
pub use gate::{GateCondition, GateFailure, GateResult, QualityGate, RatingType};
pub use index::SymbolIndex;
pub use newcode::{
    filter_to_new_code, NewCodeDetector, NewCodeError, NewCodePeriod, NewCodeResult,
};
pub use plugin::{
    DeprecatedRuleInfo, PluginCategory, PluginError, PluginManifest, PluginRegistry, PluginRule,
    PluginSeverity, RuleCondition,
};
pub use profile::{available_profiles, profile_descriptions, ProfileName, QualityProfile};
pub use secrets::{DetectedSecret, SecretSeverity, SecretType, SecretsDetector, SecretsResult};
pub use suppression::SuppressionContext;
pub use types::*;
pub use watch::{FileWatcher, WatchConfig, WatchError, WatchEvent};

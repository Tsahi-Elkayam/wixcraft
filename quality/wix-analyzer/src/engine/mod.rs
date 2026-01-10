//! Generic Static Analysis Engine
//!
//! A language-agnostic engine that supports both data-driven and code-based rules.
//! Languages are added via plugins that implement the `LanguagePlugin` trait.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────┐
//! │           Analysis Engine               │
//! │  ┌─────────────┐  ┌─────────────────┐  │
//! │  │ DataRule    │  │ CodeRule        │  │
//! │  │ Evaluator   │  │ Interface       │  │
//! │  └──────┬──────┘  └────────┬────────┘  │
//! │         └──────────┬───────┘           │
//! │                    ▼                   │
//! │         ┌─────────────────┐            │
//! │         │ LanguagePlugin  │            │
//! │         └─────────────────┘            │
//! └─────────────────────────────────────────┘
//! ```

pub mod condition;
pub mod evaluator;
pub mod plugin;
pub mod rule;
pub mod types;

pub use condition::{CompareOp, Condition, ConditionEvaluator};
pub use evaluator::{EvaluatorConfig, EvaluatorStats, RuleEvaluator};
pub use plugin::{LanguagePlugin, ParseResult, PluginCapabilities};
pub use rule::{
    CodeRule, DataRule, Diagnostic, ElementPosition, Fix, FixAction, FixTemplate, RuleCategory,
    RuleImpl, RuleSeverity, TextEdit,
};
pub use types::{Attribute, Document, Node, NodeKind};

//! Best practice analyzers

pub mod efficiency;
pub mod idioms;
pub mod maintainability;
pub mod performance;

pub use efficiency::EfficiencyAnalyzer;
pub use idioms::IdiomsAnalyzer;
pub use maintainability::MaintainabilityAnalyzer;
pub use performance::PerformanceAnalyzer;

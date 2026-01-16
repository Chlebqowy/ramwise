//! Memory analysis and insight generation
//!
//! This module contains the rule engine that analyzes memory patterns
//! and generates actionable insights.

mod engine;
mod rules;
mod insights;

pub use engine::Analyzer;
pub use insights::{Insight, Severity};
// Rule trait exported for extensibility
#[allow(unused_imports)]
pub use rules::Rule;

// Copyright (C) 2026 rezky_nightky
// SPDX-License-Identifier: GPL-3.0-only

//! Analysis engine — consumes Evidence, produces readiness and risks.
//!
//! Never performs Linux probing. Pure transformation from Evidence
//! to structured analysis.
//!
//! Architecture:
//!   readiness.rs → category scores + overall rating
//!   risk.rs       → risk identification
//!   compatibility.rs → compatibility assessment (extensible)

pub mod compatibility;
pub mod decision;
pub mod readiness;
pub mod risk;

use crate::core::evidence::Evidence;

// Re-export public types for backward compatibility.
pub use compatibility::{
    BlockingIssue, Compatibility, ComponentScore, ComponentStatus, Confidence, RecommendedAction,
    RiskLevel, assess,
};
pub use decision::{
    ActionPriority, Confidence as DecisionConfidence, DecisionAction, DecisionPlan, Difficulty,
    evaluate,
};
pub use readiness::{CategoryScore, Readiness};
pub use risk::Risk;

/// Analyze evidence and produce readiness + risks.
///
/// This is the single orchestration entry point. Each sub-engine
/// operates independently on the same Evidence slice.
pub fn analyze(evidence: &[Evidence]) -> (Readiness, Vec<Risk>) {
    let categories = readiness::compute_categories(evidence);
    let overall = readiness::overall_rating(&categories);
    let risks = risk::identify_risks(evidence);
    let stars_str = readiness::stars_label(&categories);

    (
        Readiness {
            overall,
            stars: stars_str,
            categories,
        },
        risks,
    )
}

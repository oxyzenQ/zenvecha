// Copyright (C) 2026 rezky_nightky
// SPDX-License-Identifier: GPL-3.0-only

//! Wolfzenix pipeline — explicit orchestration.
//!
//! Central flow: collect → Evidence → Analysis → Recommendation → Render.
//! No business logic outside this pipeline. Each step is a pure function
//! receiving the previous step's output.

use crate::core::analysis::{self, Compatibility, Readiness, Risk};
use crate::core::capability::Registry;
use crate::core::evidence::Evidence;
use crate::core::recommendation;

/// Result of running the analysis pipeline.
pub struct AnalysisResult {
    pub evidence: Vec<Evidence>,
    pub readiness: Readiness,
    pub risks: Vec<Risk>,
    pub compatibility: Compatibility,
    pub recommendations: Vec<String>,
}

/// Run the full analysis pipeline: collect → analyze → recommend.
pub fn run_analysis_pipeline() -> AnalysisResult {
    let reg = Registry::default();
    let evidence = reg.run_all();
    let (readiness, risks) = analysis::analyze(&evidence);
    let compatibility = analysis::compatibility::assess(&evidence);
    let recommendations = recommendation::recommend(&evidence);
    AnalysisResult {
        evidence,
        readiness,
        risks,
        compatibility,
        recommendations,
    }
}

/// Collect evidence only — for inspect and ABI commands.
pub fn collect_evidence() -> Vec<Evidence> {
    let reg = Registry::default();
    reg.run_all()
}

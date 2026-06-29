// Copyright (C) 2026 rezky_nightky
// SPDX-License-Identifier: GPL-3.0-only

//! Analyze command — development readiness assessment with full intelligence.
//!
//! Orchestration only. Delegates to pipeline and render layer.

use std::io;

use crate::core::pipeline;
use crate::core::render::analyze::{self, AnalyzeModels};

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let result = pipeline::run_analysis_pipeline();

    let stdout = io::stdout();
    let mut out = stdout.lock();

    let models = AnalyzeModels {
        evidence: &result.evidence,
        readiness: &result.readiness,
        compatibility: &result.compatibility,
        decision_plan: &result.decision_plan,
        prediction: &result.prediction,
        knowledge: &result.knowledge,
        reasoning: &result.reasoning,
        semantic: &result.semantic_descriptors,
        recs: &result.recommendations,
    };

    analyze::render(&models, &mut out)
}

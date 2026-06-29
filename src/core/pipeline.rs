// Copyright (C) 2026 rezky_nightky
// SPDX-License-Identifier: GPL-3.0-only

//! Wolfzenix pipeline — explicit orchestration.
//!
//! Central flow: collect → Evidence → Analysis → Compatibility →
//! Decision → Prediction → Knowledge → Reasoning → Recommend → Render.
//! No business logic outside this pipeline.

use crate::core::analysis::{self, Compatibility, DecisionPlan, PredictionResult, Readiness, Risk};
use crate::core::capability::Registry;
use crate::core::evidence::Evidence;
use crate::core::knowledge::resolver::{KnowledgeResult, resolve};
use crate::core::reasoning::builder::build_reasoning;
use crate::core::reasoning::model::ReasoningResult;
use crate::core::recommendation;

/// Result of running the full pipeline with reasoning.
pub struct AnalysisResult {
    pub evidence: Vec<Evidence>,
    pub readiness: Readiness,
    pub risks: Vec<Risk>,
    pub compatibility: Compatibility,
    pub decision_plan: DecisionPlan,
    pub prediction: PredictionResult,
    pub knowledge: KnowledgeResult,
    pub reasoning: ReasoningResult,
    pub recommendations: Vec<String>,
}

/// Run the full pipeline.
pub fn run_analysis_pipeline() -> AnalysisResult {
    let reg = Registry::default();
    let evidence = reg.run_all();
    let (readiness, risks) = analysis::analyze(&evidence);
    let compatibility = analysis::compatibility::assess(&evidence);
    let decision_plan = analysis::decision::evaluate(&evidence, &compatibility);
    let prediction = analysis::prediction::simulate(&evidence, &compatibility, &decision_plan);
    let knowledge = resolve(&evidence);
    let reasoning = build_reasoning(
        &evidence,
        &compatibility,
        &decision_plan,
        &prediction,
        &knowledge,
    );
    let recommendations = recommendation::recommend(&evidence);
    AnalysisResult {
        evidence,
        readiness,
        risks,
        compatibility,
        decision_plan,
        prediction,
        knowledge,
        reasoning,
        recommendations,
    }
}

/// Collect evidence only — for inspect and ABI commands.
pub fn collect_evidence() -> Vec<Evidence> {
    let reg = Registry::default();
    reg.run_all()
}

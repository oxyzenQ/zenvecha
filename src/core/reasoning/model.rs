// Copyright (C) 2026 rezky_nightky
// SPDX-License-Identifier: GPL-3.0-only

//! Reasoning domain models — traceable explanations.
//!
//! Every conclusion produced by previous engines must be explainable.
//! Reasoning nodes link conclusions back to evidence, assumptions,
//! limitations, and upstream engine dependencies.

/// A single node in the reasoning chain.
#[derive(Clone, Debug)]
pub struct ReasoningNode {
    /// What is being explained.
    pub title: &'static str,
    /// The conclusion.
    pub conclusion: String,
    /// Why this conclusion was reached.
    pub because: Vec<String>,
    /// Specific evidence items that support this conclusion.
    pub supporting_evidence: Vec<EvidenceRef>,
    /// Evidence or facts that conflict with this conclusion.
    pub conflicting_evidence: Vec<String>,
    /// What we assume to be true.
    pub assumptions: Vec<String>,
    /// What we know we don't know.
    pub limitations: Vec<String>,
    /// Why we are confident (or not) in this conclusion.
    pub confidence_reason: String,
    /// Upstream engine dependencies.
    pub dependencies: Vec<&'static str>,
}

/// A reference to a specific evidence item.
#[derive(Clone, Debug)]
pub struct EvidenceRef {
    /// The evidence ID (e.g., "kernel.release").
    pub evidence_id: &'static str,
    /// Human-readable name of the evidence.
    pub label: &'static str,
    /// The value as observed.
    pub value: String,
    /// How this evidence supports the conclusion.
    pub relevance: String,
}

/// Complete reasoning result — explains every major decision.
#[derive(Clone, Debug)]
pub struct ReasoningResult {
    /// Why the overall readiness rating was assigned.
    pub readiness_reason: ReasoningNode,
    /// Why the compatibility score was assigned.
    pub compatibility_reason: ReasoningNode,
    /// Why each blocking issue is blocking.
    pub blocking_reasons: Vec<ReasoningNode>,
    /// Why the top decision was chosen.
    pub decision_reason: Option<ReasoningNode>,
    /// Why the top prediction has its expected outcome.
    pub prediction_reason: Option<ReasoningNode>,
    /// Key insights from knowledge matching.
    pub knowledge_insights: Vec<String>,
    /// Overall system narrative.
    pub system_narrative: String,
}

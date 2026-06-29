// Copyright (C) 2026 rezky_nightky
// SPDX-License-Identifier: GPL-3.0-only

//! Reasoning builder — constructs explainable conclusions from existing models.
//!
//! Consumes Evidence, Compatibility, DecisionPlan, PredictionResult,
//! and KnowledgeResult. Produces ReasoningResult with traceable
//! explanation chains. Never recalculates. Never probes the system.

use crate::core::analysis::compatibility::Compatibility;
use crate::core::analysis::decision::DecisionPlan;
use crate::core::analysis::prediction::PredictionResult;
use crate::core::evidence::Evidence;
use crate::core::evidence_helpers;
use crate::core::knowledge::resolver::KnowledgeResult;

use super::model::{EvidenceRef, ReasoningNode, ReasoningResult};

/// Build a complete reasoning chain from all existing engine outputs.
pub fn build_reasoning(
    evidence: &[Evidence],
    compatibility: &Compatibility,
    decision_plan: &DecisionPlan,
    prediction: &PredictionResult,
    knowledge: &KnowledgeResult,
) -> ReasoningResult {
    let readiness_reason = explain_readiness(evidence, compatibility);
    let compatibility_reason = explain_compatibility(evidence, compatibility);
    let blocking_reasons = explain_blocking(evidence, compatibility);
    let decision_reason = decision_plan
        .highest_roi_action
        .as_ref()
        .map(|action| explain_decision(evidence, action));
    let prediction_reason = prediction
        .scenarios
        .first()
        .map(|s| explain_prediction(evidence, s));
    let knowledge_insights = knowledge.insights.clone();
    let system_narrative = build_narrative(evidence, compatibility, decision_plan);

    ReasoningResult {
        readiness_reason,
        compatibility_reason,
        blocking_reasons,
        decision_reason,
        prediction_reason,
        knowledge_insights,
        system_narrative,
    }
}

// ============================================================================
//  Readiness Explanation
// ============================================================================

fn explain_readiness(evidence: &[Evidence], compatibility: &Compatibility) -> ReasoningNode {
    let mut because = Vec::new();
    let mut evidence_refs = Vec::new();

    // Kernel identity
    let release = evidence_helpers::ev_s(evidence, "kernel.release");
    if release != "Unknown" {
        because.push(format!("Kernel release identified: {release}"));
        evidence_refs.push(EvidenceRef {
            evidence_id: "kernel.release",
            label: "Kernel Release",
            value: release.clone(),
            relevance: "Identifies kernel version for compatibility checks".into(),
        });
    }

    // Headers
    if evidence_helpers::ev_status_is(evidence, "build.headers", "Complete") {
        because.push("Kernel headers match running kernel".into());
        evidence_refs.push(EvidenceRef {
            evidence_id: "build.headers",
            label: "Header Integrity",
            value: "Complete".into(),
            relevance: "Headers enable external module compilation".into(),
        });
    } else {
        because.push("Kernel headers are missing or incomplete".into());
    }

    // Toolchain
    if evidence_helpers::ev_bool(evidence, "toolchain.gcc") {
        because.push("C compiler available".into());
    }
    if evidence_helpers::ev_bool(evidence, "toolchain.rustc") {
        because.push("Rust compiler available".into());
    }

    // Config
    if evidence_helpers::ev_text_value(evidence, "config.source").is_some() {
        let mods = evidence_helpers::ev_bool(evidence, "config.MODULES");
        because.push(format!("CONFIG_MODULES={}", if mods { "y" } else { "n" }));
        evidence_refs.push(EvidenceRef {
            evidence_id: "config.MODULES",
            label: "CONFIG_MODULES",
            value: if mods {
                "enabled".into()
            } else {
                "disabled".into()
            },
            relevance: "Required for kernel module development".into(),
        });
    }

    let level_label = compatibility.level;
    let score = compatibility.score;

    ReasoningNode {
        title: "Readiness Assessment",
        conclusion: format!("Kernel development readiness is {level_label} ({score}%)"),
        because,
        supporting_evidence: evidence_refs,
        conflicting_evidence: Vec::new(),
        assumptions: vec![
            "System state has not changed since evidence collection".into(),
            "Package manager is functional for installing dependencies".into(),
        ],
        limitations: vec![
            "Runtime kernel behavior not tested".into(),
            "Module loading success not simulated".into(),
        ],
        confidence_reason: format!(
            "Based on {count} direct system probes",
            count = count_probes(evidence)
        ),
        dependencies: vec!["Capability Engine", "Compatibility Engine"],
    }
}

// ============================================================================
//  Compatibility Explanation
// ============================================================================

fn explain_compatibility(evidence: &[Evidence], compatibility: &Compatibility) -> ReasoningNode {
    let mut because = Vec::new();
    let mut evidence_refs = Vec::new();

    for comp in &compatibility.components {
        let status_str = match comp.status {
            crate::core::analysis::ComponentStatus::Good => "pass",
            crate::core::analysis::ComponentStatus::Partial => "partial",
            crate::core::analysis::ComponentStatus::Missing => "fail",
            crate::core::analysis::ComponentStatus::Blocking => "blocking",
        };
        because.push(format!(
            "{}: {status_str} ({score}%) — {detail}",
            comp.name,
            score = comp.score,
            detail = comp.detail
        ));
    }

    // Key evidence items
    let headers_ok = evidence_helpers::ev_status_is(evidence, "build.headers", "Complete");
    evidence_refs.push(EvidenceRef {
        evidence_id: "build.headers",
        label: "Kernel Headers",
        value: if headers_ok {
            "Present".into()
        } else {
            "Missing".into()
        },
        relevance: "Most impactful single factor for build compatibility".into(),
    });

    let gcc_ok = evidence_helpers::ev_bool(evidence, "toolchain.gcc");
    evidence_refs.push(EvidenceRef {
        evidence_id: "toolchain.gcc",
        label: "C Compiler",
        value: if gcc_ok {
            "Available".into()
        } else {
            "Missing".into()
        },
        relevance: "Required for any kernel module compilation".into(),
    });

    let rust_ok = evidence_helpers::ev_bool(evidence, "config.RUST");
    evidence_refs.push(EvidenceRef {
        evidence_id: "config.RUST",
        label: "Rust Support",
        value: if rust_ok {
            "Enabled".into()
        } else {
            "Not enabled".into()
        },
        relevance: "Required for Rust kernel module development".into(),
    });

    ReasoningNode {
        title: "Compatibility Assessment",
        conclusion: format!(
            "Overall compatibility: {} ({}%)",
            compatibility.level, compatibility.score
        ),
        because,
        supporting_evidence: evidence_refs,
        conflicting_evidence: Vec::new(),
        assumptions: vec![
            "Installed headers match running kernel ABI".into(),
            "System configuration is representative of normal operation".into(),
        ],
        limitations: vec![
            "Secure boot status not evaluated".into(),
            "Kernel module runtime behavior not tested".into(),
        ],
        confidence_reason: compatibility.confidence.label().to_string(),
        dependencies: vec!["Capability Engine", "Compatibility Engine"],
    }
}

// ============================================================================
//  Blocking Issues Explanation
// ============================================================================

fn explain_blocking(evidence: &[Evidence], compatibility: &Compatibility) -> Vec<ReasoningNode> {
    compatibility
        .blocking_issues
        .iter()
        .map(|issue| {
            let evidence_refs = match issue.component {
                "Kernel Headers" => vec![EvidenceRef {
                    evidence_id: "build.headers",
                    label: "Header Integrity",
                    value: evidence_helpers::ev_s(evidence, "build.headers"),
                    relevance: "Headers missing — blocks ALL external module compilation".into(),
                }],
                "Toolchain" => vec![EvidenceRef {
                    evidence_id: "toolchain.gcc",
                    label: "C Compiler",
                    value: if evidence_helpers::ev_bool(evidence, "toolchain.gcc") {
                        "Found".into()
                    } else {
                        "Not found".into()
                    },
                    relevance: "Without C compiler, no kernel code can be built".into(),
                }],
                "Module Support" => vec![EvidenceRef {
                    evidence_id: "config.MODULES",
                    label: "CONFIG_MODULES",
                    value: if evidence_helpers::ev_bool(evidence, "config.MODULES") {
                        "enabled".into()
                    } else {
                        "disabled".into()
                    },
                    relevance: "Without module support, no external kernel code can be loaded"
                        .into(),
                }],
                _ => Vec::new(),
            };

            ReasoningNode {
                title: "Blocking Issue",
                conclusion: format!("{}: {}", issue.component, issue.description),
                because: vec![format!(
                    "{} is a hard requirement for kernel development",
                    issue.component
                )],
                supporting_evidence: evidence_refs,
                conflicting_evidence: Vec::new(),
                assumptions: vec![
                    "Fixing this issue will unblock further development steps".into(),
                ],
                limitations: vec![format!(
                    "May reveal additional issues once {} is resolved",
                    issue.component
                )],
                confidence_reason: "Direct evidence — this is a binary condition".into(),
                dependencies: vec!["Capability Engine", "Compatibility Engine"],
            }
        })
        .collect()
}

// ============================================================================
//  Decision Explanation
// ============================================================================

fn explain_decision(
    evidence: &[Evidence],
    action: &crate::core::analysis::decision::DecisionAction,
) -> ReasoningNode {
    let mut evidence_refs = Vec::new();

    // Gather relevant evidence for this action
    if action.title.contains("headers") {
        evidence_refs.push(EvidenceRef {
            evidence_id: "build.headers",
            label: "Header Status",
            value: evidence_helpers::ev_s(evidence, "build.headers"),
            relevance: "Headers are missing — this is the root cause".into(),
        });
        evidence_refs.push(EvidenceRef {
            evidence_id: "kernel.release",
            label: "Kernel Version",
            value: evidence_helpers::ev_s(evidence, "kernel.release"),
            relevance: "Headers must match this kernel version".into(),
        });
    }

    if action.title.contains("gcc") || action.title.contains("build toolchain") {
        evidence_refs.push(EvidenceRef {
            evidence_id: "toolchain.gcc",
            label: "C Compiler",
            value: "Not found".into(),
            relevance: "Compiler is missing — cannot build kernel modules".into(),
        });
    }

    ReasoningNode {
        title: "Recommended Action",
        conclusion: action.title.clone(),
        because: vec![action.why.clone()],
        supporting_evidence: evidence_refs,
        conflicting_evidence: Vec::new(),
        assumptions: vec!["User has required permissions to execute this action".into()],
        limitations: vec![format!(
            "Estimated {} min — actual time may vary by system",
            action.estimated_minutes
        )],
        confidence_reason: format!(
            "ROI {:.2}/min — {} difficulty, {} priority",
            action.roi,
            action.difficulty.label(),
            action.priority.label()
        ),
        dependencies: vec![
            "Capability Engine",
            "Compatibility Engine",
            "Decision Engine",
        ],
    }
}

// ============================================================================
//  Prediction Explanation
// ============================================================================

fn explain_prediction(
    evidence: &[Evidence],
    scenario: &crate::core::analysis::prediction::Scenario,
) -> ReasoningNode {
    let delta = if scenario.score_delta >= 0 {
        format!("+{}%", scenario.score_delta)
    } else {
        format!("{}%", scenario.score_delta)
    };

    let mut because = Vec::new();
    because.push(format!(
        "Taking action '{}' is expected to improve compatibility by {delta}",
        scenario.action
    ));

    for cap in &scenario.unlocked_capabilities {
        because.push(format!("Will unlock: {cap}"));
    }

    if scenario.requires_reboot {
        because.push("This action requires a system reboot".into());
    }

    let refs = vec![EvidenceRef {
        evidence_id: "kernel.release",
        label: "Current Kernel",
        value: evidence_helpers::ev_s(evidence, "kernel.release"),
        relevance: "Baseline for prediction".into(),
    }];

    ReasoningNode {
        title: "Prediction",
        conclusion: format!(
            "Expected: {}% ({delta}) — Risk: {}, Confidence: {}%",
            scenario.expected_score,
            scenario.expected_risk.label(),
            scenario.confidence.percentage()
        ),
        because,
        supporting_evidence: refs,
        conflicting_evidence: Vec::new(),
        assumptions: scenario.assumptions.clone(),
        limitations: vec![
            "Predictions are based on known system state — runtime surprises possible".into(),
        ],
        confidence_reason: format!(
            "{}% — based on {} confidence signals",
            scenario.confidence.percentage(),
            5
        ),
        dependencies: vec![
            "Capability Engine",
            "Compatibility Engine",
            "Decision Engine",
            "Prediction Engine",
        ],
    }
}

// ============================================================================
//  System Narrative
// ============================================================================

fn build_narrative(
    evidence: &[Evidence],
    compatibility: &Compatibility,
    decision_plan: &DecisionPlan,
) -> String {
    let release = evidence_helpers::ev_s(evidence, "kernel.release");
    let arch = evidence_helpers::ev_s(evidence, "kernel.arch");
    let mut narrative = String::new();

    if release != "Unknown" {
        narrative.push_str(&format!(
            "System runs Linux {release} on {arch}. ",
            release = release,
            arch = arch
        ));
    }

    narrative.push_str(&format!(
        "Kernel development readiness is {} ({}%). ",
        compatibility.level, compatibility.score
    ));

    if !compatibility.blocking_issues.is_empty() {
        narrative.push_str(&format!(
            "{} blocking issue(s) detected. ",
            compatibility.blocking_issues.len()
        ));
    }

    if let Some(action) = &decision_plan.highest_roi_action {
        narrative.push_str(&format!(
            "Highest priority action: {}. Estimated fix time: ~{} min.",
            action.title, decision_plan.estimated_total_fix_minutes
        ));
    } else {
        narrative.push_str("No actions required — system is ready.");
    }

    narrative
}

fn count_probes(evidence: &[Evidence]) -> usize {
    evidence.len()
}

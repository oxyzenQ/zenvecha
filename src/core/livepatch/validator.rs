// Copyright (C) 2026 rezky_nightky
// SPDX-License-Identifier: GPL-3.0-only

//! Livepatch validator — safety consensus engine.
//!
//! Consumes CapabilityGraph + SemanticDescriptors to determine if
//! a livepatch can be safely applied. Pure validation — no execution.
//!
//! Validation gates (all must pass):
//!   1. Capability check — livepatch capability exists in graph
//!   2. Dependency check — all required dependencies available
//!   3. Safety constraint — semantic states satisfy policy
//!   4. Forbidden state — no forbidden semantic state present

use crate::core::caps::kernel_cap::graph::CapabilityGraph;
use crate::core::semantic::model::SemanticDescriptor;

use super::model::{
    LivepatchSafetyPolicy, RejectionCategory, RejectionReason, ValidationCheck, ValidationContext,
    ValidationVerdict,
};

/// Validate that a livepatch can be safely applied.
///
/// Returns a ValidationContext with detailed per-check results.
/// The engine uses `verdict` to decide: Approved → proceed; Rejected → abort.
pub fn validate(graph: &CapabilityGraph, semantic: &[SemanticDescriptor]) -> ValidationContext {
    let mut graph_checks = Vec::new();
    let mut semantic_checks = Vec::new();

    // ── Gate 1: Required capabilities must exist ──
    for cap_id in LivepatchSafetyPolicy::required_capabilities() {
        let node = graph.node(cap_id);
        graph_checks.push(ValidationCheck {
            check_name: format!("capability: {cap_id}"),
            passed: node.is_some(),
            dependency_kind: None,
            semantic_domain: None,
            expected: "present in graph".into(),
            actual: if node.is_some() {
                "found".into()
            } else {
                "missing".into()
            },
        });
    }

    // ── Gate 2: Dependency chain must be resolvable ──
    if let Some(livepatch_node) = graph.node("security.livepatch") {
        for dep in &livepatch_node.depends_on {
            let dep_node = graph.node(dep.target_id);
            graph_checks.push(ValidationCheck {
                check_name: format!("dependency: {} → {}", livepatch_node.id, dep.target_id),
                passed: dep_node.is_some(),
                dependency_kind: Some(dep.kind),
                semantic_domain: None,
                expected: format!("{} present", dep.target_id),
                actual: if dep_node.is_some() {
                    "found".into()
                } else {
                    "missing".into()
                },
            });
        }
    }

    // ── Gate 3: Required semantic states ──
    for (domain, expected_state) in LivepatchSafetyPolicy::required_semantic_states() {
        let actual = semantic
            .iter()
            .find(|d| d.domain == *domain)
            .map(|d| &d.state);

        let passed = actual == Some(expected_state);
        semantic_checks.push(ValidationCheck {
            check_name: format!("semantic: {}.state", domain.label()),
            passed,
            dependency_kind: None,
            semantic_domain: Some(*domain),
            expected: expected_state.label().to_string(),
            actual: actual
                .map(|s| s.label().to_string())
                .unwrap_or_else(|| "unknown".into()),
        });
    }

    // ── Gate 4: Forbidden semantic states ──
    for (domain, forbidden) in LivepatchSafetyPolicy::forbidden_semantic_states() {
        let actual = semantic
            .iter()
            .find(|d| d.domain == *domain)
            .map(|d| &d.state);

        let passed = actual != Some(forbidden);
        semantic_checks.push(ValidationCheck {
            check_name: format!(
                "forbidden: {}.state != {}",
                domain.label(),
                forbidden.label()
            ),
            passed,
            dependency_kind: None,
            semantic_domain: Some(*domain),
            expected: format!("not {}", forbidden.label()),
            actual: actual
                .map(|s| s.label().to_string())
                .unwrap_or_else(|| "unknown".into()),
        });
    }

    // ── Verdict ──
    let all_passed =
        graph_checks.iter().all(|c| c.passed) && semantic_checks.iter().all(|c| c.passed);

    let verdict = if all_passed {
        ValidationVerdict::Approved
    } else {
        ValidationVerdict::Rejected
    };

    ValidationContext {
        graph_checks,
        semantic_checks,
        verdict,
    }
}

/// Build a structured rejection reason from a failed validation.
pub fn build_rejection(ctx: &ValidationContext) -> Option<RejectionReason> {
    if ctx.verdict != ValidationVerdict::Rejected {
        return None;
    }

    // Find the first failed check
    let failed = ctx
        .graph_checks
        .iter()
        .chain(ctx.semantic_checks.iter())
        .find(|c| !c.passed)?;

    let category = if failed.check_name.starts_with("capability:") {
        RejectionCategory::CapabilityMissing
    } else if failed.check_name.starts_with("dependency:") {
        RejectionCategory::DependencyUnavailable
    } else {
        RejectionCategory::SafetyConstraint
    };

    let resolution = match category {
        RejectionCategory::CapabilityMissing => {
            format!("Enable {} in kernel config and rebuild", failed.check_name)
        }
        RejectionCategory::DependencyUnavailable => {
            "Enable required dependency before attempting livepatch".into()
        }
        RejectionCategory::SafetyConstraint => {
            format!(
                "System state must be {} (currently: {})",
                failed.expected, failed.actual
            )
        }
        _ => "Cannot resolve automatically".into(),
    };

    Some(RejectionReason {
        category,
        failed_check: failed.check_name.clone(),
        detail: format!("Expected '{}' but got '{}'", failed.expected, failed.actual),
        resolution,
    })
}

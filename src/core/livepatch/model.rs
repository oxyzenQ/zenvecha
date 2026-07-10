// Copyright (C) 2026 rezky_nightky
// SPDX-License-Identifier: GPL-3.0-only

//! Livepatch domain models — safe execution without reboot.
//!
//! Describes WHAT to patch, HOW it was validated, and WHAT happened.
//! No execution logic — pure domain types.

use crate::core::caps::kernel_cap::graph::DependencyKind;
use crate::core::semantic::model::{SemanticDomain, SemanticState};

// ============================================================================
//  Livepatch Request
// ============================================================================

/// A patch request from userspace.
///
/// Userspace (Decision Engine) decides WHICH function to patch.
/// The Livepatch Engine validates and executes.
#[derive(Clone, Debug)]
pub struct LivepatchRequest {
    /// Symbol name of the function to patch (e.g. "sys_read").
    pub symbol_name: String,
    /// Memory address of the target function.
    pub target_address: usize,
    /// New code or function pointer to redirect to.
    pub new_address: usize,
    /// Human-readable description of the patch purpose.
    pub description: String,
    /// Whether this is a trial run (validate only, don't apply).
    pub dry_run: bool,
}

// ============================================================================
//  Livepatch Result
// ============================================================================

/// Outcome of a livepatch operation.
#[derive(Clone, Debug)]
pub struct LivepatchResult {
    /// Whether the patch was accepted and applied.
    pub applied: bool,
    /// If rejected, structured reason.
    pub rejection: Option<RejectionReason>,
    /// Post-patch verification status.
    pub verification: Option<VerificationResult>,
    /// Timestamp of the operation (seconds since module load).
    pub timestamp_secs: u64,
}

/// Structured rejection — never a generic error string.
#[derive(Clone, Debug)]
pub struct RejectionReason {
    /// Category of rejection.
    pub category: RejectionCategory,
    /// Which specific check failed.
    pub failed_check: String,
    /// Human-readable explanation.
    pub detail: String,
    /// What the user must do to resolve this.
    pub resolution: String,
}

/// Why a patch was rejected.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RejectionCategory {
    /// Missing kernel capability (e.g., CONFIG_LIVEPATCH=n).
    CapabilityMissing,
    /// Capability exists but a required dependency is unavailable.
    DependencyUnavailable,
    /// Semantic safety check failed (e.g., runtime risk too high).
    SafetyConstraint,
    /// The target symbol doesn't exist or can't be patched.
    InvalidTarget,
    /// The kernel module reported an error during application.
    ExecutionFailed,
    /// Post-patch verification showed the patch didn't take effect.
    VerificationFailed,
}

impl RejectionCategory {
    pub fn label(self) -> &'static str {
        match self {
            RejectionCategory::CapabilityMissing => "capability_missing",
            RejectionCategory::DependencyUnavailable => "dependency_unavailable",
            RejectionCategory::SafetyConstraint => "safety_constraint",
            RejectionCategory::InvalidTarget => "invalid_target",
            RejectionCategory::ExecutionFailed => "execution_failed",
            RejectionCategory::VerificationFailed => "verification_failed",
        }
    }
}

// ============================================================================
//  Verification Result
// ============================================================================

/// Post-patch verification.
#[derive(Clone, Debug)]
pub struct VerificationResult {
    /// Whether the patch is confirmed active.
    pub confirmed: bool,
    /// The old function address (before patch).
    pub old_address: usize,
    /// The new function address (after patch).
    pub new_address: usize,
    /// Whether the redirect was observed (ftrace confirms redirection).
    pub redirect_observed: bool,
}

// ============================================================================
//  Validation Context
// ============================================================================

/// All checks that were performed and their outcomes.
#[derive(Clone, Debug)]
pub struct ValidationContext {
    /// Capability graph checks performed.
    pub graph_checks: Vec<ValidationCheck>,
    /// Semantic descriptor checks performed.
    pub semantic_checks: Vec<ValidationCheck>,
    /// Overall verdict.
    pub verdict: ValidationVerdict,
}

#[derive(Clone, Debug)]
pub struct ValidationCheck {
    /// What was checked.
    pub check_name: String,
    /// Whether it passed.
    pub passed: bool,
    /// Dependency kind if this was a graph check.
    pub dependency_kind: Option<DependencyKind>,
    /// Semantic domain if this was a semantic check.
    pub semantic_domain: Option<SemanticDomain>,
    /// Expected state.
    pub expected: String,
    /// Actual state.
    pub actual: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ValidationVerdict {
    /// All checks passed — safe to apply.
    Approved,
    /// One or more checks failed — DO NOT apply.
    Rejected,
    /// All checks passed but one or more are warnings.
    ApprovedWithWarnings,
}

impl ValidationVerdict {
    pub fn label(self) -> &'static str {
        match self {
            ValidationVerdict::Approved => "approved",
            ValidationVerdict::Rejected => "rejected",
            ValidationVerdict::ApprovedWithWarnings => "approved_with_warnings",
        }
    }
}

// ============================================================================
//  Safety Policy — the minimum bar for livepatch
// ============================================================================

/// Pre-defined safety constraints for livepatch execution.
///
/// These are the gates that MUST pass before any patch is applied.
/// They consume only CapabilityGraph + SemanticDescriptors.
pub struct LivepatchSafetyPolicy;

impl LivepatchSafetyPolicy {
    /// Required capability IDs that must exist in the graph.
    pub fn required_capabilities() -> &'static [&'static str] {
        &[
            "config.LIVEPATCH",
            "config.MODULES",
            "tracing.ftrace",
            "security.livepatch",
        ]
    }

    /// Semantic constraints that must be satisfied.
    pub fn required_semantic_states() -> &'static [(SemanticDomain, SemanticState)] {
        &[
            // Runtime risk must be Low — never patch an unstable system.
            (SemanticDomain::RuntimeRisk, SemanticState::RuntimeRiskLow),
            // Stability tier must NOT be Unstable.
            // (checked in validator as "must not be Unstable")
        ]
    }

    /// Semantic states that are FORBIDDEN.
    pub fn forbidden_semantic_states() -> &'static [(SemanticDomain, SemanticState)] {
        &[
            (
                SemanticDomain::RuntimeRisk,
                SemanticState::RuntimeRiskCritical,
            ),
            (
                SemanticDomain::StabilityTier,
                SemanticState::StabilityUnstable,
            ),
        ]
    }
}

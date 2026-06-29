// Copyright (C) 2026 rezky_nightky
// SPDX-License-Identifier: GPL-3.0-only

//! Livepatch execution engine — safe, atomic, no-reboot patching.
//!
//! Flow:
//!   1. Validate (CapabilityGraph + SemanticDescriptors)
//!   2. If rejected → return structured RejectionReason
//!   3. If approved → send to kernel module → atomic apply
//!   4. Verify post-patch state
//!
//! This engine NEVER decides WHAT to patch — Decision Engine does that.
//! This engine only validates safety and executes.

use crate::core::caps::kernel_cap::graph::CapabilityGraph;
use crate::core::semantic::model::SemanticDescriptor;

use super::model::{
    LivepatchRequest, LivepatchResult, RejectionCategory, RejectionReason, VerificationResult,
};
use super::validator;

/// Execute a livepatch with full safety validation.
///
/// This is the single entry point for all livepatch operations.
/// Userspace calls this with a request; the engine validates,
/// executes, and verifies.
pub fn execute(
    request: &LivepatchRequest,
    graph: &CapabilityGraph,
    semantic: &[SemanticDescriptor],
) -> LivepatchResult {
    // ── Phase 1: Validation ──
    let ctx = validator::validate(graph, semantic);

    if let Some(rejection) = validator::build_rejection(&ctx) {
        return LivepatchResult {
            applied: false,
            rejection: Some(rejection),
            verification: None,
            timestamp_secs: 0,
        };
    }

    // ── Phase 2: Dry-run check ──
    if request.dry_run {
        return LivepatchResult {
            applied: false,
            rejection: None,
            verification: None,
            timestamp_secs: 0,
        };
    }

    // ── Phase 3: Send to kernel module ──
    //
    // In a real system, this writes the patch payload to
    // /proc/zenvecha/livepatch/apply and reads the result.
    //
    // Kernel-side (kernel/livepatch/executor.rs):
    //   1. Validate symbol exists in kallsyms
    //   2. Take text_mutex (ensures no concurrent code modification)
    //   3. stop_machine() — halt all CPUs
    //   4. Install ftrace handler at target address
    //   5. Resume CPUs
    //   6. Report success/failure to userspace
    //
    // For now: simulate the kernel interface.
    let kernel_result = apply_via_kernel(request);

    // ── Phase 4: Verification ──
    let verification = if kernel_result.applied {
        verify_patch(request)
    } else {
        None
    };

    LivepatchResult {
        applied: kernel_result.applied,
        rejection: kernel_result.rejection,
        verification,
        timestamp_secs: 0,
    }
}

/// Send patch payload to kernel module via /proc/zenvecha/livepatch/.
///
/// In production, this writes a structured payload and reads structured
/// response. The kernel module performs:
///   1. Symbol lookup via kallsyms
///   2. Atomic code modification via ftrace + stop_machine
///   3. Returns success/error code
fn apply_via_kernel(request: &LivepatchRequest) -> LivepatchResult {
    // Check if kernel module is loaded
    if !std::path::Path::new("/proc/zenvecha").is_dir() {
        return LivepatchResult {
            applied: false,
            rejection: Some(RejectionReason {
                category: RejectionCategory::ExecutionFailed,
                failed_check: "kernel.module_loaded".into(),
                detail: "Zenvecha kernel module is not loaded".into(),
                resolution: "Load the Zenvecha kernel module first".into(),
            }),
            verification: None,
            timestamp_secs: 0,
        };
    }

    // Try writing payload to /proc/zenvecha/livepatch/apply
    let payload = format!(
        "symbol={}\ntarget=0x{:x}\nnew=0x{:x}\ndesc={}\n",
        request.symbol_name, request.target_address, request.new_address, request.description
    );

    let result = std::fs::write("/proc/zenvecha/livepatch/apply", payload.as_bytes());

    match result {
        Ok(()) => {
            // Read back the result
            let status =
                std::fs::read_to_string("/proc/zenvecha/livepatch/status").unwrap_or_default();
            let applied = status.trim() == "applied";

            if applied {
                LivepatchResult {
                    applied: true,
                    rejection: None,
                    verification: None,
                    timestamp_secs: 0,
                }
            } else {
                LivepatchResult {
                    applied: false,
                    rejection: Some(RejectionReason {
                        category: RejectionCategory::ExecutionFailed,
                        failed_check: "kernel.livepatch.status".into(),
                        detail: format!("Kernel module returned: {status}"),
                        resolution: "Check kernel log (dmesg) for detailed error".into(),
                    }),
                    verification: None,
                    timestamp_secs: 0,
                }
            }
        }
        Err(e) => LivepatchResult {
            applied: false,
            rejection: Some(RejectionReason {
                category: RejectionCategory::ExecutionFailed,
                failed_check: "proc.livepatch.write".into(),
                detail: format!("Failed to write to kernel module: {e}"),
                resolution: "Ensure kernel module is loaded and proc interface is writable".into(),
            }),
            verification: None,
            timestamp_secs: 0,
        },
    }
}

/// Verify the patch is active by reading /proc/zenvecha/livepatch/verify.
fn verify_patch(request: &LivepatchRequest) -> Option<VerificationResult> {
    let status = std::fs::read_to_string("/proc/zenvecha/livepatch/verify")
        .ok()
        .unwrap_or_default();

    let confirmed = status.contains("verified");
    let redirect = status.contains("redirect_observed");

    Some(VerificationResult {
        confirmed,
        old_address: request.target_address,
        new_address: request.new_address,
        redirect_observed: redirect,
    })
}

// ============================================================================
//  Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::semantic::model::{SemanticDomain, SemanticState};

    #[test]
    fn test_validate_rejects_when_livepatch_missing() {
        let graph = CapabilityGraph::known();
        // Semantic: runtime risk is Critical → should reject
        let semantic = vec![
            SemanticDescriptor {
                domain: SemanticDomain::RuntimeRisk,
                state: SemanticState::RuntimeRiskCritical,
                source_evidence: vec![],
                rationale: "test",
            },
            SemanticDescriptor {
                domain: SemanticDomain::StabilityTier,
                state: SemanticState::StabilityUnstable,
                source_evidence: vec![],
                rationale: "test",
            },
        ];

        let request = LivepatchRequest {
            symbol_name: "test_fn".into(),
            target_address: 0x1000,
            new_address: 0x2000,
            description: "test".into(),
            dry_run: true,
        };

        let result = execute(&request, &graph, &semantic);
        assert!(!result.applied, "should reject with critical risk");
        assert!(result.rejection.is_some(), "should have rejection reason");
    }

    #[test]
    fn test_validate_approves_when_safe() {
        let graph = CapabilityGraph::known();
        let semantic = vec![
            SemanticDescriptor {
                domain: SemanticDomain::RuntimeRisk,
                state: SemanticState::RuntimeRiskLow,
                source_evidence: vec![],
                rationale: "test",
            },
            SemanticDescriptor {
                domain: SemanticDomain::StabilityTier,
                state: SemanticState::StabilityProduction,
                source_evidence: vec![],
                rationale: "test",
            },
        ];

        let request = LivepatchRequest {
            symbol_name: "test_fn".into(),
            target_address: 0x1000,
            new_address: 0x2000,
            description: "test".into(),
            dry_run: true,
        };

        // Dry-run with low risk + production stability should pass validation
        // (module may not be loaded in test, but dry_run exits before kernel call)
        let result = execute(&request, &graph, &semantic);
        assert!(!result.applied, "dry run should not apply");
        assert!(
            result.rejection.is_none(),
            "dry run should not reject on safe system"
        );
    }
}

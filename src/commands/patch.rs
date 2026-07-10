// Copyright (C) 2026 rezky_nightky
// SPDX-License-Identifier: GPL-3.0-only

//! Livepatch CLI command — thin orchestration.
//!
//! Commands:
//!   zenvecha patch apply   — safely apply a livepatch (validate→execute→verify)
//!   zenvecha patch revert  — rollback the last applied patch
//!   zenvecha patch status  — show current patch state
//!   zenvecha patch dry-run — validate only, don't apply
//!
//! Orchestration only. Delegates to LivepatchEngine.

use std::io::{self, Write};

use crate::core::caps::kernel_cap::graph::CapabilityGraph;
use crate::core::livepatch::engine;
use crate::core::livepatch::model::LivepatchRequest;
use crate::core::pipeline;

pub fn run(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let sub = args.get(1).map(|s| s.as_str()).unwrap_or("status");

    match sub {
        "apply" => cmd_apply(),
        "revert" => cmd_revert(),
        "dry-run" => cmd_dry_run(),
        "status" => cmd_status(),
        _ => {
            writeln!(io::stderr(), "zenvecha patch: unknown subcommand '{sub}'")?;
            writeln!(
                io::stderr(),
                "Usage: zenvecha patch [apply|revert|dry-run|status]"
            )?;
            Ok(())
        }
    }
}

/// Apply a livepatch to the dummy target function.
///
/// This is the end-to-end path:
///   1. Collect Evidence + Semantic state
///   2. Build the LivepatchRequest
///   3. Validate against CapabilityGraph + Semantic
///   4. Execute via kernel module
///   5. Display result
fn cmd_apply() -> Result<(), Box<dyn std::error::Error>> {
    let result = pipeline::run_analysis_pipeline();
    let graph = CapabilityGraph::known();
    let stdout = io::stdout();
    let mut out = stdout.lock();

    writeln!(out, "══ Zenvecha Livepatch — Safe Execution ══")?;
    writeln!(out)?;

    // Show pre-patch semantic state
    writeln!(out, "Pre-Patch System State:")?;
    for d in &result.semantic_descriptors {
        writeln!(out, "  {}: {}", d.domain.label(), d.state.label())?;
    }
    writeln!(out)?;

    // Build request — target the dummy function
    let request = LivepatchRequest {
        symbol_name: "zenvecha_dummy_func".into(),
        target_address: 0x0, // kernel resolves via kallsyms
        new_address: 0x0,    // kernel resolves via kallsyms
        description: "Zenvecha PoC: patch dummy_func (42 → 99)".into(),
        dry_run: false,
    };

    writeln!(out, "Target: {}", request.symbol_name)?;
    writeln!(out, "Payload: return 42 → return 99")?;
    writeln!(out)?;

    // Execute (validate + apply + verify)
    let patch_result = engine::execute(&request, &graph, &result.semantic_descriptors);

    if patch_result.applied {
        writeln!(out, "✅ PATCH APPLIED SUCCESSFULLY")?;
        writeln!(out)?;
        if let Some(ref verif) = patch_result.verification {
            writeln!(out, "Verification:")?;
            writeln!(out, "  Confirmed: {}", verif.confirmed)?;
            writeln!(out, "  Redirect observed: {}", verif.redirect_observed)?;
            writeln!(
                out,
                "  Old addr: 0x{:x} → New addr: 0x{:x}",
                verif.old_address, verif.new_address
            )?;
        }
        writeln!(out)?;
        writeln!(
            out,
            "The function 'zenvecha_dummy_func()' now returns 99 instead of 42."
        )?;
        writeln!(
            out,
            "No reboot required. All CPUs redirected via ftrace + stop_machine()."
        )?;
    } else if let Some(ref rejection) = patch_result.rejection {
        writeln!(out, "❌ PATCH REJECTED")?;
        writeln!(out)?;
        writeln!(out, "Category  : {}", rejection.category.label())?;
        writeln!(out, "Failed at : {}", rejection.failed_check)?;
        writeln!(out, "Detail    : {}", rejection.detail)?;
        writeln!(out, "Resolution: {}", rejection.resolution)?;
    } else {
        writeln!(out, "Patch was not applied (dry-run or module not loaded).")?;
        writeln!(
            out,
            "Tip: run 'zenvecha patch dry-run' to validate without applying."
        )?;
    }

    Ok(())
}

/// Validate the livepatch setup without applying.
fn cmd_dry_run() -> Result<(), Box<dyn std::error::Error>> {
    let result = pipeline::run_analysis_pipeline();
    let graph = CapabilityGraph::known();
    let stdout = io::stdout();
    let mut out = stdout.lock();

    writeln!(out, "══ Zenvecha Livepatch — Dry Run ══")?;
    writeln!(out)?;

    // Show validation gates
    let ctx = crate::core::livepatch::validator::validate(&graph, &result.semantic_descriptors);

    writeln!(
        out,
        "Validation Gates ({} total):",
        ctx.graph_checks.len() + ctx.semantic_checks.len()
    )?;
    writeln!(out)?;

    writeln!(out, "Capability Checks:")?;
    for check in &ctx.graph_checks {
        let icon = if check.passed { "✅" } else { "❌" };
        writeln!(out, "  {icon} {}: {}", check.check_name, check.actual)?;
    }
    writeln!(out)?;

    writeln!(out, "Semantic Checks:")?;
    for check in &ctx.semantic_checks {
        let icon = if check.passed { "✅" } else { "❌" };
        writeln!(
            out,
            "  {icon} {}: expected '{}' → actual '{}'",
            check.check_name, check.expected, check.actual
        )?;
    }
    writeln!(out)?;

    writeln!(out, "Verdict: {}", ctx.verdict.label())?;
    if ctx.verdict == crate::core::livepatch::model::ValidationVerdict::Approved {
        writeln!(out, "All gates passed. Safe to run 'zenvecha patch apply'.")?;
    } else {
        writeln!(
            out,
            "One or more gates failed. Resolve issues before applying."
        )?;
    }

    Ok(())
}

/// Revert the last applied patch.
fn cmd_revert() -> Result<(), Box<dyn std::error::Error>> {
    let stdout = io::stdout();
    let mut out = stdout.lock();

    writeln!(out, "══ Zenvecha Livepatch — Revert ══")?;
    writeln!(out)?;

    // Write revert command to /proc/zenvecha/livepatch/revert
    let module_loaded = std::path::Path::new("/proc/zenvecha").is_dir();

    if !module_loaded {
        writeln!(out, "❌ Zenvecha kernel module is not loaded.")?;
        writeln!(out, "Cannot revert patch without the kernel module.")?;
        return Ok(());
    }

    match std::fs::write("/proc/zenvecha/livepatch/revert", "revert\n") {
        Ok(()) => {
            let status =
                std::fs::read_to_string("/proc/zenvecha/livepatch/status").unwrap_or_default();
            writeln!(out, "Revert requested. Status: {}", status.trim())?;
            writeln!(out)?;
            writeln!(
                out,
                "The function 'zenvecha_dummy_func()' should now return 42 again."
            )?;
        }
        Err(e) => {
            writeln!(out, "❌ Failed to revert: {e}")?;
        }
    }

    Ok(())
}

/// Show current livepatch status.
fn cmd_status() -> Result<(), Box<dyn std::error::Error>> {
    let stdout = io::stdout();
    let mut out = stdout.lock();

    writeln!(out, "══ Zenvecha Livepatch — Status ══")?;
    writeln!(out)?;

    let module_loaded = std::path::Path::new("/proc/zenvecha").is_dir();
    if !module_loaded {
        writeln!(out, "Kernel module: NOT LOADED")?;
        writeln!(out, "Load the Zenvecha kernel module to enable livepatch.")?;
        return Ok(());
    }

    writeln!(out, "Kernel module: LOADED ✅")?;

    let status = std::fs::read_to_string("/proc/zenvecha/livepatch/status")
        .ok()
        .unwrap_or_else(|| "unknown".into());
    writeln!(out, "Patch status : {}", status.trim())?;

    let verify = std::fs::read_to_string("/proc/zenvecha/livepatch/verify")
        .ok()
        .unwrap_or_else(|| "no verification data".into());
    writeln!(out, "Verification : {}", verify.trim())?;

    // Show semantic context
    let result = pipeline::run_analysis_pipeline();
    writeln!(out)?;
    writeln!(out, "System Semantic State:")?;
    for d in &result.semantic_descriptors {
        let icon = match d.state {
            crate::core::semantic::model::SemanticState::RuntimeRiskLow => "🟢",
            crate::core::semantic::model::SemanticState::RuntimeRiskMedium => "🟡",
            crate::core::semantic::model::SemanticState::RuntimeRiskHigh => "🟠",
            crate::core::semantic::model::SemanticState::RuntimeRiskCritical => "🔴",
            _ => "⚪",
        };
        writeln!(out, "  {icon} {}: {}", d.domain.label(), d.state.label())?;
    }

    Ok(())
}

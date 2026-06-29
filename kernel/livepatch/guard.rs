// SPDX-License-Identifier: GPL-2.0-only
// Copyright (C) 2026 rezky_nightky

//! Kernel livepatch safety guards — prevent kernel panics.
//!
//! Every operation that touches kernel execution flow must pass
//! through a guard. Guards check preconditions, wrap dangerous
//! operations in error-handling, and return structured errors.
//!
//! ## Safety Principles
//!
//!   1. Check BEFORE acting — never act and then discover failure
//!   2. Return errors, never panic — userspace handles rejection
//!   3. Isolate dangerous code — stop_machine lives in a guarded scope
//!   4. Recover gracefully — any unexpected state → structured error
//!
//! ## Guard Lifecycle
//!
//!   Module Init:
//!     preflight_checks() → Ok(()) or refuse to load
//!
//!   Patch Request:
//!     validate_target() → verify symbol + address
//!     guarded_stop_machine() → atomic apply with error capture
//!
//!   Module Unload:
//!     require_no_active_patches() → refuse unload if patches active

#![allow(dead_code)] // Architectural skeleton — compiled inside real kernel tree

use alloc::string::String;
use alloc::vec::Vec;

// ============================================================================
//  Pre-flight Guard — runs at module init
// ============================================================================

/// Result of a pre-flight check.
#[derive(Clone, Debug)]
pub struct PreflightResult {
    /// All checks passed.
    pub ok: bool,
    /// List of failures (empty if ok).
    pub failures: Vec<PreflightFailure>,
}

#[derive(Clone, Debug)]
pub struct PreflightFailure {
    /// Which check failed.
    pub check: &'static str,
    /// Why it failed.
    pub reason: String,
    /// Is this fatal? (true = module must abort load)
    pub fatal: bool,
}

/// Run all pre-flight checks at module init.
///
/// If ANY fatal check fails, the module MUST refuse to load.
/// Userspace receives the structured failure reason via dmesg.
pub fn preflight_checks() -> PreflightResult {
    let mut failures = Vec::new();

    // ── Check 1: CONFIG_LIVEPATCH ──
    // In real kernel code:
    //   if !cfg!(CONFIG_LIVEPATCH) {
    //       failures.push(PreflightFailure {
    //           check: "CONFIG_LIVEPATCH",
    //           reason: "Livepatch support not compiled into this kernel".into(),
    //           fatal: true,
    //       });
    //   }
    check_config("LIVEPATCH", true, &mut failures);

    // ── Check 2: CONFIG_FUNCTION_TRACER ──
    check_config("FUNCTION_TRACER", true, &mut failures);

    // ── Check 3: CONFIG_MODULES ──
    check_config("MODULES", true, &mut failures);

    // ── Check 4: Running in safe environment ──
    // Some container runtimes restrict kernel code modification.
    // Check /proc/1/environ for container indicators.
    check_container_safety(&mut failures);

    // ── Check 5: Kernel lockdown ──
    // If kernel lockdown is in 'confidentiality' mode, livepatch is blocked.
    // Read /sys/kernel/security/lockdown — if "confidentiality", refuse.
    check_lockdown_mode(&mut failures);

    PreflightResult {
        ok: failures.iter().all(|f| !f.fatal),
        failures,
    }
}

fn check_config(name: &str, required: bool, failures: &mut Vec<PreflightFailure>) {
    // In real kernel code: IS_ENABLED(CONFIG_{name})
    let enabled = true; // stub — real code checks kernel config
    if required && !enabled {
        failures.push(PreflightFailure {
            check: "CONFIG_LIVEPATCH",
            reason: format!("CONFIG_{name} is not enabled in this kernel"),
            fatal: true,
        });
    }
}

fn check_container_safety(failures: &mut Vec<PreflightFailure>) {
    // In real kernel code:
    //   - Check if init process is systemd-nspawn or docker-init
    //   - Check cgroup namespace for container indicators
    //   - Check seccomp profile for restricted syscalls
    //
    // If running in a restricted container that blocks kernel code
    // modification, refuse to load.
    //
    // This is a WARNING, not fatal — containers with CAP_SYS_MODULE
    // can still use livepatch.

    // Stub: assume safe
    let _ = failures;
}

fn check_lockdown_mode(failures: &mut Vec<PreflightFailure>) {
    // In real kernel code, read the lockdown LSM state:
    //
    //   let lockdown = security_locked_down(LOCKDOWN_KERNEL_MODULE);
    //   if lockdown == LOCKDOWN_CONFIDENTIALITY_MAX {
    //       failures.push(PreflightFailure {
    //           check: "kernel_lockdown",
    //           reason: "Kernel lockdown is in confidentiality mode".into(),
    //           fatal: true,
    //       });
    //   } else if lockdown == LOCKDOWN_INTEGRITY_MAX {
    //       failures.push(PreflightFailure {
    //           check: "kernel_lockdown",
    //           reason: "Kernel lockdown is in integrity mode — some patches may be restricted".into(),
    //           fatal: false, // non-fatal warning
    //       });
    //   }

    let _ = failures;
}

// ============================================================================
//  Target Validation Guard — runs before every patch
// ============================================================================

/// Validate a patch target before attempting to apply.
pub fn validate_target(symbol_name: &str, _target_address: usize) -> Result<(), String> {
    // In real kernel code:
    //
    //   let addr = kallsyms_lookup_name(symbol_name);
    //   if addr.is_null() {
    //       return Err(format!("Symbol '{}' not found in kallsyms", symbol_name));
    //   }
    //
    //   // Check symbol is in .text section (only code can be livepatched)
    //   if !is_text_symbol(addr) {
    //       return Err(format!("Symbol '{}' is not in .text section", symbol_name));
    //   }
    //
    //   // Check symbol is not already patched
    //   if is_currently_patched(addr) {
    //       return Err(format!("Symbol '{}' is already patched", symbol_name));
    //   }
    //
    //   // Check symbol is not blacklisted (NOKPROBE, NOTRACE)
    //   if is_blacklisted(addr) {
    //       return Err(format!("Symbol '{}' is marked NOTRACE/NOKPROBE", symbol_name));
    //   }

    let _ = symbol_name;
    Ok(())
}

// ============================================================================
//  Atomic Execution Guard — wraps stop_machine safely
// ============================================================================

/// Result of a guarded atomic operation.
pub struct GuardedExecResult {
    pub success: bool,
    pub error: Option<String>,
    /// If the pre-patch state was saved, it's here for rollback.
    pub saved_state: Option<SavedPatchState>,
}

/// Saved state before a patch — enables safe rollback.
pub struct SavedPatchState {
    pub symbol_name: String,
    pub old_address: usize,
    pub old_handler: usize,
}

/// Execute a guarded stop_machine operation.
///
/// This is the CRITICAL SECTION of livepatch. Every failure path
/// must be handled — the kernel must NEVER oops here.
///
/// Safety contract:
///   1. text_mutex MUST be acquired before calling this
///   2. stop_machine MUST complete (no partial application)
///   3. On ANY error, restore original state
///   4. Always release text_mutex (even on panic path)
pub fn guarded_stop_machine(
    symbol_name: &str,
    old_address: usize,
    new_address: usize,
) -> GuardedExecResult {
    // In real kernel code:
    //
    //   // Step 0: Save state for rollback
    //   let saved = SavedPatchState {
    //       symbol_name: symbol_name.into(),
    //       old_address,
    //       old_handler: read_ftrace_handler(old_address),
    //   };
    //
    //   // Step 1: Acquire text_mutex
    //   let mutex_result = mutex_lock_interruptible(&text_mutex);
    //   if mutex_result != 0 {
    //       return GuardedExecResult {
    //           success: false,
    //           error: Some("Failed to acquire text_mutex".into()),
    //           saved_state: None,
    //       };
    //   }
    //
    //   // Step 2: stop_machine with error capture
    //   //         The closure runs on a single CPU while all others are halted.
    //   let result = stop_machine(|_data| {
    //       // SAFETY: all other CPUs are halted — safe to modify code.
    //       //
    //       // Try to install ftrace handler. If this fails, we MUST
    //       // report the error — the stop_machine callback returns a
    //       // negative errno on failure.
    //       match install_ftrace_handler(old_address, new_address) {
    //           Ok(()) => 0,
    //           Err(e) => {
    //               pr_err!("zenvecha: ftrace install failed for {}: {}", symbol_name, e);
    //               -EIO
    //           }
    //       }
    //   }, NULL, NULL);
    //
    //   // Step 3: Always release text_mutex (even on failure)
    //   mutex_unlock(&text_mutex);
    //
    //   // Step 4: Interpret result
    //   if result == 0 {
    //       // Verify immediately
    //       let actual = read_ftrace_handler(old_address);
    //       if actual == new_address {
    //           GuardedExecResult {
    //               success: true,
    //               error: None,
    //               saved_state: Some(saved),
    //           }
    //       } else {
    //           // Mismatch — something went wrong silently
    //           GuardedExecResult {
    //               success: false,
    //               error: Some(format!("Handler mismatch: expected {}, got {}", new_address, actual)),
    //               saved_state: Some(saved),
    //           }
    //       }
    //   } else {
    //       GuardedExecResult {
    //           success: false,
    //           error: Some(format!("stop_machine failed with error {}", result)),
    //           saved_state: None,
    //       }
    //   }

    GuardedExecResult {
        success: true,
        error: None,
        saved_state: Some(SavedPatchState {
            symbol_name: symbol_name.into(),
            old_address,
            old_handler: old_address,
        }),
    }
}

// ============================================================================
//  Unload Guard — prevents unsafe module removal
// ============================================================================

/// Check if it's safe to unload the module.
///
/// Returns Ok(()) if no active patches exist.
/// Returns Err with a list of active patches that must be reverted first.
pub fn require_no_active_patches() -> Result<(), Vec<String>> {
    // In real kernel code:
    //   let active = list_active_patches();
    //   if !active.is_empty() {
    //       return Err(active);
    //   }
    Ok(())
}

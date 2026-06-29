// SPDX-License-Identifier: GPL-2.0-only
// Copyright (C) 2026 rezky_nightky

//! Livepatch executor — kernel-side atomic application.
//!
//! This is the ONLY place in the kernel that modifies execution flow.
//! Every patch goes through a mandatory safety protocol:
//!
//!   1. Symbol validation   — target exists in kallsyms
//!   2. Atomicity guard     — text_mutex acquired
//!   3. CPU halt            — stop_machine() ensures no CPU in target
//!   4. Redirect install    — ftrace + livepatch ops applied
//!   5. CPU resume          — all cores now use new code
//!   6. Report              — status written to /proc/zenvecha/livepatch/
//!
//! ## Safety Contract
//!
//! The kernel module NEVER decides WHAT to patch.
//! It ONLY executes patches that userspace has already validated.
//! Userspace owns the decision; kernel owns atomic execution.
//!
//! ## Design
//!
//! This module exposes:
//!   /proc/zenvecha/livepatch/apply   — write patch payload
//!   /proc/zenvecha/livepatch/status  — read last result
//!   /proc/zenvecha/livepatch/verify  — read verification state
//!
//! ## Architecture
//!
//!   kernel/livepatch/
//!     mod.rs      → module declarations
//!     executor.rs → atomic apply protocol

#![allow(dead_code)] // Architectural skeleton — compiled inside real kernel tree

use alloc::string::String;

/// Result of a livepatch application attempt.
pub struct LivepatchExecResult {
    pub success: bool,
    pub symbol_name: String,
    pub old_address: usize,
    pub new_address: usize,
    pub error: Option<String>,
}

/// The livepatch executor.
///
/// In a real kernel module, this would:
///   - Hold a reference to the module's proc directory
///   - Dispatch proc read/write operations
///   - Use kernel::livepatch, kernel::stop_machine, kernel::ftrace
pub struct LivepatchExecutor;

impl LivepatchExecutor {
    /// Apply a livepatch atomically.
    ///
    /// Internal protocol (not visible to userspace):
    ///
    /// ```text
    /// 1. lookup_symbol(target) → old_func
    ///    ├─ Not found → return error
    ///    └─ Found → continue
    ///
    /// 2. mutex_lock(&text_mutex)
    ///    Prevents concurrent code modification
    ///
    /// 3. stop_machine(patch_handler, &payload)
    ///    Halts ALL CPUs except the one running this code
    ///    patch_handler is called on the halted CPU
    ///
    /// 4. patch_handler():
    ///    ├─ ftrace_set_filter(target, new_func)
    ///    ├─ register_ftrace_function(&ops)
    ///    └─ return 0 (success)
    ///
    /// 5. stop_machine completes → all CPUs resume
    ///
    /// 6. mutex_unlock(&text_mutex)
    ///
    /// 7. Verification:
    ///    ├─ Read target address → should equal new_func
    ///    ├─ Check ftrace filter → target should be in filter list
    ///    └─ Report result
    /// ```
    ///
    /// The key safety property: stop_machine() guarantees that NO CPU
    /// is executing the target function during the switch. This is how
    /// live kernel patching works without a reboot.
    pub fn apply(
        symbol_name: &str,
        target_address: usize,
        new_address: usize,
    ) -> LivepatchExecResult {
        // In real kernel code:
        //
        //   // Step 1: Symbol lookup
        //   let symbol = kallsyms_lookup_name(symbol_name);
        //   if symbol.is_null() {
        //       return LivepatchExecResult {
        //           success: false,
        //           error: Some("Symbol not found".into()),
        //           ...
        //       };
        //   }
        //
        //   // Step 2: Lock
        //   mutex_lock(&text_mutex);
        //
        //   // Step 3-4: Atomic switch via stop_machine
        //   let result = stop_machine(patch_fn, &payload, NULL);
        //
        //   // Step 5: Done
        //   mutex_unlock(&text_mutex);
        //
        //   // Step 6: Verify
        //   let confirmed = (read_ptr(target_address) == new_address);
        //
        //   LivepatchExecResult { success: confirmed, ... }

        LivepatchExecResult {
            success: true,
            symbol_name: symbol_name.into(),
            old_address: target_address,
            new_address,
            error: None,
        }
    }

    /// Verify a previously-applied patch is still active.
    pub fn verify(target_address: usize, expected_address: usize) -> bool {
        // In real kernel code:
        //   read_ptr(target_address) == expected_address
        //   && ftrace_filter_contains(target_address)
        let _ = (target_address, expected_address);
        true
    }
}

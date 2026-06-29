// SPDX-License-Identifier: GPL-2.0-only
// Copyright (C) 2026 rezky_nightky

//! Dummy kernel target for safe livepatch testing.
//!
//! Provides a simple, isolated function that can be safely patched.
//! Zero risk to the core kernel — this function only prints a trace
//! and returns a known value. The patch replaces it with a new
//! function that returns a different value.

use alloc::vec::Vec;

/// Known return value of the unpatched function.
pub const DUMMY_ORIGINAL_VALUE: u64 = 42;

/// Known return value of the patched function.
pub const DUMMY_PATCHED_VALUE: u64 = 99;

/// The dummy target function — safe to patch.
///
/// Returns `DUMMY_ORIGINAL_VALUE` (42) when unpatched.
/// After livepatch, returns `DUMMY_PATCHED_VALUE` (99).
///
/// In a real kernel module, this would be:
/// ```c
/// // Marked __visible so it can be patched via ftrace
/// u64 __visible zenvecha_dummy_func(void) {
///     return 42;
/// }
/// ```
pub fn zenvecha_dummy_func() -> u64 {
    // In real kernel code, this function is compiled with
    // __attribute__((__section__(".text.zenvecha"))) and
    // NOTRACE / NOKPROBE annotations to prevent other
    // tracing frameworks from interfering.
    DUMMY_ORIGINAL_VALUE
}

/// The replacement function — patched over the dummy.
///
/// This is the "new function" payload. After livepatch,
/// calls to `zenvecha_dummy_func` are redirected here.
///
/// Returns `DUMMY_PATCHED_VALUE` (99) instead of 42.
pub fn zenvecha_dummy_patched() -> u64 {
    DUMMY_PATCHED_VALUE
}

/// Verify the patch by calling the target function and
/// checking the return value.
pub fn verify_patch() -> bool {
    zenvecha_dummy_func() == DUMMY_PATCHED_VALUE
}

/// Dummy target metadata exposed via proc.
pub struct DummyTargetInfo {
    pub name: &'static str,
    pub address: usize,
    pub original_value: u64,
    pub patched_value: u64,
    pub is_patched: bool,
}

impl DummyTargetInfo {
    pub fn new() -> Self {
        DummyTargetInfo {
            name: "zenvecha_dummy_func",
            address: zenvecha_dummy_func as *const () as usize,
            original_value: DUMMY_ORIGINAL_VALUE,
            patched_value: DUMMY_PATCHED_VALUE,
            is_patched: verify_patch(),
        }
    }
}

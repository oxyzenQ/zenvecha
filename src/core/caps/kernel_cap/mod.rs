// Copyright (C) 2026 rezky_nightky
// SPDX-License-Identifier: GPL-3.0-only

//! Kernel capability collector — userspace bridge.
//!
//! One file per capability domain — mirrors kernel/capability/probes/ structure.
//! Every provider follows the Capability trait contract.
//!
//! ## Provider Contract
//!
//! Every kernel capability provider:
//!   1. Implements `Capability` trait
//!   2. Owns exactly ONE domain (version, symbols, btf, etc.)
//!   3. Returns `Evidence` via `probe()` — never renders, scores, or decides
//!   4. Reads from `/proc/zenvecha/{key}` — gracefully degrades to `missing`
//!   5. Uses the standard proc key namespace: `domain.subdomain.field`
//!
//! ## Adding a Provider
//!
//!   1. Create one new file: `kernel_cap/{domain}.rs`
//!   2. Implement `Capability` for your struct
//!   3. Re-export from this mod.rs
//!   4. Register in `capability::register_all()`
//!   5. Zero modifications to existing providers
//!
//! ## Evidence ID Namespace
//!
//!   kernel.module_loaded     — module status (bool)
//!   kernel.version.module    — kernel release from module (text)
//!   kernel.symbols.module    — symbol count from module (text)
//!   kernel.btf.module        — BTF status from module (bool)
//!   kernel.loader.module     — module loader from module (text)
//!   kernel.tracing.ftrace    — ftrace availability (bool)
//!   kernel.tracing.kprobes   — kprobes availability (bool)

pub mod btf;
pub mod loader;
pub mod module;
pub mod symbols;
pub mod tracing;
pub mod version;

pub use btf::KernelBtfStatus;
pub use loader::KernelModuleLoader;
pub use module::KernelModuleStatus;
pub use symbols::KernelSymbolCount;
pub use tracing::{KernelTracingFtrace, KernelTracingKprobes};
pub use version::KernelVersionFromModule;

// ============================================================================
//  Shared helpers — used by all providers
// ============================================================================

/// Read a single /proc/zenvecha entry. Returns None if module not loaded.
pub(crate) fn read_proc(key: &str) -> Option<String> {
    let path = format!("/proc/zenvecha/{key}");
    std::fs::read_to_string(&path)
        .ok()
        .map(|s| s.trim().to_string())
}

/// Check if the Zenvecha kernel module is loaded.
pub(crate) fn module_loaded() -> bool {
    std::path::Path::new("/proc/zenvecha").is_dir()
}

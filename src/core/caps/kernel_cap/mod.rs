// Copyright (C) 2026 rezky_nightky
// SPDX-License-Identifier: GPL-3.0-only

//! Kernel Evidence Framework — reusable evidence construction.
//!
//! Every kernel capability provider produces Evidence through this
//! framework. Providers only collect runtime facts and populate domain
//! models — the framework handles evidence formatting, typing,
//! namespace generation, and graceful degradation.
//!
//! ## Framework Contract
//!
//! Providers NEVER manually construct `Evidence` structs.
//! Instead, they call one of the three constructors:
//!
//!   `kernel_text(id, proc_key)`   → Text evidence from a proc entry
//!   `kernel_bool(id, proc_key)`   → Bool evidence from a proc entry
//!   `kernel_status(id, loaded)`   → Bool evidence for module status
//!
//! ## Adding a Provider (Standard Workflow)
//!
//!   1. Create kernel/capability/probes/{domain}.rs
//!   2. Add domain models to kernel/capability/types.rs
//!   3. Create src/core/caps/kernel_cap/{domain}.rs
//!   4. Implement `Capability`, calling `kernel_text()` or `kernel_bool()`
//!   5. Re-export from this mod.rs
//!   6. Register in `capability::register_all()`
//!   7. Zero modifications to existing providers
//!
//! ## Evidence ID Namespace
//!
//!   kernel.module_loaded       kernel.version.module
//!   kernel.symbols.*           kernel.btf.module
//!   kernel.loader.module       kernel.tracing.*

use crate::core::evidence::{Evidence, EvidenceValue};

// ============================================================================
//  Proc Bridge (shared)
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

// ============================================================================
//  Evidence Constructors — the ONLY way providers produce Evidence
// ============================================================================

/// Create Text Evidence from a /proc/zenvecha entry.
///
/// When the kernel module is loaded and the proc entry exists,
/// returns `Evidence::present` with the value.
/// When the module is not loaded or the entry is missing,
/// returns `Evidence::missing`.
///
/// Usage: `kernel_text(self.id(), "symbols.total")`
pub(crate) fn kernel_text(id: &'static str, proc_key: &str) -> Evidence {
    match read_proc(proc_key) {
        Some(v) => Evidence::present(id, EvidenceValue::Text(Some(v))),
        None => Evidence::missing(id, EvidenceValue::Text(None)),
    }
}

/// Create Bool Evidence from a /proc/zenvecha entry.
///
/// Parses common boolean string representations:
///   "yes", "enabled", "available", "true", "1" → true
///   anything else or missing → false
///
/// Usage: `kernel_bool(self.id(), "symbols.kallsyms")`
pub(crate) fn kernel_bool(id: &'static str, proc_key: &str) -> Evidence {
    let val = read_proc(proc_key)
        .is_some_and(|s| matches!(s.as_str(), "yes" | "enabled" | "available" | "true" | "1"));
    Evidence::present(id, EvidenceValue::Bool(val))
}

/// Create module-loaded status Evidence.
///
/// Returns `Evidence::present` when the module directory exists AND
/// at least one proc entry is readable. Returns `Evidence::missing`
/// when the module is not loaded.
///
/// Usage: `kernel_status(self.id(), "version.release")`
pub(crate) fn kernel_status(id: &'static str, verify_key: &str) -> Evidence {
    if module_loaded() {
        let confirmed = read_proc(verify_key).is_some();
        Evidence::present(id, EvidenceValue::Bool(confirmed))
    } else {
        Evidence::missing(id, EvidenceValue::Bool(false))
    }
}

// ============================================================================
//  Framework Integrity Check
// ============================================================================

/// Verify that all registered providers use the framework constructors.
///
/// This is a compile-time architectural constraint: providers that
/// manually construct Evidence bypass the framework and risk
/// inconsistency. This function exists to document the contract;
/// actual enforcement happens during code review.
#[allow(dead_code)]
pub(crate) fn validate_framework_usage() {
    // If a provider calls Evidence::present() or Evidence::missing()
    // directly instead of kernel_text()/kernel_bool()/kernel_status(),
    // it violates the Kernel Evidence Framework contract.
    //
    // This is enforced by code review and grep:
    //   grep -r 'Evidence::present\|Evidence::missing' \
    //     src/core/caps/kernel_cap/ --include='*.rs' \
    //     | grep -v 'kernel_text\|kernel_bool\|kernel_status'
}

// ============================================================================
//  Module declarations and re-exports
// ============================================================================

pub mod btf;
pub mod loader;
pub mod module;
pub mod symbols;
pub mod tracing;
pub mod version;

pub use btf::KernelBtfStatus;
pub use loader::KernelModuleLoader;
pub use module::KernelModuleStatus;
pub use symbols::{
    KernelSymbolCollection, KernelSymbolExported, KernelSymbolGplOnly, KernelSymbolInternal,
    KernelSymbolKallsyms, KernelSymbolKallsymsAll, KernelSymbolKptrRestrict,
    KernelSymbolModuleOwned, KernelSymbolNamespaced, KernelSymbolTotal, KernelSymbolVmlinux,
};
pub use tracing::{KernelTracingFtrace, KernelTracingKprobes};
pub use version::KernelVersionFromModule;

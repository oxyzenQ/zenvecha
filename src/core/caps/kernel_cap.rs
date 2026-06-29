// Copyright (C) 2026 rezky_nightky
// SPDX-License-Identifier: GPL-3.0-only

//! Kernel capability collector — userspace bridge to the Zenvecha kernel module.
//!
//! Reads structured facts from /proc/zenvecha/* and presents them as
//! standard Capability implementations. Each Capability maps to one
//! proc entry from the kernel module.
//!
//! When the kernel module is not loaded, these capabilities gracefully
//! return missing evidence — they never crash and never block the pipeline.
//!
//! Architecture (Phase 7):
//!   Kernel Module → /proc/zenvecha/* → kernel_cap::* → Evidence → Pipeline
//!
//! To add a new kernel capability fact:
//!   1. Add the probe in kernel/capability/probes/
//!   2. Add one Capability struct here
//!   3. Register in capability.rs register_all()

use crate::core::capability::Capability;
use crate::core::evidence::{Evidence, EvidenceValue};

// ============================================================================
//  Proc helpers
// ============================================================================

/// Read a single /proc/zenvecha entry. Returns None if module not loaded.
fn read_proc(key: &str) -> Option<String> {
    let path = format!("/proc/zenvecha/{key}");
    std::fs::read_to_string(&path)
        .ok()
        .map(|s| s.trim().to_string())
}

/// Best-effort proc read.
fn probe_val(key: &str) -> Option<String> {
    read_proc(key)
}

/// Check if the Zenvecha kernel module is loaded.
fn module_loaded() -> bool {
    std::path::Path::new("/proc/zenvecha").is_dir()
}

// ============================================================================
//  Kernel Module Status
// ============================================================================

pub struct KernelModuleStatus;

impl Capability for KernelModuleStatus {
    fn id(&self) -> &'static str {
        "kernel.module_loaded"
    }
    fn label(&self) -> &'static str {
        "Zenvecha Kernel Module"
    }
    fn probe(&self) -> Evidence {
        Evidence::present(self.id(), EvidenceValue::Bool(module_loaded()))
    }
}

// ============================================================================
//  Kernel Version (from kernel module)
// ============================================================================

pub struct KernelVersionFromModule;

impl Capability for KernelVersionFromModule {
    fn id(&self) -> &'static str {
        "kernel.version.module"
    }
    fn label(&self) -> &'static str {
        "Kernel Version (module)"
    }
    fn probe(&self) -> Evidence {
        match probe_val("version.release") {
            Some(v) => Evidence::present(self.id(), EvidenceValue::Text(Some(v))),
            None => Evidence::missing(self.id(), EvidenceValue::Text(None)),
        }
    }
}

// ============================================================================
//  Symbol Count (from kernel module)
// ============================================================================

pub struct KernelSymbolCount;

impl Capability for KernelSymbolCount {
    fn id(&self) -> &'static str {
        "kernel.symbols.module_count"
    }
    fn label(&self) -> &'static str {
        "Kernel Symbols (module)"
    }
    fn probe(&self) -> Evidence {
        match probe_val("symbols.count") {
            Some(v) => Evidence::present(self.id(), EvidenceValue::Text(Some(v))),
            None => Evidence::missing(self.id(), EvidenceValue::Text(None)),
        }
    }
}

// ============================================================================
//  BTF Status (from kernel module)
// ============================================================================

pub struct KernelBtfStatus;

impl Capability for KernelBtfStatus {
    fn id(&self) -> &'static str {
        "kernel.btf.module"
    }
    fn label(&self) -> &'static str {
        "BTF Status (module)"
    }
    fn probe(&self) -> Evidence {
        let val = probe_val("btf.available")
            .map(|s| s == "yes")
            .unwrap_or(false);
        Evidence::present(self.id(), EvidenceValue::Bool(val))
    }
}

// ============================================================================
//  Module Loader (from kernel module)
// ============================================================================

pub struct KernelModuleLoader;

impl Capability for KernelModuleLoader {
    fn id(&self) -> &'static str {
        "kernel.loader.module"
    }
    fn label(&self) -> &'static str {
        "Module Loader (module)"
    }
    fn probe(&self) -> Evidence {
        match probe_val("modules.loader") {
            Some(v) => Evidence::present(self.id(), EvidenceValue::Text(Some(v))),
            None => Evidence::missing(self.id(), EvidenceValue::Text(None)),
        }
    }
}

// ============================================================================
//  Tracing (from kernel module)
// ============================================================================

pub struct KernelTracingFtrace;

impl Capability for KernelTracingFtrace {
    fn id(&self) -> &'static str {
        "kernel.tracing.ftrace"
    }
    fn label(&self) -> &'static str {
        "ftrace (kernel)"
    }
    fn probe(&self) -> Evidence {
        let val = probe_val("tracing.ftrace")
            .map(|s| s == "available")
            .unwrap_or(false);
        Evidence::present(self.id(), EvidenceValue::Bool(val))
    }
}

pub struct KernelTracingKprobes;

impl Capability for KernelTracingKprobes {
    fn id(&self) -> &'static str {
        "kernel.tracing.kprobes"
    }
    fn label(&self) -> &'static str {
        "kprobes (kernel)"
    }
    fn probe(&self) -> Evidence {
        let val = probe_val("tracing.kprobes")
            .map(|s| s == "available")
            .unwrap_or(false);
        Evidence::present(self.id(), EvidenceValue::Bool(val))
    }
}

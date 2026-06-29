// SPDX-License-Identifier: GPL-2.0-only
// Copyright (C) 2026 rezky_nightky

//! Structured kernel capability types.
//!
//! The kernel module produces structured facts, never human-readable reports.
//! These types are the kernel-space contract — userspace reads them via
//! proc/sysfs and converts to Evidence for the Wolfzenix pipeline.
//!
//! Design rules:
//!   - No rendering
//!   - No scoring
//!   - No CLI awareness
//!   - No human-readable formatting
//!   - One type per capability domain

/// A single capability fact from kernel space.
pub struct Capability {
    /// Unique capability key (e.g. "symbols.count", "btf.available").
    pub key: &'static str,
    /// Structured value as a plain string (proc-compatible).
    pub value: &'static str,
}

/// Complete set of capabilities discovered by the kernel module.
pub struct CapabilitySet {
    pub capabilities: &'static [Capability],
}

/// A kernel symbol with ownership information.
pub struct ExportedSymbol {
    pub name: &'static str,
    pub address: usize,
    pub owner: Option<&'static str>,
}

/// Capability categories the kernel module can discover.
pub enum CapabilityKind {
    /// Kernel version (release string, major, minor, patch).
    Version,
    /// Exported symbol list.
    Symbols,
    /// kallsyms availability (all, none, restricted).
    Kallsyms,
    /// BTF (BPF Type Format) availability.
    Btf,
    /// Module loader status.
    ModuleLoader,
    /// Tracing infrastructure (ftrace, kprobes, tracepoints).
    Tracing,
    /// Livepatch support.
    Livepatch,
    /// Rust for Linux support.
    RustSupport,
    /// CPU architecture.
    Architecture,
}

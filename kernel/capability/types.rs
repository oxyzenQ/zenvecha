// SPDX-License-Identifier: GPL-2.0-only
// Copyright (C) 2026 rezky_nightky

//! Structured kernel capability types.
//!
//! ## Architecture
//!
//! Two layers:
//!   1. Compile-time contract: `CapabilityDescriptor` — defines what each
//!      probe promises to discover (used by the skeletal architecture).
//!   2. Runtime data: `ProbeResult` — what was actually discovered at
//!      runtime (used by real kernel modules with dynamic allocation).
//!
//! The current implementation uses layer 1 (static descriptors) as
//! the architectural skeleton. When built inside a real kernel tree
//! with CONFIG_RUST=y, probes produce `ProbeResult` values allocated
//! via the kernel memory allocator (KernelAlloc or vmalloc).
//!
//! ## Design rules:
//!   - No rendering
//!   - No scoring
//!   - No CLI awareness
//!   - No human-readable formatting
//!   - One type per capability domain

/// Compile-time capability descriptor — defines what a probe discovers.
/// Used by the architectural skeleton. At runtime, this becomes a
/// `ProbeResult` with dynamically-allocated values.
pub struct CapabilityDescriptor {
    pub key: &'static str,
    pub value: &'static str,
}

/// Result of a single capability probe at runtime.
///
/// Produced by real kernel modules with dynamic allocation.
/// Userspace reads these via /proc/zenvecha/{key}.
#[derive(Clone, Debug)]
pub struct ProbeResult {
    /// Unique key matching a CapabilityDescriptor.
    pub key: String,
    /// The discovered value — empty string means probe ran but found nothing.
    pub value: String,
    /// Whether the probe succeeded, partially succeeded, or failed.
    pub status: ProbeStatus,
    /// Human-readable error if probe failed.
    pub error: Option<String>,
}

/// Outcome of a capability probe.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProbeStatus {
    /// Probe completed successfully, data is reliable.
    Success,
    /// Probe ran but the resource was unavailable (e.g., CONFIG disabled).
    Unavailable,
    /// Probe encountered a recoverable error (e.g., partial data).
    Partial,
    /// Probe failed entirely (e.g., permission denied, kernel panic avoided).
    Failed,
}

impl ProbeStatus {
    pub fn label(self) -> &'static str {
        match self {
            ProbeStatus::Success => "success",
            ProbeStatus::Unavailable => "unavailable",
            ProbeStatus::Partial => "partial",
            ProbeStatus::Failed => "failed",
        }
    }
}

/// Complete set of probe results from kernel module discovery.
pub struct ProbeResultSet {
    pub results: Vec<ProbeResult>,
}

/// A kernel symbol with ownership information (runtime).
pub struct ExportedSymbol {
    pub name: String,
    pub address: usize,
    pub owner: Option<String>,
}

/// Capability categories the kernel module can discover.
pub enum CapabilityKind {
    Version,
    Symbols,
    Kallsyms,
    Btf,
    ModuleLoader,
    Tracing,
    Livepatch,
    RustSupport,
    Architecture,
    /// Reserved for future providers.
    Custom(&'static str),
}

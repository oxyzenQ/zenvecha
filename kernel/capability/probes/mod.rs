// SPDX-License-Identifier: GPL-2.0-only
// Copyright (C) 2026 rezky_nightky

//! Probe trait and registry.
//!
//! ## Compile-Time Skeleton (current)
//!
//! Every probe implements `Probe` with `&'static [CapabilityDescriptor]`.
//! This defines the contract — what each probe promises to discover.
//! Used for architectural validation and documentation.
//!
//! ## Runtime Contract (when built inside a real kernel tree)
//!
//! Probes implement `RuntimeProbe` which produces dynamically-allocated
//! `ProbeResult` values. Results are exposed via /proc/zenvecha/{key}.
//!
//! ## Provider Contract
//!
//! Adding a new capability provider requires:
//!   1. One new file in probes/{domain}.rs
//!   2. One entry in all_probes()
//!   3. One corresponding userspace Capability in src/core/caps/kernel_cap/
//!   4. Zero modifications to existing probes
//!
//! Rules for every probe:
//!   - Read-only: never modify kernel state
//!   - Isolated: never depend on other probes
//!   - Structured: produce key=value facts, never formatted text
//!   - Graceful: return ProbeStatus::Unavailable, never panic

use super::types::CapabilityDescriptor;

/// Compile-time probe contract — defines what each probe discovers.
///
/// The `discover()` method returns static capability descriptors.
/// At runtime, these become dynamic `ProbeResult` values read from
/// /proc/zenvecha/.
pub trait Probe {
    /// Unique name for this probe (e.g. "version", "symbols", "btf").
    /// This maps to the proc directory: /proc/zenvecha/{name}/
    fn name(&self) -> &'static str;

    /// Discover capabilities. Returns capability descriptors.
    fn discover(&self) -> &'static [CapabilityDescriptor];
}

/// Registry of all kernel capability probes.
///
/// Adding a new probe = add one entry to this list + create the probe file.
/// No existing code needs modification.
pub fn all_probes() -> &'static [&'static dyn Probe] {
    &[
        &probes::version::VersionProbe,
        &probes::symbols::SymbolsProbe,
        &probes::kallsyms::KallsymsProbe,
        &probes::btf::BtfProbe,
        &probes::modules::ModuleLoaderProbe,
        &probes::tracing::TracingProbe,
        &probes::arch::ArchitectureProbe,
    ]
}

// SPDX-License-Identifier: GPL-2.0-only
// Copyright (C) 2026 rezky_nightky

//! Probe trait and registry.
//!
//! Every capability probe implements the Probe trait. Adding a new
//! capability requires creating ONE new file in probes/ — nothing else
//! needs modification.

use super::types::Capability;

/// A kernel capability probe. One implementation per capability domain.
///
/// Each probe is a read-only inspection of kernel state. No probe
/// modifies kernel state, no probe inserts hooks, no probe patches memory.
pub trait Probe {
    /// Unique name for this probe (e.g. "version", "symbols", "btf").
    fn name(&self) -> &'static str;

    /// Discover capabilities. Returns zero or more capability facts.
    fn discover(&self) -> &'static [Capability];
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

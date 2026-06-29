// SPDX-License-Identifier: GPL-2.0-only
// Copyright (C) 2026 rezky_nightky

//! Zenvecha Kernel Capability Layer.
//!
//! ## Architecture
//!
//! Two layers of types:
//!   1. CapabilityDescriptor — compile-time contract (what probes promise)
//!   2. ProbeResult           — runtime contract (what was actually found)
//!
//! ## Directory Structure
//!
//!   kernel/capability/
//!     types.rs       → CapabilityDescriptor, ProbeResult, ProbeStatus,
//!                       ExportedSymbol, CapabilityKind
//!     probes/
//!       mod.rs       → Probe trait + all_probes() registry
//!       version.rs   → kernel version
//!       symbols.rs   → exported symbols
//!       kallsyms.rs  → kallsyms availability
//!       btf.rs       → BTF availability
//!       modules.rs   → module loader status
//!       tracing.rs   → tracing frameworks
//!       arch.rs      → CPU architecture
//!
//! ## Adding a Provider
//!
//!   1. Create probes/{domain}.rs — implement Probe
//!   2. Add entry to probes/mod.rs all_probes()
//!   3. Create src/core/caps/kernel_cap/{domain}.rs — implement Capability
//!   4. Register in src/core/capability.rs register_all()
//!   5. Zero modifications to any existing file (except steps 2+4)

pub mod probes;
pub mod types;

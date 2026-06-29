// SPDX-License-Identifier: GPL-2.0-only
// Copyright (C) 2026 rezky_nightky

//! Zenvecha Kernel Capability Layer.
//!
//! Structured kernel capability discovery. Every probe is read-only.
//! Facts flow from kernel → proc → userspace → Evidence → Pipeline.
//!
//! Design:
//!   - One trait: Probe
//!   - One registry: all_probes()
//!   - One new file per capability domain
//!   - Zero changes to existing probes when adding new ones
//!
//! Architecture:
//!   kernel/capability/
//!     types.rs       → Capability, CapabilitySet, CapabilityKind
//!     probes/
//!       mod.rs       → Probe trait + all_probes() registry
//!       version.rs   → kernel version
//!       symbols.rs   → exported symbols
//!       kallsyms.rs  → kallsyms availability
//!       btf.rs       → BTF availability
//!       modules.rs   → module loader status
//!       tracing.rs   → tracing frameworks
//!       arch.rs      → CPU architecture

pub mod probes;
pub mod types;

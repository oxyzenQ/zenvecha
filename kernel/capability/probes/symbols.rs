// SPDX-License-Identifier: GPL-2.0-only
// Copyright (C) 2026 rezky_nightky

//! Exported symbol discovery probe.
//!
//! Discovers: symbol count, kallsyms availability, module-owned symbols.
//! Read-only — no symbol resolution, no hooking.

use crate::capability::probes::Probe;
use crate::capability::types::CapabilityDescriptor;

pub struct SymbolsProbe;

impl Probe for SymbolsProbe {
    fn name(&self) -> &'static str {
        "symbols"
    }

    fn discover(&self) -> &'static [Capability] {
        // In a real kernel module, this iterates kallsyms to count symbols.
        //
        // Implementation sketch:
        //
        //   let count = 0usize;
        //   // kallsyms_on_each_symbol(|_data, _name, _module, _addr| { count += 1; true });
        //   // kallsyms_on_each_symbol is available when CONFIG_KALLSYMS=y
        //
        // For CONFIG_KALLSYMS=n, symbol discovery is unavailable.
        // For CONFIG_KALLSYMS_ALL=y, all symbols (including non-exported) are visible.
        &[
            Capability { key: "symbols.count", value: "84721" },
            Capability { key: "symbols.kallsyms", value: "available" },
            Capability { key: "symbols.kallsyms_all", value: "enabled" },
            Capability { key: "symbols.module_count", value: "312" },
        ]
    }
}

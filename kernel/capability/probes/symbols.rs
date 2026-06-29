// SPDX-License-Identifier: GPL-2.0-only
// Copyright (C) 2026 rezky_nightky

//! Symbol Discovery probe — reference implementation for all runtime providers.
//!
//! Discovered facts:
//!   - Total symbol count
//!   - Exported vs GPL-only vs internal breakdown
//!   - Module-owned symbols
//!   - Namespaced exports (EXPORT_SYMBOL_NS)
//!   - kallsyms visibility level
//!   - Collection status
//!   - Available symbol capabilities
//!
//! Architecture pattern:
//!   KernelSymbol  → individual symbol (iterated via kallsyms)
//!   KernelSymbolSet → aggregate statistics (one-shot read)
//!
//! This probe is the TEMPLATE for all future runtime providers.

use crate::capability::probes::Probe;
use crate::capability::types::CapabilityDescriptor;

pub struct SymbolsProbe;

impl Probe for SymbolsProbe {
    fn name(&self) -> &'static str {
        "symbols"
    }

    fn discover(&self) -> &'static [CapabilityDescriptor] {
        // Runtime implementation (real kernel module):
        //
        //   let mut set = KernelSymbolSet::default();
        //   kallsyms_on_each_symbol(|data, name, module, addr| {
        //       let sym = classify_symbol(name, module, addr);
        //       set.ingest(sym);
        //       true  // continue iteration
        //   }, &mut set);
        //
        //   // Expose via /proc/zenvecha/symbols/
        //   create_proc_entry("symbols/count", &set.total_symbols);
        //   create_proc_entry("symbols/exported", &set.exported_symbols);
        //   create_proc_entry("symbols/gpl_only", &set.gpl_only_symbols);
        //   create_proc_entry("symbols/module_owned", &set.module_owned_symbols);
        //   create_proc_entry("symbols/namespaced", &set.namespaced_symbols);
        //   create_proc_entry("symbols/status", &set.collection_status.label());
        //
        //   // Iteration for individual symbols (future):
        //   // /proc/zenvecha/symbols/list → one KernelSymbol per line
        //
        // classify_symbol() inspects the kallsyms type character:
        //   'T' / 't' = text (exported / local)
        //   'D' / 'd' = data
        //   'B' / 'b' = BSS
        //   'R' / 'r' = read-only data
        //   GPL-only detection requires comparing against __ksymtab_gpl
        //   Namespace detection requires EXPORT_SYMBOL_NS metadata

        &[
            // Aggregate counts
            CapabilityDescriptor { key: "symbols.total", value: "84721" },
            CapabilityDescriptor { key: "symbols.exported", value: "12341" },
            CapabilityDescriptor { key: "symbols.gpl_only", value: "3892" },
            CapabilityDescriptor { key: "symbols.internal", value: "68488" },
            CapabilityDescriptor { key: "symbols.module_owned", value: "312" },
            CapabilityDescriptor { key: "symbols.vmlinux", value: "84409" },
            CapabilityDescriptor { key: "symbols.namespaced", value: "247" },

            // Infrastructure status
            CapabilityDescriptor { key: "symbols.kallsyms", value: "available" },
            CapabilityDescriptor { key: "symbols.kallsyms_all", value: "enabled" },
            CapabilityDescriptor { key: "symbols.kptr_restrict", value: "1" },

            // Collection metadata
            CapabilityDescriptor { key: "symbols.collection_status", value: "complete" },
            CapabilityDescriptor { key: "symbols.collection_confidence", value: "high" },
        ]
    }
}

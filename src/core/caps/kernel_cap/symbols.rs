// Copyright (C) 2026 rezky_nightky
// SPDX-License-Identifier: GPL-3.0-only

//! Symbol Discovery collector — reference implementation.
//!
//! Converts kernel module symbol facts into structured Evidence.
//! This is the TEMPLATE for all future runtime provider collectors.
//!
//! ## Architecture Pattern (follow this for every new provider)
//!
//!   1. Define domain model types matching the kernel types
//!   2. Implement one Capability per evidence fact (single responsibility)
//!   3. Read from /proc/zenvecha/{domain}/{key}
//!   4. Parse into typed Evidence values, never raw strings
//!   5. Gracefully degrade when kernel module is not loaded
//!
//! ## Evidence produced
//!
//!   kernel.symbols.total          — text (count)
//!   kernel.symbols.exported       — text (count)
//!   kernel.symbols.gpl_only       — text (count)
//!   kernel.symbols.internal       — text (count)
//!   kernel.symbols.module_owned   — text (count)
//!   kernel.symbols.vmlinux        — text (count)
//!   kernel.symbols.namespaced     — text (count)
//!   kernel.symbols.kallsyms       — bool
//!   kernel.symbols.kallsyms_all   — bool
//!   kernel.symbols.kptr_restrict  — text (restriction level)
//!   kernel.symbols.collection     — text (status label)

use crate::core::capability::Capability;
use crate::core::caps::kernel_cap::read_proc;
use crate::core::evidence::{Evidence, EvidenceValue};

// ============================================================================
//  Symbol Counts
// ============================================================================

fn count_evidence(id: &'static str, proc_key: &str) -> Evidence {
    match read_proc(proc_key) {
        Some(v) => Evidence::present(id, EvidenceValue::Text(Some(v))),
        None => Evidence::missing(id, EvidenceValue::Text(None)),
    }
}

pub struct KernelSymbolTotal;

impl Capability for KernelSymbolTotal {
    fn id(&self) -> &'static str {
        "kernel.symbols.total"
    }
    fn label(&self) -> &'static str {
        "Total Symbols (module)"
    }
    fn probe(&self) -> Evidence {
        count_evidence(self.id(), "symbols.total")
    }
}

pub struct KernelSymbolExported;

impl Capability for KernelSymbolExported {
    fn id(&self) -> &'static str {
        "kernel.symbols.exported"
    }
    fn label(&self) -> &'static str {
        "Exported Symbols (module)"
    }
    fn probe(&self) -> Evidence {
        count_evidence(self.id(), "symbols.exported")
    }
}

pub struct KernelSymbolGplOnly;

impl Capability for KernelSymbolGplOnly {
    fn id(&self) -> &'static str {
        "kernel.symbols.gpl_only"
    }
    fn label(&self) -> &'static str {
        "GPL-Only Symbols (module)"
    }
    fn probe(&self) -> Evidence {
        count_evidence(self.id(), "symbols.gpl_only")
    }
}

pub struct KernelSymbolInternal;

impl Capability for KernelSymbolInternal {
    fn id(&self) -> &'static str {
        "kernel.symbols.internal"
    }
    fn label(&self) -> &'static str {
        "Internal Symbols (module)"
    }
    fn probe(&self) -> Evidence {
        count_evidence(self.id(), "symbols.internal")
    }
}

pub struct KernelSymbolModuleOwned;

impl Capability for KernelSymbolModuleOwned {
    fn id(&self) -> &'static str {
        "kernel.symbols.module_owned"
    }
    fn label(&self) -> &'static str {
        "Module-Owned Symbols (module)"
    }
    fn probe(&self) -> Evidence {
        count_evidence(self.id(), "symbols.module_owned")
    }
}

pub struct KernelSymbolVmlinux;

impl Capability for KernelSymbolVmlinux {
    fn id(&self) -> &'static str {
        "kernel.symbols.vmlinux"
    }
    fn label(&self) -> &'static str {
        "Vmlinux Symbols (module)"
    }
    fn probe(&self) -> Evidence {
        count_evidence(self.id(), "symbols.vmlinux")
    }
}

pub struct KernelSymbolNamespaced;

impl Capability for KernelSymbolNamespaced {
    fn id(&self) -> &'static str {
        "kernel.symbols.namespaced"
    }
    fn label(&self) -> &'static str {
        "Namespaced Symbols (module)"
    }
    fn probe(&self) -> Evidence {
        count_evidence(self.id(), "symbols.namespaced")
    }
}

// ============================================================================
//  Infrastructure Status
// ============================================================================

fn bool_evidence(id: &'static str, proc_key: &str) -> Evidence {
    let val = read_proc(proc_key)
        .map(|s| s == "available" || s == "enabled" || s == "yes")
        .unwrap_or(false);
    Evidence::present(id, EvidenceValue::Bool(val))
}

pub struct KernelSymbolKallsyms;

impl Capability for KernelSymbolKallsyms {
    fn id(&self) -> &'static str {
        "kernel.symbols.kallsyms"
    }
    fn label(&self) -> &'static str {
        "kallsyms Available (module)"
    }
    fn probe(&self) -> Evidence {
        bool_evidence(self.id(), "symbols.kallsyms")
    }
}

pub struct KernelSymbolKallsymsAll;

impl Capability for KernelSymbolKallsymsAll {
    fn id(&self) -> &'static str {
        "kernel.symbols.kallsyms_all"
    }
    fn label(&self) -> &'static str {
        "kallsyms All Symbols (module)"
    }
    fn probe(&self) -> Evidence {
        bool_evidence(self.id(), "symbols.kallsyms_all")
    }
}

pub struct KernelSymbolKptrRestrict;

impl Capability for KernelSymbolKptrRestrict {
    fn id(&self) -> &'static str {
        "kernel.symbols.kptr_restrict"
    }
    fn label(&self) -> &'static str {
        "kptr_restrict Level (module)"
    }
    fn probe(&self) -> Evidence {
        match read_proc("symbols.kptr_restrict") {
            Some(v) => Evidence::present(self.id(), EvidenceValue::Text(Some(v))),
            None => Evidence::missing(self.id(), EvidenceValue::Text(None)),
        }
    }
}

// ============================================================================
//  Collection Metadata
// ============================================================================

pub struct KernelSymbolCollection;

impl Capability for KernelSymbolCollection {
    fn id(&self) -> &'static str {
        "kernel.symbols.collection"
    }
    fn label(&self) -> &'static str {
        "Symbol Collection Status (module)"
    }
    fn probe(&self) -> Evidence {
        match read_proc("symbols.collection_status") {
            Some(v) => Evidence::present(self.id(), EvidenceValue::Text(Some(v))),
            None => Evidence::missing(self.id(), EvidenceValue::Text(None)),
        }
    }
}

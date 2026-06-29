// Copyright (C) 2026 rezky_nightky
// SPDX-License-Identifier: GPL-3.0-only

//! Symbol Discovery collector — reference implementation.
//!
//! Uses the Kernel Evidence Framework (kernel_text, kernel_bool).
//! This is the TEMPLATE for all future runtime provider collectors.
//!
//! Every Capability:
//!   1. Defines a unique evidence ID
//!   2. Calls one framework constructor
//!   3. Returns Evidence — never constructs it manually

use crate::core::capability::Capability;
use crate::core::caps::kernel_cap::{kernel_bool, kernel_text};
use crate::core::evidence::Evidence;

// Each macro invocation creates a Capability struct + impl in 5 lines.
// Without it, each provider would be ~12 lines of identical boilerplate.
macro_rules! text_cap {
    ($name:ident, $id:literal, $label:literal, $proc_key:literal) => {
        pub struct $name;
        impl Capability for $name {
            fn id(&self) -> &'static str {
                $id
            }
            fn label(&self) -> &'static str {
                $label
            }
            fn probe(&self) -> Evidence {
                kernel_text(self.id(), $proc_key)
            }
        }
    };
}

macro_rules! bool_cap {
    ($name:ident, $id:literal, $label:literal, $proc_key:literal) => {
        pub struct $name;
        impl Capability for $name {
            fn id(&self) -> &'static str {
                $id
            }
            fn label(&self) -> &'static str {
                $label
            }
            fn probe(&self) -> Evidence {
                kernel_bool(self.id(), $proc_key)
            }
        }
    };
}

// ── Symbol Counts ──
text_cap!(
    KernelSymbolTotal,
    "kernel.symbols.total",
    "Total Symbols (module)",
    "symbols.total"
);
text_cap!(
    KernelSymbolExported,
    "kernel.symbols.exported",
    "Exported Symbols (module)",
    "symbols.exported"
);
text_cap!(
    KernelSymbolGplOnly,
    "kernel.symbols.gpl_only",
    "GPL-Only Symbols (module)",
    "symbols.gpl_only"
);
text_cap!(
    KernelSymbolInternal,
    "kernel.symbols.internal",
    "Internal Symbols (module)",
    "symbols.internal"
);
text_cap!(
    KernelSymbolModuleOwned,
    "kernel.symbols.module_owned",
    "Module-Owned Symbols (module)",
    "symbols.module_owned"
);
text_cap!(
    KernelSymbolVmlinux,
    "kernel.symbols.vmlinux",
    "Vmlinux Symbols (module)",
    "symbols.vmlinux"
);
text_cap!(
    KernelSymbolNamespaced,
    "kernel.symbols.namespaced",
    "Namespaced Symbols (module)",
    "symbols.namespaced"
);

// ── Infrastructure Status ──
bool_cap!(
    KernelSymbolKallsyms,
    "kernel.symbols.kallsyms",
    "kallsyms Available (module)",
    "symbols.kallsyms"
);
bool_cap!(
    KernelSymbolKallsymsAll,
    "kernel.symbols.kallsyms_all",
    "kallsyms All Symbols (module)",
    "symbols.kallsyms_all"
);

// ── kptr_restrict (needs raw text, not bool) ──
pub struct KernelSymbolKptrRestrict;
impl Capability for KernelSymbolKptrRestrict {
    fn id(&self) -> &'static str {
        "kernel.symbols.kptr_restrict"
    }
    fn label(&self) -> &'static str {
        "kptr_restrict Level (module)"
    }
    fn probe(&self) -> Evidence {
        kernel_text(self.id(), "symbols.kptr_restrict")
    }
}

// ── Collection Metadata ──
pub struct KernelSymbolCollection;
impl Capability for KernelSymbolCollection {
    fn id(&self) -> &'static str {
        "kernel.symbols.collection"
    }
    fn label(&self) -> &'static str {
        "Symbol Collection Status (module)"
    }
    fn probe(&self) -> Evidence {
        kernel_text(self.id(), "symbols.collection_status")
    }
}

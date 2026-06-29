// Copyright (C) 2026 rezky_nightky
// SPDX-License-Identifier: GPL-3.0-only

//! Symbol capabilities — kallsyms, symbol count, vmlinux, symvers.

use crate::core::capability::Capability;
use crate::core::evidence::{Evidence, EvidenceValue};
use crate::system;

pub struct KallsymsInfo;
impl Capability for KallsymsInfo {
    fn id(&self) -> &'static str {
        "symbols.kallsyms"
    }
    fn label(&self) -> &'static str {
        "Kallsyms"
    }
    fn probe(&self) -> Evidence {
        let ks = system::kallsyms::inspect_kallsyms();
        let status = if ks.exists && ks.readable {
            if ks.root_only {
                "readable (root)"
            } else {
                "readable"
            }
        } else if ks.exists {
            "permission denied"
        } else {
            "hidden"
        };
        Evidence::present(self.id(), EvidenceValue::Status(status))
    }
}

pub struct SymbolCount;
impl Capability for SymbolCount {
    fn id(&self) -> &'static str {
        "symbols.count"
    }
    fn label(&self) -> &'static str {
        "Symbol Count"
    }
    fn probe(&self) -> Evidence {
        let release = system::kernel::kernel_release();
        let info = system::symbols::inspect_symbols(release.as_deref());
        Evidence::present(
            self.id(),
            EvidenceValue::Count(info.symbol_count.unwrap_or(0)),
        )
    }
}

pub struct VmlinuxInfo;
impl Capability for VmlinuxInfo {
    fn id(&self) -> &'static str {
        "symbols.vmlinux"
    }
    fn label(&self) -> &'static str {
        "VMLinux"
    }
    fn probe(&self) -> Evidence {
        let release = system::kernel::kernel_release();
        let info = system::symbols::inspect_symbols(release.as_deref());
        match info.vmlinux_path {
            Some(p) => {
                let sz = info
                    .vmlinux_size
                    .map(|s| format!(" ({})", system::capabilities::human_size(s)))
                    .unwrap_or_default();
                let bid = info
                    .vmlinux_build_id
                    .map(|b| format!(" BuildID={b}"))
                    .unwrap_or_default();
                Evidence::present(self.id(), EvidenceValue::Literal(format!("{p}{sz}{bid}")))
            }
            None => Evidence::missing(self.id(), EvidenceValue::Text(None)),
        }
    }
}

pub struct ModuleSymvers;
impl Capability for ModuleSymvers {
    fn id(&self) -> &'static str {
        "symbols.symvers"
    }
    fn label(&self) -> &'static str {
        "Module.symvers"
    }
    fn probe(&self) -> Evidence {
        let release = system::kernel::kernel_release();
        let info = system::symbols::inspect_symbols(release.as_deref());
        match info.module_symvers_path {
            Some(p) => Evidence::present(self.id(), EvidenceValue::Text(Some(p))),
            None => Evidence::missing(self.id(), EvidenceValue::Text(None)),
        }
    }
}

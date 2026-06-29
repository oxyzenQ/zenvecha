// Copyright (C) 2026 rezky_nightky
// SPDX-License-Identifier: GPL-3.0-only

//! Kernel symbol count probe — reads from Zenvecha kernel module.
//!
//! Proc key: symbols.count

use crate::core::capability::Capability;
use crate::core::caps::kernel_cap::read_proc;
use crate::core::evidence::{Evidence, EvidenceValue};

pub struct KernelSymbolCount;

impl Capability for KernelSymbolCount {
    fn id(&self) -> &'static str {
        "kernel.symbols.module"
    }
    fn label(&self) -> &'static str {
        "Kernel Symbols (module)"
    }
    fn probe(&self) -> Evidence {
        match read_proc("symbols.count") {
            Some(v) => Evidence::present(self.id(), EvidenceValue::Text(Some(v))),
            None => Evidence::missing(self.id(), EvidenceValue::Text(None)),
        }
    }
}

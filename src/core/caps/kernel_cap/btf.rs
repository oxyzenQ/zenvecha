// Copyright (C) 2026 rezky_nightky
// SPDX-License-Identifier: GPL-3.0-only

//! BTF (BPF Type Format) probe — reads from Zenvecha kernel module.
//!
//! Proc key: btf.available

use crate::core::capability::Capability;
use crate::core::caps::kernel_cap::read_proc;
use crate::core::evidence::{Evidence, EvidenceValue};

pub struct KernelBtfStatus;

impl Capability for KernelBtfStatus {
    fn id(&self) -> &'static str {
        "kernel.btf.module"
    }
    fn label(&self) -> &'static str {
        "BTF Status (module)"
    }
    fn probe(&self) -> Evidence {
        let val = read_proc("btf.available")
            .map(|s| s == "yes")
            .unwrap_or(false);
        Evidence::present(self.id(), EvidenceValue::Bool(val))
    }
}

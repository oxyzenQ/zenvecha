// Copyright (C) 2026 rezky_nightky
// SPDX-License-Identifier: GPL-3.0-only

//! Kernel version probe — reads from Zenvecha kernel module.
//!
//! Proc key: version.release

use crate::core::capability::Capability;
use crate::core::caps::kernel_cap::read_proc;
use crate::core::evidence::{Evidence, EvidenceValue};

pub struct KernelVersionFromModule;

impl Capability for KernelVersionFromModule {
    fn id(&self) -> &'static str {
        "kernel.version.module"
    }
    fn label(&self) -> &'static str {
        "Kernel Version (module)"
    }
    fn probe(&self) -> Evidence {
        match read_proc("version.release") {
            Some(v) => Evidence::present(self.id(), EvidenceValue::Text(Some(v))),
            None => Evidence::missing(self.id(), EvidenceValue::Text(None)),
        }
    }
}

// Copyright (C) 2026 rezky_nightky
// SPDX-License-Identifier: GPL-3.0-only

//! Module loader probe — reads from Zenvecha kernel module.
//!
//! Proc key: modules.loader

use crate::core::capability::Capability;
use crate::core::caps::kernel_cap::read_proc;
use crate::core::evidence::{Evidence, EvidenceValue};

pub struct KernelModuleLoader;

impl Capability for KernelModuleLoader {
    fn id(&self) -> &'static str {
        "kernel.loader.module"
    }
    fn label(&self) -> &'static str {
        "Module Loader (module)"
    }
    fn probe(&self) -> Evidence {
        match read_proc("modules.loader") {
            Some(v) => Evidence::present(self.id(), EvidenceValue::Text(Some(v))),
            None => Evidence::missing(self.id(), EvidenceValue::Text(None)),
        }
    }
}

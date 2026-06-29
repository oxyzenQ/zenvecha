// Copyright (C) 2026 rezky_nightky
// SPDX-License-Identifier: GPL-3.0-only

//! Kernel module presence probe.
//!
//! Detects whether the Zenvecha kernel module is loaded
//! by checking for /proc/zenvecha directory.

use crate::core::capability::Capability;
use crate::core::caps::kernel_cap::read_proc;
use crate::core::evidence::{Evidence, EvidenceValue};

pub struct KernelModuleStatus;

impl Capability for KernelModuleStatus {
    fn id(&self) -> &'static str {
        "kernel.module_loaded"
    }
    fn label(&self) -> &'static str {
        "Zenvecha Kernel Module"
    }
    fn probe(&self) -> Evidence {
        let loaded = super::module_loaded();
        if loaded {
            // Verify at least one proc entry is readable
            let version_exists = read_proc("version.release").is_some();
            Evidence::present(self.id(), EvidenceValue::Bool(version_exists))
        } else {
            Evidence::missing(self.id(), EvidenceValue::Bool(false))
        }
    }
}

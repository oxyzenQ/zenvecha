// Copyright (C) 2026 rezky_nightky
// SPDX-License-Identifier: GPL-3.0-only

//! Kernel module presence probe.
//!
//! Uses: kernel_status()

use crate::core::capability::Capability;
use crate::core::caps::kernel_cap::kernel_status;
use crate::core::evidence::Evidence;

pub struct KernelModuleStatus;

impl Capability for KernelModuleStatus {
    fn id(&self) -> &'static str {
        "kernel.module_loaded"
    }
    fn label(&self) -> &'static str {
        "Zenvecha Kernel Module"
    }
    fn probe(&self) -> Evidence {
        kernel_status(self.id(), "version.release")
    }
}

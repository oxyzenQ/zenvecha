// Copyright (C) 2026 rezky_nightky
// SPDX-License-Identifier: GPL-3.0-only

//! Kernel version probe.
//!
//! Uses: kernel_text()

use crate::core::capability::Capability;
use crate::core::caps::kernel_cap::kernel_text;
use crate::core::evidence::Evidence;

pub struct KernelVersionFromModule;

impl Capability for KernelVersionFromModule {
    fn id(&self) -> &'static str {
        "kernel.version.module"
    }
    fn label(&self) -> &'static str {
        "Kernel Version (module)"
    }
    fn probe(&self) -> Evidence {
        kernel_text(self.id(), "version.release")
    }
}

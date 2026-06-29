// Copyright (C) 2026 rezky_nightky
// SPDX-License-Identifier: GPL-3.0-only

//! Module loader probe.
//!
//! Uses: kernel_text()

use crate::core::capability::Capability;
use crate::core::caps::kernel_cap::kernel_text;
use crate::core::evidence::Evidence;

pub struct KernelModuleLoader;

impl Capability for KernelModuleLoader {
    fn id(&self) -> &'static str {
        "kernel.loader.module"
    }
    fn label(&self) -> &'static str {
        "Module Loader (module)"
    }
    fn probe(&self) -> Evidence {
        kernel_text(self.id(), "modules.loader")
    }
}

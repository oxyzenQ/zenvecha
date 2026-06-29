// Copyright (C) 2026 rezky_nightky
// SPDX-License-Identifier: GPL-3.0-only

//! BTF (BPF Type Format) probe.
//!
//! Uses: kernel_bool()

use crate::core::capability::Capability;
use crate::core::caps::kernel_cap::kernel_bool;
use crate::core::evidence::Evidence;

pub struct KernelBtfStatus;

impl Capability for KernelBtfStatus {
    fn id(&self) -> &'static str {
        "kernel.btf.module"
    }
    fn label(&self) -> &'static str {
        "BTF Status (module)"
    }
    fn probe(&self) -> Evidence {
        kernel_bool(self.id(), "btf.available")
    }
}

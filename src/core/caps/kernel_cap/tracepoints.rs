// Copyright (C) 2026 rezky_nightky
// SPDX-License-Identifier: GPL-3.0-only

//! Tracepoint capability probes.
//!
//! Uses: kernel_text()

use crate::core::capability::Capability;
use crate::core::caps::kernel_cap::kernel_text;
use crate::core::evidence::Evidence;

/// Total tracepoint count available in the kernel.
pub struct KernelTracepointCount;

impl Capability for KernelTracepointCount {
    fn id(&self) -> &'static str {
        "kernel.tracepoints.count"
    }
    fn label(&self) -> &'static str {
        "Tracepoint Count (module)"
    }
    fn probe(&self) -> Evidence {
        // /proc/zenvecha/tracepoints/count → e.g. "1427"
        kernel_text(self.id(), "tracepoints.count")
    }
}

/// Tracepoint subsystems available (e.g. "sched,block,net,irq").
pub struct KernelTracepointSubsystems;

impl Capability for KernelTracepointSubsystems {
    fn id(&self) -> &'static str {
        "kernel.tracepoints.subsystems"
    }
    fn label(&self) -> &'static str {
        "Tracepoint Subsystems (module)"
    }
    fn probe(&self) -> Evidence {
        // /proc/zenvecha/tracepoints/subsystems → "sched,block,net,..."
        kernel_text(self.id(), "tracepoints.subsystems")
    }
}

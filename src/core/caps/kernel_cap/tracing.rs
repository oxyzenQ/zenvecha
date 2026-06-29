// Copyright (C) 2026 rezky_nightky
// SPDX-License-Identifier: GPL-3.0-only

//! Tracing infrastructure probes.
//!
//! Uses: kernel_bool()

use crate::core::capability::Capability;
use crate::core::caps::kernel_cap::kernel_bool;
use crate::core::evidence::Evidence;

pub struct KernelTracingFtrace;

impl Capability for KernelTracingFtrace {
    fn id(&self) -> &'static str {
        "kernel.tracing.ftrace"
    }
    fn label(&self) -> &'static str {
        "ftrace (kernel)"
    }
    fn probe(&self) -> Evidence {
        kernel_bool(self.id(), "tracing.ftrace")
    }
}

pub struct KernelTracingKprobes;

impl Capability for KernelTracingKprobes {
    fn id(&self) -> &'static str {
        "kernel.tracing.kprobes"
    }
    fn label(&self) -> &'static str {
        "kprobes (kernel)"
    }
    fn probe(&self) -> Evidence {
        kernel_bool(self.id(), "tracing.kprobes")
    }
}

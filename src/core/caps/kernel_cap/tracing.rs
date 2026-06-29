// Copyright (C) 2026 rezky_nightky
// SPDX-License-Identifier: GPL-3.0-only

//! Tracing infrastructure probes — reads from Zenvecha kernel module.
//!
//! Proc keys: tracing.ftrace, tracing.kprobes

use crate::core::capability::Capability;
use crate::core::caps::kernel_cap::read_proc;
use crate::core::evidence::{Evidence, EvidenceValue};

/// ftrace availability from kernel module.
pub struct KernelTracingFtrace;

impl Capability for KernelTracingFtrace {
    fn id(&self) -> &'static str {
        "kernel.tracing.ftrace"
    }
    fn label(&self) -> &'static str {
        "ftrace (kernel)"
    }
    fn probe(&self) -> Evidence {
        let val = read_proc("tracing.ftrace")
            .map(|s| s == "available")
            .unwrap_or(false);
        Evidence::present(self.id(), EvidenceValue::Bool(val))
    }
}

/// kprobes availability from kernel module.
pub struct KernelTracingKprobes;

impl Capability for KernelTracingKprobes {
    fn id(&self) -> &'static str {
        "kernel.tracing.kprobes"
    }
    fn label(&self) -> &'static str {
        "kprobes (kernel)"
    }
    fn probe(&self) -> Evidence {
        let val = read_proc("tracing.kprobes")
            .map(|s| s == "available")
            .unwrap_or(false);
        Evidence::present(self.id(), EvidenceValue::Bool(val))
    }
}

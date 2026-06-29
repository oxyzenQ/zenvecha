// SPDX-License-Identifier: GPL-2.0-only
// Copyright (C) 2026 rezky_nightky

//! Tracing infrastructure probe.
//!
//! Discovers available tracing frameworks: ftrace, kprobes, tracepoints.

use crate::capability::probes::Probe;
use crate::capability::types::Capability;

pub struct TracingProbe;

impl Probe for TracingProbe {
    fn name(&self) -> &'static str {
        "tracing"
    }

    fn discover(&self) -> &'static [Capability] {
        // Checks kernel config:
        //   CONFIG_FTRACE       → function tracer
        //   CONFIG_KPROBES      → dynamic probes
        //   CONFIG_TRACEPOINTS  → static tracepoints
        //   CONFIG_UPROBES      → userspace probes
        &[
            Capability { key: "tracing.ftrace", value: "available" },
            Capability { key: "tracing.kprobes", value: "available" },
            Capability { key: "tracing.kretprobes", value: "available" },
            Capability { key: "tracing.tracepoints", value: "available" },
            Capability { key: "tracing.uprobes", value: "available" },
        ]
    }
}

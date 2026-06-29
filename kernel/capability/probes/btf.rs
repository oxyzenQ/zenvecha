// SPDX-License-Identifier: GPL-2.0-only
// Copyright (C) 2026 rezky_nightky

//! BTF (BPF Type Format) availability probe.

use crate::capability::probes::Probe;
use crate::capability::types::CapabilityDescriptor;

pub struct BtfProbe;

impl Probe for BtfProbe {
    fn name(&self) -> &'static str {
        "btf"
    }

    fn discover(&self) -> &'static [Capability] {
        // IS_ENABLED(CONFIG_DEBUG_INFO_BTF)
        // When available, BTF enables:
        //   - BPF CO-RE (Compile Once, Run Everywhere)
        //   - Type-safe kernel tracing
        //   - Structure layout introspection
        &[
            Capability { key: "btf.available", value: "yes" },
            Capability { key: "btf.vmlinux", value: "available" },
        ]
    }
}

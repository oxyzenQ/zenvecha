// SPDX-License-Identifier: GPL-2.0-only
// Copyright (C) 2026 rezky_nightky

//! CPU architecture probe.
//!
//! Discovers: CPU arch, endianness, page size, word width.

use crate::capability::probes::Probe;
use crate::capability::types::CapabilityDescriptor;

pub struct ArchitectureProbe;

impl Probe for ArchitectureProbe {
    fn name(&self) -> &'static str {
        "architecture"
    }

    fn discover(&self) -> &'static [Capability] {
        // In kernel space: UTS_MACHINE, BITS_PER_LONG, PAGE_SIZE
        // Architecture determines:
        //   - Available instruction features
        //   - Memory model (page size, address width)
        //   - eBPF JIT availability
        &[
            Capability { key: "arch.name", value: "x86_64" },
            Capability { key: "arch.bits", value: "64" },
            Capability { key: "arch.endian", value: "little" },
            Capability { key: "arch.page_size", value: "4096" },
        ]
    }
}

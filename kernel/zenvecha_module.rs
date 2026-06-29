// SPDX-License-Identifier: GPL-2.0-only
// Copyright (C) 2026 rezky_nightky

//! Zenvecha Kernel Module — Wolfzenix Kernel Capability Platform.
//!
//! ## Contract
//!
//! This module discovers kernel capabilities and exposes them as
//! structured key=value pairs via /proc/zenvecha/*.
//!
//! **The module NEVER:**
//!   - Renders human-readable output
//!   - Computes scores or recommendations
//!   - Makes decisions or performs reasoning
//!   - Knows anything about the CLI or pipeline
//!
//! **The module ONLY:**
//!   - Discovers facts about the running kernel
//!   - Exposes structured data via proc filesystem
//!
//! ## Two-Layer Type System
//!
//!   Compile-time:  Probe → discover() → &'static [CapabilityDescriptor]
//!   Runtime:       Probe → run() → ProbeResult (dynamically allocated)
//!
//! The compile-time skeleton (current) defines the contract.
//! The runtime implementation uses kernel allocators for dynamic data.
//!
//! ## Data Flow
//!
//!   Kernel Space:
//!     probes/* → all_probes() → /proc/zenvecha/{probe}/{key}
//!   Userspace:
//!     kernel_cap/{domain}.rs → Evidence → Pipeline → Render
//!
//! ## Future Providers
//!
//!   Supported without modification:
//!     livepatch, tracepoints, ftrace, kprobes, scheduler, memory,
//!     security, eBPF, module verifier, ABI inspector

#![no_std]

use kernel::prelude::*;
use kernel::printk;

mod capability;

module! {
    type: ZenvechaModule,
    name: "zenvecha",
    author: "rezky_nightky",
    description: "Wolfzenix Kernel Capability Discovery — structured kernel facts",
    license: "GPL",
}

struct ZenvechaModule;

impl kernel::Module for ZenvechaModule {
    fn init(_module: &'static ThisModule) -> Result<Self> {
        printk!("zenvecha: Wolfzenix kernel capability discovery loaded\n");

        // Discover all capabilities via the probe registry.
        // Each probe's descriptors define the proc entries to create.
        //
        // Runtime behavior (in a real kernel build):
        //   for probe in capability::probes::all_probes() {
        //       let dir = proc_mkdir(probe.name(), zenvecha_proc_root);
        //       for desc in probe.discover() {
        //           // Create /proc/zenvecha/{probe}/{key}
        //           // populated with ProbeResult values at read time
        //       }
        //   }

        let probes = capability::probes::all_probes();
        let total_descriptors: usize = probes.iter().map(|p| p.discover().len()).sum();

        printk!(
            "zenvecha: {} probes loaded, {} capability descriptors defined\n",
            probes.len(),
            total_descriptors,
        );

        Ok(ZenvechaModule)
    }
}

impl Drop for ZenvechaModule {
    fn drop(&mut self) {
        printk!("zenvecha: module unloaded\n");
    }
}

// SPDX-License-Identifier: GPL-2.0-only
// Copyright (C) 2026 rezky_nightky

//! Zenvecha Kernel Module — Wolfzenix Kernel Capability Platform.
//!
//! This module discovers kernel capabilities and exposes them
//! as structured facts to userspace via proc filesystem.
//!
//! The module NEVER:
//!   - Renders human-readable output
//!   - Computes scores or recommendations
//!   - Makes decisions
//!   - Knows anything about CLI or pipeline
//!
//! The module ONLY:
//!   - Discovers facts about the running kernel
//!   - Exposes structured key=value pairs
//!
//! Architecture:
//!   Kernel Space:
//!     zenvecha_module → capability/probes/* → /proc/zenvecha/*
//!   Userspace:
//!     caps/kernel_cap.rs → Evidence → Pipeline → Render

#![no_std]

use kernel::prelude::*;
use kernel::printk;

mod capability;

module! {
    type: ZenvechaModule,
    name: "zenvecha",
    author: "rezky_nightky",
    description: "Wolfzenix Kernel Capability Discovery",
    license: "GPL",
}

struct ZenvechaModule;

impl kernel::Module for ZenvechaModule {
    fn init(_module: &'static ThisModule) -> Result<Self> {
        printk!(
            "zenvecha: Wolfzenix kernel capability discovery loaded\n"
        );

        // Discover all capabilities via the probe registry.
        // In a full implementation, each probe's output is registered
        // as a proc file under /proc/zenvecha/<probe_name>.
        //
        // Example:
        //   for probe in capability::probes::all_probes() {
        //       create_proc_entry(probe.name(), probe.discover());
        //   }
        //
        // This provides: /proc/zenvecha/version, /proc/zenvecha/symbols,
        // /proc/zenvecha/kallsyms, /proc/zenvecha/btf, etc.

        let probes = capability::probes::all_probes();
        let total_caps: usize = probes.iter().map(|p| p.discover().len()).sum();

        printk!(
            "zenvecha: {} probes loaded, {} total capability facts\n",
            probes.len(),
            total_caps,
        );

        Ok(ZenvechaModule)
    }
}

impl Drop for ZenvechaModule {
    fn drop(&mut self) {
        printk!("zenvecha: module unloaded\n");
    }
}

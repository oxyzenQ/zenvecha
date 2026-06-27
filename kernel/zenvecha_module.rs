// Copyright (C) 2026 rezky_nightky
// SPDX-License-Identifier: GPL-3.0-only
//
// Zenvecha kernel module — Phase 1 "Kernel Hello"
//
// Loads, prints a message, unloads cleanly.
// Built using the Linux kernel's Rust for Linux (R4L) infrastructure.
//
// Compilation requires:
//   - Linux kernel 6.x with CONFIG_RUST=y
//   - Rust toolchain matching the kernel's required version
//
// See docs/testing-vm.md for VM testing instructions.

#![no_std]
#![feature(allocator_api)]

use kernel::prelude::*;

module! {
    type: ZenvechaModule,
    name: "zenvecha",
    author: "rezky_nightky (oxyzenQ)",
    description: "Zenvecha — Safe runtime kernel patching research",
    license: "GPL",
}

struct ZenvechaModule;

impl kernel::Module for ZenvechaModule {
    fn init(_module: &'static ThisModule) -> Result<Self> {
        pr_info!("zenvecha loaded\n");
        Ok(ZenvechaModule)
    }
}

impl Drop for ZenvechaModule {
    fn drop(&mut self) {
        pr_info!("zenvecha unloaded\n");
    }
}

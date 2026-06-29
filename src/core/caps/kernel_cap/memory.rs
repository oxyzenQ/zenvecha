// Copyright (C) 2026 rezky_nightky
// SPDX-License-Identifier: GPL-3.0-only

//! Memory subsystem probes.
//!
//! Uses: kernel_text()

use crate::core::capability::Capability;
use crate::core::caps::kernel_cap::kernel_text;
use crate::core::evidence::Evidence;

/// Page size in bytes.
pub struct KernelPageSize;

impl Capability for KernelPageSize {
    fn id(&self) -> &'static str {
        "kernel.memory.page_size"
    }
    fn label(&self) -> &'static str {
        "Page Size (module)"
    }
    fn probe(&self) -> Evidence {
        kernel_text(self.id(), "memory.page_size")
    }
}

/// Huge page sizes available.
pub struct KernelHugePages;

impl Capability for KernelHugePages {
    fn id(&self) -> &'static str {
        "kernel.memory.hugepages"
    }
    fn label(&self) -> &'static str {
        "Huge Pages (module)"
    }
    fn probe(&self) -> Evidence {
        // /proc/zenvecha/memory/hugepages → "2M,1G" or empty
        kernel_text(self.id(), "memory.hugepages")
    }
}

/// Memory model (e.g., SPARSEMEM, FLATMEM).
pub struct KernelMemoryModel;

impl Capability for KernelMemoryModel {
    fn id(&self) -> &'static str {
        "kernel.memory.model"
    }
    fn label(&self) -> &'static str {
        "Memory Model (module)"
    }
    fn probe(&self) -> Evidence {
        kernel_text(self.id(), "memory.model")
    }
}

// Copyright (C) 2026 rezky_nightky
// SPDX-License-Identifier: GPL-3.0-only

//! Scheduler capability probes.
//!
//! Uses: kernel_text(), kernel_bool()

use crate::core::capability::Capability;
use crate::core::caps::kernel_cap::kernel_text;
use crate::core::evidence::Evidence;

/// Available scheduling classes (CFS, RT, Deadline).
pub struct KernelSchedulerClasses;

impl Capability for KernelSchedulerClasses {
    fn id(&self) -> &'static str {
        "kernel.scheduler.classes"
    }
    fn label(&self) -> &'static str {
        "Scheduler Classes (module)"
    }
    fn probe(&self) -> Evidence {
        // /proc/zenvecha/scheduler/classes → e.g. "cfs,rt,deadline"
        kernel_text(self.id(), "scheduler.classes")
    }
}

/// Preemption model (none, voluntary, full).
pub struct KernelPreemption;

impl Capability for KernelPreemption {
    fn id(&self) -> &'static str {
        "kernel.scheduler.preemption"
    }
    fn label(&self) -> &'static str {
        "Preemption Model (module)"
    }
    fn probe(&self) -> Evidence {
        // /proc/zenvecha/scheduler/preemption → "voluntary" | "full" | "none"
        kernel_text(self.id(), "scheduler.preemption")
    }
}

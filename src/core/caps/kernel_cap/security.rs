// Copyright (C) 2026 rezky_nightky
// SPDX-License-Identifier: GPL-3.0-only

//! Kernel security probes — lockdown, LSMs.
//!
//! Uses: kernel_text(), kernel_bool()

use crate::core::capability::Capability;
use crate::core::caps::kernel_cap::{kernel_bool, kernel_text};
use crate::core::evidence::Evidence;

/// Kernel lockdown mode (integrity, confidentiality, none).
pub struct KernelLockdown;

impl Capability for KernelLockdown {
    fn id(&self) -> &'static str {
        "kernel.security.lockdown"
    }
    fn label(&self) -> &'static str {
        "Kernel Lockdown (module)"
    }
    fn probe(&self) -> Evidence {
        // /proc/zenvecha/security/lockdown → "none" | "integrity" | "confidentiality"
        kernel_text(self.id(), "security.lockdown")
    }
}

/// Active Linux Security Modules.
pub struct KernelActiveLsms;

impl Capability for KernelActiveLsms {
    fn id(&self) -> &'static str {
        "kernel.security.lsms"
    }
    fn label(&self) -> &'static str {
        "Active LSMs (module)"
    }
    fn probe(&self) -> Evidence {
        // /proc/zenvecha/security/lsms → "selinux,apparmor,bpf" etc.
        kernel_text(self.id(), "security.lsms")
    }
}

/// Kernel Address Space Layout Randomization.
pub struct KernelKaslr;

impl Capability for KernelKaslr {
    fn id(&self) -> &'static str {
        "kernel.security.kaslr"
    }
    fn label(&self) -> &'static str {
        "KASLR Status (module)"
    }
    fn probe(&self) -> Evidence {
        kernel_bool(self.id(), "security.kaslr")
    }
}

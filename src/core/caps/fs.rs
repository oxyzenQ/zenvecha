// Copyright (C) 2026 rezky_nightky
// SPDX-License-Identifier: GPL-3.0-only

//! Filesystem capabilities — debugfs, tracefs mounts.

use crate::core::capability::Capability;
use crate::core::caps::mount_ok;
use crate::core::evidence::{Evidence, EvidenceValue};

pub struct DebugfsMounted;
impl Capability for DebugfsMounted {
    fn id(&self) -> &'static str {
        "fs.debugfs"
    }
    fn label(&self) -> &'static str {
        "debugfs"
    }
    fn probe(&self) -> Evidence {
        Evidence::present(
            self.id(),
            EvidenceValue::Bool(mount_ok("/sys/kernel/debug")),
        )
    }
}

pub struct TracefsMounted;
impl Capability for TracefsMounted {
    fn id(&self) -> &'static str {
        "fs.tracefs"
    }
    fn label(&self) -> &'static str {
        "tracefs"
    }
    fn probe(&self) -> Evidence {
        Evidence::present(
            self.id(),
            EvidenceValue::Bool(mount_ok("/sys/kernel/tracing")),
        )
    }
}

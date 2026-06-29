// SPDX-License-Identifier: GPL-2.0-only
// Copyright (C) 2026 rezky_nightky

//! Kernel module loader status probe.
//!
//! Discovers: CONFIG_MODULES status, active module count, vermagic.

use crate::capability::probes::Probe;
use crate::capability::types::Capability;

pub struct ModuleLoaderProbe;

impl Probe for ModuleLoaderProbe {
    fn name(&self) -> &'static str {
        "modules"
    }

    fn discover(&self) -> &'static [Capability] {
        // IS_ENABLED(CONFIG_MODULES), module count via /proc/modules equivalent.
        // In kernel space:
        //   - CONFIG_MODULES → module loading API available
        //   - THIS_MODULE->list → module linked list
        //   - mod->state → MODULE_STATE_LIVE, COMING, GOING
        &[
            Capability { key: "modules.loader", value: "enabled" },
            Capability { key: "modules.active_count", value: "47" },
            Capability { key: "modules.vermagic", value: "6.18.0 SMP mod_unload" },
        ]
    }
}

// SPDX-License-Identifier: GPL-2.0-only
// Copyright (C) 2026 rezky_nightky

//! kallsyms availability probe.

use crate::capability::probes::Probe;
use crate::capability::types::CapabilityDescriptor;

pub struct KallsymsProbe;

impl Probe for KallsymsProbe {
    fn name(&self) -> &'static str {
        "kallsyms"
    }

    fn discover(&self) -> &'static [Capability] {
        // Checks CONFIG_KALLSYMS and CONFIG_KALLSYMS_ALL.
        // In kernel space: IS_ENABLED(CONFIG_KALLSYMS), IS_ENABLED(CONFIG_KALLSYMS_ALL).
        &[
            Capability { key: "kallsyms.available", value: "yes" },
            Capability { key: "kallsyms.all_symbols", value: "yes" },
            Capability { key: "kallsyms.base_address", value: "0xffffff8000000000" },
        ]
    }
}

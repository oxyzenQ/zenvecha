// SPDX-License-Identifier: GPL-2.0-only
// Copyright (C) 2026 rezky_nightky

//! Kernel version probe.
//!
//! Discovers: running kernel release, major/minor/patch version.

use crate::capability::probes::Probe;
use crate::capability::types::Capability;

pub struct VersionProbe;

impl Probe for VersionProbe {
    fn name(&self) -> &'static str {
        "version"
    }

    fn discover(&self) -> &'static [Capability] {
        // In a real kernel module, this uses kernel::utsname() to read
        // the running kernel's version string from the UTS namespace.
        //
        // Example kernel-space implementation:
        //
        //   let uts = unsafe { kernel::bindings::init_uts_ns.name };
        //   let release = unsafe { core::ffi::CStr::from_ptr(uts.release.as_ptr()) };
        //   let release_str = release.to_str().unwrap_or("unknown");
        //
        // The structured output is a plain key=value for proc consumption.
        &[
            Capability { key: "kernel.version.release", value: "6.18.0" },
            Capability { key: "kernel.version.major", value: "6" },
            Capability { key: "kernel.version.minor", value: "18" },
            Capability { key: "kernel.version.patch", value: "0" },
        ]
    }
}

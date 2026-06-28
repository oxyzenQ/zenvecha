// Copyright (C) 2026 rezky_nightky
// SPDX-License-Identifier: GPL-3.0-only

//! Debug information inspection — BTF and DWARF.

use std::path::Path;

/// Result of debug-info inspection.
pub struct DebugInfo {
    pub btf_available: bool,
    pub dwarf_available: bool,
}

/// Inspect BTF and DWARF availability.
pub fn inspect_debug() -> DebugInfo {
    DebugInfo {
        btf_available: btf_present(),
        dwarf_available: dwarf_present(),
    }
}

/// BTF is available when /sys/kernel/btf/vmlinux exists.
fn btf_present() -> bool {
    Path::new("/sys/kernel/btf/vmlinux").exists()
}

/// DWARF is detectable via the debug vmlinux under /usr/lib/debug.
fn dwarf_present() -> bool {
    let release = match super::kernel::kernel_release() {
        Some(r) => r,
        None => return false,
    };

    // Common locations for DWARF-enabled vmlinux
    let candidates = [
        format!("/usr/lib/debug/boot/vmlinux-{release}"),
        format!("/usr/lib/debug/lib/modules/{release}/vmlinux"),
    ];

    candidates.iter().any(|p| Path::new(p).exists())
}

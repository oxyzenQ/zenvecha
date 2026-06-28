// Copyright (C) 2026 rezky_nightky
// SPDX-License-Identifier: GPL-3.0-only

//! Doctor renderer — formats system readiness check output.
//!
//! Accepts already-collected Evidence. Never inspects the system.
//! Only formatting.

use std::io::{self, Write};

use crate::core::evidence::Evidence;
use crate::core::evidence_helpers;

/// Render doctor output from collected evidence.
pub fn render(
    evidence: &[Evidence],
    out: &mut io::StdoutLock<'_>,
    fix_mode: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    writeln!(out, "Zenvecha Doctor")?;
    writeln!(out)?;

    let mut all_ok = true;

    let cfg_ok = evidence_helpers::ev_text_value(evidence, "config.source").is_some();
    all_ok &= check(
        out,
        "Kernel config readable",
        cfg_ok,
        if fix_mode && !cfg_ok {
            "Ensure /boot/config-$(uname -r) exists or CONFIG_IKCONFIG_PROC is enabled"
        } else {
            "No kernel config found — some checks will be limited"
        },
    )?;

    let release = evidence_helpers::ev_s(evidence, "kernel.release");
    let rel_ok = release != "Unknown";
    all_ok &= check(
        out,
        "Kernel release detected",
        rel_ok,
        "Cannot determine running kernel version",
    )?;

    let arch_ok = evidence_helpers::ev_s(evidence, "kernel.arch") != "Unknown";
    all_ok &= check(
        out,
        "Architecture detected",
        arch_ok,
        "Cannot determine system architecture",
    )?;

    let ks_ok = evidence_helpers::ev_status_is(evidence, "symbols.kallsyms", "readable")
        || evidence_helpers::ev_status_is(evidence, "symbols.kallsyms", "readable (root)");
    all_ok &= check(
        out,
        "/proc/kallsyms readable",
        ks_ok,
        if fix_mode && !ks_ok {
            "Run: echo 0 > /proc/sys/kernel/kptr_restrict (as root)"
        } else {
            "Symbol table not readable — symbol analysis unavailable"
        },
    )?;

    let mod_dir = if release != "Unknown" {
        std::path::Path::new(&format!("/lib/modules/{release}")).is_dir()
    } else {
        false
    };
    all_ok &= check(
        out,
        "Modules directory exists",
        mod_dir,
        "Cannot inspect kernel modules",
    )?;

    all_ok &= check(
        out,
        "debugfs mounted",
        evidence_helpers::ev_bool(evidence, "fs.debugfs"),
        if fix_mode && !evidence_helpers::ev_bool(evidence, "fs.debugfs") {
            "Run: mount -t debugfs none /sys/kernel/debug (as root)"
        } else {
            "debugfs not mounted — some debug info unavailable"
        },
    )?;

    all_ok &= check(
        out,
        "tracefs mounted",
        evidence_helpers::ev_bool(evidence, "fs.tracefs"),
        if fix_mode && !evidence_helpers::ev_bool(evidence, "fs.tracefs") {
            "Run: mount -t tracefs none /sys/kernel/tracing (as root)"
        } else {
            "tracefs not mounted — tracing features unavailable"
        },
    )?;

    all_ok &= check(
        out,
        "BTF available",
        evidence_helpers::ev_bool(evidence, "debug.btf"),
        "BTF not available — some introspection features limited",
    )?;

    writeln!(out)?;
    if all_ok {
        writeln!(
            out,
            "All checks passed. System is ready for kernel development."
        )?;
    } else {
        writeln!(out, "Some checks failed. Review the output above.")?;
        if !fix_mode {
            writeln!(out, "Run 'zenvecha doctor --fix' for remediation guidance.")?;
        }
    }

    Ok(())
}

fn check(out: &mut io::StdoutLock<'_>, label: &str, ok: bool, hint: &str) -> io::Result<bool> {
    let mark = if ok { "✔" } else { "✘" };
    writeln!(out, "  {mark} {label}")?;
    if !ok {
        writeln!(out, "    → {hint}")?;
    }
    Ok(ok)
}

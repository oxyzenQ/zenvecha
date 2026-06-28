// Copyright (C) 2026 rezky_nightky
// SPDX-License-Identifier: GPL-3.0-only

//! Doctor command — system readiness check with optional --fix mode.
//!
//! Thin orchestrator. All probes from Registry, rendering only.

use std::io::{self, Write};

use crate::core::capability::Registry;
use crate::core::evidence::{Evidence, EvidenceValue};

pub fn run(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let fix_mode = args.iter().any(|a| a == "--fix");

    let reg = Registry::default();
    let evidence = reg.run_all();

    let stdout = io::stdout();
    let mut out = stdout.lock();

    render(&evidence, &mut out, fix_mode)
}

fn render(
    evidence: &[Evidence],
    out: &mut io::StdoutLock<'_>,
    fix_mode: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    writeln!(out, "Zenvecha Doctor")?;
    writeln!(out)?;

    let mut all_ok = true;

    // Check 1: Kernel config access
    let cfg_ok = ev_text_value(evidence, "config.source").is_some();
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

    // Check 2: Kernel release
    let release = ev_s(evidence, "kernel.release");
    let rel_ok = release != "Unknown";
    all_ok &= check(
        out,
        "Kernel release detected",
        rel_ok,
        "Cannot determine running kernel version",
    )?;

    // Check 3: Architecture
    let arch_ok = ev_s(evidence, "kernel.arch") != "Unknown";
    all_ok &= check(
        out,
        "Architecture detected",
        arch_ok,
        "Cannot determine system architecture",
    )?;

    // Check 4: kallsyms
    let ks_ok = ev_status_is(evidence, "symbols.kallsyms", "readable")
        || ev_status_is(evidence, "symbols.kallsyms", "readable (root)");
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

    // Check 5: Modules directory
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

    // Check 6: Debugfs
    all_ok &= check(
        out,
        "debugfs mounted",
        ev_bool(evidence, "fs.debugfs"),
        if fix_mode && !ev_bool(evidence, "fs.debugfs") {
            "Run: mount -t debugfs none /sys/kernel/debug (as root)"
        } else {
            "debugfs not mounted — some debug info unavailable"
        },
    )?;

    // Check 7: tracefs
    all_ok &= check(
        out,
        "tracefs mounted",
        ev_bool(evidence, "fs.tracefs"),
        if fix_mode && !ev_bool(evidence, "fs.tracefs") {
            "Run: mount -t tracefs none /sys/kernel/tracing (as root)"
        } else {
            "tracefs not mounted — tracing features unavailable"
        },
    )?;

    // Check 8: BTF
    all_ok &= check(
        out,
        "BTF available",
        ev_bool(evidence, "debug.btf"),
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

/* helpers */

fn ev_s(evidence: &[Evidence], id: &str) -> String {
    evidence
        .iter()
        .find(|e| e.id == id)
        .map_or_else(|| "Unknown".into(), |e| e.value.display())
}

fn ev_bool(evidence: &[Evidence], id: &str) -> bool {
    evidence
        .iter()
        .find(|e| e.id == id)
        .is_some_and(|e| match &e.value {
            EvidenceValue::Bool(b) => *b,
            EvidenceValue::Config(cv) => cv.is_enabled(),
            _ => false,
        })
}

fn ev_text_value(evidence: &[Evidence], id: &str) -> Option<String> {
    evidence
        .iter()
        .find(|e| e.id == id)
        .and_then(|e| match &e.value {
            EvidenceValue::Text(Some(s)) => Some(s.clone()),
            EvidenceValue::Literal(s) => Some(s.clone()),
            _ => None,
        })
}

fn ev_status_is(evidence: &[Evidence], id: &str, expected: &str) -> bool {
    evidence
        .iter()
        .find(|e| e.id == id)
        .is_some_and(|e| match &e.value {
            EvidenceValue::Status(s) => *s == expected,
            _ => false,
        })
}

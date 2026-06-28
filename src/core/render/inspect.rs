// Copyright (C) 2026 rezky_nightky
// SPDX-License-Identifier: GPL-3.0-only

//! Inspect renderer — formats kernel capability discovery output.
//!
//! Accepts already-collected Evidence. Never inspects the system.
//! Only formatting.

use std::io::{self, Write};

use crate::core::evidence::Evidence;
use crate::core::evidence_helpers;

/// Render inspect output from collected evidence.
pub fn render(
    evidence: &[Evidence],
    out: &mut io::StdoutLock<'_>,
) -> Result<(), Box<dyn std::error::Error>> {
    writeln!(out, "Zenvecha Inspect")?;
    writeln!(out)?;

    // Kernel identity
    writeln!(out, "Kernel")?;
    print_kv(
        out,
        "  Version",
        &evidence_helpers::ev_s(evidence, "kernel.release"),
    )?;
    print_kv(
        out,
        "  Architecture",
        &evidence_helpers::ev_s(evidence, "kernel.arch"),
    )?;
    print_kv(
        out,
        "  Distribution",
        &evidence_helpers::ev_s(evidence, "kernel.distro"),
    )?;
    {
        let rustc = evidence_helpers::ev_bool(evidence, "toolchain.rustc");
        if rustc {
            writeln!(out, "  Rust compiler : installed")?;
        } else {
            writeln!(out, "  Rust compiler : not found")?;
        }
    }
    writeln!(out)?;

    // Configuration
    writeln!(out, "Configuration")?;
    {
        let src = evidence_helpers::ev_text_value(evidence, "config.source");
        if let Some(s) = src {
            writeln!(out, "  Source : {s}")?;
        } else {
            writeln!(out, "  Source : not available")?;
        }
    }
    writeln!(out)?;

    let cfg_keys = [
        ("MODULES", "config.MODULES"),
        ("MODULE_SIG", "config.MODULE_SIG"),
        ("KALLSYMS", "config.KALLSYMS"),
        ("KALLSYMS_ALL", "config.KALLSYMS_ALL"),
        ("BPF", "config.BPF"),
        ("DEBUG_INFO_BTF", "config.DEBUG_INFO_BTF"),
        ("RUST", "config.RUST"),
        ("RUST_IS_AVAILABLE", "config.RUST_IS_AVAILABLE"),
        ("LIVEPATCH", "config.LIVEPATCH"),
    ];
    let cfg_available = evidence_helpers::ev_text_value(evidence, "config.source").is_some();
    for (key, id) in &cfg_keys {
        let val = evidence_helpers::ev_config_label(evidence, id, cfg_available);
        writeln!(out, "  CONFIG_{key:.<30} {val}")?;
    }
    writeln!(out)?;

    // Module environment
    writeln!(out, "Module Environment")?;
    print_kv(
        out,
        "  Running kernel",
        &evidence_helpers::ev_s(evidence, "kernel.release"),
    )?;
    print_kv(
        out,
        "  Installed headers",
        &evidence_helpers::ev_s(evidence, "build.headers"),
    )?;
    {
        let release = evidence_helpers::ev_s(evidence, "kernel.release");
        if release != "Unknown" {
            writeln!(out, "  Modules directory : /lib/modules/{release}")?;
        } else {
            writeln!(out, "  Modules directory : Unknown")?;
        }
    }
    print_bool(
        out,
        "  Build directory",
        evidence_helpers::ev_text_known(evidence, "build.dir"),
    )?;
    print_bool(
        out,
        "  Headers",
        evidence_helpers::ev_status_is(evidence, "build.headers", "Complete"),
    )?;
    {
        let sig = evidence_helpers::ev_bool(evidence, "config.MODULE_SIG");
        if sig {
            writeln!(out, "  Signing support : enabled")?;
        } else {
            writeln!(out, "  Signing support : disabled")?;
        }
    }
    writeln!(out)?;

    // Debug information
    writeln!(out, "Debug Information")?;
    print_bool(
        out,
        "  BTF",
        evidence_helpers::ev_bool(evidence, "debug.btf"),
    )?;
    print_bool(
        out,
        "  DWARF",
        evidence_helpers::ev_bool(evidence, "debug.dwarf"),
    )?;
    writeln!(out)?;

    // Symbol information
    writeln!(out, "Symbol Information")?;
    let ks = evidence_helpers::ev_status_value(evidence, "symbols.kallsyms");
    match ks.as_deref() {
        Some("readable") => writeln!(out, "  /proc/kallsyms : present, readable")?,
        Some("readable (root)") => {
            writeln!(out, "  /proc/kallsyms : present, readable (root only)")?
        }
        Some("permission denied") => {
            writeln!(out, "  /proc/kallsyms : present, permission denied")?
        }
        _ => writeln!(out, "  /proc/kallsyms : not found")?,
    }
    writeln!(out)?;

    // Capability summary
    writeln!(out, "Kernel Capability Summary")?;
    writeln!(out)?;

    let r4l_ok = evidence_helpers::ev_bool(evidence, "config.RUST")
        || evidence_helpers::ev_bool(evidence, "config.RUST_IS_AVAILABLE");
    let r4l_known = evidence_helpers::ev_config_known(evidence, "config.RUST")
        || evidence_helpers::ev_config_known(evidence, "config.RUST_IS_AVAILABLE");
    let mod_support = evidence_helpers::ev_bool(evidence, "config.MODULES");
    let modsig_ok = evidence_helpers::ev_bool(evidence, "config.MODULE_SIG");
    let modsig_known = evidence_helpers::ev_config_known(evidence, "config.MODULE_SIG");
    let btf_ok = evidence_helpers::ev_bool(evidence, "config.DEBUG_INFO_BTF")
        && evidence_helpers::ev_bool(evidence, "debug.btf");
    let btf_known = evidence_helpers::ev_config_known(evidence, "config.DEBUG_INFO_BTF");
    let lp_ok = evidence_helpers::ev_bool(evidence, "config.LIVEPATCH");
    let lp_known = evidence_helpers::ev_config_known(evidence, "config.LIVEPATCH");
    let ks_ok = evidence_helpers::ev_status_is(evidence, "symbols.kallsyms", "readable")
        || evidence_helpers::ev_status_is(evidence, "symbols.kallsyms", "readable (root)");

    print_tri(out, "Rust for Linux", r4l_ok, r4l_known)?;
    print_tri(out, "Modules", mod_support, mod_support)?;
    print_tri(out, "Module Signing", modsig_ok, modsig_known)?;
    print_tri(out, "BTF", btf_ok, btf_known)?;
    print_tri(out, "Livepatch", lp_ok, lp_known)?;
    print_tri(out, "Kallsyms", ks_ok, true)?;
    writeln!(out)?;

    // Suitable for
    let mod_dev_ok = mod_support
        && evidence_helpers::ev_status_is(evidence, "build.headers", "Complete")
        && evidence_helpers::ev_bool(evidence, "toolchain.gcc");
    writeln!(out, "Suitable for:")?;
    print_check(out, "module development", mod_dev_ok)?;
    print_check(out, "symbol analysis", ks_ok)?;
    print_check(out, "live patching", lp_ok && ks_ok && mod_support)?;

    Ok(())
}

fn print_kv(out: &mut io::StdoutLock<'_>, label: &str, value: &str) -> io::Result<()> {
    if value == "Unknown" || value.is_empty() {
        writeln!(out, "{label} : Unknown")
    } else {
        writeln!(out, "{label} : {value}")
    }
}

fn print_bool(out: &mut io::StdoutLock<'_>, label: &str, val: bool) -> io::Result<()> {
    writeln!(
        out,
        "{label} : {}",
        if val { "present" } else { "not present" }
    )
}

fn print_check(out: &mut io::StdoutLock<'_>, label: &str, ok: bool) -> io::Result<()> {
    let mark = if ok { "✔" } else { "✘" };
    writeln!(out, "  {mark} {label}")
}

fn print_tri(out: &mut io::StdoutLock<'_>, label: &str, ok: bool, known: bool) -> io::Result<()> {
    if !known {
        writeln!(out, "  ? {label}")
    } else if ok {
        writeln!(out, "  ✔ {label}")
    } else {
        writeln!(out, "  ✘ {label}")
    }
}

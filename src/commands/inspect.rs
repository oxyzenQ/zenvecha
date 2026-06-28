// Copyright (C) 2026 rezky_nightky
// SPDX-License-Identifier: GPL-3.0-only

//! Inspect command — kernel capability discovery.
//!
//! Orchestrates capability probes via Registry, renders results.
//! No business logic in this file — all logic lives in core/ and system/.

use std::io::{self, Write};

use crate::core::capability::Registry;
use crate::core::evidence::{Evidence, EvidenceValue};

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let reg = Registry::default();
    let evidence = reg.run_all();

    let stdout = io::stdout();
    let mut out = stdout.lock();

    render(&evidence, &mut out)
}

fn render(
    evidence: &[Evidence],
    out: &mut io::StdoutLock<'_>,
) -> Result<(), Box<dyn std::error::Error>> {
    writeln!(out, "Zenvecha Inspect")?;
    writeln!(out)?;

    // Kernel identity
    writeln!(out, "Kernel")?;
    print_kv(out, "  Version", &ev_s(evidence, "kernel.release"))?;
    print_kv(out, "  Architecture", &ev_s(evidence, "kernel.arch"))?;
    print_kv(out, "  Distribution", &ev_s(evidence, "kernel.distro"))?;
    {
        let rustc = ev_bool(evidence, "toolchain.rustc");
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
        let src = ev_text_value(evidence, "config.source");
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
    let cfg_available = ev_text_value(evidence, "config.source").is_some();
    for (key, id) in &cfg_keys {
        let val = ev_config_label(evidence, id, cfg_available);
        writeln!(out, "  CONFIG_{key:.<30} {val}")?;
    }
    writeln!(out)?;

    // Module environment
    writeln!(out, "Module Environment")?;
    print_kv(out, "  Running kernel", &ev_s(evidence, "kernel.release"))?;
    print_kv(out, "  Installed headers", &ev_s(evidence, "build.headers"))?;
    {
        let release = ev_s(evidence, "kernel.release");
        if release != "Unknown" {
            writeln!(out, "  Modules directory : /lib/modules/{release}")?;
        } else {
            writeln!(out, "  Modules directory : Unknown")?;
        }
    }
    print_bool(
        out,
        "  Build directory",
        ev_text_known(evidence, "build.dir"),
    )?;
    print_bool(
        out,
        "  Headers",
        ev_status_is(evidence, "build.headers", "Complete"),
    )?;
    {
        let sig = ev_bool(evidence, "config.MODULE_SIG");
        if sig {
            writeln!(out, "  Signing support : enabled")?;
        } else {
            writeln!(out, "  Signing support : disabled")?;
        }
    }
    writeln!(out)?;

    // Debug information
    writeln!(out, "Debug Information")?;
    print_bool(out, "  BTF", ev_bool(evidence, "debug.btf"))?;
    print_bool(out, "  DWARF", ev_bool(evidence, "debug.dwarf"))?;
    writeln!(out)?;

    // Symbol information
    writeln!(out, "Symbol Information")?;
    let ks = ev_status_value(evidence, "symbols.kallsyms");
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

    let r4l_ok = ev_bool(evidence, "config.RUST") || ev_bool(evidence, "config.RUST_IS_AVAILABLE");
    let r4l_known = ev_config_known(evidence, "config.RUST")
        || ev_config_known(evidence, "config.RUST_IS_AVAILABLE");
    let mod_support = ev_bool(evidence, "config.MODULES");
    let modsig_ok = ev_bool(evidence, "config.MODULE_SIG");
    let modsig_known = ev_config_known(evidence, "config.MODULE_SIG");
    let btf_ok = ev_bool(evidence, "config.DEBUG_INFO_BTF") && ev_bool(evidence, "debug.btf");
    let btf_known = ev_config_known(evidence, "config.DEBUG_INFO_BTF");
    let lp_ok = ev_bool(evidence, "config.LIVEPATCH");
    let lp_known = ev_config_known(evidence, "config.LIVEPATCH");
    let ks_ok = ev_status_is(evidence, "symbols.kallsyms", "readable")
        || ev_status_is(evidence, "symbols.kallsyms", "readable (root)");

    print_tri(out, "Rust for Linux", r4l_ok, r4l_known)?;
    print_tri(out, "Modules", mod_support, mod_support)?;
    print_tri(out, "Module Signing", modsig_ok, modsig_known)?;
    print_tri(out, "BTF", btf_ok, btf_known)?;
    print_tri(out, "Livepatch", lp_ok, lp_known)?;
    print_tri(out, "Kallsyms", ks_ok, true)?;
    writeln!(out)?;

    // Suitable for
    let mod_dev_ok = mod_support
        && ev_status_is(evidence, "build.headers", "Complete")
        && ev_bool(evidence, "toolchain.gcc");
    writeln!(out, "Suitable for:")?;
    print_check(out, "module development", mod_dev_ok)?;
    print_check(out, "symbol analysis", ks_ok)?;
    print_check(out, "live patching", lp_ok && ks_ok && mod_support)?;

    Ok(())
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
            EvidenceValue::Count(n) => *n > 0,
            _ => false,
        })
}

fn ev_config_known(evidence: &[Evidence], id: &str) -> bool {
    evidence
        .iter()
        .find(|e| e.id == id)
        .is_some_and(|e| match &e.value {
            EvidenceValue::Config(cv) => cv.is_known(),
            _ => false,
        })
}

fn ev_config_label(evidence: &[Evidence], id: &str, cfg_available: bool) -> String {
    evidence
        .iter()
        .find(|e| e.id == id)
        .map_or("Unknown".into(), |e| match &e.value {
            EvidenceValue::Config(cv) => cv.label(cfg_available).to_string(),
            v => v.display(),
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

fn ev_text_known(evidence: &[Evidence], id: &str) -> bool {
    evidence.iter().find(|e| e.id == id).is_some_and(|e| {
        matches!(
            &e.value,
            EvidenceValue::Text(Some(_)) | EvidenceValue::Path(Some(_))
        )
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

fn ev_status_value(evidence: &[Evidence], id: &str) -> Option<String> {
    evidence
        .iter()
        .find(|e| e.id == id)
        .and_then(|e| match &e.value {
            EvidenceValue::Status(s) => Some(s.to_string()),
            _ => None,
        })
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

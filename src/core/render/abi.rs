// Copyright (C) 2026 rezky_nightky
// SPDX-License-Identifier: GPL-3.0-only

//! ABI renderer — formats kernel ABI & compatibility intelligence output.
//!
//! Accepts already-collected Evidence. Never inspects the system.
//! Only formatting.

use std::io::{self, Write};

use crate::core::evidence::Evidence;
use crate::core::evidence_helpers;

/// Render ABI output from collected evidence.
pub fn render(
    evidence: &[Evidence],
    out: &mut io::StdoutLock<'_>,
) -> Result<(), Box<dyn std::error::Error>> {
    let abi = evidence_helpers::ev_text_value(evidence, "abi.info");
    let (uts, vermagic, layout, compression) = if let Some(ref a) = abi {
        (
            field(a, "utsrelease="),
            field(a, "vermagic="),
            field(a, "layout="),
            field(a, "compression="),
        )
    } else {
        (None, None, None, None)
    };

    writeln!(out, "Kernel ABI")?;
    writeln!(out)?;
    writeln!(
        out,
        "  Utsrelease : {}",
        uts.as_deref().unwrap_or("Unknown")
    )?;
    writeln!(
        out,
        "  Vermagic   : {}",
        vermagic.as_deref().unwrap_or("Unknown")
    )?;
    writeln!(
        out,
        "  Module layout : {}",
        layout.as_deref().unwrap_or("Unknown")
    )?;
    writeln!(out)?;

    writeln!(out, "Module Loader")?;
    writeln!(
        out,
        "  Signing support  : {}",
        if evidence_helpers::ev_bool(evidence, "config.MODULE_SIG") {
            "yes"
        } else {
            "no"
        }
    )?;
    writeln!(
        out,
        "  Compression      : {}",
        compression.as_deref().unwrap_or("Unknown")
    )?;
    writeln!(
        out,
        "  Livepatch        : {}",
        if evidence_helpers::ev_bool(evidence, "config.LIVEPATCH") {
            "enabled"
        } else {
            "disabled"
        }
    )?;
    writeln!(out)?;

    writeln!(out, "Module.symvers (running kernel)")?;
    match evidence_helpers::ev_text_value(evidence, "symbols.symvers") {
        Some(p) => writeln!(out, "  Path : {p}")?,
        None => writeln!(out, "  Status : not found")?,
    }
    writeln!(out)?;

    writeln!(out, "Kernel Symbols")?;
    writeln!(
        out,
        "  Total : {}",
        evidence_helpers::ev_s(evidence, "symbols.count")
    )?;
    let ks = evidence_helpers::ev_status_value(evidence, "symbols.kallsyms");
    match ks.as_deref() {
        Some("readable") | Some("readable (root)") => {
            writeln!(out, "  Source : /proc/kallsyms (readable)")?
        }
        Some("permission denied") => {
            writeln!(out, "  Source : /proc/kallsyms (permission denied)")?
        }
        Some("hidden") => writeln!(out, "  Source : hidden")?,
        _ => writeln!(out, "  Source : not available")?,
    }
    writeln!(out)?;

    writeln!(out, "VMLinux")?;
    match evidence_helpers::ev_text_value(evidence, "symbols.vmlinux") {
        Some(v) => writeln!(out, "  Info : {v}")?,
        None => writeln!(out, "  Status : not found")?,
    }
    writeln!(out)?;

    writeln!(out, "Compiler ABI")?;
    let comp = evidence_helpers::ev_text_value(evidence, "compiler.abi")
        .unwrap_or_else(|| "Unknown".into());
    writeln!(out, "  Kernel compiler  : {comp}")?;
    let conf = evidence_helpers::ev_confidence(evidence, "compiler.abi");
    writeln!(out, "  Confidence       : {}", conf)?;
    writeln!(out)?;

    Ok(())
}

fn field(data: &str, prefix: &str) -> Option<String> {
    data.split_whitespace()
        .find(|p| p.starts_with(prefix))
        .map(|p| p.strip_prefix(prefix).unwrap_or(p).to_string())
}

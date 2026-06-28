// Copyright (C) 2026 rezky_nightky
// SPDX-License-Identifier: GPL-3.0-only

//! ABI command — kernel ABI & compatibility intelligence.
//!
//! Thin orchestrator — all data from Registry, rendering only.

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
    // Utsrelease
    let abi = ev_text_value(evidence, "abi.info");
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
        if ev_bool(evidence, "config.MODULE_SIG") {
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
        if ev_bool(evidence, "config.LIVEPATCH") {
            "enabled"
        } else {
            "disabled"
        }
    )?;
    writeln!(out)?;

    // Module.symvers
    writeln!(out, "Module.symvers (running kernel)")?;
    match ev_text_value(evidence, "symbols.symvers") {
        Some(p) => writeln!(out, "  Path : {p}")?,
        None => writeln!(out, "  Status : not found")?,
    }
    writeln!(out)?;

    // Kernel Symbols
    writeln!(out, "Kernel Symbols")?;
    writeln!(out, "  Total : {}", ev_s(evidence, "symbols.count"))?;
    let ks = ev_status_value(evidence, "symbols.kallsyms");
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

    // VMLinux
    writeln!(out, "VMLinux")?;
    match ev_text_value(evidence, "symbols.vmlinux") {
        Some(v) => writeln!(out, "  Info : {v}")?,
        None => writeln!(out, "  Status : not found")?,
    }
    writeln!(out)?;

    // Compiler ABI
    writeln!(out, "Compiler ABI")?;
    let comp = ev_text_value(evidence, "compiler.abi").unwrap_or_else(|| "Unknown".into());
    writeln!(out, "  Kernel compiler  : {comp}")?;
    let conf = ev_confidence(evidence, "compiler.abi");
    writeln!(out, "  Confidence       : {}", conf)?;
    writeln!(out)?;

    Ok(())
}

/* helpers */

fn field(data: &str, prefix: &str) -> Option<String> {
    data.split_whitespace()
        .find(|p| p.starts_with(prefix))
        .map(|p| p.strip_prefix(prefix).unwrap_or(p).to_string())
}

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

fn ev_status_value(evidence: &[Evidence], id: &str) -> Option<String> {
    evidence
        .iter()
        .find(|e| e.id == id)
        .and_then(|e| match &e.value {
            EvidenceValue::Status(s) => Some(s.to_string()),
            _ => None,
        })
}

fn ev_confidence(evidence: &[Evidence], id: &str) -> &'static str {
    evidence
        .iter()
        .find(|e| e.id == id)
        .map_or("low", |e| e.confidence.label())
}

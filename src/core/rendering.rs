// Copyright (C) 2026 rezky_nightky
// SPDX-License-Identifier: GPL-3.0-only

//! Renderer module — transforms Evidence + analysis into output.
//!
//! Renderers are the ONLY place where terminal/JSON output is produced.
//! Capabilities never print. Commands only orchestrate.

use std::io::{self, Write};

use super::analysis::{self};
use super::evidence::{Evidence, EvidenceValue};

// ============================================================================
//  Human-readable report renderer
// ============================================================================

pub fn render_human(
    evidence: &[Evidence],
    out: &mut io::StdoutLock<'_>,
) -> Result<(), Box<dyn std::error::Error>> {
    let (readiness, risks) = analysis::analyze(evidence);
    let recs = super::recommendation::recommend(evidence);

    // Header
    writeln!(out, "Zenvecha Kernel Intelligence Report")?;
    writeln!(out)?;

    // Kernel Identity
    render_section(out, "Kernel Identity")?;
    render_kv(out, "  Version", &ev_s(evidence, "kernel.release"))?;
    render_kv(out, "  Architecture", &ev_s(evidence, "kernel.arch"))?;
    render_kv(out, "  Distribution", &ev_s(evidence, "kernel.distro"))?;
    render_kv(out, "  Rust compiler", &ev_s(evidence, "toolchain.rustc"))?;
    writeln!(out)?;

    // Readiness
    render_section(out, "Readiness")?;
    writeln!(out, "  Overall : {}", readiness.overall)?;
    writeln!(out)?;
    for cat in &readiness.categories {
        writeln!(out, "  {} {}", stars_fmt(cat.stars), cat.name)?;
    }
    writeln!(out)?;

    // Capabilities
    render_section(out, "Capability Matrix")?;
    writeln!(out)?;

    let caps = [
        ("Rust for Linux", "config.RUST", "config.RUST_IS_AVAILABLE"),
        ("Modules", "config.MODULES", ""),
        ("Module Signing", "config.MODULE_SIG", ""),
        ("BTF", "config.DEBUG_INFO_BTF", "debug.btf"),
        ("Livepatch", "config.LIVEPATCH", ""),
        ("Kallsyms", "symbols.kallsyms", ""),
    ];

    for (name, primary, secondary) in &caps {
        let primary_val = ev_config_bool(evidence, primary);
        let secondary_val = if secondary.is_empty() {
            None
        } else {
            ev_config_bool(evidence, secondary)
        };

        let (status, _) = cap_status(primary_val, secondary_val);
        writeln!(out, "  {} {}", tri_icon(status), name)?;
    }
    writeln!(out)?;

    // Risks
    if !risks.is_empty() {
        render_section(out, "Compatibility Risks")?;
        writeln!(out)?;
        for risk in &risks {
            writeln!(out, "  ⚠  {}", risk.description)?;
        }
        writeln!(out)?;
    }

    // Recommendations
    if !recs.is_empty() {
        render_section(out, "Recommendations")?;
        writeln!(out)?;
        let end = recs.len().min(10);
        for (i, rec) in recs.iter().take(end).enumerate() {
            writeln!(out, "  {}. {}", i + 1, rec)?;
        }
        writeln!(out)?;
    }

    // Environment Facts
    render_section(out, "Environment Facts")?;
    writeln!(out)?;
    for fact in collect_facts(evidence) {
        writeln!(out, "  • {}", fact)?;
    }
    writeln!(out)?;

    Ok(())
}

// ============================================================================
//  Compact renderer
// ============================================================================

pub fn render_compact(
    evidence: &[Evidence],
    out: &mut io::StdoutLock<'_>,
) -> Result<(), Box<dyn std::error::Error>> {
    let (readiness, risks) = analysis::analyze(evidence);
    let recs = super::recommendation::recommend(evidence);

    writeln!(out, "Zenvecha v{}", env!("CARGO_PKG_VERSION"))?;
    writeln!(
        out,
        "Kernel: {} ({})",
        ev_s(evidence, "kernel.release"),
        ev_s(evidence, "kernel.arch")
    )?;
    writeln!(
        out,
        "Status: {} | Symbols: {} | Modules: {}",
        readiness.overall,
        ev_count(evidence, "symbols.count"),
        ev_s(evidence, "modules.loader"),
    )?;

    if let Some(bid) = ev_literal(evidence, "symbols.vmlinux") {
        writeln!(out, "VMLinux: {bid}")?;
    }
    if ev_bool(evidence, "debug.btf") {
        writeln!(out, "BTF: available")?;
    }
    if ev_bool(evidence, "config.RUST") {
        writeln!(out, "Rust: enabled")?;
    } else if ev_bool(evidence, "config.RUST_IS_AVAILABLE") {
        writeln!(out, "Rust: compiler available, not enabled")?;
    }

    if !risks.is_empty() {
        writeln!(out, "Risks: {}", risks.len())?;
    }
    if !recs.is_empty() {
        let end = recs.len().min(3);
        writeln!(out, "Top recommendations:")?;
        for (i, rec) in recs.iter().take(end).enumerate() {
            writeln!(out, "  {}. {}", i + 1, rec)?;
        }
    }

    Ok(())
}

// ============================================================================
//  JSON renderer
// ============================================================================

pub fn render_json(
    evidence: &[Evidence],
    out: &mut io::StdoutLock<'_>,
) -> Result<(), Box<dyn std::error::Error>> {
    let (readiness, risks) = analysis::analyze(evidence);
    let recs = super::recommendation::recommend(evidence);

    let mut buf = String::new();
    buf.push_str("{\n");

    // Meta
    buf.push_str(&format!(
        "  \"version\": \"{}\",\n",
        esc(env!("CARGO_PKG_VERSION")),
    ));

    // Readiness
    buf.push_str("  \"readiness\": {\n");
    buf.push_str(&format!(
        "    \"overall\": \"{}\",\n",
        esc(readiness.overall)
    ));
    buf.push_str(&format!("    \"score\": \"{}\",\n", esc(readiness.stars)));
    buf.push_str("    \"categories\": [\n");
    for (i, cat) in readiness.categories.iter().enumerate() {
        let comma = if i + 1 < readiness.categories.len() {
            ","
        } else {
            ""
        };
        buf.push_str(&format!(
            "      {{\"name\":\"{}\",\"stars\":{}}}{comma}\n",
            esc(cat.name),
            cat.stars,
        ));
    }
    buf.push_str("    ]\n  },\n");

    // Capabilities
    buf.push_str("  \"capabilities\": {\n");
    let cap_keys = [
        ("Rust for Linux", "config.RUST"),
        ("Modules", "config.MODULES"),
        ("Module Signing", "config.MODULE_SIG"),
        ("BTF", "config.DEBUG_INFO_BTF"),
        ("Livepatch", "config.LIVEPATCH"),
        ("Kallsyms", "symbols.kallsyms"),
    ];
    for (i, (name, id)) in cap_keys.iter().enumerate() {
        let comma = if i + 1 < cap_keys.len() { "," } else { "" };
        let val = ev_bool(evidence, id);
        buf.push_str(&format!("    \"{}\": {}{comma}\n", esc(name), val,));
    }
    buf.push_str("  },\n");

    // Risks
    buf.push_str("  \"risks\": [\n");
    for (i, risk) in risks.iter().enumerate() {
        let comma = if i + 1 < risks.len() { "," } else { "" };
        buf.push_str(&format!(
            "    {{\"description\":\"{}\",\"severity\":\"{}\"}}{comma}\n",
            esc(&risk.description),
            esc(risk.severity),
        ));
    }
    buf.push_str("  ],\n");

    // Recommendations
    buf.push_str("  \"recommendations\": [\n");
    let end = recs.len().min(10);
    for (i, rec) in recs.iter().take(end).enumerate() {
        let comma = if i + 1 < end { "," } else { "" };
        buf.push_str(&format!("    \"{}\"{comma}\n", esc(rec),));
    }
    buf.push_str("  ],\n");

    // Facts
    buf.push_str("  \"facts\": [\n");
    let facts = collect_facts(evidence);
    for (i, fact) in facts.iter().enumerate() {
        let comma = if i + 1 < facts.len() { "," } else { "" };
        buf.push_str(&format!("    \"{}\"{comma}\n", esc(fact),));
    }
    buf.push_str("  ]\n");

    buf.push_str("}\n");

    write!(out, "{buf}")?;
    Ok(())
}

// ============================================================================
//  Helpers
// ============================================================================

fn render_section(out: &mut io::StdoutLock<'_>, title: &str) -> io::Result<()> {
    writeln!(out, "── {title} ──")
}

fn render_kv(out: &mut io::StdoutLock<'_>, label: &str, value: &str) -> io::Result<()> {
    if value.is_empty() || value == "Unknown" {
        writeln!(out, "{label} : Unknown")
    } else {
        writeln!(out, "{label} : {value}")
    }
}

fn stars_fmt(n: u8) -> String {
    match n {
        5 => "★★★★★".into(),
        4 => "★★★★☆".into(),
        3 => "★★★☆☆".into(),
        2 => "★★☆☆☆".into(),
        1 => "★☆☆☆☆".into(),
        _ => "☆☆☆☆☆".into(),
    }
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
            EvidenceValue::Count(n) => *n > 0,
            _ => false,
        })
}

fn ev_config_bool(evidence: &[Evidence], id: &str) -> Option<bool> {
    evidence
        .iter()
        .find(|e| e.id == id)
        .and_then(|e| match &e.value {
            EvidenceValue::Bool(b) => Some(*b),
            EvidenceValue::Config(cv) => {
                if cv.is_known() {
                    Some(cv.is_enabled())
                } else {
                    None
                }
            }
            _ => None,
        })
}

fn ev_count(evidence: &[Evidence], id: &str) -> String {
    evidence
        .iter()
        .find(|e| e.id == id)
        .map_or_else(|| "0".into(), |e| e.value.display())
}

fn ev_literal(evidence: &[Evidence], id: &str) -> Option<String> {
    evidence
        .iter()
        .find(|e| e.id == id)
        .and_then(|e| match &e.value {
            EvidenceValue::Literal(s) => Some(s.clone()),
            EvidenceValue::Text(Some(s)) => Some(s.clone()),
            _ => None,
        })
}

fn cap_status(primary: Option<bool>, secondary: Option<bool>) -> (&'static str, bool) {
    match (primary, secondary) {
        (Some(true), _) | (_, Some(true)) => ("enabled", true),
        (Some(false), Some(false)) | (Some(false), None) => ("disabled", false),
        (None, None) => ("unknown", false),
        (None, Some(false)) => ("unknown", false),
    }
}

fn tri_icon(status: &str) -> &'static str {
    match status {
        "enabled" => "✔",
        "disabled" => "✘",
        _ => "?",
    }
}

fn collect_facts(evidence: &[Evidence]) -> Vec<String> {
    let mut facts: Vec<String> = Vec::new();

    // Symbol count
    let sym_count = ev_count(evidence, "symbols.count");
    if sym_count != "0" {
        facts.push(format!("{sym_count} kernel symbols"));
    }

    // Modules loaded
    let loader = ev_text_value(evidence, "modules.loader");
    if let Some(ref l) = loader
        && let Some(loaded) = extract_field(l, "loaded=")
    {
        facts.push(format!("{loaded} modules loaded"));
    }

    // BTF
    if ev_bool(evidence, "debug.btf") {
        facts.push("BTF type information available".into());
    }

    // Module signing
    if ev_bool(evidence, "config.MODULE_SIG") {
        facts.push("Kernel module signing active".into());
    }

    // Architecture
    let arch = ev_s(evidence, "kernel.arch");
    if arch != "Unknown" {
        facts.push(format!("Architecture: {arch}"));
    }

    // Kernel vs headers
    let release = ev_text_value(evidence, "kernel.release");
    if let Some(ref r) = release
        && ev_status_is(evidence, "build.headers", "Complete")
    {
        facts.push(format!("Running kernel matches installed headers ({r})"));
    }

    // Config available
    let cfg = ev_text_value(evidence, "config.source");
    if cfg.is_some() {
        facts.push("Kernel config available".into());
    }

    // Debug/Trace mounts
    if ev_bool(evidence, "fs.debugfs") {
        facts.push("debugfs mounted".into());
    }
    if ev_bool(evidence, "fs.tracefs") {
        facts.push("tracefs mounted".into());
    }

    // Module compression
    if let Some(ref l) = loader
        && let Some(comp) = extract_field(l, "compression=")
        && comp != "none"
        && comp != "Unknown"
    {
        facts.push(format!("Module compression: {comp}"));
    }

    // VMLinux
    if let Some(vml) = ev_literal(evidence, "symbols.vmlinux") {
        facts.push(format!("VMLinux: {vml}"));
    }

    // Module.symvers
    if let Some(sym) = ev_text_value(evidence, "symbols.symvers") {
        facts.push(format!("Module.symvers (source tree): {sym}"));
    }

    facts
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

fn extract_field(data: &str, prefix: &str) -> Option<String> {
    data.split_whitespace()
        .find(|p| p.starts_with(prefix))
        .map(|p| p.strip_prefix(prefix).unwrap_or(p).to_string())
}

fn esc(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            c if c.is_control() => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out
}

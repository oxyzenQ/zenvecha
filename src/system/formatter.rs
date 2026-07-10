// Copyright (C) 2026 rezky_nightky
// SPDX-License-Identifier: GPL-3.0-only

//! Human-readable and compact report formatters.
//!
//! Receive `ReportContext`, write to any `Write` sink.

use std::io::Write;

use crate::system::capabilities::{capability_matrix, collect_facts, collect_risks};
use crate::system::report::ReportContext;

/// Write the full human-readable report.
pub fn write_human(ctx: &ReportContext, out: &mut dyn Write) -> std::io::Result<()> {
    let capabilities = capability_matrix(ctx);
    let risks = collect_risks(ctx);
    let facts = collect_facts(ctx);

    writeln!(out, "══════════════════════════════════════════")?;
    writeln!(out, "  Zenvecha Kernel Intelligence Report")?;
    writeln!(out, "══════════════════════════════════════════")?;
    writeln!(out)?;

    // 1. System Summary
    writeln!(out, "── System Summary ──")?;
    kv(out, "Distribution", ctx.distro.as_deref())?;
    kv(out, "Kernel", ctx.release.as_deref())?;
    kv(out, "Architecture", ctx.arch.as_deref())?;
    if let Some(ref c) = ctx.tools.rustc {
        writeln!(out, "  Compiler     : {c}")?;
    }
    if let Some(ref l) = ctx.tools.llvm_version {
        writeln!(out, "  LLVM         : {l}")?;
    }
    writeln!(out)?;

    // 2. Capability Matrix
    writeln!(out, "── Kernel Capability Matrix ──")?;
    writeln!(out)?;
    for cap in &capabilities {
        let mark = match cap.status {
            crate::system::capabilities::CapabilityStatus::Enabled => "✔",
            crate::system::capabilities::CapabilityStatus::Disabled => "✘",
            crate::system::capabilities::CapabilityStatus::Unknown => "?",
        };
        writeln!(out, "  {mark} {}", cap.name)?;
        writeln!(out, "    {}", cap.evidence)?;
        writeln!(out)?;
    }

    // 3. Development Readiness
    writeln!(out, "── Development Readiness ──")?;
    writeln!(out)?;
    for s in &ctx.scores {
        writeln!(out, "  {}  {}", s.render(), s.name)?;
    }
    writeln!(out)?;
    writeln!(out, "  Overall : {}", ctx.overall)?;
    writeln!(out)?;

    // 4. Compatibility Risks
    if !risks.is_empty() {
        writeln!(out, "── Compatibility Risks ──")?;
        writeln!(out)?;
        for r in &risks {
            writeln!(out, "  ⚠  {r}")?;
        }
        writeln!(out)?;
    }

    // 5. Recommendations
    let recs = crate::system::recommend::generate(&crate::system::recommend::RecCtx {
        rustc_installed: ctx.tools.rustc.is_some(),
        bindgen_installed: ctx.tools.bindgen.is_some(),
        llvm_installed: ctx.tools.llvm_version.is_some(),
        headers_available: ctx.mod_info.headers_available,
        build_dir_present: ctx.bld.build_dir.is_some(),
        source_dir_present: ctx.bld.source_dir.is_some(),
        config_rust: ctx.rust_cfg,
        config_rust_available: ctx.rust_avail,
        config_modules: ctx
            .config_text
            .as_deref()
            .map_or(crate::system::config::ConfigValue::Missing, |t| {
                crate::system::config::config_value(t, "MODULES")
            }),
        config_btf: ctx
            .config_text
            .as_deref()
            .map_or(crate::system::config::ConfigValue::Missing, |t| {
                crate::system::config::config_value(t, "DEBUG_INFO_BTF")
            }),
        btf_available: ctx.dbg.btf_available,
        signing_required: ctx.mod_info.signing_required,
        signing_enabled: ctx.mod_info.signing_enabled == Some(true),
        debugfs_ok: ctx.debugfs_ok,
        tracefs_ok: ctx.tracefs_ok,
        release: ctx.release.as_deref(),
        headers_ver: ctx.mod_info.installed_header_version.as_deref(),
    });
    let recs: Vec<&String> = recs.iter().take(10).collect();
    if !recs.is_empty() {
        writeln!(out, "── Recommended Next Actions ──")?;
        writeln!(out)?;
        for (i, r) in recs.iter().enumerate() {
            writeln!(out, "  {}. {r}", i + 1)?;
        }
        writeln!(out)?;
    }

    // 6. Environment Facts
    if !facts.is_empty() {
        writeln!(out, "── Environment Facts ──")?;
        writeln!(out)?;
        for f in &facts {
            writeln!(out, "  • {f}")?;
        }
        writeln!(out)?;
    }

    Ok(())
}

/// Write the compact report (one line per section).
pub fn write_compact(ctx: &ReportContext, out: &mut dyn Write) -> std::io::Result<()> {
    let capabilities = capability_matrix(ctx);
    let risks = collect_risks(ctx);
    let facts = collect_facts(ctx);

    writeln!(out, "Zenvecha Report v{}", env!("CARGO_PKG_VERSION"))?;
    writeln!(out)?;
    writeln!(
        out,
        "System: {} | {} | {}",
        ctx.distro.as_deref().unwrap_or("?"),
        ctx.release.as_deref().unwrap_or("?"),
        ctx.arch.as_deref().unwrap_or("?")
    )?;

    write!(out, "Caps: ")?;
    for (i, cap) in capabilities.iter().enumerate() {
        if i > 0 {
            write!(out, ", ")?;
        }
        let mark = match cap.status {
            crate::system::capabilities::CapabilityStatus::Enabled => "+",
            crate::system::capabilities::CapabilityStatus::Disabled => "-",
            crate::system::capabilities::CapabilityStatus::Unknown => "?",
        };
        write!(out, "{mark}{}", cap.name)?;
    }
    writeln!(out)?;

    write!(out, "Score: ")?;
    for s in &ctx.scores {
        write!(out, "{}:{}★ ", s.name, s.stars)?;
    }
    writeln!(out, "| {}", ctx.overall)?;

    if !risks.is_empty() {
        writeln!(out, "Risks: {}", risks.len())?;
    }
    if !facts.is_empty() {
        writeln!(out, "Facts: {}", facts.len())?;
    }

    Ok(())
}

fn kv(out: &mut dyn Write, label: &str, value: Option<&str>) -> std::io::Result<()> {
    match value {
        Some(v) if !v.is_empty() => writeln!(out, "  {label:<14} : {v}"),
        _ => writeln!(out, "  {label:<14} : Unknown"),
    }
}

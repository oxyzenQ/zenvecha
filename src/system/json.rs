// Copyright (C) 2026 rezky_nightky
// SPDX-License-Identifier: GPL-3.0-only

//! Manual JSON serializer — no serde dependency.
//!
//! Builds valid JSON strings from ReportContext. Minimal, deterministic.

use std::io::Write;

use crate::system::capabilities::{capability_matrix, collect_facts, collect_risks};
use crate::system::report::ReportContext;

/// Write the report as JSON.
pub fn write_json(ctx: &ReportContext, out: &mut dyn Write) -> std::io::Result<()> {
    let capabilities = capability_matrix(ctx);
    let risks = collect_risks(ctx);
    let facts = collect_facts(ctx);

    writeln!(out, "{{")?;

    // system
    writeln!(out, "  \"system\": {{")?;
    jstr(out, "distribution", ctx.distro.as_deref(), true)?;
    jstr(out, "kernel", ctx.release.as_deref(), true)?;
    jstr(out, "architecture", ctx.arch.as_deref(), true)?;
    jstr(out, "compiler", ctx.tools.rustc.as_deref(), true)?;
    jstr(out, "llvm", ctx.tools.llvm_version.as_deref(), false)?;
    writeln!(out, "  }},")?;

    // capabilities
    writeln!(out, "  \"capabilities\": [")?;
    for (i, cap) in capabilities.iter().enumerate() {
        let status = match cap.status {
            crate::system::capabilities::CapabilityStatus::Enabled => "enabled",
            crate::system::capabilities::CapabilityStatus::Disabled => "disabled",
            crate::system::capabilities::CapabilityStatus::Unknown => "unknown",
        };
        let comma = if i + 1 < capabilities.len() { "," } else { "" };
        writeln!(out, "    {{")?;
        writeln!(out, "      \"name\": \"{}\",", cap.name)?;
        writeln!(out, "      \"status\": \"{}\",", status)?;
        writeln!(out, "      \"evidence\": \"{}\"", esc(&cap.evidence))?;
        writeln!(out, "    }}{comma}")?;
    }
    writeln!(out, "  ],")?;

    // readiness
    writeln!(out, "  \"readiness\": {{")?;
    writeln!(out, "    \"overall\": \"{}\",", ctx.overall)?;
    writeln!(out, "    \"scores\": [")?;
    for (i, s) in ctx.scores.iter().enumerate() {
        let comma = if i + 1 < ctx.scores.len() { "," } else { "" };
        writeln!(
            out,
            "      {{ \"name\": \"{}\", \"stars\": {} }}{comma}",
            s.name, s.stars
        )?;
    }
    writeln!(out, "    ]")?;
    writeln!(out, "  }},")?;

    // risks
    writeln!(out, "  \"risks\": [")?;
    for (i, r) in risks.iter().enumerate() {
        let comma = if i + 1 < risks.len() { "," } else { "" };
        writeln!(out, "    \"{}\"{comma}", esc(r))?;
    }
    writeln!(out, "  ],")?;

    // recommendations
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
    writeln!(out, "  \"recommendations\": [")?;
    let recs: Vec<&String> = recs.iter().take(10).collect();
    for (i, r) in recs.iter().enumerate() {
        let comma = if i + 1 < recs.len() { "," } else { "" };
        writeln!(out, "    \"{}\"{comma}", esc(r))?;
    }
    writeln!(out, "  ],")?;

    // facts
    writeln!(out, "  \"facts\": [")?;
    for (i, f) in facts.iter().enumerate() {
        let comma = if i + 1 < facts.len() { "," } else { "" };
        writeln!(out, "    \"{}\"{comma}", esc(f))?;
    }
    writeln!(out, "  ]")?;

    writeln!(out, "}}")?;
    Ok(())
}

fn jstr(out: &mut dyn Write, key: &str, val: Option<&str>, comma: bool) -> std::io::Result<()> {
    let c = if comma { "," } else { "" };
    match val {
        Some(v) if !v.is_empty() => writeln!(out, "    \"{key}\": \"{}\"{c}", esc(v)),
        _ => writeln!(out, "    \"{key}\": null{c}"),
    }
}

fn esc(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\t', "\\t")
}

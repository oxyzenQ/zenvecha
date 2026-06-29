// Copyright (C) 2026 rezky_nightky
// SPDX-License-Identifier: GPL-3.0-only

//! Analyze renderer — formats development readiness assessment output.
//!
//! Accepts pre-computed models. Never inspects the system. Only formatting.

use std::io::{self, Write};

use crate::core::analysis::{CategoryScore, Compatibility, ComponentScore, Readiness};
use crate::core::evidence::Evidence;
use crate::core::evidence_helpers;

/// Render analyze output from pre-computed models.
pub fn render(
    evidence: &[Evidence],
    readiness: &Readiness,
    compatibility: &Compatibility,
    recs: &[String],
    out: &mut io::StdoutLock<'_>,
) -> Result<(), Box<dyn std::error::Error>> {
    writeln!(out, "Zenvecha Analyze")?;
    writeln!(out)?;

    // ── Compatibility Score ──
    writeln!(out, "Compatibility Score")?;
    writeln!(out)?;
    let bar = score_bar(compatibility.score);
    writeln!(out, "  {}  {}%", bar, compatibility.score)?;
    writeln!(out, "  Level      : {}", compatibility.level)?;
    writeln!(out, "  Confidence  : {}", compatibility.confidence.label())?;
    writeln!(out, "  Risk        : {}", compatibility.risk.label())?;
    if compatibility.blocking_issues.is_empty() {
        writeln!(out, "  Blocking    : none")?;
    } else {
        writeln!(
            out,
            "  Blocking    : {} issue(s)",
            compatibility.blocking_issues.len()
        )?;
    }
    writeln!(out)?;

    // Per-component scores
    writeln!(out, "Component Scores")?;
    writeln!(out)?;
    for comp in &compatibility.components {
        let icon = component_icon(comp);
        writeln!(out, "  {icon} {:<28} {:>3}%", comp.name, comp.score)?;
        writeln!(out, "    {}", comp.detail)?;
    }
    writeln!(out)?;

    // Blocking issues
    if !compatibility.blocking_issues.is_empty() {
        writeln!(out, "Blocking Issues")?;
        writeln!(out)?;
        for issue in &compatibility.blocking_issues {
            writeln!(out, "  ✘ {}: {}", issue.component, issue.description)?;
        }
        writeln!(out)?;
    }

    // Next best action
    writeln!(out, "Next Best Action")?;
    writeln!(out)?;
    writeln!(out, "  {}", compatibility.next_best_action)?;
    writeln!(out)?;

    // Estimated fix time
    if compatibility.estimated_fix_minutes > 0 {
        writeln!(
            out,
            "  Estimated fix time : ~{} min",
            compatibility.estimated_fix_minutes
        )?;
    } else {
        writeln!(out, "  No fixes required")?;
    }
    writeln!(out)?;

    // ── Build Environment ──
    writeln!(out, "Build Environment")?;
    print_kv(
        out,
        "  Header integrity",
        &evidence_helpers::ev_s(evidence, "build.headers"),
    )?;

    if evidence_helpers::ev_status_is(evidence, "build.headers", "Complete")
        && !evidence_helpers::ev_text_known(evidence, "build.source")
    {
        writeln!(
            out,
            "  Kernel source       : not installed (header tree only)"
        )?;
    } else if !evidence_helpers::ev_status_is(evidence, "build.headers", "Complete") {
        print_kv(
            out,
            "  Kernel source",
            &evidence_helpers::ev_s(evidence, "build.source"),
        )?;
    }

    print_kv(
        out,
        "  Build directory",
        &evidence_helpers::ev_s(evidence, "build.dir"),
    )?;
    print_kv(
        out,
        "  Source directory",
        &evidence_helpers::ev_s(evidence, "build.source"),
    )?;
    print_path_kv(
        out,
        "Module.symvers (source tree)",
        &evidence_helpers::ev_s(evidence, "symbols.symvers"),
    )?;
    print_bool_kv(
        out,
        "compile_commands.json",
        evidence_helpers::ev_bool(evidence, "build.compile_commands"),
    )?;
    writeln!(out)?;

    // Toolchain
    writeln!(out, "Toolchain")?;
    print_bool_kv(
        out,
        "  rustc",
        evidence_helpers::ev_bool(evidence, "toolchain.rustc"),
    )?;
    print_bool_kv(
        out,
        "  bindgen",
        evidence_helpers::ev_bool(evidence, "toolchain.bindgen"),
    )?;
    print_bool_kv(
        out,
        "  llvm",
        evidence_helpers::ev_bool(evidence, "toolchain.llvm"),
    )?;
    print_bool_kv(
        out,
        "  make",
        evidence_helpers::ev_bool(evidence, "toolchain.make"),
    )?;
    print_bool_kv(
        out,
        "  gcc",
        evidence_helpers::ev_bool(evidence, "toolchain.gcc"),
    )?;
    writeln!(out)?;

    // Filesystem
    writeln!(out, "Filesystem")?;
    writeln!(
        out,
        "  debugfs : {}",
        if evidence_helpers::ev_bool(evidence, "fs.debugfs") {
            "mounted"
        } else {
            "not mounted"
        }
    )?;
    writeln!(
        out,
        "  tracefs : {}",
        if evidence_helpers::ev_bool(evidence, "fs.tracefs") {
            "mounted"
        } else {
            "not mounted"
        }
    )?;
    writeln!(out)?;

    // Rust for Linux
    writeln!(out, "Rust for Linux")?;
    writeln!(
        out,
        "  Status : {}",
        evidence_helpers::ev_s(evidence, "config.RUST")
    )?;
    writeln!(
        out,
        "  Compiler : {}",
        evidence_helpers::ev_s(evidence, "config.RUST_IS_AVAILABLE")
    )?;
    let rust_detail = if evidence_helpers::ev_bool(evidence, "config.RUST")
        && evidence_helpers::ev_bool(evidence, "config.RUST_IS_AVAILABLE")
    {
        "Compatible — CONFIG_RUST=y and Rust compiler is available"
    } else if evidence_helpers::ev_bool(evidence, "config.RUST") {
        "Partially Compatible — CONFIG_RUST=y but compiler not detected"
    } else if evidence_helpers::ev_bool(evidence, "config.RUST_IS_AVAILABLE") {
        "Partially Compatible — compiler available but CONFIG_RUST not set"
    } else {
        "Not Compatible — neither CONFIG_RUST nor Rust compiler found"
    };
    writeln!(out, "  Compatibility : {rust_detail}")?;
    writeln!(out)?;

    // Overall Status
    writeln!(out, "Overall Status")?;
    writeln!(out)?;
    writeln!(
        out,
        "  {}  {}",
        stars_str(&readiness.categories),
        readiness.overall
    )?;
    writeln!(out)?;
    for cat in &readiness.categories {
        writeln!(out, "  {}  {}", category_stars(cat.stars), cat.name)?;
    }
    writeln!(out)?;

    // Recommendations
    if !recs.is_empty() {
        writeln!(out, "Recommendations")?;
        writeln!(out)?;
        let end = recs.len().min(10);
        for (i, rec) in recs.iter().take(end).enumerate() {
            writeln!(out, "  {}. {rec}", i + 1)?;
        }
        writeln!(out)?;
    }

    Ok(())
}

fn print_kv(out: &mut io::StdoutLock<'_>, label: &str, value: &str) -> io::Result<()> {
    if value == "Unknown" || value.is_empty() {
        writeln!(out, "{label} : Unknown")
    } else {
        writeln!(out, "{label} : {value}")
    }
}

fn print_path_kv(out: &mut io::StdoutLock<'_>, label: &str, value: &str) -> io::Result<()> {
    if value == "Unknown" || value.is_empty() {
        writeln!(out, "{label} : not found")
    } else {
        writeln!(out, "{label} : {value}")
    }
}

fn print_bool_kv(out: &mut io::StdoutLock<'_>, label: &str, val: bool) -> io::Result<()> {
    writeln!(out, "{label} : {}", if val { "yes" } else { "no" })
}

fn stars_str(categories: &[CategoryScore]) -> &'static str {
    let total: u8 = categories.iter().map(|c| c.stars).sum();
    let max = (categories.len() * 5) as u8;
    if total >= max {
        "★★★★★"
    } else if total as f64 >= max as f64 * 0.8 {
        "★★★★☆"
    } else if total as f64 >= max as f64 * 0.6 {
        "★★★☆☆"
    } else if total as f64 >= max as f64 * 0.4 {
        "★★☆☆☆"
    } else {
        "★☆☆☆☆"
    }
}

fn category_stars(n: u8) -> String {
    match n {
        5 => "★★★★★".into(),
        4 => "★★★★☆".into(),
        3 => "★★★☆☆".into(),
        2 => "★★☆☆☆".into(),
        1 => "★☆☆☆☆".into(),
        _ => "☆☆☆☆☆".into(),
    }
}

fn score_bar(score: u8) -> String {
    let filled = (score / 10).min(10) as usize;
    let empty = 10 - filled;
    format!("[{}{}]", "█".repeat(filled), "░".repeat(empty))
}

fn component_icon(comp: &ComponentScore) -> &'static str {
    match comp.status {
        crate::core::analysis::ComponentStatus::Good => "✓",
        crate::core::analysis::ComponentStatus::Partial => "~",
        crate::core::analysis::ComponentStatus::Missing => "✗",
        crate::core::analysis::ComponentStatus::Blocking => "✘",
    }
}

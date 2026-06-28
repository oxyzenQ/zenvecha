// Copyright (C) 2026 rezky_nightky
// SPDX-License-Identifier: GPL-3.0-only

//! Analyze command — development readiness assessment.
//!
//! Thin orchestrator. All analysis from core engine, rendering only.

use std::io::{self, Write};

use crate::core::analysis;
use crate::core::capability::Registry;
use crate::core::evidence::{Evidence, EvidenceValue};
use crate::core::recommendation;

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
    let (readiness, _risks) = analysis::analyze(evidence);
    let recs = recommendation::recommend(evidence);

    writeln!(out, "Zenvecha Analyze")?;
    writeln!(out)?;

    // Build Environment
    writeln!(out, "Build Environment")?;
    print_kv(out, "  Header integrity", &ev_s(evidence, "build.headers"))?;

    // Header integrity vs kernel source distinction
    if ev_status_is(evidence, "build.headers", "Complete")
        && !ev_text_known(evidence, "build.source")
    {
        writeln!(
            out,
            "  Kernel source       : not installed (header tree only)"
        )?;
    } else if !ev_status_is(evidence, "build.headers", "Complete") {
        print_kv(out, "  Kernel source", &ev_s(evidence, "build.source"))?;
    }

    print_kv(out, "  Build directory", &ev_s(evidence, "build.dir"))?;
    print_kv(out, "  Source directory", &ev_s(evidence, "build.source"))?;
    print_path_kv(
        out,
        "Module.symvers (source tree)",
        &ev_s(evidence, "symbols.symvers"),
    )?;
    print_bool_kv(
        out,
        "compile_commands.json",
        ev_bool(evidence, "build.compile_commands"),
    )?;
    writeln!(out)?;

    // Toolchain
    writeln!(out, "Toolchain")?;
    print_bool_kv(out, "  rustc", ev_bool(evidence, "toolchain.rustc"))?;
    print_bool_kv(out, "  bindgen", ev_bool(evidence, "toolchain.bindgen"))?;
    print_bool_kv(out, "  llvm", ev_bool(evidence, "toolchain.llvm"))?;
    print_bool_kv(out, "  make", ev_bool(evidence, "toolchain.make"))?;
    print_bool_kv(out, "  gcc", ev_bool(evidence, "toolchain.gcc"))?;
    writeln!(out)?;

    // Filesystem
    writeln!(out, "Filesystem")?;
    writeln!(
        out,
        "  debugfs : {}",
        if ev_bool(evidence, "fs.debugfs") {
            "mounted"
        } else {
            "not mounted"
        }
    )?;
    writeln!(
        out,
        "  tracefs : {}",
        if ev_bool(evidence, "fs.tracefs") {
            "mounted"
        } else {
            "not mounted"
        }
    )?;
    writeln!(out)?;

    // Rust for Linux
    writeln!(out, "Rust for Linux")?;
    writeln!(out, "  Status : {}", ev_s(evidence, "config.RUST"))?;
    writeln!(
        out,
        "  Compiler : {}",
        ev_s(evidence, "config.RUST_IS_AVAILABLE")
    )?;
    let rust_detail =
        if ev_bool(evidence, "config.RUST") && ev_bool(evidence, "config.RUST_IS_AVAILABLE") {
            "Compatible — kernel has CONFIG_RUST=y and Rust compiler is available"
        } else if ev_bool(evidence, "config.RUST") {
            "Partially Compatible — CONFIG_RUST=y but compiler not detected"
        } else if ev_bool(evidence, "config.RUST_IS_AVAILABLE") {
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

fn stars_str(categories: &[crate::core::analysis::CategoryScore]) -> &'static str {
    let total: u8 = categories.iter().map(|c| c.stars).sum();
    let max = (categories.len() * 5) as u8;
    if total >= max {
        return "★★★★★";
    }
    if total as f64 >= max as f64 * 0.8 {
        return "★★★★☆";
    }
    if total as f64 >= max as f64 * 0.6 {
        return "★★★☆☆";
    }
    if total as f64 >= max as f64 * 0.4 {
        return "★★☆☆☆";
    }
    "★☆☆☆☆"
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

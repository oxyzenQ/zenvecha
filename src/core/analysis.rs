// Copyright (C) 2026 rezky_nightky
// SPDX-License-Identifier: GPL-3.0-only

//! Analysis engine — consumes Evidence, produces readiness and risks.
//!
//! Never performs Linux probing. Pure transformation from Evidence
//! to structured analysis.

use super::evidence::{Evidence, EvidenceValue};

/// Overall readiness assessment.
#[derive(Clone, Debug)]
pub struct Readiness {
    pub overall: &'static str,
    pub stars: &'static str,
    pub categories: Vec<CategoryScore>,
}

#[derive(Clone, Debug)]
pub struct CategoryScore {
    pub name: &'static str,
    pub stars: u8,
    pub note: &'static str,
}

/// A risk identified from evidence.
#[derive(Clone, Debug)]
pub struct Risk {
    pub description: String,
    pub severity: &'static str, // "critical", "warning", "info"
}

/// Analyze evidence and produce readiness + risks.
pub fn analyze(evidence: &[Evidence]) -> (Readiness, Vec<Risk>) {
    let categories = compute_categories(evidence);
    let overall = overall_rating(&categories);
    let risks = identify_risks(evidence);
    let stars_str = stars_label(&categories);

    (
        Readiness {
            overall,
            stars: stars_str,
            categories,
        },
        risks,
    )
}

fn compute_categories(evidence: &[Evidence]) -> Vec<CategoryScore> {
    let config_mod = ev_bool(evidence, "config.MODULES");
    let config_btf = ev_bool(evidence, "config.DEBUG_INFO_BTF");
    let btf_ok = ev_bool(evidence, "debug.btf");
    let config_rust = ev_bool(evidence, "config.RUST");
    let rust_avail = ev_bool(evidence, "config.RUST_IS_AVAILABLE");
    let kallsyms = ev_bool(evidence, "symbols.kallsyms");
    let headers = ev_status_is(evidence, "build.headers", "Complete");
    let build_dir = ev_text_known(evidence, "build.dir");
    let source_dir = ev_text_known(evidence, "build.source");
    let debugfs = ev_bool(evidence, "fs.debugfs");
    let tracefs = ev_bool(evidence, "fs.tracefs");
    let rustc = ev_bool(evidence, "toolchain.rustc");
    let bindgen = ev_bool(evidence, "toolchain.bindgen");
    let llvm = ev_bool(evidence, "toolchain.llvm");
    let config_sig = ev_bool(evidence, "config.MODULE_SIG");

    vec![
        CategoryScore {
            name: "Kernel",
            stars: kernel_stars(kallsyms, config_mod, config_btf, btf_ok),
            note: "",
        },
        CategoryScore {
            name: "Headers",
            stars: header_stars(headers, build_dir, source_dir),
            note: "",
        },
        CategoryScore {
            name: "Module Dev",
            stars: module_dev_stars(config_mod, headers, config_sig),
            note: "",
        },
        CategoryScore {
            name: "Rust",
            stars: rust_stars(config_rust, rust_avail, rustc, bindgen),
            note: "",
        },
        CategoryScore {
            name: "Debug",
            stars: debug_stars(debugfs, tracefs, btf_ok, config_btf),
            note: "",
        },
        CategoryScore {
            name: "Toolchain",
            stars: toolchain_stars(rustc, bindgen, llvm),
            note: "",
        },
    ]
}

fn kernel_stars(kallsyms: bool, mods: bool, cfg_btf: bool, btf_ok: bool) -> u8 {
    let mut s: u8 = 0;
    if kallsyms {
        s += 1;
    }
    if mods {
        s += 1;
    }
    if cfg_btf && btf_ok {
        s += 2;
    } else if cfg_btf || btf_ok {
        s += 1;
    }
    s += 1; // base: kernel is running
    s.min(5)
}

fn header_stars(headers: bool, build: bool, source: bool) -> u8 {
    let mut s: u8 = 0;
    if headers {
        s += 2;
    }
    if build {
        s += 1;
    }
    if source {
        s += 2;
    }
    s.min(5)
}

fn module_dev_stars(mods: bool, headers: bool, sig: bool) -> u8 {
    let mut s: u8 = 0;
    if mods {
        s += 2;
    }
    if headers {
        s += 1;
    }
    if sig {
        s += 1;
    }
    s += 1;
    s.min(5)
}

fn rust_stars(rust_cfg: bool, rust_avail: bool, rustc: bool, bindgen: bool) -> u8 {
    let mut s: u8 = 0;
    if rust_cfg {
        s += 3;
    } else if rust_avail {
        s += 1;
    }
    if rustc {
        s += 1;
    }
    if bindgen {
        s += 1;
    }
    s.min(5)
}

fn debug_stars(debugfs: bool, tracefs: bool, btf: bool, cfg_btf: bool) -> u8 {
    let mut s: u8 = 0;
    if debugfs {
        s += 1;
    }
    if tracefs {
        s += 1;
    }
    if btf && cfg_btf {
        s += 2;
    } else if btf {
        s += 1;
    }
    s += 1;
    s.min(5)
}

fn toolchain_stars(rustc: bool, bindgen: bool, llvm: bool) -> u8 {
    let mut s: u8 = 0;
    if rustc {
        s += 2;
    }
    if bindgen {
        s += 1;
    }
    if llvm {
        s += 1;
    }
    s += 1;
    s.min(5)
}

fn overall_rating(categories: &[CategoryScore]) -> &'static str {
    let weak = categories.iter().filter(|c| c.stars <= 2).count();
    let strong = categories.iter().filter(|c| c.stars >= 4).count();
    if strong == categories.len() {
        "Ready"
    } else if weak <= 1 && strong as f64 >= categories.len() as f64 * 0.5 {
        "Mostly Ready"
    } else if weak <= 1 {
        "Needs Work"
    } else {
        "Needs Attention"
    }
}

fn stars_label(categories: &[CategoryScore]) -> &'static str {
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

fn identify_risks(evidence: &[Evidence]) -> Vec<Risk> {
    let mut risks = Vec::new();

    if !ev_bool(evidence, "config.RUST") && !ev_bool(evidence, "config.RUST_IS_AVAILABLE") {
        risks.push(Risk {
            description: "CONFIG_RUST not enabled — Rust-for-Linux unavailable".into(),
            severity: "warning",
        });
    }

    if !ev_bool(evidence, "config.MODULES") {
        risks.push(Risk {
            description: "CONFIG_MODULES not set — kernel module support missing".into(),
            severity: "critical",
        });
    }

    if !ev_bool(evidence, "symbols.kallsyms") {
        risks.push(Risk {
            description: "Kallsyms hidden — symbol analysis limited".into(),
            severity: "warning",
        });
    }

    if ev_bool(evidence, "config.DEBUG_INFO_BTF") && !ev_bool(evidence, "debug.btf") {
        risks.push(Risk {
            description: "CONFIG_DEBUG_INFO_BTF=y but BTF data not found".into(),
            severity: "warning",
        });
    }

    risks
}

/* ---------- evidence helpers ----------------------------------------------- */

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

// Copyright (C) 2026 rezky_nightky
// SPDX-License-Identifier: GPL-3.0-only

//! Readiness engine — computes categorized readiness scores from Evidence.
//!
//! Each category is scored independently. No cross-category knowledge.
//! Categories receive immutable Evidence and return CategoryScore.

use crate::core::evidence::Evidence;
use crate::core::evidence_helpers;

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

/// Compute all readiness categories from evidence.
pub fn compute_categories(evidence: &[Evidence]) -> Vec<CategoryScore> {
    let config_mod = evidence_helpers::ev_bool(evidence, "config.MODULES");
    let config_btf = evidence_helpers::ev_bool(evidence, "config.DEBUG_INFO_BTF");
    let btf_ok = evidence_helpers::ev_bool(evidence, "debug.btf");
    let config_rust = evidence_helpers::ev_bool(evidence, "config.RUST");
    let rust_avail = evidence_helpers::ev_bool(evidence, "config.RUST_IS_AVAILABLE");
    let kallsyms = evidence_helpers::ev_bool(evidence, "symbols.kallsyms");
    let headers = evidence_helpers::ev_status_is(evidence, "build.headers", "Complete");
    let build_dir = evidence_helpers::ev_text_known(evidence, "build.dir");
    let source_dir = evidence_helpers::ev_text_known(evidence, "build.source");
    let debugfs = evidence_helpers::ev_bool(evidence, "fs.debugfs");
    let tracefs = evidence_helpers::ev_bool(evidence, "fs.tracefs");
    let rustc = evidence_helpers::ev_bool(evidence, "toolchain.rustc");
    let bindgen = evidence_helpers::ev_bool(evidence, "toolchain.bindgen");
    let llvm = evidence_helpers::ev_bool(evidence, "toolchain.llvm");
    let config_sig = evidence_helpers::ev_bool(evidence, "config.MODULE_SIG");

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

/// Compute overall rating from category scores.
pub fn overall_rating(categories: &[CategoryScore]) -> &'static str {
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

/// Compute star label from category scores.
pub fn stars_label(categories: &[CategoryScore]) -> &'static str {
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

// Copyright (C) 2026 rezky_nightky
// SPDX-License-Identifier: GPL-3.0-only

//! Recommendation engine — generates actionable advice from Evidence.
//!
//! Recommendations are derived exclusively from Evidence. No duplicated
//! logic inside commands. Max 10 recommendations.

use super::evidence::{Evidence, EvidenceValue};

/// Generate prioritized recommendations from evidence.
pub fn recommend(evidence: &[Evidence]) -> Vec<String> {
    let mut recs: Vec<String> = Vec::new();

    // --- environment first (highest impact) --------------------------------

    // Header mismatch → reboot or install
    let headers_complete = ev_status_is(evidence, "build.headers", "Complete");
    let headers_partial = ev_status_is(evidence, "build.headers", "Partial");
    let build_dir = ev_text_known(evidence, "build.dir");
    let source_dir = ev_text_known(evidence, "build.source");
    let release = ev_text(evidence, "kernel.release");

    if !headers_complete {
        // If headers are present but wrong version → reboot
        if headers_partial {
            recs.push(format!(
                "Install kernel headers matching running kernel{release_suffix}",
                release_suffix = release.map(|r| format!(" ({r})")).unwrap_or_default()
            ));
        } else {
            recs.push("Install kernel headers matching running kernel".into());
        }
    }

    if !build_dir {
        recs.push("Install kernel headers to populate /lib/modules/$(uname -r)/build".into());
    }
    if !source_dir {
        recs.push(
            "Install kernel source or create symlink from /lib/modules/$(uname -r)/source".into(),
        );
    }

    // --- toolchain ----------------------------------------------------------

    let rustc = ev_bool(evidence, "toolchain.rustc");
    let bindgen = ev_bool(evidence, "toolchain.bindgen");
    let llvm = ev_bool(evidence, "toolchain.llvm");
    let config_rust = ev_bool(evidence, "config.RUST");
    let rust_avail = ev_bool(evidence, "config.RUST_IS_AVAILABLE");

    if !rustc {
        recs.push(
            "Install Rust: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh".into(),
        );
    }
    if !bindgen && (config_rust || rust_avail) {
        recs.push("Install bindgen: cargo install bindgen-cli".into());
    }
    if !llvm {
        recs.push("Install LLVM/clang for kernel compilation".into());
    }

    // --- Rust ---------------------------------------------------------------

    if !config_rust && !rust_avail {
        recs.push("Enable CONFIG_RUST=y in kernel configuration".into());
    }
    if rust_avail && !config_rust {
        recs.push("Compile kernel with CONFIG_RUST=y (compiler is available)".into());
    }

    // --- modules ------------------------------------------------------------

    if !ev_bool(evidence, "config.MODULES") {
        recs.push("Enable CONFIG_MODULES=y in kernel configuration".into());
    }

    let signing_req = ev_text(evidence, "modules.loader").is_some_and(|t| t.contains("signed=yes"));
    if !signing_req && ev_bool(evidence, "config.MODULES") {
        recs.push("Set up module signing keys for kernel module development".into());
    }

    // --- debug --------------------------------------------------------------

    if !ev_bool(evidence, "fs.debugfs") {
        recs.push("Mount debugfs: mount -t debugfs none /sys/kernel/debug".into());
    }
    if !ev_bool(evidence, "fs.tracefs") {
        recs.push("Mount tracefs: mount -t tracefs none /sys/kernel/tracing".into());
    }

    // Cap at 10
    recs.truncate(10);
    recs
}

/* ---------- helpers -------------------------------------------------------- */

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

fn ev_text(evidence: &[Evidence], id: &str) -> Option<String> {
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

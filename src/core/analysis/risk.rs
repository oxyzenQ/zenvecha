// Copyright (C) 2026 rezky_nightky
// SPDX-License-Identifier: GPL-3.0-only

//! Risk engine — identifies risks from Evidence.
//!
//! Independent engine. Receives immutable Evidence, returns Risk models.
//! No cross-engine knowledge.

use crate::core::evidence::Evidence;
use crate::core::evidence_helpers;

/// A risk identified from evidence.
#[derive(Clone, Debug)]
pub struct Risk {
    pub description: String,
    pub severity: &'static str, // "critical", "warning", "info"
}

/// Identify risks from evidence.
///
/// Each risk is derived exclusively from Evidence. No system probing,
/// no printing, no recommendation logic.
pub fn identify_risks(evidence: &[Evidence]) -> Vec<Risk> {
    let mut risks = Vec::new();

    if !evidence_helpers::ev_bool(evidence, "config.RUST")
        && !evidence_helpers::ev_bool(evidence, "config.RUST_IS_AVAILABLE")
    {
        risks.push(Risk {
            description: "CONFIG_RUST not enabled — Rust-for-Linux unavailable".into(),
            severity: "warning",
        });
    }

    if !evidence_helpers::ev_bool(evidence, "config.MODULES") {
        risks.push(Risk {
            description: "CONFIG_MODULES not set — kernel module support missing".into(),
            severity: "critical",
        });
    }

    if !evidence_helpers::ev_bool(evidence, "symbols.kallsyms") {
        risks.push(Risk {
            description: "Kallsyms hidden — symbol analysis limited".into(),
            severity: "warning",
        });
    }

    if evidence_helpers::ev_bool(evidence, "config.DEBUG_INFO_BTF")
        && !evidence_helpers::ev_bool(evidence, "debug.btf")
    {
        risks.push(Risk {
            description: "CONFIG_DEBUG_INFO_BTF=y but BTF data not found".into(),
            severity: "warning",
        });
    }

    risks
}

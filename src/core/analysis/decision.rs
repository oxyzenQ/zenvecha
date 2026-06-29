// Copyright (C) 2026 rezky_nightky
// SPDX-License-Identifier: GPL-3.0-only

//! Decision Engine — transforms Compatibility into an Action Plan.
//!
//! The Decision Engine answers:
//!   - What should the user do next?
//!   - Why? How much improvement? How long?
//!   - Are there alternatives? Which has highest ROI?
//!
//! Never coupled to rendering. Pure domain model consumed by any frontend
//! (CLI, JSON, report, REST API, TUI, GUI, AI advisor).

use crate::core::analysis::compatibility::Compatibility;
use crate::core::evidence::Evidence;
use crate::core::evidence_helpers;

// ============================================================================
//  Domain Models
// ============================================================================

/// A full decision plan derived from compatibility assessment.
#[derive(Clone, Debug)]
pub struct DecisionPlan {
    /// Current overall compatibility score (0–100).
    pub current_score: u8,
    /// Expected score after all recommended actions.
    pub expected_score: u8,
    /// Confidence in the expected score projection.
    pub confidence: Confidence,
    /// The single highest-ROI action.
    pub highest_roi_action: Option<DecisionAction>,
    /// All actions ranked by priority (best first).
    pub ranked_actions: Vec<DecisionAction>,
    /// Issues that block kernel development.
    pub blocking_issues: Vec<String>,
    /// Opportunities — things that would help but aren't required.
    pub opportunities: Vec<String>,
    /// Total estimated time to fix all issues (minutes).
    pub estimated_total_fix_minutes: u32,
}

/// A single recommended action with full decision metadata.
#[derive(Clone, Debug)]
pub struct DecisionAction {
    /// Rank position (1 = highest priority).
    pub rank: u8,
    /// Human-readable action title.
    pub title: String,
    /// Why this action matters.
    pub why: String,
    /// Estimated time to complete (minutes).
    pub estimated_minutes: u32,
    /// Expected score improvement from this action alone.
    pub expected_score_gain: u8,
    /// Return on investment (gain ÷ minutes, higher = better).
    pub roi: f64,
    /// Effort level.
    pub difficulty: Difficulty,
    /// Priority category.
    pub priority: ActionPriority,
    /// Whether this fixes a blocking issue.
    pub fixes_blocking: bool,
    /// Alternative approaches (if any).
    pub alternatives: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Difficulty {
    Trivial, // < 2 min
    Easy,    // 2–10 min
    Medium,  // 10–30 min
    Hard,    // 30–60 min
    Complex, // > 60 min
}

impl Difficulty {
    pub fn label(self) -> &'static str {
        match self {
            Difficulty::Trivial => "trivial",
            Difficulty::Easy => "easy",
            Difficulty::Medium => "medium",
            Difficulty::Hard => "hard",
            Difficulty::Complex => "complex",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ActionPriority {
    Critical = 0, // fixes blocking issue
    High = 1,     // high ROI + high gain
    Medium = 2,   // decent gain
    Low = 3,      // nice to have
}

impl ActionPriority {
    pub fn label(self) -> &'static str {
        match self {
            ActionPriority::Critical => "critical",
            ActionPriority::High => "high",
            ActionPriority::Medium => "medium",
            ActionPriority::Low => "low",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Confidence {
    High,
    Medium,
    Low,
}

impl Confidence {
    pub fn label(self) -> &'static str {
        match self {
            Confidence::High => "High",
            Confidence::Medium => "Medium",
            Confidence::Low => "Low",
        }
    }
}

// ============================================================================
//  Engine Entry Point
// ============================================================================

/// Evaluate evidence + compatibility and produce a DecisionPlan.
pub fn evaluate(evidence: &[Evidence], compatibility: &Compatibility) -> DecisionPlan {
    let actions = build_actions(evidence, compatibility);
    let ranked = rank_actions(actions);
    let highest_roi = ranked.first().cloned();
    let expected_score = compute_expected_score(compatibility.score, &ranked);
    let confidence = decision_confidence(compatibility, &ranked);
    let blocking_desc: Vec<String> = compatibility
        .blocking_issues
        .iter()
        .map(|b| format!("{}: {}", b.component, b.description))
        .collect();
    let opportunities = collect_opportunities(evidence, compatibility);
    let total_fix = ranked.iter().map(|a| a.estimated_minutes).sum();

    DecisionPlan {
        current_score: compatibility.score,
        expected_score,
        confidence,
        highest_roi_action: highest_roi,
        ranked_actions: ranked,
        blocking_issues: blocking_desc,
        opportunities,
        estimated_total_fix_minutes: total_fix,
    }
}

// ============================================================================
//  Action Construction
// ============================================================================

fn build_actions(evidence: &[Evidence], compatibility: &Compatibility) -> Vec<DecisionAction> {
    let mut actions = Vec::new();
    let release = evidence_helpers::ev_s(evidence, "kernel.release");
    let release_str = if release != "Unknown" {
        release
    } else {
        "$(uname -r)".into()
    };

    // ── Blocking: Kernel Headers ──
    if !evidence_helpers::ev_status_is(evidence, "build.headers", "Complete") {
        let gain = compute_header_gain(compatibility);
        actions.push(DecisionAction {
            rank: 0,
            title: format!("Install matching kernel headers for {release_str}"),
            why: "Missing headers prevent ALL external kernel module builds. This is the single most impactful fix.".into(),
            estimated_minutes: 3,
            expected_score_gain: gain,
            roi: gain as f64 / 3.0,
            difficulty: Difficulty::Trivial,
            priority: ActionPriority::Critical,
            fixes_blocking: true,
            alternatives: vec![
                format!("Install via package manager: apt install linux-headers-{release_str} or pacman -S linux-headers"),
                "Compile custom kernel with headers".into(),
                "Use distro-provided kernel package (if available)".into(),
            ],
        });
    }

    // ── Blocking: C Compiler ──
    if !evidence_helpers::ev_bool(evidence, "toolchain.gcc") {
        let gain = if compatibility.score < 30 { 25 } else { 10 };
        actions.push(DecisionAction {
            rank: 0,
            title: "Install C build toolchain (gcc + make)".into(),
            why: "No C compiler found — kernel modules require a C toolchain for compilation."
                .into(),
            estimated_minutes: 5,
            expected_score_gain: gain,
            roi: gain as f64 / 5.0,
            difficulty: Difficulty::Easy,
            priority: ActionPriority::Critical,
            fixes_blocking: true,
            alternatives: vec![
                "apt install build-essential".into(),
                "pacman -S base-devel".into(),
                "dnf groupinstall 'Development Tools'".into(),
            ],
        });
    }

    // ── Blocking: Kernel Config ──
    if evidence_helpers::ev_text_value(evidence, "config.source").is_none() {
        actions.push(DecisionAction {
            rank: 0,
            title: "Make kernel config accessible".into(),
            why: "Cannot read kernel config — compatibility assessment is incomplete without it."
                .into(),
            estimated_minutes: 2,
            expected_score_gain: 15,
            roi: 7.5,
            difficulty: Difficulty::Trivial,
            priority: ActionPriority::Critical,
            fixes_blocking: true,
            alternatives: vec![
                "Copy /boot/config-$(uname -r) if it exists".into(),
                "Enable CONFIG_IKCONFIG_PROC and rebuild kernel".into(),
                "Use zcat /proc/config.gz if available".into(),
            ],
        });
    }

    // ── Blocking: CONFIG_MODULES ──
    if evidence_helpers::ev_text_value(evidence, "config.source").is_some()
        && !evidence_helpers::ev_bool(evidence, "config.MODULES")
    {
        actions.push(DecisionAction {
            rank: 0,
            title: "Rebuild kernel with CONFIG_MODULES=y".into(),
            why: "Kernel compiled without module support — cannot load any kernel modules.".into(),
            estimated_minutes: 60,
            expected_score_gain: 20,
            roi: 0.33,
            difficulty: Difficulty::Complex,
            priority: ActionPriority::Critical,
            fixes_blocking: true,
            alternatives: vec![
                "Install a distro kernel package with modules enabled".into(),
                "Most distribution kernels already have CONFIG_MODULES=y".into(),
            ],
        });
    }

    // ── Kallsyms Access ──
    if !evidence_helpers::ev_status_is(evidence, "symbols.kallsyms", "readable")
        && !evidence_helpers::ev_status_is(evidence, "symbols.kallsyms", "readable (root)")
    {
        actions.push(DecisionAction {
            rank: 0,
            title: "Enable kallsyms access".into(),
            why: "Symbol table not readable — symbol resolution and function hooking unavailable."
                .into(),
            estimated_minutes: 1,
            expected_score_gain: 8,
            roi: 8.0,
            difficulty: Difficulty::Trivial,
            priority: ActionPriority::High,
            fixes_blocking: false,
            alternatives: vec![
                "Run: echo 0 > /proc/sys/kernel/kptr_restrict (as root)".into(),
                "Set kernel.kptr_restrict=0 in sysctl.conf".into(),
                "Run zenvecha as root for symbol access".into(),
            ],
        });
    }

    // ── BTF Data ──
    if evidence_helpers::ev_bool(evidence, "config.DEBUG_INFO_BTF")
        && !evidence_helpers::ev_bool(evidence, "debug.btf")
    {
        actions.push(DecisionAction {
            rank: 0,
            title: "Ensure BTF data available".into(),
            why: "CONFIG_DEBUG_INFO_BTF=y but BTF data not found — type-aware kernel introspection disabled.".into(),
            estimated_minutes: 10,
            expected_score_gain: 5,
            roi: 0.5,
            difficulty: Difficulty::Easy,
            priority: ActionPriority::Medium,
            fixes_blocking: false,
            alternatives: vec![
                "Rebuild kernel ensuring BTF generation succeeds".into(),
                "Check /sys/kernel/btf/vmlinux exists".into(),
            ],
        });
    }

    // ── Debugfs ──
    if !evidence_helpers::ev_bool(evidence, "fs.debugfs") {
        actions.push(DecisionAction {
            rank: 0,
            title: "Mount debugfs".into(),
            why: "debugfs provides kernel debugging interfaces used by many development tools."
                .into(),
            estimated_minutes: 1,
            expected_score_gain: 3,
            roi: 3.0,
            difficulty: Difficulty::Trivial,
            priority: ActionPriority::Medium,
            fixes_blocking: false,
            alternatives: vec![
                "mount -t debugfs none /sys/kernel/debug (as root)".into(),
                "Add to /etc/fstab for persistence".into(),
            ],
        });
    }

    // ── Tracefs ──
    if !evidence_helpers::ev_bool(evidence, "fs.tracefs") {
        actions.push(DecisionAction {
            rank: 0,
            title: "Mount tracefs".into(),
            why: "tracefs enables ftrace-based function tracing used for kernel hooking.".into(),
            estimated_minutes: 1,
            expected_score_gain: 3,
            roi: 3.0,
            difficulty: Difficulty::Trivial,
            priority: ActionPriority::Medium,
            fixes_blocking: false,
            alternatives: vec![
                "mount -t tracefs none /sys/kernel/tracing (as root)".into(),
                "Add to /etc/fstab for persistence".into(),
            ],
        });
    }

    // ── Rust Toolchain ──
    if evidence_helpers::ev_bool(evidence, "config.RUST")
        && !evidence_helpers::ev_bool(evidence, "toolchain.rustc")
    {
        actions.push(DecisionAction {
            rank: 0,
            title: "Install Rust compiler".into(),
            why:
                "Kernel has CONFIG_RUST=y but rustc not found — cannot compile Rust kernel modules."
                    .into(),
            estimated_minutes: 5,
            expected_score_gain: 10,
            roi: 2.0,
            difficulty: Difficulty::Easy,
            priority: ActionPriority::High,
            fixes_blocking: false,
            alternatives: vec![
                "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh".into(),
                "Install via package manager: apt install rustc cargo".into(),
            ],
        });
    }

    // ── Bindgen ──
    if evidence_helpers::ev_bool(evidence, "config.RUST")
        && evidence_helpers::ev_bool(evidence, "toolchain.rustc")
        && !evidence_helpers::ev_bool(evidence, "toolchain.bindgen")
    {
        actions.push(DecisionAction {
            rank: 0,
            title: "Install bindgen".into(),
            why: "bindgen generates Rust FFI bindings to kernel C headers — required for Rust kernel modules.".into(),
            estimated_minutes: 10,
            expected_score_gain: 5,
            roi: 0.5,
            difficulty: Difficulty::Easy,
            priority: ActionPriority::Medium,
            fixes_blocking: false,
            alternatives: vec![
                "cargo install bindgen-cli".into(),
                "Install via package manager if available".into(),
            ],
        });
    }

    // ── Module Signing ──
    if evidence_helpers::ev_bool(evidence, "config.MODULES")
        && !evidence_helpers::ev_bool(evidence, "config.MODULE_SIG")
    {
        actions.push(DecisionAction {
            rank: 0,
            title: "Set up module signing keys".into(),
            why: "Module signing not configured — unsigned modules will fail to load on secure boot systems.".into(),
            estimated_minutes: 15,
            expected_score_gain: 4,
            roi: 0.27,
            difficulty: Difficulty::Medium,
            priority: ActionPriority::Low,
            fixes_blocking: false,
            alternatives: vec![
                "Generate signing key and configure kernel".into(),
                "Disable secure boot in BIOS (not recommended)".into(),
            ],
        });
    }

    actions
}

// ============================================================================
//  Ranking
// ============================================================================

fn rank_actions(mut actions: Vec<DecisionAction>) -> Vec<DecisionAction> {
    // Sort: Critical first, then by ROI descending, then by score gain descending
    actions.sort_by(|a, b| {
        a.priority
            .cmp(&b.priority)
            .then_with(|| {
                b.roi
                    .partial_cmp(&a.roi)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .then_with(|| b.expected_score_gain.cmp(&a.expected_score_gain))
    });

    // Assign rank numbers
    for (i, action) in actions.iter_mut().enumerate() {
        action.rank = (i + 1) as u8;
    }

    actions
}

// ============================================================================
//  Score Projection
// ============================================================================

fn compute_expected_score(current: u8, ranked: &[DecisionAction]) -> u8 {
    let gain: u32 = ranked.iter().map(|a| a.expected_score_gain as u32).sum();
    (current as u32 + gain).min(100) as u8
}

fn decision_confidence(compatibility: &Compatibility, ranked: &[DecisionAction]) -> Confidence {
    // If we have blocking issues with clear fixes, confidence is high
    if !compatibility.blocking_issues.is_empty() && ranked.iter().any(|a| a.fixes_blocking) {
        return Confidence::High;
    }
    match compatibility.confidence {
        crate::core::analysis::Confidence::High => Confidence::High,
        crate::core::analysis::Confidence::Medium => Confidence::Medium,
        crate::core::analysis::Confidence::Low => Confidence::Low,
    }
}

fn compute_header_gain(compatibility: &Compatibility) -> u8 {
    // Headers are the most impactful single fix — they unlock all other checks
    let component = compatibility
        .components
        .iter()
        .find(|c| c.name == "Kernel Headers");
    match component {
        Some(c) if c.score == 0 => 25, // headers completely missing
        Some(c) if c.score < 50 => 15, // partial headers
        _ => 8,
    }
}

fn collect_opportunities(evidence: &[Evidence], _compatibility: &Compatibility) -> Vec<String> {
    let mut opportunities = Vec::new();

    // BTF opportunity
    if !evidence_helpers::ev_bool(evidence, "config.DEBUG_INFO_BTF") {
        opportunities
            .push("Enable CONFIG_DEBUG_INFO_BTF for type-aware kernel introspection".into());
    }

    // Rust opportunity
    if !evidence_helpers::ev_bool(evidence, "config.RUST")
        && evidence_helpers::ev_bool(evidence, "toolchain.rustc")
    {
        opportunities.push(
            "Rust compiler available — enable CONFIG_RUST=y to unlock Rust kernel modules".into(),
        );
    }

    // Livepatch opportunity
    if !evidence_helpers::ev_bool(evidence, "config.LIVEPATCH")
        && evidence_helpers::ev_bool(evidence, "config.MODULES")
    {
        opportunities.push("Enable CONFIG_LIVEPATCH for runtime kernel patching capability".into());
    }

    // DWARF opportunity
    if !evidence_helpers::ev_bool(evidence, "debug.dwarf") {
        opportunities
            .push("DWARF debug info not available — limits detailed kernel analysis".into());
    }

    opportunities
}

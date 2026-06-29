// Copyright (C) 2026 rezky_nightky
// SPDX-License-Identifier: GPL-3.0-only

//! Compatibility engine — assesses kernel development compatibility.
//!
//! Consumes Evidence, produces scored compatibility assessment.
//! Never probes the system. Pure transformation from Evidence.
//!
//! The engine answers:
//!   - How compatible is this system for kernel development?
//!   - What's blocking vs what's nice-to-have?
//!   - What should the user do next?

use crate::core::evidence::Evidence;
use crate::core::evidence_helpers;

/// Confidence level in the assessment.
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

/// Overall risk level.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RiskLevel {
    None,
    Low,
    Warning,
    Critical,
}

impl RiskLevel {
    pub fn label(self) -> &'static str {
        match self {
            RiskLevel::None => "None",
            RiskLevel::Low => "Low",
            RiskLevel::Warning => "Warning",
            RiskLevel::Critical => "Critical",
        }
    }
}

/// Component weight for weighted scoring.
#[derive(Clone, Debug)]
pub struct ComponentWeight {
    pub name: &'static str,
    pub weight: f64, // 0.0–1.0, all weights must sum to 1.0
}

/// A scored component in the compatibility assessment.
#[derive(Clone, Debug)]
pub struct ComponentScore {
    pub name: &'static str,
    pub score: u8, // 0-100
    pub status: ComponentStatus,
    pub detail: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ComponentStatus {
    Good,
    Partial,
    Missing,
    Blocking,
}

impl ComponentStatus {
    pub fn label(self) -> &'static str {
        match self {
            ComponentStatus::Good => "good",
            ComponentStatus::Partial => "partial",
            ComponentStatus::Missing => "missing",
            ComponentStatus::Blocking => "blocking",
        }
    }
}

/// An issue that must be resolved before kernel development.
#[derive(Clone, Debug)]
pub struct BlockingIssue {
    pub component: &'static str,
    pub description: String,
    pub severity: &'static str, // "critical", "warning"
}

/// A recommended action with estimated effort.
#[derive(Clone, Debug)]
pub struct RecommendedAction {
    pub priority: u8, // 1 = highest
    pub action: String,
    pub estimated_minutes: u32,
    pub component: &'static str,
}

/// Full compatibility assessment.
#[derive(Clone, Debug)]
pub struct Compatibility {
    /// Overall compatibility score 0-100.
    pub score: u8,
    /// Human-readable level label.
    pub level: &'static str,
    /// Confidence in this assessment.
    pub confidence: Confidence,
    /// Overall risk level.
    pub risk: RiskLevel,
    /// Per-component scores.
    pub components: Vec<ComponentScore>,
    /// Weight configuration used (core components weighted higher).
    pub weights: Vec<ComponentWeight>,
    /// Issues that absolutely block kernel development.
    pub blocking_issues: Vec<BlockingIssue>,
    /// Recommended next actions, ordered by priority.
    pub recommended_actions: Vec<RecommendedAction>,
    /// Total estimated fix time in minutes.
    pub estimated_fix_minutes: u32,
    /// Components that need attention.
    pub affected_components: Vec<&'static str>,
    /// The single most impactful next action.
    pub next_best_action: String,
}

/// Core component weights — sum = 1.0.
/// Headers and toolchain dominate because without them nothing works.
fn component_weights() -> Vec<ComponentWeight> {
    vec![
        ComponentWeight {
            name: "Kernel Headers",
            weight: 0.25,
        },
        ComponentWeight {
            name: "Toolchain",
            weight: 0.20,
        },
        ComponentWeight {
            name: "Kernel Config",
            weight: 0.15,
        },
        ComponentWeight {
            name: "Build Environment",
            weight: 0.10,
        },
        ComponentWeight {
            name: "Modules",
            weight: 0.10,
        },
        ComponentWeight {
            name: "Symbol Resolution",
            weight: 0.08,
        },
        ComponentWeight {
            name: "Rust Support",
            weight: 0.05,
        },
        ComponentWeight {
            name: "Debug Capabilities",
            weight: 0.04,
        },
        ComponentWeight {
            name: "Filesystem Mounts",
            weight: 0.03,
        },
    ]
}

/// Assess compatibility from evidence.
pub fn assess(evidence: &[Evidence]) -> Compatibility {
    let components = score_components(evidence);
    let weights = component_weights();
    let score = weighted_overall_score(&components, &weights);
    let blocking_issues = detect_blocking(evidence);
    let recommended_actions = recommend_actions(evidence, &blocking_issues);
    let affected_components = affected(&components);
    let confidence = assess_confidence(evidence);
    let risk = assess_risk(&components, &blocking_issues);
    let estimated_fix_minutes = recommended_actions
        .iter()
        .map(|a| a.estimated_minutes)
        .sum();
    let next_best_action = next_best(&recommended_actions, &blocking_issues);
    let level = score_label(score);

    Compatibility {
        score,
        level,
        confidence,
        risk,
        components,
        weights,
        blocking_issues,
        recommended_actions,
        estimated_fix_minutes,
        affected_components,
        next_best_action,
    }
}

fn score_components(evidence: &[Evidence]) -> Vec<ComponentScore> {
    vec![
        score_kernel_headers(evidence),
        score_build_env(evidence),
        score_kernel_config(evidence),
        score_modules(evidence),
        score_symbols(evidence),
        score_rust_support(evidence),
        score_toolchain(evidence),
        score_debug_capabilities(evidence),
        score_filesystem_mounts(evidence),
    ]
}

fn score_kernel_headers(evidence: &[Evidence]) -> ComponentScore {
    let headers_ok = evidence_helpers::ev_status_is(evidence, "build.headers", "Complete");
    let headers_partial = evidence_helpers::ev_status_is(evidence, "build.headers", "Partial");
    let release = evidence_helpers::ev_s(evidence, "kernel.release");

    if headers_ok {
        ComponentScore {
            name: "Kernel Headers",
            score: 100,
            status: ComponentStatus::Good,
            detail: format!("Headers match running kernel ({release})"),
        }
    } else if headers_partial {
        ComponentScore {
            name: "Kernel Headers",
            score: 40,
            status: ComponentStatus::Partial,
            detail: format!("Headers found but incomplete for {release}"),
        }
    } else {
        ComponentScore {
            name: "Kernel Headers",
            score: 0,
            status: ComponentStatus::Blocking,
            detail: "Kernel headers not installed — required for module development".into(),
        }
    }
}

fn score_build_env(evidence: &[Evidence]) -> ComponentScore {
    let build_dir = evidence_helpers::ev_text_known(evidence, "build.dir");
    let source_dir = evidence_helpers::ev_text_known(evidence, "build.source");
    let compile_cmds = evidence_helpers::ev_bool(evidence, "build.compile_commands");

    let mut score = 0u8;
    if build_dir {
        score += 40;
    }
    if source_dir {
        score += 40;
    }
    if compile_cmds {
        score += 20;
    }

    let status = if score >= 80 {
        ComponentStatus::Good
    } else if score >= 40 {
        ComponentStatus::Partial
    } else {
        ComponentStatus::Missing
    };

    let parts: Vec<&str> = [
        build_dir.then_some("build dir"),
        source_dir.then_some("source tree"),
        compile_cmds.then_some("compile_commands.json"),
    ]
    .into_iter()
    .flatten()
    .collect();

    ComponentScore {
        name: "Build Environment",
        score,
        status,
        detail: if parts.is_empty() {
            "No build environment components found".into()
        } else {
            format!("Found: {}", parts.join(", "))
        },
    }
}

fn score_kernel_config(evidence: &[Evidence]) -> ComponentScore {
    let config_available = evidence_helpers::ev_text_value(evidence, "config.source").is_some();
    let modules = evidence_helpers::ev_bool(evidence, "config.MODULES");
    let mod_sig = evidence_helpers::ev_bool(evidence, "config.MODULE_SIG");
    let bpf = evidence_helpers::ev_bool(evidence, "config.BPF");
    let kallsyms = evidence_helpers::ev_bool(evidence, "config.KALLSYMS");
    let livepatch = evidence_helpers::ev_bool(evidence, "config.LIVEPATCH");

    if !config_available {
        return ComponentScore {
            name: "Kernel Configuration",
            score: 10,
            status: ComponentStatus::Blocking,
            detail: "Cannot read kernel config — most checks impossible".into(),
        };
    }

    let mut score = 30u8; // base for having config
    if modules {
        score += 30;
    }
    if mod_sig {
        score += 10;
    }
    if bpf {
        score += 10;
    }
    if kallsyms {
        score += 10;
    }
    if livepatch {
        score += 10;
    }

    let missing: Vec<&str> = [
        (!modules).then_some("MODULES"),
        (!mod_sig).then_some("MODULE_SIG"),
        (!bpf).then_some("BPF"),
        (!kallsyms).then_some("KALLSYMS"),
    ]
    .into_iter()
    .flatten()
    .collect();

    ComponentScore {
        name: "Kernel Configuration",
        score: score.min(100),
        status: if modules {
            ComponentStatus::Good
        } else {
            ComponentStatus::Blocking
        },
        detail: if missing.is_empty() {
            "All key config options enabled".into()
        } else {
            format!("Missing: {}", missing.join(", "))
        },
    }
}

fn score_modules(evidence: &[Evidence]) -> ComponentScore {
    let cfg_available = evidence_helpers::ev_text_value(evidence, "config.source").is_some();
    let mod_support = evidence_helpers::ev_bool(evidence, "config.MODULES");
    let signing = evidence_helpers::ev_bool(evidence, "config.MODULE_SIG");
    let loader = evidence_helpers::ev_text_value(evidence, "modules.loader");

    if !cfg_available {
        return ComponentScore {
            name: "Module Environment",
            score: 0,
            status: ComponentStatus::Missing,
            detail: "Config not available — cannot assess module support".into(),
        };
    }

    let mut score = 0u8;
    if mod_support {
        score += 50;
    }
    if signing {
        score += 25;
    }
    if loader.is_some() {
        score += 25;
    }

    ComponentScore {
        name: "Module Environment",
        score,
        status: if mod_support {
            ComponentStatus::Good
        } else {
            ComponentStatus::Blocking
        },
        detail: if mod_support {
            "Module support enabled".into()
        } else {
            "CONFIG_MODULES not set — cannot build kernel modules".into()
        },
    }
}

fn score_symbols(evidence: &[Evidence]) -> ComponentScore {
    let ks_ok = evidence_helpers::ev_status_is(evidence, "symbols.kallsyms", "readable")
        || evidence_helpers::ev_status_is(evidence, "symbols.kallsyms", "readable (root)");
    let sym_count = evidence_helpers::ev_count(evidence, "symbols.count");
    let vmlinux = evidence_helpers::ev_literal(evidence, "symbols.vmlinux");
    let symvers = evidence_helpers::ev_text_value(evidence, "symbols.symvers");

    let mut score = 0u8;
    let mut found: Vec<&str> = Vec::new();
    let mut missing: Vec<&str> = Vec::new();

    if ks_ok {
        score += 35;
        found.push("kallsyms");
    } else {
        missing.push("kallsyms");
    }
    if sym_count != "0" {
        score += 15;
        found.push("symbols");
    } else {
        missing.push("symbols");
    }
    if vmlinux.is_some() {
        score += 25;
        found.push("vmlinux");
    } else {
        missing.push("vmlinux");
    }
    if symvers.is_some() {
        score += 25;
        found.push("symvers");
    } else {
        missing.push("symvers");
    }

    ComponentScore {
        name: "Symbol Information",
        score,
        status: if ks_ok {
            ComponentStatus::Good
        } else if score > 0 {
            ComponentStatus::Partial
        } else {
            ComponentStatus::Missing
        },
        detail: {
            let found_str = if found.is_empty() {
                "none".to_string()
            } else {
                found.join(", ")
            };
            let missing_str = if missing.is_empty() {
                "none".to_string()
            } else {
                missing.join(", ")
            };
            format!("Found: {found_str}. Missing: {missing_str}")
        },
    }
}

fn score_rust_support(evidence: &[Evidence]) -> ComponentScore {
    let rust_cfg = evidence_helpers::ev_bool(evidence, "config.RUST");
    let rust_avail = evidence_helpers::ev_bool(evidence, "config.RUST_IS_AVAILABLE");
    let rustc = evidence_helpers::ev_bool(evidence, "toolchain.rustc");
    let bindgen = evidence_helpers::ev_bool(evidence, "toolchain.bindgen");
    let cfg_available = evidence_helpers::ev_text_value(evidence, "config.source").is_some();

    if !cfg_available {
        return ComponentScore {
            name: "Rust for Linux",
            score: 0,
            status: ComponentStatus::Missing,
            detail: "Cannot check Rust config — kernel config not available".into(),
        };
    }

    let mut score = 0u8;
    let mut notes: Vec<&str> = Vec::new();

    if rust_cfg {
        score += 50;
        notes.push("CONFIG_RUST=y");
    } else if rust_avail {
        score += 25;
        notes.push("compiler available, CONFIG_RUST not set");
    } else {
        notes.push("Rust support not enabled");
    }
    if rustc {
        score += 25;
        notes.push("rustc installed");
    }
    if bindgen {
        score += 25;
        notes.push("bindgen installed");
    }

    ComponentScore {
        name: "Rust for Linux",
        score,
        status: if rust_cfg && rustc && bindgen {
            ComponentStatus::Good
        } else if rust_avail || rustc {
            ComponentStatus::Partial
        } else {
            ComponentStatus::Missing
        },
        detail: notes.join(", "),
    }
}

fn score_toolchain(evidence: &[Evidence]) -> ComponentScore {
    let rustc = evidence_helpers::ev_bool(evidence, "toolchain.rustc");
    let bindgen = evidence_helpers::ev_bool(evidence, "toolchain.bindgen");
    let llvm = evidence_helpers::ev_bool(evidence, "toolchain.llvm");
    let make = evidence_helpers::ev_bool(evidence, "toolchain.make");
    let gcc = evidence_helpers::ev_bool(evidence, "toolchain.gcc");

    let mut score = 0u8;
    let mut missing: Vec<&str> = Vec::new();

    if gcc {
        score += 30;
    } else {
        missing.push("gcc");
    }
    if make {
        score += 15;
    } else {
        missing.push("make");
    }
    if rustc {
        score += 20;
    }
    if bindgen {
        score += 15;
    }
    if llvm {
        score += 20;
    }

    ComponentScore {
        name: "Toolchain",
        score,
        status: if gcc && make {
            ComponentStatus::Good
        } else if gcc {
            ComponentStatus::Partial
        } else {
            ComponentStatus::Blocking
        },
        detail: if missing.is_empty() {
            "All core tools available".into()
        } else {
            format!("Missing: {}", missing.join(", "))
        },
    }
}

fn score_debug_capabilities(evidence: &[Evidence]) -> ComponentScore {
    let btf = evidence_helpers::ev_bool(evidence, "debug.btf");
    let dwarf = evidence_helpers::ev_bool(evidence, "debug.dwarf");
    let cfg_btf = evidence_helpers::ev_bool(evidence, "config.DEBUG_INFO_BTF");

    let mut score = 0u8;
    let mut notes: Vec<&str> = Vec::new();

    if btf {
        score += 50;
        notes.push("BTF available");
    } else if cfg_btf {
        score += 20;
        notes.push("BTF configured but data missing");
    }
    if dwarf {
        score += 50;
        notes.push("DWARF available");
    }

    ComponentScore {
        name: "Debug Capabilities",
        score,
        status: if btf && dwarf {
            ComponentStatus::Good
        } else if btf || dwarf {
            ComponentStatus::Partial
        } else {
            ComponentStatus::Missing
        },
        detail: if notes.is_empty() {
            "No debug data available".into()
        } else {
            notes.join(", ")
        },
    }
}

fn score_filesystem_mounts(evidence: &[Evidence]) -> ComponentScore {
    let debugfs = evidence_helpers::ev_bool(evidence, "fs.debugfs");
    let tracefs = evidence_helpers::ev_bool(evidence, "fs.tracefs");

    let mut score = 0u8;
    if debugfs {
        score += 50;
    }
    if tracefs {
        score += 50;
    }

    ComponentScore {
        name: "Filesystem Mounts",
        score,
        status: if score == 100 {
            ComponentStatus::Good
        } else if score >= 50 {
            ComponentStatus::Partial
        } else {
            ComponentStatus::Missing
        },
        detail: format!(
            "debugfs: {}, tracefs: {}",
            if debugfs { "mounted" } else { "not mounted" },
            if tracefs { "mounted" } else { "not mounted" },
        ),
    }
}

/// Weighted scoring — core components (headers, toolchain) contribute more.
fn weighted_overall_score(components: &[ComponentScore], weights: &[ComponentWeight]) -> u8 {
    if components.is_empty() {
        return 0;
    }
    let weighted: f64 = components
        .iter()
        .map(|c| {
            let w = weights
                .iter()
                .find(|cw| cw.name == c.name)
                .map(|cw| cw.weight)
                .unwrap_or(0.0);
            c.score as f64 * w
        })
        .sum();
    weighted.round() as u8
}

fn score_label(score: u8) -> &'static str {
    match score {
        95..=100 => "Excellent",
        80..=94 => "Good",
        60..=79 => "Adequate",
        40..=59 => "Needs Work",
        20..=39 => "Poor",
        _ => "Critical",
    }
}

fn detect_blocking(evidence: &[Evidence]) -> Vec<BlockingIssue> {
    let mut issues = Vec::new();

    // No kernel config — blocks everything
    if evidence_helpers::ev_text_value(evidence, "config.source").is_none() {
        issues.push(BlockingIssue {
            component: "Kernel Configuration",
            description:
                "Cannot read kernel config — install kernel headers or enable CONFIG_IKCONFIG_PROC"
                    .into(),
            severity: "critical",
        });
    }

    // No MODULES — blocks module development
    if !evidence_helpers::ev_bool(evidence, "config.MODULES")
        && evidence_helpers::ev_text_value(evidence, "config.source").is_some()
    {
        issues.push(BlockingIssue {
            component: "Module Support",
            description: "CONFIG_MODULES not set — kernel module support missing".into(),
            severity: "critical",
        });
    }

    // No headers — blocks compilation
    if !evidence_helpers::ev_status_is(evidence, "build.headers", "Complete") {
        let release = evidence_helpers::ev_s(evidence, "kernel.release");
        issues.push(BlockingIssue {
            component: "Kernel Headers",
            description: format!("Kernel headers not installed for {release}"),
            severity: "critical",
        });
    }

    // No C compiler — blocks everything
    if !evidence_helpers::ev_bool(evidence, "toolchain.gcc") {
        issues.push(BlockingIssue {
            component: "Toolchain",
            description: "No C compiler found — install gcc or clang".into(),
            severity: "critical",
        });
    }

    issues
}

fn recommend_actions(evidence: &[Evidence], blocking: &[BlockingIssue]) -> Vec<RecommendedAction> {
    let mut actions = Vec::new();
    let release = evidence_helpers::ev_s(evidence, "kernel.release");
    let release_str = if release != "Unknown" {
        release
    } else {
        "$(uname -r)".into()
    };

    // 1. Blocking issues first
    for issue in blocking {
        match issue.component {
            "Kernel Headers" => {
                actions.push(RecommendedAction {
                    priority: 1,
                    action: format!(
                        "Install kernel headers: apt install linux-headers-{release_str} or pacman -S linux-headers"
                    ),
                    estimated_minutes: 3,
                    component: "headers",
                });
            }
            "Toolchain" => {
                actions.push(RecommendedAction {
                    priority: 1,
                    action:
                        "Install build tools: apt install build-essential or pacman -S base-devel"
                            .into(),
                    estimated_minutes: 5,
                    component: "toolchain",
                });
            }
            "Module Support" => {
                actions.push(RecommendedAction {
                    priority: 1,
                    action: "Rebuild kernel with CONFIG_MODULES=y".into(),
                    estimated_minutes: 60,
                    component: "kernel",
                });
            }
            "Kernel Configuration" => {
                actions.push(RecommendedAction {
                    priority: 1,
                    action: "Install kernel config: ensure /boot/config-$(uname -r) exists or enable CONFIG_IKCONFIG_PROC".into(),
                    estimated_minutes: 2,
                    component: "config",
                });
            }
            _ => {}
        }
    }

    // 2. Symbols — kallsyms access
    if !evidence_helpers::ev_status_is(evidence, "symbols.kallsyms", "readable")
        && !evidence_helpers::ev_status_is(evidence, "symbols.kallsyms", "readable (root)")
    {
        actions.push(RecommendedAction {
            priority: 2,
            action: "Enable kallsyms: echo 0 > /proc/sys/kernel/kptr_restrict (as root)".into(),
            estimated_minutes: 1,
            component: "symbols",
        });
    }

    // 3. BTF
    if evidence_helpers::ev_bool(evidence, "config.DEBUG_INFO_BTF")
        && !evidence_helpers::ev_bool(evidence, "debug.btf")
    {
        actions.push(RecommendedAction {
            priority: 2,
            action: "BTF configured but data not found — ensure /sys/kernel/btf/vmlinux exists"
                .into(),
            estimated_minutes: 10,
            component: "debug",
        });
    }

    // 4. Debugfs
    if !evidence_helpers::ev_bool(evidence, "fs.debugfs") {
        actions.push(RecommendedAction {
            priority: 3,
            action: "Mount debugfs: mount -t debugfs none /sys/kernel/debug (as root)".into(),
            estimated_minutes: 1,
            component: "filesystem",
        });
    }

    // 5. Tracefs
    if !evidence_helpers::ev_bool(evidence, "fs.tracefs") {
        actions.push(RecommendedAction {
            priority: 3,
            action: "Mount tracefs: mount -t tracefs none /sys/kernel/tracing (as root)".into(),
            estimated_minutes: 1,
            component: "filesystem",
        });
    }

    // 6. Rust toolchain
    if evidence_helpers::ev_bool(evidence, "config.RUST")
        && !evidence_helpers::ev_bool(evidence, "toolchain.rustc")
    {
        actions.push(RecommendedAction {
            priority: 2,
            action: "Install Rust: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
                .into(),
            estimated_minutes: 5,
            component: "rust",
        });
    }

    // 7. Bindgen
    if evidence_helpers::ev_bool(evidence, "config.RUST")
        && evidence_helpers::ev_bool(evidence, "toolchain.rustc")
        && !evidence_helpers::ev_bool(evidence, "toolchain.bindgen")
    {
        actions.push(RecommendedAction {
            priority: 3,
            action: "Install bindgen: cargo install bindgen-cli".into(),
            estimated_minutes: 10,
            component: "rust",
        });
    }

    // 8. Module signing
    if evidence_helpers::ev_bool(evidence, "config.MODULES")
        && !evidence_helpers::ev_bool(evidence, "config.MODULE_SIG")
    {
        actions.push(RecommendedAction {
            priority: 3,
            action: "Set up module signing keys for kernel module development".into(),
            estimated_minutes: 15,
            component: "modules",
        });
    }

    // Sort by priority
    actions.sort_by_key(|a| a.priority);
    actions.truncate(7);
    actions
}

fn affected(components: &[ComponentScore]) -> Vec<&'static str> {
    components
        .iter()
        .filter(|c| c.status == ComponentStatus::Blocking || c.status == ComponentStatus::Missing)
        .map(|c| c.name)
        .collect()
}

fn assess_confidence(evidence: &[Evidence]) -> Confidence {
    let config_available = evidence_helpers::ev_text_value(evidence, "config.source").is_some();
    let ks_readable = evidence_helpers::ev_status_is(evidence, "symbols.kallsyms", "readable")
        || evidence_helpers::ev_status_is(evidence, "symbols.kallsyms", "readable (root)");
    let headers_ok = evidence_helpers::ev_status_is(evidence, "build.headers", "Complete");

    let signals = [config_available, ks_readable, headers_ok];
    let good = signals.iter().filter(|&&s| s).count();

    if good == 3 {
        Confidence::High
    } else if good >= 1 {
        Confidence::Medium
    } else {
        Confidence::Low
    }
}

fn assess_risk(components: &[ComponentScore], blocking: &[BlockingIssue]) -> RiskLevel {
    if blocking.len() >= 3 {
        return RiskLevel::Critical;
    }
    if !blocking.is_empty() {
        return RiskLevel::Warning;
    }
    let low_components = components.iter().filter(|c| c.score < 40).count();
    if low_components >= 3 {
        RiskLevel::Warning
    } else if low_components >= 1 {
        RiskLevel::Low
    } else {
        RiskLevel::None
    }
}

fn next_best(actions: &[RecommendedAction], blocking: &[BlockingIssue]) -> String {
    if !blocking.is_empty() {
        let first = &blocking[0];
        return format!("→ Resolve: {}", first.description);
    }
    if let Some(action) = actions.first() {
        return format!("→ {}", action.action);
    }
    "✓ System ready for kernel development".into()
}

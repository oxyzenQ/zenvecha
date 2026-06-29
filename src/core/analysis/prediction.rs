// Copyright (C) 2026 rezky_nightky
// SPDX-License-Identifier: GPL-3.0-only

//! Prediction Engine — simulates future consequences of every possible action.
//!
//! Transforms Zenvecha from answering "What should I do?" into answering
//! "If you choose this action, what will most likely happen?"
//!
//! Pure domain engine. Never prints, never probes the system, never renders.
//! Deterministic — derived exclusively from Evidence + Compatibility + Decision.
//!
//! Consumable by any frontend: CLI, JSON, report, REST API, TUI, GUI, AI.

use crate::core::analysis::compatibility::Compatibility;
use crate::core::analysis::decision::{DecisionAction, DecisionPlan};
use crate::core::evidence::Evidence;
use crate::core::evidence_helpers;

// ============================================================================
//  Domain Models
// ============================================================================

/// Full prediction result — current state + simulated future scenarios.
#[derive(Clone, Debug)]
pub struct PredictionResult {
    /// Current overall compatibility score (0–100).
    pub current_score: u8,
    /// All simulated scenarios, ordered by expected score (best first).
    pub scenarios: Vec<Scenario>,
}

/// A simulated future state resulting from taking one action.
#[derive(Clone, Debug)]
pub struct Scenario {
    /// Human-readable scenario title.
    pub title: String,
    /// The action that triggers this scenario.
    pub action: String,
    /// Expected compatibility score after this action.
    pub expected_score: u8,
    /// Score change (positive = improvement, zero = no change, negative = regression).
    pub score_delta: i8,
    /// Expected risk level after this action.
    pub expected_risk: PredictedRisk,
    /// Confidence in this prediction.
    pub confidence: PredictionConfidence,
    /// Expected build success probability (0–100).
    pub expected_build_success: u8,
    /// Estimated time to complete (minutes).
    pub estimated_minutes: u32,
    /// Whether this action requires a system reboot.
    pub requires_reboot: bool,
    /// Capabilities unlocked by this action.
    pub unlocked_capabilities: Vec<String>,
    /// What might break or change as a side effect.
    pub breaking_changes: Vec<String>,
    /// Warnings the user should be aware of.
    pub warnings: Vec<String>,
    /// Assumptions underlying this prediction.
    pub assumptions: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PredictedRisk {
    None,
    Low,
    Medium,
    High,
    Critical,
}

impl PredictedRisk {
    pub fn label(self) -> &'static str {
        match self {
            PredictedRisk::None => "None",
            PredictedRisk::Low => "Low",
            PredictedRisk::Medium => "Medium",
            PredictedRisk::High => "High",
            PredictedRisk::Critical => "Critical",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PredictionConfidence {
    High,   // >85% certainty
    Medium, // 60-85%
    Low,    // <60%
}

impl PredictionConfidence {
    pub fn label(self) -> &'static str {
        match self {
            PredictionConfidence::High => "High",
            PredictionConfidence::Medium => "Medium",
            PredictionConfidence::Low => "Low",
        }
    }

    pub fn percentage(self) -> u8 {
        match self {
            PredictionConfidence::High => 92,
            PredictionConfidence::Medium => 72,
            PredictionConfidence::Low => 45,
        }
    }
}

// ============================================================================
//  Engine Entry Point
// ============================================================================

/// Simulate future scenarios for every ranked action in the decision plan.
///
/// Deterministic. No randomness. Derived exclusively from inputs.
pub fn simulate(
    evidence: &[Evidence],
    compatibility: &Compatibility,
    decision_plan: &DecisionPlan,
) -> PredictionResult {
    let mut scenarios: Vec<Scenario> = decision_plan
        .ranked_actions
        .iter()
        .map(|action| simulate_one(evidence, compatibility, action))
        .collect();

    // Sort by expected score descending (best outcome first)
    scenarios.sort_by_key(|s| std::cmp::Reverse(s.expected_score));

    PredictionResult {
        current_score: compatibility.score,
        scenarios,
    }
}

// ============================================================================
//  Single Scenario Simulation
// ============================================================================

fn simulate_one(
    evidence: &[Evidence],
    compatibility: &Compatibility,
    action: &DecisionAction,
) -> Scenario {
    let expected_score = compatibility
        .score
        .saturating_add(action.expected_score_gain)
        .min(100);
    let score_delta = expected_score as i8 - compatibility.score as i8;
    let expected_risk = predict_risk_after(compatibility, action);
    let confidence = predict_confidence(evidence, action, expected_score);
    let build_success = predict_build_success(evidence, action);
    let requires_reboot = predict_reboot(action);
    let unlocked = predict_unlocked(evidence, action);
    let breaking = predict_breaking_changes(action);
    let warnings = predict_warnings(evidence, action);
    let assumptions = build_assumptions(evidence, action);

    Scenario {
        title: format!("Apply: {}", action.title),
        action: action.title.clone(),
        expected_score,
        score_delta,
        expected_risk,
        confidence,
        expected_build_success: build_success,
        estimated_minutes: action.estimated_minutes,
        requires_reboot,
        unlocked_capabilities: unlocked,
        breaking_changes: breaking,
        warnings,
        assumptions,
    }
}

// ============================================================================
//  Risk Prediction
// ============================================================================

fn predict_risk_after(compatibility: &Compatibility, action: &DecisionAction) -> PredictedRisk {
    // Blocking fixes reduce risk dramatically
    if action.fixes_blocking {
        // After fixing headers, risk drops significantly
        if action.title.contains("headers") {
            return if compatibility.blocking_issues.len() <= 2 {
                PredictedRisk::Low
            } else {
                PredictedRisk::Medium
            };
        }
        return if compatibility.blocking_issues.len() <= 1 {
            PredictedRisk::Low
        } else {
            PredictedRisk::Medium
        };
    }

    // Kernel rebuilds carry medium risk (ABI changes possible)
    if action.title.contains("Rebuild kernel") || action.title.contains("CONFIG_MODULES") {
        return PredictedRisk::Medium;
    }

    // System config changes (sysctl, mounts) — low risk
    if action.title.contains("kptr_restrict") || action.title.contains("Mount") {
        return PredictedRisk::Low;
    }

    // Toolchain installs — no risk
    if action.title.contains("Install") && action.difficulty as u8 <= 1 {
        return PredictedRisk::None;
    }

    // Default: assess based on existing risk
    match compatibility.risk {
        crate::core::analysis::RiskLevel::Critical => PredictedRisk::Medium,
        crate::core::analysis::RiskLevel::Warning => PredictedRisk::Low,
        crate::core::analysis::RiskLevel::Low => PredictedRisk::Low,
        crate::core::analysis::RiskLevel::None => PredictedRisk::None,
    }
}

// ============================================================================
//  Confidence Prediction
// ============================================================================

fn predict_confidence(
    evidence: &[Evidence],
    action: &DecisionAction,
    expected_score: u8,
) -> PredictionConfidence {
    let mut signals = 0u8;
    let mut total = 0u8;

    // Signal 1: Config available → high confidence
    total += 1;
    if evidence_helpers::ev_text_value(evidence, "config.source").is_some() {
        signals += 1;
    }

    // Signal 2: Release known → high confidence
    total += 1;
    if evidence_helpers::ev_s(evidence, "kernel.release") != "Unknown" {
        signals += 1;
    }

    // Signal 3: Low difficulty action → higher confidence
    total += 1;
    if action.difficulty as u8 <= 1 {
        signals += 1;
    }

    // Signal 4: High expected score → higher confidence in outcome
    total += 1;
    if expected_score >= 80 {
        signals += 1;
    }

    // Signal 5: Fixes blocking issue → we know exactly what will improve
    total += 1;
    if action.fixes_blocking {
        signals += 1;
    }

    let ratio = signals as f64 / total as f64;
    if ratio >= 0.8 {
        PredictionConfidence::High
    } else if ratio >= 0.5 {
        PredictionConfidence::Medium
    } else {
        PredictionConfidence::Low
    }
}

// ============================================================================
//  Build Success Prediction
// ============================================================================

fn predict_build_success(evidence: &[Evidence], action: &DecisionAction) -> u8 {
    // Header installation → enables builds
    if action.title.contains("headers") {
        return if evidence_helpers::ev_bool(evidence, "toolchain.gcc") {
            95
        } else {
            70 // still need gcc
        };
    }

    // Toolchain install → enables builds
    if action.title.contains("gcc") || action.title.contains("build toolchain") {
        return 90;
    }

    // Kernel rebuild → complex, medium success rate
    if action.title.contains("Rebuild kernel") {
        return 65;
    }

    // Module signing → improves module load success
    if action.title.contains("signing") {
        return 85;
    }

    // Config access → enables better assessments
    if action.title.contains("config") {
        return if evidence_helpers::ev_bool(evidence, "toolchain.gcc")
            && evidence_helpers::ev_bool(evidence, "config.MODULES")
        {
            88
        } else {
            60
        };
    }

    // Default: depends on current state
    let has_gcc = evidence_helpers::ev_bool(evidence, "toolchain.gcc");
    let has_headers = evidence_helpers::ev_status_is(evidence, "build.headers", "Complete");
    if has_gcc && has_headers {
        80
    } else if has_gcc || has_headers {
        50
    } else {
        25
    }
}

// ============================================================================
//  Reboot Prediction
// ============================================================================

fn predict_reboot(action: &DecisionAction) -> bool {
    // Only kernel rebuilds require reboot
    action.title.contains("Rebuild kernel") || action.title.contains("CONFIG_")
}

// ============================================================================
//  Unlocked Capabilities
// ============================================================================

fn predict_unlocked(_evidence: &[Evidence], action: &DecisionAction) -> Vec<String> {
    let mut unlocked = Vec::new();

    // Headers → unlock module development
    if action.title.contains("headers") {
        unlocked.push("External kernel module compilation".into());
        unlocked.push("Kernel source inspection".into());
    }

    // GCC → unlock compilation
    if action.title.contains("gcc") || action.title.contains("build toolchain") {
        unlocked.push("C kernel module compilation".into());
    }

    // kallsyms → unlock symbol resolution
    if action.title.contains("kptr_restrict") || action.title.contains("kallsyms") {
        unlocked.push("Kernel symbol resolution".into());
        unlocked.push("Function hooking capability".into());
    }

    // Rust → unlock Rust modules
    if action.title.contains("Rust") || action.title.contains("rustc") {
        unlocked.push("Rust kernel module development".into());
    }

    // Bindgen → unlock Rust FFI
    if action.title.contains("bindgen") {
        unlocked.push("Rust FFI kernel bindings generation".into());
    }

    // BTF → unlock type introspection
    if action.title.contains("BTF") {
        unlocked.push("Type-aware kernel introspection".into());
    }

    // Debugfs → unlock debugging interfaces
    if action.title.contains("debugfs") {
        unlocked.push("Kernel debugging interfaces".into());
    }

    // Tracefs → unlock tracing
    if action.title.contains("tracefs") {
        unlocked.push("Kernel function tracing (ftrace)".into());
        unlocked.push("Performance analysis".into());
    }

    // MODULES → unlock everything module-related
    if action.title.contains("CONFIG_MODULES") {
        unlocked.push("Kernel module loading".into());
        unlocked.push("External module development".into());
        unlocked.push("Runtime kernel extension".into());
    }

    // Livepatch → unlock runtime patching
    if action.title.contains("livepatch") || action.title.contains("LIVEPATCH") {
        unlocked.push("Runtime kernel patching (livepatch)".into());
        unlocked.push("Zero-downtime kernel fixes".into());
    }

    unlocked
}

// ============================================================================
//  Breaking Changes
// ============================================================================

fn predict_breaking_changes(action: &DecisionAction) -> Vec<String> {
    let mut changes = Vec::new();

    if action.title.contains("Rebuild kernel") {
        changes.push("Custom kernel may differ from distro kernel".into());
        changes.push("Existing modules may need recompilation".into());
        changes.push("ABI compatibility not guaranteed across versions".into());
    }

    if action.title.contains("CONFIG_") {
        changes.push("Kernel configuration change requires rebuild".into());
        changes.push("May affect system boot behavior".into());
    }

    if action.title.contains("signing") && action.title.contains("key") {
        changes.push("Module signing keys must be managed securely".into());
        changes.push("Secure boot configuration may need update".into());
    }

    changes
}

// ============================================================================
//  Warnings
// ============================================================================

fn predict_warnings(evidence: &[Evidence], action: &DecisionAction) -> Vec<String> {
    let mut warnings = Vec::new();

    if predict_reboot(action) {
        warnings.push("This action requires a system reboot to take effect".into());
    }

    if action.estimated_minutes > 30 {
        warnings.push(format!(
            "This is a significant time investment (~{} min)",
            action.estimated_minutes
        ));
    }

    if action.title.contains("headers")
        && evidence_helpers::ev_s(evidence, "kernel.release") == "Unknown"
    {
        warnings.push("Cannot verify kernel version — headers may not match running kernel".into());
    }

    if action.title.contains("Rebuild kernel") {
        warnings.push("Kernel compilation may fail on first attempt".into());
        warnings.push("Backup current kernel before proceeding".into());
    }

    if action.expected_score_gain < 3 {
        warnings.push("This action provides minimal score improvement".into());
    }

    warnings
}

// ============================================================================
//  Assumptions
// ============================================================================

fn build_assumptions(evidence: &[Evidence], action: &DecisionAction) -> Vec<String> {
    let mut assumptions = Vec::new();
    let release = evidence_helpers::ev_s(evidence, "kernel.release");

    if action.title.contains("headers") {
        assumptions.push(format!("Package manager has linux-headers for {release}"));
        assumptions.push("Headers will match running kernel version".into());
    }

    if action.title.contains("gcc") || action.title.contains("build toolchain") {
        assumptions.push("System has internet access for package installation".into());
        assumptions.push("User has sudo/root access".into());
    }

    if action.title.contains("rustc") {
        assumptions.push("Rustup installation succeeds without network issues".into());
    }

    if action.title.contains("Rebuild kernel") {
        assumptions.push("User has sufficient disk space (~20GB)".into());
        assumptions.push("Build environment meets kernel compilation requirements".into());
        assumptions.push(format!("Kernel source for {release} is available"));
    }

    if action.title.contains("kptr_restrict") || action.title.contains("Mount") {
        assumptions.push("User has root access".into());
    }

    if action.estimated_minutes > 5 {
        assumptions.push("System remains stable and connected during the operation".into());
    }

    assumptions
}

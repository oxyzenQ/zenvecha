// Copyright (C) 2026 rezky_nightky
// SPDX-License-Identifier: GPL-3.0-only

//! Semantic normalization engine.
//!
//! Deterministic, rule-based mapping of raw Evidence → SemanticDescriptor.
//! Rules are applied in declaration order. Each rule is a pure function:
//!   `fn(&[Evidence]) -> Option<SemanticDescriptor>`
//!
//! No AI. No randomness. Reproducible across runs.

use crate::core::evidence::Evidence;
use crate::core::evidence_helpers;

use super::model::{SemanticDescriptor, SemanticDomain, SemanticState};

/// A normalization rule: inspects evidence, produces zero or one semantic descriptor.
type NormalizeRule = fn(&[Evidence]) -> Option<SemanticDescriptor>;

/// Run all normalization rules against the evidence set.
///
/// Each rule independently inspects evidence and may produce
/// zero or one semantic descriptor. Rules never modify evidence.
pub fn normalize(evidence: &[Evidence]) -> Vec<SemanticDescriptor> {
    let rules: &[NormalizeRule] = &[
        normalize_security_posture,
        normalize_memory_class,
        normalize_scheduler_class,
        normalize_performance_tier,
        normalize_stability_tier,
        normalize_feature_availability,
        normalize_runtime_risk,
    ];

    rules.iter().filter_map(|rule| rule(evidence)).collect()
}

// ============================================================================
//  Normalization Rules — one function per semantic domain
// ============================================================================

/// SECURITY_POSTURE: derived from lockdown + LSMs + KASLR
fn normalize_security_posture(evidence: &[Evidence]) -> Option<SemanticDescriptor> {
    let lockdown = evidence_helpers::ev_s(evidence, "kernel.security.lockdown");
    let lsms = evidence_helpers::ev_s(evidence, "kernel.security.lsms");
    let kaslr = evidence_helpers::ev_bool(evidence, "kernel.security.kaslr");

    // Domain from kernel module — if module not loaded, fall back to config-based assessment
    let state = if lockdown.contains("confidentiality") {
        SemanticState::SecurityPostureHigh
    } else if lockdown.contains("integrity")
        || lockdown.contains("none")
        || lsms.contains("selinux")
        || kaslr
    {
        SemanticState::SecurityPostureMedium
    } else {
        // Fall back to config: check if any security config is enabled
        let selinux_cfg = evidence_helpers::ev_bool(evidence, "config.SELINUX");
        let lockdown_cfg = evidence_helpers::ev_bool(evidence, "config.LOCKDOWN");
        if selinux_cfg || lockdown_cfg {
            SemanticState::SecurityPostureMedium
        } else {
            SemanticState::SecurityPostureLow
        }
    };

    Some(SemanticDescriptor {
        domain: SemanticDomain::SecurityPosture,
        state,
        source_evidence: vec![
            "kernel.security.lockdown",
            "kernel.security.lsms",
            "kernel.security.kaslr",
        ],
        rationale: "Derived from lockdown mode, active LSMs, and KASLR status",
    })
}

/// MEMORY_CLASS: derived from page size + huge pages + memory model
fn normalize_memory_class(evidence: &[Evidence]) -> Option<SemanticDescriptor> {
    let page_size = evidence_helpers::ev_s(evidence, "kernel.memory.page_size");
    let hugepages = evidence_helpers::ev_s(evidence, "kernel.memory.hugepages");

    let state = if hugepages.contains("1G") {
        SemanticState::MemoryHugePageOptimized
    } else if page_size.contains("65536") || page_size.contains("64K") || hugepages.contains("2M") {
        SemanticState::MemoryHighPerformance
    } else {
        SemanticState::MemoryBaseline
    };

    Some(SemanticDescriptor {
        domain: SemanticDomain::MemoryClass,
        state,
        source_evidence: vec![
            "kernel.memory.page_size",
            "kernel.memory.hugepages",
            "kernel.memory.model",
        ],
        rationale: "Derived from page size, huge page availability, and memory model",
    })
}

/// SCHEDULER_CLASS: derived from scheduler classes + preemption
fn normalize_scheduler_class(evidence: &[Evidence]) -> Option<SemanticDescriptor> {
    let classes = evidence_helpers::ev_s(evidence, "kernel.scheduler.classes");
    let preempt = evidence_helpers::ev_s(evidence, "kernel.scheduler.preemption");

    let state = if classes.contains("deadline") || preempt.contains("full") {
        SemanticState::SchedulerRealTime
    } else if preempt.contains("voluntary") {
        SemanticState::SchedulerDesktop
    } else if preempt.contains("low_latency") {
        SemanticState::SchedulerLowLatency
    } else {
        SemanticState::SchedulerServer
    };

    Some(SemanticDescriptor {
        domain: SemanticDomain::SchedulerClass,
        state,
        source_evidence: vec!["kernel.scheduler.classes", "kernel.scheduler.preemption"],
        rationale: "Derived from available scheduling classes and preemption model",
    })
}

/// PERFORMANCE_TIER: derived from memory + scheduler + BTF + tracing
fn normalize_performance_tier(evidence: &[Evidence]) -> Option<SemanticDescriptor> {
    let mem_class = normalize_memory_class(evidence);
    let sched_class = normalize_scheduler_class(evidence);
    let btf = evidence_helpers::ev_bool(evidence, "kernel.btf.module");

    let state = match (
        mem_class.as_ref().map(|d| &d.state),
        sched_class.as_ref().map(|d| &d.state),
    ) {
        (Some(SemanticState::MemoryHugePageOptimized), Some(SemanticState::SchedulerRealTime)) => {
            SemanticState::PerformanceHigh
        }
        (Some(SemanticState::MemoryHighPerformance), _)
        | (_, Some(SemanticState::SchedulerRealTime)) => SemanticState::PerformanceHigh,
        _ if btf => SemanticState::PerformanceMedium,
        _ => SemanticState::PerformanceLow,
    };

    Some(SemanticDescriptor {
        domain: SemanticDomain::PerformanceTier,
        state,
        source_evidence: vec![
            "kernel.memory.page_size",
            "kernel.scheduler.preemption",
            "kernel.btf.module",
        ],
        rationale: "Composite of memory class, scheduler capability, and BTF availability",
    })
}

/// STABILITY_TIER: derived from lockdown + preemption + headers + config
fn normalize_stability_tier(evidence: &[Evidence]) -> Option<SemanticDescriptor> {
    let lockdown = evidence_helpers::ev_s(evidence, "kernel.security.lockdown");
    let headers = evidence_helpers::ev_s(evidence, "build.headers");
    let preempt = evidence_helpers::ev_s(evidence, "kernel.scheduler.preemption");

    // Production: lockdown active, headers complete, preemption=voluntary
    let state = if lockdown.contains("confidentiality")
        && headers.contains("Complete")
        && preempt.contains("voluntary")
    {
        SemanticState::StabilityProduction
    } else if headers.contains("Complete") || lockdown.contains("integrity") {
        SemanticState::StabilityStaging
    } else if headers.contains("Partial") {
        SemanticState::StabilityDevelopment
    } else {
        SemanticState::StabilityUnstable
    };

    Some(SemanticDescriptor {
        domain: SemanticDomain::StabilityTier,
        state,
        source_evidence: vec![
            "kernel.security.lockdown",
            "build.headers",
            "kernel.scheduler.preemption",
        ],
        rationale: "Composite of security posture, build readiness, and scheduler stability",
    })
}

/// FEATURE_AVAILABILITY: aggregate of key feature flags
fn normalize_feature_availability(evidence: &[Evidence]) -> Option<SemanticDescriptor> {
    let btf = evidence_helpers::ev_bool(evidence, "kernel.btf.module");
    let ftrace = evidence_helpers::ev_bool(evidence, "kernel.tracing.ftrace");
    let kprobes = evidence_helpers::ev_bool(evidence, "kernel.tracing.kprobes");
    let tracepoints = evidence_helpers::ev_s(evidence, "kernel.tracepoints.count");
    let has_tracepoints = !tracepoints.is_empty() && tracepoints != "0";

    let available = [btf, ftrace, kprobes, has_tracepoints]
        .iter()
        .filter(|&&x| x)
        .count();

    let state = if available >= 4 {
        SemanticState::FeatureAvailable
    } else if available >= 2 {
        SemanticState::FeaturePartial
    } else {
        SemanticState::FeatureUnavailable
    };

    Some(SemanticDescriptor {
        domain: SemanticDomain::FeatureAvailability,
        state,
        source_evidence: vec![
            "kernel.btf.module",
            "kernel.tracing.ftrace",
            "kernel.tracing.kprobes",
            "kernel.tracepoints.count",
        ],
        rationale: "Aggregate of BTF, ftrace, kprobes, and tracepoint availability",
    })
}

/// RUNTIME_RISK: composite risk assessment from security + stability
fn normalize_runtime_risk(evidence: &[Evidence]) -> Option<SemanticDescriptor> {
    let lockdown = evidence_helpers::ev_s(evidence, "kernel.security.lockdown");
    let headers = evidence_helpers::ev_s(evidence, "build.headers");
    let modules = evidence_helpers::ev_bool(evidence, "config.MODULES");

    let mut risk_score = 0u8;

    // Low lockdown → higher risk
    if lockdown.contains("none") || lockdown.is_empty() {
        risk_score += 2;
    }
    // Incomplete headers → higher risk
    if !headers.contains("Complete") {
        risk_score += 2;
    }
    // No modules → higher risk (harder to load kernel modules safely)
    if !modules {
        risk_score += 1;
    }

    let state = match risk_score {
        0 => SemanticState::RuntimeRiskLow,
        1..=2 => SemanticState::RuntimeRiskMedium,
        3 => SemanticState::RuntimeRiskHigh,
        _ => SemanticState::RuntimeRiskCritical,
    };

    Some(SemanticDescriptor {
        domain: SemanticDomain::RuntimeRisk,
        state,
        source_evidence: vec![
            "kernel.security.lockdown",
            "build.headers",
            "config.MODULES",
        ],
        rationale: "Composite risk from security posture, build readiness, and module availability",
    })
}

// ============================================================================
//  Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::evidence::{Evidence, EvidenceValue};

    fn make_text(id: &'static str, value: &str) -> Evidence {
        Evidence::present(id, EvidenceValue::Text(Some(value.into())))
    }

    fn make_bool(id: &'static str, value: bool) -> Evidence {
        Evidence::present(id, EvidenceValue::Bool(value))
    }

    #[test]
    fn test_security_posture_high_with_lockdown_confidentiality() {
        let ev = vec![make_text("kernel.security.lockdown", "confidentiality")];
        let result = normalize(&ev);
        let sec = result
            .iter()
            .find(|d| d.domain == SemanticDomain::SecurityPosture)
            .unwrap();
        assert_eq!(sec.state, SemanticState::SecurityPostureHigh);
    }

    #[test]
    fn test_security_posture_low_without_security() {
        let ev: Vec<Evidence> = vec![];
        let result = normalize(&ev);
        let sec = result
            .iter()
            .find(|d| d.domain == SemanticDomain::SecurityPosture)
            .unwrap();
        assert_eq!(sec.state, SemanticState::SecurityPostureLow);
    }

    #[test]
    fn test_memory_huge_page_optimized() {
        let ev = vec![
            make_text("kernel.memory.page_size", "4096"),
            make_text("kernel.memory.hugepages", "2M,1G"),
        ];
        let result = normalize(&ev);
        let mem = result
            .iter()
            .find(|d| d.domain == SemanticDomain::MemoryClass)
            .unwrap();
        assert_eq!(mem.state, SemanticState::MemoryHugePageOptimized);
    }

    #[test]
    fn test_scheduler_realtime() {
        let ev = vec![
            make_text("kernel.scheduler.classes", "cfs,rt,deadline"),
            make_text("kernel.scheduler.preemption", "full"),
        ];
        let result = normalize(&ev);
        let sched = result
            .iter()
            .find(|d| d.domain == SemanticDomain::SchedulerClass)
            .unwrap();
        assert_eq!(sched.state, SemanticState::SchedulerRealTime);
    }

    #[test]
    fn test_feature_availability_all_available() {
        let ev = vec![
            make_bool("kernel.btf.module", true),
            make_bool("kernel.tracing.ftrace", true),
            make_bool("kernel.tracing.kprobes", true),
            make_text("kernel.tracepoints.count", "1427"),
        ];
        let result = normalize(&ev);
        let feat = result
            .iter()
            .find(|d| d.domain == SemanticDomain::FeatureAvailability)
            .unwrap();
        assert_eq!(feat.state, SemanticState::FeatureAvailable);
    }

    #[test]
    fn test_feature_availability_none() {
        let ev: Vec<Evidence> = vec![];
        let result = normalize(&ev);
        let feat = result
            .iter()
            .find(|d| d.domain == SemanticDomain::FeatureAvailability)
            .unwrap();
        assert_eq!(feat.state, SemanticState::FeatureUnavailable);
    }

    #[test]
    fn test_all_seven_domains_produced() {
        let ev = vec![
            make_text("kernel.security.lockdown", "integrity"),
            make_text("kernel.memory.page_size", "4096"),
            make_text("kernel.scheduler.preemption", "voluntary"),
            make_bool("kernel.btf.module", true),
            make_bool("kernel.tracing.ftrace", true),
            make_bool("kernel.tracing.kprobes", true),
            make_text("kernel.tracepoints.count", "500"),
            make_text("build.headers", "Complete"),
            make_bool("config.MODULES", true),
        ];
        let result = normalize(&ev);
        let domains: Vec<SemanticDomain> = result.iter().map(|d| d.domain).collect();
        assert!(
            domains.contains(&SemanticDomain::SecurityPosture),
            "missing security"
        );
        assert!(
            domains.contains(&SemanticDomain::MemoryClass),
            "missing memory"
        );
        assert!(
            domains.contains(&SemanticDomain::SchedulerClass),
            "missing scheduler"
        );
        assert!(
            domains.contains(&SemanticDomain::PerformanceTier),
            "missing performance"
        );
        assert!(
            domains.contains(&SemanticDomain::StabilityTier),
            "missing stability"
        );
        assert!(
            domains.contains(&SemanticDomain::FeatureAvailability),
            "missing features"
        );
        assert!(
            domains.contains(&SemanticDomain::RuntimeRisk),
            "missing risk"
        );
    }
}

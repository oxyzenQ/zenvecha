// Copyright (C) 2026 rezky_nightky
// SPDX-License-Identifier: GPL-3.0-only

//! Semantic domain models — normalized system state descriptors.
//!
//! Raw kernel facts are strings: `security.lockdown = "integrity"`.
//! Semantic descriptors are typed states: `SecurityPosture::Medium`.
//!
//! Engines match on typed variants, not string comparisons.
//! This layer is deterministic, rule-based, and reproducible.

// ============================================================================
//  Semantic Descriptor
// ============================================================================

/// A normalized interpretation of one or more raw Evidence items.
#[derive(Clone, Debug)]
pub struct SemanticDescriptor {
    /// Which domain this descriptor belongs to.
    pub domain: SemanticDomain,
    /// The normalized state.
    pub state: SemanticState,
    /// Which Evidence IDs contributed to this descriptor.
    pub source_evidence: Vec<&'static str>,
    /// Human-readable explanation of the mapping.
    pub rationale: &'static str,
}

// ============================================================================
//  Semantic Domains
// ============================================================================

/// High-level semantic domains that raw facts map into.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SemanticDomain {
    SecurityPosture,
    MemoryClass,
    SchedulerClass,
    PerformanceTier,
    StabilityTier,
    FeatureAvailability,
    RuntimeRisk,
}

impl SemanticDomain {
    pub fn label(self) -> &'static str {
        match self {
            SemanticDomain::SecurityPosture => "security_posture",
            SemanticDomain::MemoryClass => "memory_class",
            SemanticDomain::SchedulerClass => "scheduler_class",
            SemanticDomain::PerformanceTier => "performance_tier",
            SemanticDomain::StabilityTier => "stability_tier",
            SemanticDomain::FeatureAvailability => "feature_availability",
            SemanticDomain::RuntimeRisk => "runtime_risk",
        }
    }
}

// ============================================================================
//  Semantic States — strongly-typed, not strings
// ============================================================================

/// All possible semantic states across all domains.
///
/// Engines should match on these variants, never on raw strings.
/// Adding a new domain = add variants here + add normalization rule.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SemanticState {
    // Security Posture
    SecurityPostureLow,
    SecurityPostureMedium,
    SecurityPostureHigh,

    // Memory Class
    MemoryBaseline,
    MemoryHighPerformance,
    MemoryHugePageOptimized,

    // Scheduler Class
    SchedulerDesktop,
    SchedulerServer,
    SchedulerRealTime,
    SchedulerLowLatency,

    // Performance Tier
    PerformanceLow,
    PerformanceMedium,
    PerformanceHigh,

    // Stability Tier
    StabilityProduction,
    StabilityStaging,
    StabilityDevelopment,
    StabilityUnstable,

    // Feature Availability
    FeatureAvailable,
    FeaturePartial,
    FeatureUnavailable,

    // Runtime Risk
    RuntimeRiskLow,
    RuntimeRiskMedium,
    RuntimeRiskHigh,
    RuntimeRiskCritical,

    // Fallback
    Unknown,
}

impl SemanticState {
    /// Label for rendering / serialization.
    pub fn label(&self) -> &'static str {
        match self {
            SemanticState::SecurityPostureLow => "low",
            SemanticState::SecurityPostureMedium => "medium",
            SemanticState::SecurityPostureHigh => "high",
            SemanticState::MemoryBaseline => "baseline",
            SemanticState::MemoryHighPerformance => "high_performance",
            SemanticState::MemoryHugePageOptimized => "huge_page_optimized",
            SemanticState::SchedulerDesktop => "desktop",
            SemanticState::SchedulerServer => "server",
            SemanticState::SchedulerRealTime => "realtime",
            SemanticState::SchedulerLowLatency => "low_latency",
            SemanticState::PerformanceLow => "low",
            SemanticState::PerformanceMedium => "medium",
            SemanticState::PerformanceHigh => "high",
            SemanticState::StabilityProduction => "production",
            SemanticState::StabilityStaging => "staging",
            SemanticState::StabilityDevelopment => "development",
            SemanticState::StabilityUnstable => "unstable",
            SemanticState::FeatureAvailable => "available",
            SemanticState::FeaturePartial => "partial",
            SemanticState::FeatureUnavailable => "unavailable",
            SemanticState::RuntimeRiskLow => "low",
            SemanticState::RuntimeRiskMedium => "medium",
            SemanticState::RuntimeRiskHigh => "high",
            SemanticState::RuntimeRiskCritical => "critical",
            SemanticState::Unknown => "unknown",
        }
    }
}

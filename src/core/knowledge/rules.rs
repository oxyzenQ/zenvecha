// Copyright (C) 2026 rezky_nightky
// SPDX-License-Identifier: GPL-3.0-only

//! Knowledge rule types — declarative domain models.
//!
//! Rules are immutable domain data. They describe what is true about
//! Linux kernels, independent of any specific machine.

/// Category of knowledge.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KnowledgeCategory {
    Kernel,
    Configuration,
    Rust,
    Subsystem,
    Feature,
    Deprecation,
    Security,
}

impl KnowledgeCategory {
    pub fn label(self) -> &'static str {
        match self {
            KnowledgeCategory::Kernel => "Kernel",
            KnowledgeCategory::Configuration => "Configuration",
            KnowledgeCategory::Rust => "Rust",
            KnowledgeCategory::Subsystem => "Subsystem",
            KnowledgeCategory::Feature => "Feature",
            KnowledgeCategory::Deprecation => "Deprecation",
            KnowledgeCategory::Security => "Security",
        }
    }
}

/// Impact level of a matched rule.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuleImpact {
    Informational,
    Notable,
    Important,
    Critical,
}

impl RuleImpact {
    pub fn label(self) -> &'static str {
        match self {
            RuleImpact::Informational => "info",
            RuleImpact::Notable => "notable",
            RuleImpact::Important => "important",
            RuleImpact::Critical => "critical",
        }
    }
}

/// A rule about kernel version requirements.
#[derive(Clone, Debug)]
pub struct KernelRule {
    pub id: &'static str,
    pub min_version_major: u32,
    pub min_version_minor: u32,
    pub category: KnowledgeCategory,
    pub description: &'static str,
    pub implications: &'static [&'static str],
    pub impact: RuleImpact,
}

/// A rule about kernel configuration options.
#[derive(Clone, Debug)]
pub struct ConfigRule {
    pub id: &'static str,
    pub config_key: &'static str,
    pub expected: ConfigExpectation,
    pub description: &'static str,
    pub implications: &'static [&'static str],
    pub impact: RuleImpact,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConfigExpectation {
    Enabled,
    Disabled,
    Any,
}

/// A rule about Rust for Linux support.
#[derive(Clone, Debug)]
pub struct RustRule {
    pub id: &'static str,
    pub min_version_major: u32,
    pub min_version_minor: u32,
    pub description: &'static str,
    pub implications: &'static [&'static str],
    pub impact: RuleImpact,
}

/// A rule about kernel features.
#[derive(Clone, Debug)]
pub struct FeatureRule {
    pub id: &'static str,
    pub name: &'static str,
    pub min_version_major: u32,
    pub min_version_minor: u32,
    pub requires_config: Option<&'static str>,
    pub category: KnowledgeCategory,
    pub description: &'static str,
    pub implications: &'static [&'static str],
    pub impact: RuleImpact,
}

/// A matched rule — the result of resolving a rule against evidence.
#[derive(Clone, Debug)]
pub struct MatchedRule {
    pub rule_id: &'static str,
    pub category: KnowledgeCategory,
    pub description: String,
    pub implications: Vec<String>,
    pub impact: RuleImpact,
    /// Why this rule matched (or didn't).
    pub match_reason: String,
}

/// Parsed kernel version from evidence.
#[derive(Clone, Debug, Default)]
pub struct KernelVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub raw: String,
}

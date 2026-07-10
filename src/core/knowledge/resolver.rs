// Copyright (C) 2026 rezky_nightky
// SPDX-License-Identifier: GPL-3.0-only

//! Knowledge resolver — matches domain rules against system Evidence.
//!
//! Resolves kernel version implications, configuration implications,
//! Rust support status, and feature availability.
//!
//! Pure transformation from Evidence + KnowledgeBase → KnowledgeResult.
//! Never probes the system.

use crate::core::evidence::Evidence;
use crate::core::evidence_helpers;
use crate::core::knowledge::database::KnowledgeBase;
use crate::core::knowledge::rules::{
    ConfigExpectation, KernelVersion, KnowledgeCategory, MatchedRule, RuleImpact,
};

/// Result of resolving the knowledge base against evidence.
#[derive(Clone, Debug)]
pub struct KnowledgeResult {
    /// Matched rules that apply to this system.
    pub matched_rules: Vec<MatchedRule>,
    /// Parsed kernel version.
    pub kernel_version: Option<KernelVersion>,
    /// Human-readable insights for rendering.
    pub insights: Vec<String>,
    /// How many rules were evaluated.
    pub total_rules_evaluated: usize,
    /// How many rules matched.
    pub total_rules_matched: usize,
}

impl KnowledgeResult {
    pub fn kernel_version_str(&self) -> String {
        match &self.kernel_version {
            Some(kv) => format!("{}.{}", kv.major, kv.minor),
            None => "Unknown".to_string(),
        }
    }
}

/// Resolve the knowledge base against system evidence.
pub fn resolve(evidence: &[Evidence]) -> KnowledgeResult {
    let kb = KnowledgeBase::load();
    let kv = parse_kernel_version(evidence);
    let mut matched = Vec::new();
    let mut total = 0usize;

    // Resolve kernel version rules
    total += kb.kernel_rules.len();
    for rule in &kb.kernel_rules {
        if let Some(ver) = kv.clone() {
            if version_in_range(
                &ver,
                rule.min_version_major,
                rule.min_version_minor,
                rule.max_version_major,
                rule.max_version_minor,
            ) {
                matched.push(MatchedRule {
                    rule_id: rule.id,
                    category: rule.category,
                    description: rule.description.to_string(),
                    implications: rule.implications.iter().map(|&s| s.to_string()).collect(),
                    impact: rule.impact,
                    match_reason: format!(
                        "Kernel {}.{} meets minimum {}.{}",
                        ver.major, ver.minor, rule.min_version_major, rule.min_version_minor
                    ),
                });
            }
        } else if rule.min_version_major == 0 {
            // Legacy catch-all — matches when we can't determine version
            matched.push(MatchedRule {
                rule_id: rule.id,
                category: rule.category,
                description: rule.description.to_string(),
                implications: rule.implications.iter().map(|&s| s.to_string()).collect(),
                impact: rule.impact,
                match_reason: "Kernel version unknown — legacy fallback".into(),
            });
        }
    }

    // Resolve config rules
    total += kb.config_rules.len();
    for rule in &kb.config_rules {
        let config_val =
            evidence_helpers::ev_bool(evidence, &format!("config.{}", rule.config_key));
        let config_known = evidence_helpers::ev_text_value(evidence, "config.source").is_some();

        let reason = match rule.expected {
            ConfigExpectation::Enabled => {
                if config_val {
                    matched.push(build_config_match(rule, "enabled"));
                } else if config_known {
                    // Config known but not set — this is actionable knowledge
                    matched.push(build_config_mismatch(rule, "not set"));
                }
                format!(
                    "CONFIG_{} is {}",
                    rule.config_key,
                    if config_val { "enabled" } else { "absent" }
                )
            }
            ConfigExpectation::Disabled => {
                if !config_val && config_known {
                    matched.push(build_config_match(rule, "disabled (as expected)"));
                }
                format!("CONFIG_{} checked", rule.config_key)
            }
            ConfigExpectation::Any => {
                if config_known {
                    matched.push(build_config_match(rule, "present"));
                }
                format!("CONFIG_{} present", rule.config_key)
            }
        };
        // Update match reason for the last pushed (or skip if not matched)
        if let Some(last) = matched.last_mut()
            && last.rule_id == rule.id
        {
            last.match_reason = reason;
        }
    }

    // Resolve Rust rules
    total += kb.rust_rules.len();
    for rule in &kb.rust_rules {
        if let Some(ver) = kv.clone()
            && version_gte(&ver, rule.min_version_major, rule.min_version_minor)
        {
            let rust_enabled = evidence_helpers::ev_bool(evidence, "config.RUST");
            matched.push(MatchedRule {
                rule_id: rule.id,
                category: KnowledgeCategory::Rust,
                description: rule.description.to_string(),
                implications: rule.implications.iter().map(|&s| s.to_string()).collect(),
                impact: rule.impact,
                match_reason: format!(
                    "Kernel {}.{} meets Rust minimum {}.{} — Rust is {}",
                    ver.major,
                    ver.minor,
                    rule.min_version_major,
                    rule.min_version_minor,
                    if rust_enabled {
                        "enabled"
                    } else {
                        "available but not enabled"
                    }
                ),
            });
        }
    }

    // Resolve feature rules
    total += kb.feature_rules.len();
    for rule in &kb.feature_rules {
        if let Some(ver) = kv.clone()
            && version_gte(&ver, rule.min_version_major, rule.min_version_minor)
        {
            let config_ok = match rule.requires_config {
                Some(cfg_key) => {
                    evidence_helpers::ev_bool(evidence, &format!("config.{}", cfg_key))
                }
                None => true,
            };
            if config_ok {
                matched.push(MatchedRule {
                    rule_id: rule.id,
                    category: rule.category,
                    description: rule.description.to_string(),
                    implications: rule.implications.iter().map(|&s| s.to_string()).collect(),
                    impact: rule.impact,
                    match_reason: format!(
                        "Kernel {}.{} supports {}",
                        ver.major, ver.minor, rule.name,
                    ),
                });
            }
        }
    }

    let insights = build_insights(&matched, &kv);

    KnowledgeResult {
        matched_rules: matched.clone(),
        kernel_version: kv,
        insights,
        total_rules_evaluated: total,
        total_rules_matched: matched.len(),
    }
}

// ============================================================================
//  Helpers
// ============================================================================

fn parse_kernel_version(evidence: &[Evidence]) -> Option<KernelVersion> {
    let release = evidence_helpers::ev_s(evidence, "kernel.release");
    if release == "Unknown" {
        return None;
    }
    // Parse "6.12.5-arch1-1" → (6, 12, 5)
    let parts: Vec<&str> = release.split(['.', '-']).collect();
    if parts.len() < 2 {
        return None;
    }
    let major = parts[0].parse().ok()?;
    let minor = parts[1].parse().ok()?;
    let patch = parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(0);
    Some(KernelVersion {
        major,
        minor,
        patch,
        raw: release,
    })
}

fn version_gte(ver: &KernelVersion, min_major: u32, min_minor: u32) -> bool {
    version_in_range(ver, min_major, min_minor, None, None)
}

fn version_in_range(
    ver: &KernelVersion,
    min_major: u32,
    min_minor: u32,
    max_major: Option<u32>,
    max_minor: Option<u32>,
) -> bool {
    let meets_min = ver.major > min_major || (ver.major == min_major && ver.minor >= min_minor);
    let meets_max = match (max_major, max_minor) {
        (Some(maj), Some(min)) => ver.major < maj || (ver.major == maj && ver.minor <= min),
        _ => true,
    };
    meets_min && meets_max
}

fn build_config_match(
    rule: &crate::core::knowledge::rules::ConfigRule,
    status: &str,
) -> MatchedRule {
    MatchedRule {
        rule_id: rule.id,
        category: KnowledgeCategory::Configuration,
        description: rule.description.to_string(),
        implications: rule.implications.iter().map(|&s| s.to_string()).collect(),
        impact: rule.impact,
        match_reason: format!("CONFIG_{} is {}", rule.config_key, status),
    }
}

fn build_config_mismatch(
    rule: &crate::core::knowledge::rules::ConfigRule,
    status: &str,
) -> MatchedRule {
    let _desc = format!("{} — currently {}", rule.description, status);
    MatchedRule {
        rule_id: rule.id,
        category: KnowledgeCategory::Configuration,
        description: String::new(), // filled below
        implications: rule.implications.iter().map(|&s| s.to_string()).collect(),
        impact: if matches!(rule.impact, RuleImpact::Critical) {
            RuleImpact::Critical
        } else {
            RuleImpact::Important
        },
        match_reason: format!(
            "CONFIG_{} is {} — {}",
            rule.config_key,
            status,
            rule.implications.first().unwrap_or(&"")
        ),
    }
}

fn build_insights(matched: &[MatchedRule], kv: &Option<KernelVersion>) -> Vec<String> {
    let mut insights = Vec::new();

    if let Some(ver) = kv.clone() {
        // Kernel version awareness
        if version_gte(&ver, 6, 18) {
            insights.push(format!(
                "Running Linux {}.{} — Rust support is production-ready",
                ver.major, ver.minor
            ));
        } else if version_gte(&ver, 6, 6) {
            insights.push(format!(
                "Running Linux {}.{} LTS — stable foundation, Rust support available",
                ver.major, ver.minor
            ));
        } else if version_gte(&ver, 6, 0) {
            insights.push(format!(
                "Running Linux {}.{} — Rust support is experimental, upgrade to 6.18+ recommended",
                ver.major, ver.minor
            ));
        } else {
            insights.push(format!(
                "Running Linux {}.{} — legacy kernel, consider upgrading to 6.6+ LTS",
                ver.major, ver.minor
            ));
        }
    }

    // Count by category
    let critical = matched
        .iter()
        .filter(|r| r.impact == RuleImpact::Critical)
        .count();
    let important = matched
        .iter()
        .filter(|r| r.impact == RuleImpact::Important)
        .count();
    let notable = matched
        .iter()
        .filter(|r| r.impact == RuleImpact::Notable)
        .count();

    if critical > 0 {
        insights.push(format!(
            "{critical} critical knowledge rule(s) apply to this system"
        ));
    }
    if important > 0 {
        insights.push(format!("{important} important knowledge rule(s) matched"));
    }
    if notable > 0 {
        insights.push(format!("{notable} notable kernel feature(s) identified"));
    }

    insights
}

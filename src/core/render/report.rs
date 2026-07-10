// Copyright (C) 2026 rezky_nightky
// SPDX-License-Identifier: GPL-3.0-only

//! Report renderers — transforms Evidence + analysis into human/compact/JSON output.
//!
//! Renderers are the ONLY place where terminal/JSON output is produced.
//! Renderers never inspect the system, never collect evidence, never compute scores.
//! Only formatting.

use std::io::{self, Write};

use crate::core::analysis::{Readiness, Risk};
use crate::core::evidence::Evidence;
use crate::core::evidence_helpers;

// ============================================================================
//  Human-readable report renderer
// ============================================================================

pub fn render_human(
    evidence: &[Evidence],
    readiness: &Readiness,
    risks: &[Risk],
    recs: &[String],
    out: &mut io::StdoutLock<'_>,
) -> Result<(), Box<dyn std::error::Error>> {
    writeln!(out, "Zenvecha Kernel Intelligence Report")?;
    writeln!(out)?;

    render_section(out, "Kernel Identity")?;
    render_kv(
        out,
        "  Version",
        &evidence_helpers::ev_s(evidence, "kernel.release"),
    )?;
    render_kv(
        out,
        "  Architecture",
        &evidence_helpers::ev_s(evidence, "kernel.arch"),
    )?;
    render_kv(
        out,
        "  Distribution",
        &evidence_helpers::ev_s(evidence, "kernel.distro"),
    )?;
    render_kv(
        out,
        "  Rust compiler",
        &evidence_helpers::ev_s(evidence, "toolchain.rustc"),
    )?;
    writeln!(out)?;

    render_section(out, "Readiness")?;
    writeln!(out, "  Overall : {}", readiness.overall)?;
    writeln!(out)?;
    for cat in &readiness.categories {
        writeln!(out, "  {} {}", stars_fmt(cat.stars), cat.name)?;
    }
    writeln!(out)?;

    render_section(out, "Capability Matrix")?;
    writeln!(out)?;

    let caps = [
        ("Rust for Linux", "config.RUST", "config.RUST_IS_AVAILABLE"),
        ("Modules", "config.MODULES", ""),
        ("Module Signing", "config.MODULE_SIG", ""),
        ("BTF", "config.DEBUG_INFO_BTF", "debug.btf"),
        ("Livepatch", "config.LIVEPATCH", ""),
        ("Kallsyms", "symbols.kallsyms", ""),
    ];

    for (name, primary, secondary) in &caps {
        let primary_val = evidence_helpers::ev_config_bool(evidence, primary);
        let secondary_val = if secondary.is_empty() {
            None
        } else {
            evidence_helpers::ev_config_bool(evidence, secondary)
        };
        let (status, _) = cap_status(primary_val, secondary_val);
        writeln!(out, "  {} {}", tri_icon(status), name)?;
    }
    writeln!(out)?;

    if !risks.is_empty() {
        render_section(out, "Compatibility Risks")?;
        writeln!(out)?;
        for risk in risks {
            writeln!(out, "  ⚠  {}", risk.description)?;
        }
        writeln!(out)?;
    }

    if !recs.is_empty() {
        render_section(out, "Recommendations")?;
        writeln!(out)?;
        let end = recs.len().min(10);
        for (i, rec) in recs.iter().take(end).enumerate() {
            writeln!(out, "  {}. {}", i + 1, rec)?;
        }
        writeln!(out)?;
    }

    render_section(out, "Environment Facts")?;
    writeln!(out)?;
    for fact in collect_facts(evidence) {
        writeln!(out, "  • {}", fact)?;
    }
    writeln!(out)?;

    Ok(())
}

/// Full human report — includes all Wolfzenix engine outputs.
pub fn render_human_full(
    result: &crate::core::pipeline::AnalysisResult,
    out: &mut io::StdoutLock<'_>,
) -> Result<(), Box<dyn std::error::Error>> {
    let evidence = &result.evidence;
    let compatibility = &result.compatibility;
    let knowledge = &result.knowledge;
    let reasoning = &result.reasoning;

    writeln!(out, "══ Zenvecha Kernel Intelligence Report ══")?;
    writeln!(out)?;

    // Kernel Identity
    render_section(out, "Kernel Identity")?;
    render_kv(
        out,
        "  Version",
        &evidence_helpers::ev_s(evidence, "kernel.release"),
    )?;
    render_kv(
        out,
        "  Architecture",
        &evidence_helpers::ev_s(evidence, "kernel.arch"),
    )?;
    writeln!(out)?;

    // Compatibility Score
    render_section(out, "Compatibility")?;
    writeln!(
        out,
        "  Score : {}% ({})",
        compatibility.score, compatibility.level
    )?;
    writeln!(out, "  Risk  : {}", compatibility.risk.label())?;
    writeln!(out)?;
    for c in &compatibility.components {
        let icon = match c.status {
            crate::core::analysis::ComponentStatus::Good => "✔",
            crate::core::analysis::ComponentStatus::Partial => "◐",
            crate::core::analysis::ComponentStatus::Missing => "✘",
            crate::core::analysis::ComponentStatus::Blocking => "🚫",
        };
        writeln!(out, "  {icon} {} — {}%", c.name, c.score)?;
    }
    writeln!(out)?;

    // Blocking Issues
    if !compatibility.blocking_issues.is_empty() {
        render_section(out, "Blocking Issues")?;
        writeln!(out)?;
        for issue in &compatibility.blocking_issues {
            writeln!(out, "  🚫 {}: {}", issue.component, issue.description)?;
        }
        writeln!(out)?;
    }

    // Decision Plan
    let dp = &result.decision_plan;
    render_section(out, "Decision Plan")?;
    writeln!(out, "  Current  : {}%", dp.current_score)?;
    writeln!(
        out,
        "  Expected : {}% (+{}%)",
        dp.expected_score,
        dp.expected_score.saturating_sub(dp.current_score)
    )?;
    if !dp.ranked_actions.is_empty() {
        writeln!(out)?;
        for a in &dp.ranked_actions {
            writeln!(
                out,
                "  ▶ {} (+{}% in {} min, ROI {:.2})",
                a.title, a.expected_score_gain, a.estimated_minutes, a.roi
            )?;
        }
    } else {
        writeln!(out, "  No actions required — system is ready")?;
    }
    writeln!(out)?;

    // Prediction
    let pred = &result.prediction;
    render_section(out, "Prediction")?;
    writeln!(out, "  Current Score : {}%", pred.current_score)?;
    if !pred.scenarios.is_empty() {
        writeln!(out)?;
        for s in &pred.scenarios {
            let delta = if s.score_delta >= 0 {
                format!("+{}", s.score_delta)
            } else {
                format!("{}", s.score_delta)
            };
            writeln!(
                out,
                "  📈 If \"{}\" → {}% ({delta})",
                s.action, s.expected_score
            )?;
        }
    }
    writeln!(out)?;

    // Knowledge
    render_section(out, "Kernel Intelligence")?;
    writeln!(out, "  Kernel: {}", knowledge.kernel_version_str())?;
    writeln!(
        out,
        "  Rules matched: {}/{}",
        knowledge.total_rules_matched, knowledge.total_rules_evaluated
    )?;
    for insight in &knowledge.insights {
        writeln!(out, "  • {}", insight)?;
    }
    writeln!(out)?;

    // Reasoning
    render_section(out, "Reasoning")?;
    writeln!(out, "  {}", reasoning.system_narrative)?;
    writeln!(out)?;
    writeln!(out, "  ▸ {}", reasoning.readiness_reason.conclusion)?;
    for line in &reasoning.readiness_reason.because {
        writeln!(out, "    ↓ {line}")?;
    }
    writeln!(out)?;
    writeln!(out, "  ↳ {}", reasoning.readiness_reason.confidence_reason)?;
    writeln!(out)?;

    // Recommendations
    if !result.recommendations.is_empty() {
        let is_optional = compatibility.blocking_issues.is_empty();
        let label = if is_optional {
            "Optional Improvements"
        } else {
            "Recommendations"
        };
        render_section(out, label)?;
        writeln!(out)?;
        for (i, rec) in result.recommendations.iter().enumerate() {
            writeln!(out, "  {}. {rec}", i + 1)?;
        }
        writeln!(out)?;
    }

    Ok(())
}

// ============================================================================
//  Compact renderer
// ============================================================================

pub fn render_compact(
    evidence: &[Evidence],
    readiness: &Readiness,
    risks: &[Risk],
    recs: &[String],
    out: &mut io::StdoutLock<'_>,
) -> Result<(), Box<dyn std::error::Error>> {
    writeln!(out, "Zenvecha v{}", env!("CARGO_PKG_VERSION"))?;
    writeln!(
        out,
        "Kernel: {} ({})",
        evidence_helpers::ev_s(evidence, "kernel.release"),
        evidence_helpers::ev_s(evidence, "kernel.arch")
    )?;
    writeln!(
        out,
        "Status: {} | Symbols: {} | Modules: {}",
        readiness.overall,
        evidence_helpers::ev_count(evidence, "symbols.count"),
        evidence_helpers::ev_s(evidence, "modules.loader"),
    )?;

    if let Some(bid) = evidence_helpers::ev_literal(evidence, "symbols.vmlinux") {
        writeln!(out, "VMLinux: {bid}")?;
    }
    if evidence_helpers::ev_bool(evidence, "debug.btf") {
        writeln!(out, "BTF: available")?;
    }
    if evidence_helpers::ev_bool(evidence, "config.RUST") {
        writeln!(out, "Rust: enabled")?;
    } else if evidence_helpers::ev_bool(evidence, "config.RUST_IS_AVAILABLE") {
        writeln!(out, "Rust: compiler available, not enabled")?;
    }

    if !risks.is_empty() {
        writeln!(out, "Risks: {}", risks.len())?;
    }
    if !recs.is_empty() {
        let end = recs.len().min(3);
        writeln!(out, "Top recommendations:")?;
        for (i, rec) in recs.iter().take(end).enumerate() {
            writeln!(out, "  {}. {}", i + 1, rec)?;
        }
    }

    Ok(())
}

// ============================================================================
//  JSON renderer
// ============================================================================

pub fn render_json(
    evidence: &[Evidence],
    readiness: &Readiness,
    risks: &[Risk],
    recs: &[String],
    out: &mut io::StdoutLock<'_>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut buf = String::new();
    buf.push_str("{\n");

    buf.push_str(&format!(
        "  \"version\": \"{}\",\n",
        esc(env!("CARGO_PKG_VERSION")),
    ));

    buf.push_str("  \"readiness\": {\n");
    buf.push_str(&format!(
        "    \"overall\": \"{}\",\n",
        esc(readiness.overall)
    ));
    buf.push_str(&format!("    \"score\": \"{}\",\n", esc(readiness.stars)));
    buf.push_str("    \"categories\": [\n");
    for (i, cat) in readiness.categories.iter().enumerate() {
        let comma = if i + 1 < readiness.categories.len() {
            ","
        } else {
            ""
        };
        buf.push_str(&format!(
            "      {{\"name\":\"{}\",\"stars\":{}}}{comma}\n",
            esc(cat.name),
            cat.stars,
        ));
    }
    buf.push_str("    ]\n  },\n");

    buf.push_str("  \"capabilities\": {\n");
    let cap_keys = [
        ("Rust for Linux", "config.RUST"),
        ("Modules", "config.MODULES"),
        ("Module Signing", "config.MODULE_SIG"),
        ("BTF", "config.DEBUG_INFO_BTF"),
        ("Livepatch", "config.LIVEPATCH"),
        ("Kallsyms", "symbols.kallsyms"),
    ];
    for (i, (name, id)) in cap_keys.iter().enumerate() {
        let comma = if i + 1 < cap_keys.len() { "," } else { "" };
        let val = evidence_helpers::ev_bool(evidence, id);
        buf.push_str(&format!("    \"{}\": {}{comma}\n", esc(name), val,));
    }
    buf.push_str("  },\n");

    buf.push_str("  \"risks\": [\n");
    for (i, risk) in risks.iter().enumerate() {
        let comma = if i + 1 < risks.len() { "," } else { "" };
        buf.push_str(&format!(
            "    {{\"description\":\"{}\",\"severity\":\"{}\"}}{comma}\n",
            esc(&risk.description),
            esc(risk.severity),
        ));
    }
    buf.push_str("  ],\n");

    buf.push_str("  \"recommendations\": [\n");
    let end = recs.len().min(10);
    for (i, rec) in recs.iter().take(end).enumerate() {
        let comma = if i + 1 < end { "," } else { "" };
        buf.push_str(&format!("    \"{}\"{comma}\n", esc(rec),));
    }
    buf.push_str("  ],\n");

    buf.push_str("  \"facts\": [\n");
    let facts = collect_facts(evidence);
    for (i, fact) in facts.iter().enumerate() {
        let comma = if i + 1 < facts.len() { "," } else { "" };
        buf.push_str(&format!("    \"{}\"{comma}\n", esc(fact),));
    }
    buf.push_str("  ]\n");

    buf.push_str("}\n");

    write!(out, "{buf}")?;
    Ok(())
}

/// Full JSON report — includes all Wolfzenix engine outputs.
pub fn render_json_full(
    result: &crate::core::pipeline::AnalysisResult,
    out: &mut io::StdoutLock<'_>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut b = String::new();
    b.push_str("{\n");
    b.push_str(&format!(
        "  \"version\": \"{}\",\n",
        esc(env!("CARGO_PKG_VERSION"))
    ));

    // Readiness
    b.push_str(&format!(
        "  \"readiness\": {{ \"overall\": \"{}\", \"score\": \"{}\" }},\n",
        esc(result.readiness.overall),
        esc(result.readiness.stars)
    ));

    // Compatibility
    let compat = &result.compatibility;
    b.push_str("  \"compatibility\": {\n");
    b.push_str(&format!("    \"score\": {},\n", compat.score));
    b.push_str(&format!("    \"level\": \"{}\",\n", esc(compat.level)));
    b.push_str(&format!(
        "    \"confidence\": \"{}\",\n",
        esc(compat.confidence.label())
    ));
    b.push_str(&format!(
        "    \"risk\": \"{}\",\n",
        esc(compat.risk.label())
    ));
    b.push_str("    \"components\": [\n");
    for (i, c) in compat.components.iter().enumerate() {
        let comma = if i + 1 < compat.components.len() {
            ","
        } else {
            ""
        };
        b.push_str(&format!(
            "      {{ \"name\": \"{}\", \"score\": {}, \"status\": \"{}\" }}{comma}\n",
            esc(c.name),
            c.score,
            esc(c.status.label()),
        ));
    }
    b.push_str("    ]\n  },\n");

    // Decision
    let dp = &result.decision_plan;
    b.push_str("  \"decision\": {\n");
    b.push_str(&format!("    \"current_score\": {},\n", dp.current_score));
    b.push_str(&format!("    \"expected_score\": {},\n", dp.expected_score));
    b.push_str(&format!(
        "    \"estimated_total_minutes\": {},\n",
        dp.estimated_total_fix_minutes
    ));
    b.push_str("    \"actions\": [\n");
    for (i, a) in dp.ranked_actions.iter().enumerate() {
        let comma = if i + 1 < dp.ranked_actions.len() {
            ","
        } else {
            ""
        };
        b.push_str(&format!(
            "      {{ \"title\": \"{}\", \"gain\": {}, \"minutes\": {}, \"roi\": {:.2} }}{comma}\n",
            esc(&a.title),
            a.expected_score_gain,
            a.estimated_minutes,
            a.roi
        ));
    }
    b.push_str("    ]\n  },\n");

    // Prediction
    let pred = &result.prediction;
    b.push_str("  \"prediction\": {\n");
    b.push_str(&format!("    \"current_score\": {},\n", pred.current_score));
    b.push_str("    \"scenarios\": [\n");
    for (i, s) in pred.scenarios.iter().enumerate() {
        let comma = if i + 1 < pred.scenarios.len() {
            ","
        } else {
            ""
        };
        b.push_str(&format!(
            "      {{ \"action\": \"{}\", \"expected_score\": {}, \"delta\": {}, \"confidence\": \"{}\" }}{comma}\n",
            esc(&s.action), s.expected_score, s.score_delta, esc(s.confidence.label())
        ));
    }
    b.push_str("    ]\n  },\n");

    // Knowledge
    let kn = &result.knowledge;
    b.push_str("  \"knowledge\": {\n");
    b.push_str(&format!(
        "    \"kernel_version\": \"{}\",\n",
        esc(&kn.kernel_version_str())
    ));
    b.push_str(&format!(
        "    \"matched_rules\": {},\n",
        kn.matched_rules.len()
    ));
    b.push_str(&format!(
        "    \"total_rules_evaluated\": {},\n",
        kn.total_rules_evaluated
    ));
    b.push_str("    \"insights\": [\n");
    for (i, insight) in kn.insights.iter().enumerate() {
        let comma = if i + 1 < kn.insights.len() { "," } else { "" };
        b.push_str(&format!("      \"{}\"{comma}\n", esc(insight)));
    }
    b.push_str("    ]\n  },\n");

    // Reasoning
    let reason = &result.reasoning;
    b.push_str("  \"reasoning\": {\n");
    b.push_str(&format!(
        "    \"narrative\": \"{}\",\n",
        esc(&reason.system_narrative)
    ));
    b.push_str(&format!(
        "    \"readiness_conclusion\": \"{}\",\n",
        esc(&reason.readiness_reason.conclusion)
    ));
    b.push_str(&format!(
        "    \"readiness_confidence\": \"{}\"\n",
        esc(&reason.readiness_reason.confidence_reason)
    ));
    b.push_str("  },\n");

    // Recommendations
    b.push_str("  \"recommendations\": [\n");
    let end = result.recommendations.len().min(10);
    for (i, rec) in result.recommendations.iter().take(end).enumerate() {
        let comma = if i + 1 < end { "," } else { "" };
        b.push_str(&format!("    \"{}\"{comma}\n", esc(rec)));
    }
    b.push_str("  ]\n");

    b.push_str("}\n");
    write!(out, "{b}")?;
    Ok(())
}

// ============================================================================
//  Helpers
// ============================================================================

fn render_section(out: &mut io::StdoutLock<'_>, title: &str) -> io::Result<()> {
    writeln!(out, "── {title} ──")
}

fn render_kv(out: &mut io::StdoutLock<'_>, label: &str, value: &str) -> io::Result<()> {
    if value.is_empty() || value == "Unknown" {
        writeln!(out, "{label} : Unknown")
    } else {
        writeln!(out, "{label} : {value}")
    }
}

fn stars_fmt(n: u8) -> String {
    match n {
        5 => "★★★★★".into(),
        4 => "★★★★☆".into(),
        3 => "★★★☆☆".into(),
        2 => "★★☆☆☆".into(),
        1 => "★☆☆☆☆".into(),
        _ => "☆☆☆☆☆".into(),
    }
}

fn cap_status(primary: Option<bool>, secondary: Option<bool>) -> (&'static str, bool) {
    match (primary, secondary) {
        (Some(true), _) | (_, Some(true)) => ("enabled", true),
        (Some(false), Some(false)) | (Some(false), None) => ("disabled", false),
        (None, None) => ("unknown", false),
        (None, Some(false)) => ("unknown", false),
    }
}

fn tri_icon(status: &str) -> &'static str {
    match status {
        "enabled" => "✔",
        "disabled" => "✘",
        _ => "?",
    }
}

fn collect_facts(evidence: &[Evidence]) -> Vec<String> {
    let mut facts: Vec<String> = Vec::new();

    let sym_count = evidence_helpers::ev_count(evidence, "symbols.count");
    if sym_count != "0" {
        facts.push(format!("{sym_count} kernel symbols"));
    }

    let loader = evidence_helpers::ev_text_value(evidence, "modules.loader");
    if let Some(ref l) = loader
        && let Some(loaded) = extract_field(l, "loaded=")
    {
        facts.push(format!("{loaded} modules loaded"));
    }

    if evidence_helpers::ev_bool(evidence, "debug.btf") {
        facts.push("BTF type information available".into());
    }

    if evidence_helpers::ev_bool(evidence, "config.MODULE_SIG") {
        facts.push("Kernel module signing active".into());
    }

    let arch = evidence_helpers::ev_s(evidence, "kernel.arch");
    if arch != "Unknown" {
        facts.push(format!("Architecture: {arch}"));
    }

    let release = evidence_helpers::ev_text_value(evidence, "kernel.release");
    if let Some(ref r) = release
        && evidence_helpers::ev_status_is(evidence, "build.headers", "Complete")
    {
        facts.push(format!("Running kernel matches installed headers ({r})"));
    }

    let cfg = evidence_helpers::ev_text_value(evidence, "config.source");
    if cfg.is_some() {
        facts.push("Kernel config available".into());
    }

    if evidence_helpers::ev_bool(evidence, "fs.debugfs") {
        facts.push("debugfs mounted".into());
    }
    if evidence_helpers::ev_bool(evidence, "fs.tracefs") {
        facts.push("tracefs mounted".into());
    }

    if let Some(ref l) = loader
        && let Some(comp) = extract_field(l, "compression=")
        && comp != "none"
        && comp != "Unknown"
    {
        facts.push(format!("Module compression: {comp}"));
    }

    if let Some(vml) = evidence_helpers::ev_literal(evidence, "symbols.vmlinux") {
        facts.push(format!("VMLinux: {vml}"));
    }

    if let Some(sym) = evidence_helpers::ev_text_value(evidence, "symbols.symvers") {
        facts.push(format!("Module.symvers (source tree): {sym}"));
    }

    facts
}

fn extract_field(data: &str, prefix: &str) -> Option<String> {
    data.split_whitespace()
        .find(|p| p.starts_with(prefix))
        .map(|p| p.strip_prefix(prefix).unwrap_or(p).to_string())
}

fn esc(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            c if c.is_control() => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out
}

// Copyright (C) 2026 rezky_nightky
// SPDX-License-Identifier: GPL-3.0-only

//! Evidence model — the single source of truth.
//!
//! Every capability produces Evidence. Evidence contains only facts —
//! never formatted strings, never scoring, never recommendations.
//! Renderers, analyzers, and recommenders consume Evidence downstream.

use crate::system::config::ConfigValue;

/// Unique capability identifier.
///
/// Namespaced: `domain.key`. Examples:
/// - `kernel.release`
/// - `config.MODULES`
/// - `symbols.count`
/// - `headers.integrity`
pub type CapabilityId = &'static str;

/// Status of a capability probe.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProbeStatus {
    /// Probe succeeded, data is reliable.
    Present,
    /// Probe ran but found nothing (not an error).
    Missing,
    /// Probe failed or was denied access.
    Denied,
    /// Probe was skipped (configuration or environment prevents it).
    Skipped,
}

impl ProbeStatus {
    pub fn label(self) -> &'static str {
        match self {
            ProbeStatus::Present => "present",
            ProbeStatus::Missing => "missing",
            ProbeStatus::Denied => "denied",
            ProbeStatus::Skipped => "skipped",
        }
    }
}

/// Confidence level of the evidence.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Confidence {
    High,
    Medium,
    Low,
}

impl Confidence {
    pub fn label(self) -> &'static str {
        match self {
            Confidence::High => "high",
            Confidence::Medium => "medium",
            Confidence::Low => "low",
        }
    }
}

/// Severity of the finding — for analysis and recommendation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Severity {
    Info,
    Warning,
    Critical,
}

impl Severity {
    pub fn label(self) -> &'static str {
        match self {
            Severity::Info => "info",
            Severity::Warning => "warning",
            Severity::Critical => "critical",
        }
    }
}

/// Typed evidence value.
///
/// Each variant carries the raw data. Renderers match on the variant
/// and format appropriately. New variants can be added as capabilities grow.
#[derive(Clone, Debug)]
pub enum EvidenceValue {
    /// A string value, or None if not found.
    Text(Option<String>),
    /// A boolean value.
    Bool(bool),
    /// A kernel configuration value (y/m/n/missing).
    Config(ConfigValue),
    /// A count (symbols, modules, CRCs, etc.)
    Count(u64),
    /// A file size in bytes.
    Size(u64),
    /// A status label (for status-like values).
    Status(&'static str),
    /// A path that may or may not exist.
    Path(Option<String>),
    /// A string with no Optional wrapper — always has a value.
    Literal(String),
}

impl EvidenceValue {
    /// Returns the value as a display string (for simple renderers).
    /// Complex renderers should match on the variant.
    pub fn display(&self) -> String {
        match self {
            EvidenceValue::Text(Some(s)) => s.clone(),
            EvidenceValue::Text(None) => "Unknown".into(),
            EvidenceValue::Bool(true) => "yes".into(),
            EvidenceValue::Bool(false) => "no".into(),
            EvidenceValue::Config(cv) => cv.label(true).to_string(),
            EvidenceValue::Count(n) => n.to_string(),
            EvidenceValue::Size(n) => format_size(*n),
            EvidenceValue::Status(s) => s.to_string(),
            EvidenceValue::Path(Some(p)) => p.clone(),
            EvidenceValue::Path(None) => "not found".into(),
            EvidenceValue::Literal(s) => s.clone(),
        }
    }
}

/// A single piece of evidence from a capability probe.
///
/// This is the core data type in the Wolfzenix architecture.
/// Everything downstream — rendering, analysis, recommendations —
/// flows from Evidence.
#[derive(Clone, Debug)]
pub struct Evidence {
    /// Unique capability identifier (e.g., "kernel.release").
    pub id: CapabilityId,
    /// Whether the probe succeeded.
    pub status: ProbeStatus,
    /// How confident we are in this evidence.
    pub confidence: Confidence,
    /// The actual data.
    pub value: EvidenceValue,
}

impl Evidence {
    /// Create a new evidence item from a successful probe.
    pub fn present(id: CapabilityId, value: EvidenceValue) -> Self {
        Evidence {
            id,
            status: ProbeStatus::Present,
            confidence: Confidence::High,
            value,
        }
    }

    /// Evidence for a probe that found nothing.
    pub fn missing(id: CapabilityId, value: EvidenceValue) -> Self {
        Evidence {
            id,
            status: ProbeStatus::Missing,
            confidence: Confidence::High,
            value,
        }
    }

    /// Evidence with explicit confidence.
    pub fn with_confidence(self, confidence: Confidence) -> Self {
        Evidence { confidence, ..self }
    }
}

fn format_size(bytes: u64) -> String {
    if bytes >= 1_048_576 {
        format!("{:.1} MB", bytes as f64 / 1_048_576.0)
    } else if bytes >= 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{bytes} B")
    }
}

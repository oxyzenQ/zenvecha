// Copyright (C) 2026 rezky_nightky
// SPDX-License-Identifier: GPL-3.0-only

//! Shared evidence accessor helpers.
//!
//! Single source of truth for querying Evidence slices.
//! Every module that reads Evidence uses these functions.
//! No duplication allowed.

use super::evidence::{Evidence, EvidenceValue};

/// Get the display string for an evidence item.
pub fn ev_s(evidence: &[Evidence], id: &str) -> String {
    evidence
        .iter()
        .find(|e| e.id == id)
        .map_or_else(|| "Unknown".into(), |e| e.value.display())
}

/// Get a boolean value from evidence.
/// Handles Bool, Config (is_enabled), and Count (>0) variants.
pub fn ev_bool(evidence: &[Evidence], id: &str) -> bool {
    evidence
        .iter()
        .find(|e| e.id == id)
        .is_some_and(|e| match &e.value {
            EvidenceValue::Bool(b) => *b,
            EvidenceValue::Config(cv) => cv.is_enabled(),
            EvidenceValue::Count(n) => *n > 0,
            _ => false,
        })
}

/// Get a text or literal value as Option<String>.
pub fn ev_text_value(evidence: &[Evidence], id: &str) -> Option<String> {
    evidence
        .iter()
        .find(|e| e.id == id)
        .and_then(|e| match &e.value {
            EvidenceValue::Text(Some(s)) => Some(s.clone()),
            EvidenceValue::Literal(s) => Some(s.clone()),
            _ => None,
        })
}

/// Check if evidence has a known text or path value.
pub fn ev_text_known(evidence: &[Evidence], id: &str) -> bool {
    evidence.iter().find(|e| e.id == id).is_some_and(|e| {
        matches!(
            &e.value,
            EvidenceValue::Text(Some(_)) | EvidenceValue::Path(Some(_))
        )
    })
}

/// Check if a status evidence item matches the expected value.
pub fn ev_status_is(evidence: &[Evidence], id: &str, expected: &str) -> bool {
    evidence
        .iter()
        .find(|e| e.id == id)
        .is_some_and(|e| match &e.value {
            EvidenceValue::Status(s) => *s == expected,
            _ => false,
        })
}

/// Get the raw status value as Option<String>.
pub fn ev_status_value(evidence: &[Evidence], id: &str) -> Option<String> {
    evidence
        .iter()
        .find(|e| e.id == id)
        .and_then(|e| match &e.value {
            EvidenceValue::Status(s) => Some(s.to_string()),
            _ => None,
        })
}

/// Check if a config evidence item has a known value (not Missing).
pub fn ev_config_known(evidence: &[Evidence], id: &str) -> bool {
    evidence
        .iter()
        .find(|e| e.id == id)
        .is_some_and(|e| match &e.value {
            EvidenceValue::Config(cv) => cv.is_known(),
            _ => false,
        })
}

/// Get a config label string, considering whether config is available.
pub fn ev_config_label(evidence: &[Evidence], id: &str, cfg_available: bool) -> String {
    evidence
        .iter()
        .find(|e| e.id == id)
        .map_or("Unknown".into(), |e| match &e.value {
            EvidenceValue::Config(cv) => cv.label(cfg_available).to_string(),
            v => v.display(),
        })
}

/// Get a config boolean (returns None if unknown/missing).
pub fn ev_config_bool(evidence: &[Evidence], id: &str) -> Option<bool> {
    evidence
        .iter()
        .find(|e| e.id == id)
        .and_then(|e| match &e.value {
            EvidenceValue::Bool(b) => Some(*b),
            EvidenceValue::Config(cv) => {
                if cv.is_known() {
                    Some(cv.is_enabled())
                } else {
                    None
                }
            }
            _ => None,
        })
}

/// Get a count value as string.
pub fn ev_count(evidence: &[Evidence], id: &str) -> String {
    evidence
        .iter()
        .find(|e| e.id == id)
        .map_or_else(|| "0".into(), |e| e.value.display())
}

/// Get the confidence label for an evidence item.
pub fn ev_confidence(evidence: &[Evidence], id: &str) -> &'static str {
    evidence
        .iter()
        .find(|e| e.id == id)
        .map_or("low", |e| e.confidence.label())
}

/// Get a literal or text value as Option<String>.
pub fn ev_literal(evidence: &[Evidence], id: &str) -> Option<String> {
    evidence
        .iter()
        .find(|e| e.id == id)
        .and_then(|e| match &e.value {
            EvidenceValue::Literal(s) => Some(s.clone()),
            EvidenceValue::Text(Some(s)) => Some(s.clone()),
            _ => None,
        })
}

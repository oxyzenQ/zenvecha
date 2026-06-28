// Copyright (C) 2026 rezky_nightky
// SPDX-License-Identifier: GPL-3.0-only

//! Compatibility engine — assesses compatibility from Evidence.
//!
//! Placeholder engine. Receives immutable Evidence, returns compatibility
//! assessments. Extensible for future compatibility checks.

use crate::core::evidence::Evidence;

/// Compatibility assessment result.
#[derive(Clone, Debug, Default)]
pub struct Compatibility {
    /// Overall compatibility level.
    pub level: &'static str,
    /// Detailed compatibility notes.
    pub notes: Vec<String>,
}

/// Assess compatibility from evidence.
pub fn assess(_evidence: &[Evidence]) -> Compatibility {
    // Compatibility assessment will be extended in future releases.
    Compatibility::default()
}

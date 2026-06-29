// Copyright (C) 2026 rezky_nightky
// SPDX-License-Identifier: GPL-3.0-only

//! Kernel Semantic Normalization Layer.
//!
//! Maps raw kernel facts into consistent, typed semantic descriptors.
//! Deterministic, rule-based, reproducible. No AI. No randomness.
//!
//! Architecture:
//!   model.rs     → SemanticDescriptor, SemanticDomain, SemanticState
//!   normalize.rs → Rule engine (7 domains, 7 normalization rules)
//!
//! Position in pipeline:
//!   Kernel → Evidence → [Semantic Layer] → Pipeline → ...
//!                        ↑ optional enhancement
//!
//! Engines consume semantic descriptors alongside raw Evidence.
//! Phase 6 engines remain unchanged — semantic layer is additive.

pub mod model;
pub mod normalize;

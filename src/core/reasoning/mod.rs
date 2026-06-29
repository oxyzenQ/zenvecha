// Copyright (C) 2026 rezky_nightky
// SPDX-License-Identifier: GPL-3.0-only

//! Reasoning Engine — traceable explanations for every conclusion.
//!
//! Every score, risk, recommendation, prediction, and decision
//! must be traceable back to evidence. No magic values.
//!
//! Architecture:
//!   model.rs   → ReasoningNode, EvidenceRef, ReasoningResult
//!   builder.rs → constructs reasoning from existing engine outputs

pub mod builder;
pub mod model;

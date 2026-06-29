// Copyright (C) 2026 rezky_nightky
// SPDX-License-Identifier: GPL-3.0-only

//! Wolfzenix core — capability-driven kernel intelligence engine.
//!
//! Architecture:
//!   Core → Capability Registry → Evidence → Analysis → Renderers
//!
//! Capabilities detect. Evidence flows. Renderers display.

pub mod analysis;
pub mod capability;
pub mod caps;
pub mod evidence;
pub mod evidence_helpers;
pub mod knowledge;
pub mod pipeline;
pub mod recommendation;
pub mod render;
pub mod rendering;

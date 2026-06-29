// Copyright (C) 2026 rezky_nightky
// SPDX-License-Identifier: GPL-3.0-only

//! Report command — unified kernel intelligence report.
//!
//! Orchestration only. Delegates to pipeline and render layer.

use crate::core::pipeline;
use crate::core::rendering;

pub fn run(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let compact = args.iter().any(|a| a == "--compact");
    let json_mode = args.iter().any(|a| a == "--json");

    let result = pipeline::run_analysis_pipeline();

    let stdout = std::io::stdout();
    let mut out = stdout.lock();

    if json_mode {
        rendering::render_json(
            &result.evidence,
            &result.readiness,
            &result.risks,
            &result.recommendations,
            &mut out,
        )?;
    } else if compact {
        rendering::render_compact(
            &result.evidence,
            &result.readiness,
            &result.risks,
            &result.recommendations,
            &mut out,
        )?;
    } else {
        rendering::render_human(
            &result.evidence,
            &result.readiness,
            &result.risks,
            &result.recommendations,
            &mut out,
        )?;
    }

    Ok(())
}

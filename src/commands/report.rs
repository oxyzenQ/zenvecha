// Copyright (C) 2026 rezky_nightky
// SPDX-License-Identifier: GPL-3.0-only

//! Report command — full Wolfzenix intelligence report.
//!
//! Orchestration only. Delegates to pipeline and render layer.
//! Supports: human (default), compact, json.

use std::io;

use crate::core::pipeline;
use crate::core::render::report as render_report;

pub fn run(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let compact = args.iter().any(|a| a == "--compact");
    let json_mode = args.iter().any(|a| a == "--json");

    let result = pipeline::run_analysis_pipeline();

    let stdout = io::stdout();
    let mut out = stdout.lock();

    if json_mode {
        render_report::render_json_full(&result, &mut out)?;
    } else if compact {
        render_report::render_compact(
            &result.evidence,
            &result.readiness,
            &result.risks,
            &result.recommendations,
            &mut out,
        )?;
    } else {
        render_report::render_human_full(&result, &mut out)?;
    }

    Ok(())
}

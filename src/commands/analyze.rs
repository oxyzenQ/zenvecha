// Copyright (C) 2026 rezky_nightky
// SPDX-License-Identifier: GPL-3.0-only

//! Analyze command — development readiness assessment.
//!
//! Orchestration only. Delegates to pipeline and render layer.

use std::io;

use crate::core::pipeline;
use crate::core::render::analyze as render;

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let result = pipeline::run_analysis_pipeline();

    let stdout = io::stdout();
    let mut out = stdout.lock();
    render::render(
        &result.evidence,
        &result.readiness,
        &result.recommendations,
        &mut out,
    )
}

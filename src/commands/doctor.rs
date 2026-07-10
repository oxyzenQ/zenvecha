// Copyright (C) 2026 rezky_nightky
// SPDX-License-Identifier: GPL-3.0-only

//! Doctor command — system readiness check with optional --fix mode.
//!
//! Orchestration only. Delegates to pipeline and render layer.

use std::io;

use crate::core::pipeline;
use crate::core::render::doctor;

pub fn run(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let fix_mode = args.iter().any(|a| a == "--fix");
    let evidence = pipeline::collect_evidence();

    let stdout = io::stdout();
    let mut out = stdout.lock();
    doctor::render(&evidence, &mut out, fix_mode)
}

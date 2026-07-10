// Copyright (C) 2026 rezky_nightky
// SPDX-License-Identifier: GPL-3.0-only

//! Inspect command — kernel capability discovery.
//!
//! Orchestration only. Delegates to pipeline and render layer.

use std::io;

use crate::core::pipeline;
use crate::core::render::inspect;

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let evidence = pipeline::collect_evidence();

    let stdout = io::stdout();
    let mut out = stdout.lock();
    inspect::render(&evidence, &mut out)
}

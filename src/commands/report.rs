// Copyright (C) 2026 rezky_nightky
// SPDX-License-Identifier: GPL-3.0-only

//! Report command — unified kernel intelligence report.
//!
//! Thin orchestrator. All data from Registry, all rendering from core.

use crate::core::capability::Registry;
use crate::core::rendering;

pub fn run(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let compact = args.iter().any(|a| a == "--compact");
    let json_mode = args.iter().any(|a| a == "--json");

    let reg = Registry::default();
    let evidence = reg.run_all();

    let stdout = std::io::stdout();
    let mut out = stdout.lock();

    if json_mode {
        rendering::render_json(&evidence, &mut out)?;
    } else if compact {
        rendering::render_compact(&evidence, &mut out)?;
    } else {
        rendering::render_human(&evidence, &mut out)?;
    }

    Ok(())
}

// Copyright (C) 2026 rezky_nightky
// SPDX-License-Identifier: GPL-3.0-only

//! Report command — unified kernel intelligence report.
//!
//! Gathers all inspections once, then formats in human, compact, or JSON mode.

use crate::system::{formatter, json, report};

pub fn run(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let compact = args.iter().any(|a| a == "--compact");
    let json_mode = args.iter().any(|a| a == "--json");

    let ctx = report::gather();

    let stdout = std::io::stdout();
    let mut out = stdout.lock();

    if json_mode {
        json::write_json(&ctx, &mut out)?;
    } else if compact {
        formatter::write_compact(&ctx, &mut out)?;
    } else {
        formatter::write_human(&ctx, &mut out)?;
    }

    Ok(())
}

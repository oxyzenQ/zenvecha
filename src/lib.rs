// Copyright (C) 2026 rezky_nightky
// SPDX-License-Identifier: GPL-3.0-only

//! Zenvecha library — shared types, CLI, and core services.

pub mod cli;

/// Run the Zenvecha CLI.
pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    cli::dispatch()
}

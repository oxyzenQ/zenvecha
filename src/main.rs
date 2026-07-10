// Copyright (C) 2026 rezky_nightky
// SPDX-License-Identifier: GPL-3.0-only

//! Zenvecha — Safe runtime kernel patching.
//!
//! Bootstrap and CLI wiring only. All logic lives in modules.
//! Target: <150 LOC for main.rs.

use std::process;

fn main() {
    zenvecha::run().unwrap_or_else(|err| {
        eprintln!("zenvecha: {err}");
        process::exit(1);
    });
}

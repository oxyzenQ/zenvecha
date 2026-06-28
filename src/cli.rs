// Copyright (C) 2026 rezky_nightky
// SPDX-License-Identifier: GPL-3.0-only

//! CLI dispatch — command routing.
//!
//! Thin router. All command logic lives under `commands/`.

use std::io::{self, Write};

const VERSION: &str = env!("CARGO_PKG_VERSION");
const BUILD: &str = "linux-x86_64";
const COMMIT: &str = env!("ZENVECHA_COMMIT_HASH");

pub fn dispatch() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        print_usage();
        return Ok(());
    }

    match args[1].as_str() {
        "-V" | "--version" => print_version(),
        "--check-update" => check_update()?,
        "doctor" => {
            crate::commands::doctor::run(&args)?;
        }
        "inspect" => crate::commands::inspect::run()?,
        "analyze" => crate::commands::analyze::run()?,
        "abi" => crate::commands::abi::run()?,
        "report" => crate::commands::report::run(&args)?,
        _ => {
            eprintln!("zenvecha: unknown command '{}'", args[1]);
            print_usage();
        }
    }

    Ok(())
}

fn print_usage() {
    let stdout = io::stdout();
    let mut out = stdout.lock();
    let _ = writeln!(out, "zenvecha — Safe runtime kernel patching");
    let _ = writeln!(out);
    let _ = writeln!(out, "USAGE:");
    let _ = writeln!(out, "  zenvecha -V, --version    Show version");
    let _ = writeln!(out, "  zenvecha --check-update    Check latest release");
    let _ = writeln!(out, "  zenvecha doctor            Check system readiness");
    let _ = writeln!(out, "  zenvecha doctor --fix      Show fix commands");
    let _ = writeln!(
        out,
        "  zenvecha inspect           Kernel capability discovery"
    );
    let _ = writeln!(
        out,
        "  zenvecha analyze           Development readiness assessment"
    );
    let _ = writeln!(
        out,
        "  zenvecha abi               Kernel ABI & compatibility intelligence"
    );
    let _ = writeln!(out, "  zenvecha report [--json|--compact]");
    let _ = writeln!(
        out,
        "                              Unified intelligence report"
    );
    let _ = writeln!(out);
    let _ = writeln!(out, "See docs/ for full documentation.");
}

fn print_version() {
    let stdout = io::stdout();
    let mut out = stdout.lock();
    let _ = writeln!(out, "zenvecha -V/--version");
    let _ = writeln!(out, "Version: v{VERSION}");
    let _ = writeln!(out, "Build: {BUILD} ({COMMIT})");
    let _ = writeln!(out, "Copyright: (c) 2026 rezky_nightky (oxyzenQ)");
    let _ = writeln!(out, "License: GPL-3.0");
    let _ = writeln!(out, "Source: https://github.com/oxyzenQ/zenvecha");
}

fn check_update() -> Result<(), Box<dyn std::error::Error>> {
    let stdout = io::stdout();
    let mut out = stdout.lock();
    let _ = writeln!(out, "zenvecha update check");
    let _ = writeln!(out);
    let _ = writeln!(
        out,
        "Checking https://github.com/oxyzenQ/zenvecha/releases/latest ..."
    );
    let _ = writeln!(out);
    let _ = writeln!(out, " Current build:");
    let _ = writeln!(out, " v{VERSION} (commit {COMMIT})");
    let _ = writeln!(out);
    let _ = writeln!(
        out,
        " Source: https://github.com/oxyzenQ/zenvecha/releases/latest"
    );
    Ok(())
}

// Copyright (C) 2026 rezky_nightky
// SPDX-License-Identifier: GPL-3.0-only

//! CLI dispatch — command routing.
//!
//! Thin router. All command logic lives under `commands/`.

use std::io::{self, Write};

const VERSION: &str = env!("CARGO_PKG_VERSION");
/// Dynamic build target: detects arch + libc env at compile time.
/// Returns e.g. "linux-amd64-gnu" (glibc, dynamic) or "linux-amd64-musl"
/// (static) for x86_64 Linux builds.
const BUILD: &str = {
    #[cfg(all(target_os = "linux", target_arch = "x86_64", target_env = "musl"))]
    {
        "linux-amd64-musl"
    }
    #[cfg(all(target_os = "linux", target_arch = "x86_64", target_env = "gnu"))]
    {
        "linux-amd64-gnu"
    }
    #[cfg(all(
        target_os = "linux",
        target_arch = "x86_64",
        not(any(target_env = "musl", target_env = "gnu"))
    ))]
    {
        "linux-amd64"
    }
    #[cfg(all(target_os = "linux", target_arch = "aarch64", target_env = "musl"))]
    {
        "linux-aarch64-musl"
    }
    #[cfg(all(target_os = "linux", target_arch = "aarch64", target_env = "gnu"))]
    {
        "linux-aarch64-gnu"
    }
    #[cfg(all(
        target_os = "linux",
        target_arch = "aarch64",
        not(any(target_env = "musl", target_env = "gnu"))
    ))]
    {
        "linux-aarch64"
    }
    #[cfg(not(any(
        all(target_os = "linux", target_arch = "x86_64"),
        all(target_os = "linux", target_arch = "aarch64")
    )))]
    {
        "unknown"
    }
};
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
        "patch" => crate::commands::patch::run(&args)?,
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
    use std::process::Command;

    const GITHUB_API_URL: &str = "https://api.github.com/repos/oxyzenQ/zenvecha/releases/latest";
    const RELEASES_URL: &str = "https://github.com/oxyzenQ/zenvecha/releases/latest";

    let output = Command::new("curl")
        .args([
            "--silent",
            "--max-time",
            "15",
            "--header",
            "Accept: application/vnd.github+json",
            "--header",
            "User-Agent: zenvecha-check-update",
            "--write-out",
            "\n%{http_code}",
            GITHUB_API_URL,
        ])
        .output();

    let output = match output {
        Ok(o) => o,
        Err(e) => {
            if e.kind() == std::io::ErrorKind::NotFound {
                eprintln!("zenvecha update check failed: curl is not available on PATH");
            } else {
                eprintln!("zenvecha update check failed: {e}");
            }
            return Ok(());
        }
    };

    if !output.status.success() {
        eprintln!("zenvecha update check failed: network request failed");
        return Ok(());
    }

    let raw = String::from_utf8_lossy(&output.stdout);
    let (body, status_str) = match raw.rsplit_once('\n') {
        Some(pair) => pair,
        None => {
            eprintln!("zenvecha update check failed: GitHub API response was malformed");
            return Ok(());
        }
    };
    let status: u16 = status_str.trim().parse().unwrap_or(0);
    if status != 200 {
        eprintln!("zenvecha update check failed: GitHub API returned HTTP {status}");
        return Ok(());
    }

    // Extract tag_name from JSON (simple parse — no serde needed)
    let latest_tag = {
        let key = "\"tag_name\"";
        let key_pos = match body.find(key) {
            Some(p) => p + key.len(),
            None => {
                eprintln!("zenvecha update check failed: could not parse latest release tag");
                return Ok(());
            }
        };
        let rest = match body.get(key_pos..) {
            Some(r) => r,
            None => {
                eprintln!("zenvecha update check failed: could not parse latest release tag");
                return Ok(());
            }
        };
        let rest = rest.trim_start();
        let rest = match rest.strip_prefix(':') {
            Some(r) => r.trim_start(),
            None => {
                eprintln!("zenvecha update check failed: could not parse latest release tag");
                return Ok(());
            }
        };
        let rest = match rest.strip_prefix('"') {
            Some(r) => r,
            None => {
                eprintln!("zenvecha update check failed: could not parse latest release tag");
                return Ok(());
            }
        };
        let end = match rest.find('"') {
            Some(e) => e,
            None => {
                eprintln!("zenvecha update check failed: could not parse latest release tag");
                return Ok(());
            }
        };
        rest[..end].to_string()
    };

    let current = format!("v{VERSION}");
    let latest = if latest_tag.starts_with('v') {
        latest_tag.clone()
    } else {
        format!("v{latest_tag}")
    };

    let status_msg = if current == latest {
        "up to date"
    } else {
        "update available"
    };

    println!("zenvecha update check");
    println!("Current: {current}");
    println!("Latest:  {latest}");
    println!("Status:  {status_msg}");
    println!("Source:  {RELEASES_URL}");
    Ok(())
}

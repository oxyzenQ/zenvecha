// Copyright (C) 2026 rezky_nightky
// SPDX-License-Identifier: GPL-3.0-only

//! CLI dispatch — command routing.

use std::io::{self, Write};
use std::process::Command;

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
        "doctor" => doctor()?,
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

// ---- doctor ----------------------------------------------------------------

fn doctor() -> Result<(), Box<dyn std::error::Error>> {
    let stdout = io::stdout();
    let mut out = stdout.lock();
    let mut ready = true;

    let _ = writeln!(out, "Zenvecha Doctor");
    let _ = writeln!(out);

    // Kernel version
    let _ = writeln!(out, "Kernel:");
    match kernel_version() {
        Some(ver) => {
            let _ = writeln!(out, " {ver}");
            if !ver.starts_with("6.") {
                let _ = writeln!(out, " WARNING: Kernel 6.x expected.");
                ready = false;
            }
        }
        None => {
            let _ = writeln!(out, " UNKNOWN");
            ready = false;
        }
    }
    let _ = writeln!(out);

    // Architecture
    let _ = writeln!(out, "Architecture:");
    match arch() {
        Some(a) => {
            let _ = writeln!(out, " {a}");
            if a != "x86_64" {
                let _ = writeln!(out, " WARNING: x86_64 expected.");
                ready = false;
            }
        }
        None => {
            let _ = writeln!(out, " UNKNOWN");
            ready = false;
        }
    }
    let _ = writeln!(out);

    // Rust toolchain
    let _ = writeln!(out, "Rust:");
    match rust_version() {
        Some(ver) => {
            let _ = writeln!(out, " {ver}");
        }
        None => {
            let _ = writeln!(out, " NOT FOUND");
            ready = false;
        }
    }
    let _ = writeln!(out);

    // Kernel headers
    let _ = writeln!(out, "Kernel headers:");
    if kernel_headers_present() {
        let _ = writeln!(out, " detected");
    } else {
        let _ = writeln!(out, " NOT FOUND");
        let _ = writeln!(
            out,
            " Install: your-distro-package-manager install linux-headers"
        );
        ready = false;
    }
    let _ = writeln!(out);

    // CONFIG_RUST (R4L)
    let _ = writeln!(out, "Rust-for-Linux:");
    if rust_for_linux_detected() {
        let _ = writeln!(out, " detected");
    } else {
        let _ = writeln!(out, " NOT DETECTED");
        let _ = writeln!(
            out,
            " Kernel must be built with CONFIG_RUST=y for module compilation."
        );
        ready = false;
    }
    let _ = writeln!(out);

    // Overall status
    let _ = writeln!(out, "Status:");
    if ready {
        let _ = writeln!(out, " READY");
    } else {
        let _ = writeln!(out, " NOT READY");
    }

    Ok(())
}

fn kernel_version() -> Option<String> {
    let raw = std::fs::read_to_string("/proc/version").ok()?;
    let parts: Vec<&str> = raw.split_whitespace().collect();
    if parts.len() >= 3 {
        Some(parts[2].to_string())
    } else {
        None
    }
}

fn arch() -> Option<String> {
    Command::new("uname").arg("-m").output().ok().and_then(|o| {
        String::from_utf8(o.stdout)
            .ok()
            .map(|s| s.trim().to_string())
    })
}

fn rust_version() -> Option<String> {
    Command::new("rustc")
        .arg("--version")
        .output()
        .ok()
        .and_then(|o| {
            String::from_utf8(o.stdout)
                .ok()
                .map(|s| s.trim().to_string())
        })
}

fn kernel_headers_present() -> bool {
    let ver = std::fs::read_to_string("/proc/version")
        .unwrap_or_default()
        .split_whitespace()
        .nth(2)
        .unwrap_or("")
        .to_string();
    if ver.is_empty() {
        return false;
    }
    let path = format!("/lib/modules/{}/build", ver);
    std::path::Path::new(&path).exists()
}

fn rust_for_linux_detected() -> bool {
    // Check if the running kernel has CONFIG_RUST=y via /proc/config.gz
    // or the installed kernel headers have rust support.
    let version = std::fs::read_to_string("/proc/version")
        .unwrap_or_default()
        .split_whitespace()
        .nth(2)
        .unwrap_or("")
        .to_string();

    if version.is_empty() {
        return false;
    }

    // Method 1: /proc/config.gz (if available)
    if let Ok(output) = Command::new("zgrep")
        .args(["CONFIG_RUST=y", "/proc/config.gz"])
        .output()
        && output.status.success()
    {
        return true;
    }

    // Method 2: kernel config in /boot
    let config_path = format!("/boot/config-{}", version);
    if let Ok(content) = std::fs::read_to_string(&config_path)
        && content.contains("CONFIG_RUST=y")
    {
        return true;
    }

    // Method 3: check kernel headers for Rust support
    let rust_kernel_h = format!("/lib/modules/{}/build/include/linux/rust.h", version);
    if std::path::Path::new(&rust_kernel_h).exists() {
        return true;
    }

    // Method 4: check for Rust Makefile in kernel build
    let rust_makefile = format!("/lib/modules/{}/build/rust/Makefile", version);
    std::path::Path::new(&rust_makefile).exists()
}

// Copyright (C) 2026 rezky_nightky
// SPDX-License-Identifier: GPL-3.0-only

//! CLI dispatch — command routing.

use std::collections::BTreeMap;
use std::io::{self, Write};
use std::path::Path;
use std::process::Command;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const BUILD: &str = "linux-x86_64";
const COMMIT: &str = env!("ZENVECHA_COMMIT_HASH");

// ---- dispatch --------------------------------------------------------------

pub fn dispatch() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        print_usage();
        return Ok(());
    }

    match args[1].as_str() {
        "-V" | "--version" => print_version(),
        "--check-update" => check_update()?,
        "doctor" => Doctor::new().run()?,
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

struct Doctor {
    checks: Vec<CheckResult>,
}

struct CheckResult {
    name: &'static str,
    detail: String,
    passed: bool,
    reason: Option<String>,
}

impl Doctor {
    fn new() -> Self {
        Self {
            checks: Vec::with_capacity(6),
        }
    }

    fn run(mut self) -> Result<(), Box<dyn std::error::Error>> {
        let stdout = io::stdout();
        let mut out = stdout.lock();

        let distro = detect_distro();
        let kern_ver = kernel_version();

        let _ = writeln!(out, "Zenvecha Doctor");
        let _ = writeln!(out);

        // If distro detected, show it
        if let Some(ref d) = distro {
            let _ = writeln!(out, "Detected distro: {d}");
            let _ = writeln!(out);
        }

        // --- Check: Kernel version ---
        let kernel_ok = match &kern_ver {
            Some(ver) => {
                let ok = ver.starts_with("6.");
                self.checks.push(CheckResult {
                    name: "Kernel version",
                    detail: ver.clone(),
                    passed: ok,
                    reason: if ok {
                        None
                    } else {
                        Some(format!(
                            "Detected kernel {ver}, Zenvecha requires Linux 6.x."
                        ))
                    },
                });
                ok
            }
            None => {
                self.checks.push(CheckResult {
                    name: "Kernel version",
                    detail: "unknown".into(),
                    passed: false,
                    reason: Some("Could not read /proc/version.".into()),
                });
                false
            }
        };

        // --- Check: CPU architecture ---
        let arch_ok = match arch() {
            Some(a) => {
                let ok = a == "x86_64";
                self.checks.push(CheckResult {
                    name: "CPU architecture",
                    detail: a.clone(),
                    passed: ok,
                    reason: if ok {
                        None
                    } else {
                        Some(format!("Detected {a}, Zenvecha requires x86_64."))
                    },
                });
                ok
            }
            None => {
                self.checks.push(CheckResult {
                    name: "CPU architecture",
                    detail: "unknown".into(),
                    passed: false,
                    reason: Some("uname -m failed.".into()),
                });
                false
            }
        };

        // --- Check: Rust compiler ---
        let rust_ok = match rust_version() {
            Some(ver) => {
                self.checks.push(CheckResult {
                    name: "Rust compiler",
                    detail: ver,
                    passed: true,
                    reason: None,
                });
                true
            }
            None => {
                self.checks.push(CheckResult {
                    name: "Rust compiler",
                    detail: "not found".into(),
                    passed: false,
                    reason: Some(
                        "rustc is not in PATH. Install: curl --proto '=https' \
                         --tlsv1.2 -sSf https://sh.rustup.rs | sh"
                            .into(),
                    ),
                });
                false
            }
        };

        // --- Check: Kernel headers ---
        let headers_ok = kernel_headers_present(kern_ver.as_deref());
        let headers_detail = if headers_ok {
            "detected".to_string()
        } else {
            "not found".to_string()
        };
        let headers_reason = if headers_ok {
            None
        } else {
            let pkg = header_package_name(distro.as_deref(), kern_ver.as_deref());
            Some(format!("Missing kernel headers. Install: {pkg}"))
        };
        self.checks.push(CheckResult {
            name: "Kernel headers",
            detail: headers_detail,
            passed: headers_ok,
            reason: headers_reason,
        });

        // --- Check: Rust-for-Linux (CONFIG_RUST) ---
        let r4l_ok = rust_for_linux_detected(kern_ver.as_deref());
        let r4l_detail = if r4l_ok {
            "detected".to_string()
        } else {
            "not detected".to_string()
        };
        let r4l_reason = if r4l_ok {
            None
        } else {
            Some(
                "Running kernel was not built with CONFIG_RUST=y. \
                 Boot a Rust-enabled kernel (linux-zen on Arch, \
                 default on CachyOS)."
                    .into(),
            )
        };
        self.checks.push(CheckResult {
            name: "Rust-for-Linux",
            detail: r4l_detail,
            passed: r4l_ok,
            reason: r4l_reason,
        });

        // --- Print results ---
        for c in &self.checks {
            let mark = if c.passed { "+" } else { "-" };
            let _ = writeln!(out, "[{mark}] {:<20} {}", c.name, c.detail);
        }
        let _ = writeln!(out);

        // --- Readiness score ---
        let passed = self.checks.iter().filter(|c| c.passed).count();
        let total = self.checks.len();
        let _ = writeln!(out, "Readiness score");
        let _ = writeln!(out);
        let _ = writeln!(out, " {passed} / {total} checks passed");
        let _ = writeln!(out);

        // --- Reasons ---
        let failures: Vec<&CheckResult> = self.checks.iter().filter(|c| !c.passed).collect();
        if !failures.is_empty() {
            let _ = writeln!(out, "Reason");
            let _ = writeln!(out);
            for f in &failures {
                if let Some(ref reason) = f.reason {
                    let _ = writeln!(out, " {}", reason);
                }
            }
            let _ = writeln!(out);
        }

        // --- Suggested actions ---
        if !headers_ok || !r4l_ok {
            let _ = writeln!(out, "Suggested actions");
            let _ = writeln!(out);
            let mut step = 1;
            if !headers_ok {
                let pkg = header_package_name(distro.as_deref(), kern_ver.as_deref());
                let _ = writeln!(out, " {step}. sudo {pkg}");
                step += 1;
            }
            if !r4l_ok {
                let _ = writeln!(out, " {step}. Boot a Rust-enabled kernel");
                step += 1;
            }
            if !rust_ok {
                let _ = writeln!(out, " {step}. Install Rust via https://rustup.rs");
            }
            let _ = writeln!(out);
        }

        // --- Status ---
        let ready = kernel_ok && arch_ok && rust_ok && headers_ok && r4l_ok;
        let _ = writeln!(out, "Status");
        let _ = writeln!(out);
        if ready {
            let _ = writeln!(out, " READY");
        } else {
            let _ = writeln!(out, " NOT READY");
        }

        Ok(())
    }
}

// ---- system queries --------------------------------------------------------

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

fn kernel_headers_present(kver: Option<&str>) -> bool {
    let ver = kver.unwrap_or("");
    if ver.is_empty() {
        return false;
    }
    Path::new(&format!("/lib/modules/{ver}/build")).exists()
}

fn rust_for_linux_detected(kver: Option<&str>) -> bool {
    let version = kver.unwrap_or("");
    if version.is_empty() {
        return false;
    }

    // Method 1: /proc/config.gz
    if let Ok(output) = Command::new("zgrep")
        .args(["CONFIG_RUST=y", "/proc/config.gz"])
        .output()
        && output.status.success()
    {
        return true;
    }

    // Method 2: /boot/config-*
    let config_path = format!("/boot/config-{version}");
    if let Ok(content) = std::fs::read_to_string(&config_path)
        && content.contains("CONFIG_RUST=y")
    {
        return true;
    }

    // Method 3: kernel headers include/linux/rust.h
    let rust_h = format!("/lib/modules/{version}/build/include/linux/rust.h");
    if Path::new(&rust_h).exists() {
        return true;
    }

    // Method 4: kernel build rust/Makefile
    let rust_makefile = format!("/lib/modules/{version}/build/rust/Makefile");
    Path::new(&rust_makefile).exists()
}

// ---- distro detection ------------------------------------------------------

fn detect_distro() -> Option<String> {
    let content = std::fs::read_to_string("/etc/os-release").ok()?;
    let mut name: Option<String> = None;
    let mut id: Option<String> = None;

    for line in content.lines() {
        if let Some(val) = line.strip_prefix("NAME=") {
            name = Some(val.trim_matches('"').to_string());
        }
        if let Some(val) = line.strip_prefix("ID=") {
            id = Some(val.trim_matches('"').to_string());
        }
    }

    name.or(id)
}

/// Map distro + kernel version to the correct header package install command.
fn header_package_name(distro: Option<&str>, kver: Option<&str>) -> String {
    let id = distro
        .and_then(|d| {
            let lower = d.to_lowercase();
            if lower.contains("cachyos") {
                Some("cachyos")
            } else if lower.contains("arch") {
                Some("arch")
            } else if lower.contains("ubuntu") {
                Some("ubuntu")
            } else if lower.contains("debian") {
                Some("debian")
            } else if lower.contains("fedora") {
                Some("fedora")
            } else {
                None
            }
        })
        .unwrap_or("unknown");

    // Build a map of known header packages for this distro
    let mut headers: BTreeMap<&str, &str> = BTreeMap::new();

    match id {
        "cachyos" => {
            let _ = headers.insert("cachyos", "pacman -S linux-cachyos-lts-headers");
            let _ = headers.insert(
                "cachyos-hardened",
                "pacman -S linux-cachyos-hardened-headers",
            );
            let _ = headers.insert("cachyos-zen", "pacman -S linux-cachyos-zen-headers");
        }
        "arch" => {
            let _ = headers.insert("arch", "pacman -S linux-headers");
            let _ = headers.insert("arch-zen", "pacman -S linux-zen-headers");
            let _ = headers.insert("arch-lts", "pacman -S linux-lts-headers");
            let _ = headers.insert("arch-hardened", "pacman -S linux-hardened-headers");
        }
        "ubuntu" | "debian" => {
            let _ = headers.insert("generic", "apt install linux-headers-$(uname -r)");
        }
        "fedora" => {
            let _ = headers.insert("generic", "dnf install kernel-headers kernel-devel");
        }
        _ => {
            let _ = headers.insert("generic", "install linux-headers for your distribution");
        }
    };

    // Try to match kernel version string to a specific variant
    if let Some(ver) = kver {
        if ver.contains("cachyos-hardened") {
            return headers
                .get("cachyos-hardened")
                .unwrap_or(&"pacman -S linux-headers")
                .to_string();
        }
        if ver.contains("cachyos-zen") {
            return headers
                .get("cachyos-zen")
                .unwrap_or(&"pacman -S linux-headers")
                .to_string();
        }
        if ver.contains("cachyos") {
            return headers
                .get("cachyos")
                .unwrap_or(&"pacman -S linux-headers")
                .to_string();
        }
        if ver.contains("zen") {
            return headers
                .get("arch-zen")
                .unwrap_or(&"pacman -S linux-zen-headers")
                .to_string();
        }
        if ver.contains("lts") {
            return headers
                .get("arch-lts")
                .unwrap_or(&"pacman -S linux-lts-headers")
                .to_string();
        }
        if ver.contains("hardened") {
            return headers
                .get("arch-hardened")
                .unwrap_or(&"pacman -S linux-hardened-headers")
                .to_string();
        }
    }

    headers
        .get(id)
        .or_else(|| headers.get("generic"))
        .copied()
        .unwrap_or("install linux-headers for your distribution")
        .to_string()
}

// Copyright (C) 2026 rezky_nightky
// SPDX-License-Identifier: GPL-3.0-only

//! Kernel identity queries — version, architecture, compiler, distro.

use std::process::Command;

/// Running kernel version from /proc/version (third field).
pub fn kernel_version() -> Option<String> {
    let raw = std::fs::read_to_string("/proc/version").ok()?;
    let parts: Vec<&str> = raw.split_whitespace().collect();
    if parts.len() >= 3 {
        Some(parts[2].to_string())
    } else {
        None
    }
}

/// Kernel release string (`uname -r`).
pub fn kernel_release() -> Option<String> {
    Command::new("uname")
        .arg("-r")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
}

/// CPU architecture (`uname -m`).
pub fn architecture() -> Option<String> {
    Command::new("uname")
        .arg("-m")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
}

/// Rust compiler version string, if in PATH.
pub fn compiler_version() -> Option<String> {
    Command::new("rustc")
        .arg("--version")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
}

/// True when `rustc` is discoverable.
pub fn compiler_available() -> bool {
    compiler_version().is_some()
}

/// Distribution name from /etc/os-release (prefers NAME, falls back to ID).
pub fn detect_distro() -> Option<String> {
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

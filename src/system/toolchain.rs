// Copyright (C) 2026 rezky_nightky
// SPDX-License-Identifier: GPL-3.0-only

//! Toolchain inspection — rustc, cargo, rustfmt, clippy, bindgen, LLVM.

use std::process::Command;

/// Result of toolchain inspection.
pub struct ToolchainInfo {
    pub rustc: Option<String>,
    pub cargo: Option<String>,
    pub rustfmt: Option<String>,
    pub clippy: Option<String>,
    pub bindgen: Option<String>,
    pub llvm_version: Option<String>,
}

/// Inspect the Rust/Linux toolchain.
pub fn inspect_toolchain() -> ToolchainInfo {
    ToolchainInfo {
        rustc: tool_version("rustc", &["--version"]),
        cargo: tool_version("cargo", &["--version"]),
        rustfmt: tool_version("rustfmt", &["--version"]),
        clippy: tool_version("cargo-clippy", &["--version"])
            .or_else(|| tool_version("clippy-driver", &["--version"])),
        bindgen: tool_version("bindgen", &["--version"]),
        llvm_version: llvm_version(),
    }
}

fn tool_version(binary: &str, args: &[&str]) -> Option<String> {
    Command::new(binary)
        .args(args)
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.lines().next().unwrap_or("").trim().to_string())
        .filter(|s| !s.is_empty())
}

fn llvm_version() -> Option<String> {
    // Prefer llvm-config
    if let Some(v) = tool_version("llvm-config", &["--version"]) {
        return Some(v);
    }
    // Fallback: extract from rustc --version --verbose
    let output = Command::new("rustc")
        .args(["--version", "--verbose"])
        .output()
        .ok()?;
    let text = String::from_utf8(output.stdout).ok()?;
    for line in text.lines() {
        if let Some(val) = line.strip_prefix("LLVM version: ") {
            return Some(val.trim().to_string());
        }
    }
    None
}

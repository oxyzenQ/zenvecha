// Copyright (C) 2026 rezky_nightky
// SPDX-License-Identifier: GPL-3.0-only

//! Kernel build environment inspection.
//!
//! Source directories, symbol maps, compile commands — never modifies.

use std::path::Path;

/// Result of build environment inspection.
pub struct BuildEnvInfo {
    pub running_kernel: Option<String>,
    pub build_dir: Option<String>,
    pub source_dir: Option<String>,
    pub module_symvers: Option<String>,
    pub system_map: Option<String>,
    pub compile_commands: bool,
    /// Minimum rustc version the kernel was built with (from build metadata).
    pub kernel_rustc_min: Option<String>,
}

/// Inspect the kernel build environment.
pub fn inspect_build_env() -> BuildEnvInfo {
    let running = crate::system::kernel::kernel_release();

    let build_dir = running.as_ref().and_then(|r| {
        let p = format!("/lib/modules/{r}/build");
        Path::new(&p).is_dir().then_some(p)
    });

    let source_dir = running.as_ref().and_then(|r| {
        let p = format!("/lib/modules/{r}/source");
        if Path::new(&p).is_dir() {
            return Some(p);
        }
        let alt = format!("/lib/modules/{r}/build");
        if Path::new(&alt).is_dir() {
            // Try to resolve the symlink target
            if let Ok(target) = std::fs::read_link(&alt) {
                let resolved = if target.is_absolute() {
                    target.to_string_lossy().to_string()
                } else {
                    format!("{alt}/../{}", target.to_string_lossy())
                };
                let normalized = normalize_path(&resolved);
                if Path::new(&normalized).is_dir() {
                    return Some(normalized);
                }
            }
        }
        // Also check /usr/src/linux
        if Path::new("/usr/src/linux").is_dir() {
            return Some("/usr/src/linux".into());
        }
        None
    });

    let module_symvers = build_dir.as_ref().and_then(|d| {
        let p = format!("{d}/Module.symvers");
        Path::new(&p).exists().then_some(p)
    });

    let system_map = running.as_ref().and_then(|r| {
        let candidates = [format!("/boot/System.map-{r}"), "/boot/System.map".into()];
        candidates.into_iter().find(|p| Path::new(p).exists())
    });

    let compile_commands = build_dir
        .as_ref()
        .map(|d| Path::new(&format!("{d}/compile_commands.json")).exists())
        .unwrap_or(false);

    let kernel_rustc_min = detect_kernel_rustc_min(build_dir.as_deref());

    BuildEnvInfo {
        running_kernel: running,
        build_dir,
        source_dir,
        module_symvers,
        system_map,
        compile_commands,
        kernel_rustc_min,
    }
}

/// Best-effort extraction of the kernel's minimum rustc version.
fn detect_kernel_rustc_min(build_dir: Option<&str>) -> Option<String> {
    let dir = build_dir?;

    // Check rust/Makefile for RUSTC_VERSION
    let makefile = format!("{dir}/rust/Makefile");
    if let Ok(content) = std::fs::read_to_string(&makefile) {
        for line in content.lines() {
            if let Some(rest) = line.strip_prefix("RUSTC_VERSION")
                && let Some(val) = rest.split('=').nth(1)
            {
                return Some(val.trim().to_string());
            }
        }
    }

    // Fallback: generated rustc_cfg
    let cfg = format!("{dir}/include/generated/rustc_cfg");
    if Path::new(&cfg).exists() {
        return Some("detected (version unknown)".into());
    }

    None
}

fn normalize_path(raw: &str) -> String {
    let mut parts: Vec<&str> = Vec::new();
    for part in raw.split('/') {
        match part {
            "" | "." => continue,
            ".." => {
                parts.pop();
            }
            _ => parts.push(part),
        }
    }
    format!("/{}", parts.join("/"))
}

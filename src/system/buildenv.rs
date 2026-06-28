// Copyright (C) 2026 rezky_nightky
// SPDX-License-Identifier: GPL-3.0-only

//! Kernel build environment inspection.
//!
//! Source directories, symbol maps, compile commands, header integrity.
//! Never modifies the system.

use std::path::Path;

/// Integrity status of kernel headers in the build tree.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HeaderStatus {
    /// All key files present (include/linux, Makefile, Kconfig).
    Ready,
    /// Some but not all key files present.
    Partial,
    /// Build directory exists but key files are missing.
    Broken,
    /// No build directory found at all.
    Missing,
}

impl HeaderStatus {
    pub fn is_ready(self) -> bool {
        matches!(self, HeaderStatus::Ready)
    }

    pub fn label(self) -> &'static str {
        match self {
            HeaderStatus::Ready => "Ready",
            HeaderStatus::Partial => "Partial",
            HeaderStatus::Broken => "Broken",
            HeaderStatus::Missing => "Missing",
        }
    }
}

/// Result of build environment inspection.
pub struct BuildEnvInfo {
    pub running_kernel: Option<String>,
    pub build_dir: Option<String>,
    pub source_dir: Option<String>,
    pub module_symvers: Option<String>,
    pub system_map: Option<String>,
    pub compile_commands: bool,
    pub header_status: HeaderStatus,
    /// Minimum rustc version the kernel was built with (from build metadata).
    pub kernel_rustc_min: Option<String>,
}

/// Inspect the kernel build environment.
pub fn inspect_build_env() -> BuildEnvInfo {
    let running = crate::system::kernel::kernel_release();

    let build_dir = find_build_dir(running.as_deref());
    let header_status = verify_headers(build_dir.as_deref());

    let source_dir = running.as_ref().and_then(|r| {
        let p = format!("/lib/modules/{r}/source");
        if Path::new(&p).is_dir() {
            return Some(p);
        }
        let alt = format!("/lib/modules/{r}/build");
        if Path::new(&alt).is_dir()
            && let Ok(target) = std::fs::read_link(&alt)
        {
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
        header_status,
        kernel_rustc_min,
    }
}

/// Verify header integrity inside a build directory.
fn verify_headers(build_dir: Option<&str>) -> HeaderStatus {
    let dir = match build_dir {
        Some(d) => d,
        None => return HeaderStatus::Missing,
    };

    let checks = [
        Path::new(&format!("{dir}/include/linux")).is_dir(),
        Path::new(&format!("{dir}/Makefile")).exists(),
        Path::new(&format!("{dir}/Kconfig")).exists(),
    ];

    let count = checks.iter().filter(|&&p| p).count();
    match count {
        3 => HeaderStatus::Ready,
        1 | 2 => HeaderStatus::Partial,
        _ => HeaderStatus::Broken,
    }
}

/// Search for the kernel build directory across distribution-specific locations.
fn find_build_dir(kernel: Option<&str>) -> Option<String> {
    let k = kernel?;

    // 1. Standard: /lib/modules/<kernel>/build
    let std = format!("/lib/modules/{k}/build");
    if Path::new(&std).is_dir() {
        return Some(std);
    }

    // 2. Arch / CachyOS: /usr/src/linux-* variants
    if let Ok(entries) = std::fs::read_dir("/usr/src") {
        for entry in entries.filter_map(|e| e.ok()) {
            let name = entry.file_name();
            let n = name.to_string_lossy();
            if n.starts_with("linux") && entry.path().is_dir() {
                // Check for include/linux inside — real kernel tree, not just a name match
                if entry.path().join("include/linux").is_dir() {
                    return Some(format!("/usr/src/{n}"));
                }
            }
        }
    }

    // 3. Fedora: /usr/src/kernels/<kernel>
    let fedora = format!("/usr/src/kernels/{k}");
    if Path::new(&fedora).is_dir() {
        return Some(fedora);
    }

    // 4. Ubuntu/Debian: /usr/src/linux-headers-<kernel>
    let deb = format!("/usr/src/linux-headers-{k}");
    if Path::new(&deb).is_dir() {
        return Some(deb);
    }

    // 5. Alternative: /usr/lib/modules/<kernel>/build (some layouts)
    let alt = format!("/usr/lib/modules/{k}/build");
    if Path::new(&alt).is_dir() {
        return Some(alt);
    }

    // 6. openSUSE: /usr/src/linux-<kernel>-obj/<arch>/<flavor>
    if let Ok(entries) = std::fs::read_dir("/usr/src") {
        for entry in entries.filter_map(|e| e.ok()) {
            let n = entry.file_name().to_string_lossy().to_string();
            if n.starts_with("linux-") && n.ends_with("-obj") && entry.path().is_dir() {
                // Look inside for arch-specific subdirs
                if let Ok(inner) = std::fs::read_dir(entry.path()) {
                    for child in inner.filter_map(|e| e.ok()) {
                        let p = child.path();
                        if p.is_dir() && p.join("include/linux").is_dir() {
                            return Some(p.to_string_lossy().to_string());
                        }
                    }
                }
            }
        }
    }

    None
}

/// Best-effort extraction of the kernel's minimum rustc version.
fn detect_kernel_rustc_min(build_dir: Option<&str>) -> Option<String> {
    let dir = build_dir?;

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

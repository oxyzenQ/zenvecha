// Copyright (C) 2026 rezky_nightky
// SPDX-License-Identifier: GPL-3.0-only

//! Module environment inspection — directories, headers, signing.

use std::path::Path;

/// Result of module environment inspection.
pub struct ModuleInfo {
    pub modules_dir: Option<String>,
    pub running_kernel: Option<String>,
    pub installed_header_version: Option<String>,
    pub build_dir_present: bool,
    pub headers_available: bool,
    pub signing_enabled: Option<bool>,
    pub signing_required: bool,
}

/// Inspect the module subsystem.
pub fn inspect_modules(config: Option<&str>) -> ModuleInfo {
    let release = super::kernel::kernel_release();
    let modules_dir = release
        .as_ref()
        .map(|r| format!("/lib/modules/{r}"))
        .filter(|p| Path::new(p).is_dir());

    let build_dir_present = modules_dir
        .as_ref()
        .map(|d| Path::new(&format!("{d}/build")).is_dir())
        .unwrap_or(false);

    let headers_available = build_dir_present;

    // Scan for installed header versions that differ from running
    let running_kernel = release.clone();
    let installed_header_version = running_kernel.as_deref().and_then(installed_header_version);

    // Module signing: prefer CONFIG_MODULE_SIG from kernel config;
    // fall back to checking if any loaded module has a signature attribute.
    let signing_enabled = match config {
        Some(cfg) => {
            let v = super::config::config_value(cfg, "MODULE_SIG");
            if v.is_known() {
                Some(v.is_enabled())
            } else {
                check_module_signatures()
            }
        }
        None => check_module_signatures(),
    };

    // CONFIG_MODULE_SIG_FORCE or /proc/sys/kernel/modules_disabled
    let signing_required = match config {
        Some(cfg) => super::config::config_value(cfg, "MODULE_SIG_FORCE").is_enabled(),
        None => check_sig_enforce(),
    };

    ModuleInfo {
        modules_dir,
        running_kernel,
        installed_header_version,
        build_dir_present,
        headers_available,
        signing_enabled,
        signing_required,
    }
}

/// Fallback: look for signature attributes on loaded modules.
fn check_module_signatures() -> Option<bool> {
    let dir = Path::new("/sys/module");
    if !dir.is_dir() {
        return None;
    }
    // If any module directory contains a 'signature' sysfs entry, signing is in use.
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            if entry.path().join("signature").exists() {
                return Some(true);
            }
        }
    }
    None
}

/// Check whether module signature enforcement is active.
fn check_sig_enforce() -> bool {
    // /proc/sys/kernel/modules_disabled == 1
    if let Ok(val) = std::fs::read_to_string("/proc/sys/kernel/modules_disabled")
        && val.trim() == "1"
    {
        return true;
    }
    false
}

/// Find an installed kernel header version that differs from the running one.
fn installed_header_version(running: &str) -> Option<String> {
    let modules = Path::new("/lib/modules");
    if !modules.exists() {
        return None;
    }
    let entries = std::fs::read_dir(modules).ok()?;
    for entry in entries.filter_map(|e| e.ok()) {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if name == running {
            continue;
        }
        if entry.path().join("build").exists() {
            return Some(name.to_string());
        }
    }
    None
}

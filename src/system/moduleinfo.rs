// Copyright (C) 2026 rezky_nightky
// SPDX-License-Identifier: GPL-3.0-only

//! Module loader inspection — loaded modules, signing, compression,
//! livepatch status. Read-only, counts directories under /sys/module.

use std::path::Path;

/// Result of module loader inspection.
pub struct ModuleLoaderInfo {
    pub loaded_count: u64,
    pub signed_supported: bool,
    pub compression: &'static str,
    pub livepatch_enabled: bool,
}

/// Inspect the module loader via /sys/module and kernel config.
pub fn inspect_loader(cfg: Option<&str>) -> ModuleLoaderInfo {
    let loaded_count = count_loaded_modules();
    let signed_supported = check_module_signing();
    let compression = detect_compression(cfg);
    let livepatch_enabled = check_livepatch(cfg);

    ModuleLoaderInfo {
        loaded_count,
        signed_supported,
        compression,
        livepatch_enabled,
    }
}

fn count_loaded_modules() -> u64 {
    let dir = match std::fs::read_dir("/sys/module") {
        Ok(d) => d,
        Err(_) => return 0,
    };
    dir.filter_map(|e| e.ok())
        .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
        .count() as u64
}

fn check_module_signing() -> bool {
    let dir = Path::new("/sys/module");
    if !dir.is_dir() {
        return false;
    }
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            if entry.path().join("signature").exists() {
                return true;
            }
        }
    }
    false
}

fn detect_compression(cfg: Option<&str>) -> &'static str {
    use crate::system::config::{ConfigValue, config_value};
    let cv = |k: &str| cfg.map_or(ConfigValue::Missing, |t| config_value(t, k));

    if cv("MODULE_COMPRESS_ZSTD").is_enabled() {
        "zstd"
    } else if cv("MODULE_COMPRESS_XZ").is_enabled() {
        "xz"
    } else if cv("MODULE_COMPRESS_GZIP").is_enabled() {
        "gzip"
    } else if cv("MODULE_COMPRESS_NONE").is_enabled() || cv("MODULE_COMPRESS") == ConfigValue::No {
        "none"
    } else {
        "Unknown"
    }
}

fn check_livepatch(cfg: Option<&str>) -> bool {
    cfg.is_some_and(|t| super::config::config_value(t, "LIVEPATCH").is_enabled())
}

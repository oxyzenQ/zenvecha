// Copyright (C) 2026 rezky_nightky
// SPDX-License-Identifier: GPL-3.0-only

//! Kernel configuration reader.
//!
//! Reads CONFIG_* values from /boot/config-* then /proc/config.gz (via zcat).
//! Never panics on missing files or tools.

use std::path::Path;
use std::process::Command;

/// Try to load the running kernel's configuration, preferring
/// /boot/config-`uname -r` then /proc/config.gz via zcat.
///
/// Returns (content, source label) on success.
pub fn read_kernel_config() -> Option<(String, String)> {
    // 1. /boot/config-$(uname -r)
    if let Some(release) = super::kernel::kernel_release() {
        let path = format!("/boot/config-{release}");
        if let Ok(content) = std::fs::read_to_string(&path)
            && !content.is_empty()
        {
            return Some((content, path));
        }
    }

    // 2. /proc/config.gz via zcat
    let gz = Path::new("/proc/config.gz");
    if gz.exists()
        && let Ok(output) = Command::new("zcat").arg("/proc/config.gz").output()
        && output.status.success()
        && let Ok(text) = String::from_utf8(output.stdout)
        && !text.is_empty()
    {
        return Some((text, "/proc/config.gz".into()));
    }

    None
}

/// Check whether a kernel config option is enabled.
///
/// Recognises:
///   CONFIG_X=y          → true
///   CONFIG_X=m          → true (compiled as module)
///   # CONFIG_X is not set → false
///
/// Returns None when the key is absent from the config text.
pub fn config_is_set(config: &str, key: &str) -> Option<bool> {
    let set_prefix = format!("CONFIG_{key}=");
    let unset_prefix = format!("# CONFIG_{key} is not set");

    for line in config.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with(&set_prefix) {
            let value = &trimmed[set_prefix.len()..];
            return Some(value == "y" || value == "m");
        }
        if trimmed == unset_prefix.as_str() {
            return Some(false);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_config_values() {
        let sample = "\
CONFIG_MODULES=y
CONFIG_BPF=m
# CONFIG_LIVEPATCH is not set
CONFIG_KALLSYMS=y
";
        assert_eq!(config_is_set(sample, "MODULES"), Some(true));
        assert_eq!(config_is_set(sample, "BPF"), Some(true)); // =m
        assert_eq!(config_is_set(sample, "LIVEPATCH"), Some(false));
        assert_eq!(config_is_set(sample, "RUST"), None);
    }

    #[test]
    fn missing_config_not_present() {
        assert_eq!(config_is_set("", "NOTHING"), None);
    }
}

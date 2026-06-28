// Copyright (C) 2026 rezky_nightky
// SPDX-License-Identifier: GPL-3.0-only

//! Kernel configuration reader.
//!
//! Reads CONFIG_* values from /boot/config-* then /proc/config.gz (via zcat).
//! Never panics on missing files or tools.
//!
//! All config queries return [`ConfigValue`] — a four-state enum:
//! `Yes`, `Module`, `No`, `Missing` — no ambiguity, no `Option<bool>`.

use std::path::Path;
use std::process::Command;

/// The state of a CONFIG_* key.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConfigValue {
    /// `CONFIG_X=y` — compiled in.
    Yes,
    /// `CONFIG_X=m` — compiled as a loadable module.
    Module,
    /// `# CONFIG_X is not set` — explicitly disabled.
    No,
    /// Key not present in the config at all.
    Missing,
}

impl ConfigValue {
    /// True when the feature is available (y or m).
    pub fn is_enabled(self) -> bool {
        matches!(self, ConfigValue::Yes | ConfigValue::Module)
    }

    /// Human-readable label for display.
    pub fn label(self) -> &'static str {
        match self {
            ConfigValue::Yes => "y",
            ConfigValue::Module => "m",
            ConfigValue::No => "not set",
            ConfigValue::Missing => "Unknown",
        }
    }

    /// True when we have a definitive answer (not Missing).
    pub fn is_known(self) -> bool {
        !matches!(self, ConfigValue::Missing)
    }
}

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

/// Look up a CONFIG_* key in the config text.
///
/// Returns:
/// - `Yes` for `CONFIG_X=y`
/// - `Module` for `CONFIG_X=m`
/// - `No` for `# CONFIG_X is not set`
/// - `Missing` when the key is absent
pub fn config_value(config: &str, key: &str) -> ConfigValue {
    let set_prefix = format!("CONFIG_{key}=");
    let unset_prefix = format!("# CONFIG_{key} is not set");

    for line in config.lines() {
        let trimmed = line.trim();
        if let Some(value) = trimmed.strip_prefix(&set_prefix) {
            return match value {
                "y" => ConfigValue::Yes,
                "m" => ConfigValue::Module,
                _ => ConfigValue::Missing,
            };
        }
        if trimmed == unset_prefix.as_str() {
            return ConfigValue::No;
        }
    }
    ConfigValue::Missing
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_yes() {
        assert_eq!(
            config_value("CONFIG_MODULES=y\n", "MODULES"),
            ConfigValue::Yes
        );
    }

    #[test]
    fn parse_module() {
        assert_eq!(config_value("CONFIG_BPF=m\n", "BPF"), ConfigValue::Module);
    }

    #[test]
    fn parse_no() {
        assert_eq!(
            config_value("# CONFIG_LIVEPATCH is not set\n", "LIVEPATCH"),
            ConfigValue::No
        );
    }

    #[test]
    fn parse_missing() {
        assert_eq!(config_value("", "NOTHING"), ConfigValue::Missing);
    }

    #[test]
    fn enabled_checks() {
        assert!(ConfigValue::Yes.is_enabled());
        assert!(ConfigValue::Module.is_enabled());
        assert!(!ConfigValue::No.is_enabled());
        assert!(!ConfigValue::Missing.is_enabled());
    }

    #[test]
    fn label_output() {
        assert_eq!(ConfigValue::Yes.label(), "y");
        assert_eq!(ConfigValue::Module.label(), "m");
        assert_eq!(ConfigValue::No.label(), "not set");
        assert_eq!(ConfigValue::Missing.label(), "Unknown");
    }
}

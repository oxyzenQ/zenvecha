// Copyright (C) 2026 rezky_nightky
// SPDX-License-Identifier: GPL-3.0-only

//! Rust-for-Linux kernel support detection.

use super::config::ConfigValue;

/// (CONFIG_RUST, CONFIG_RUST_IS_AVAILABLE) as ConfigValue.
pub fn rust_config(config: Option<&str>) -> (ConfigValue, ConfigValue) {
    let cfg = match config {
        Some(c) => c,
        None => return (ConfigValue::Missing, ConfigValue::Missing),
    };
    (
        super::config::config_value(cfg, "RUST"),
        super::config::config_value(cfg, "RUST_IS_AVAILABLE"),
    )
}

/// True when CONFIG_RUST=y OR CONFIG_RUST_IS_AVAILABLE=y.
pub fn rust_enabled(config: Option<&str>) -> bool {
    let (r, ra) = rust_config(config);
    r.is_enabled() || ra.is_enabled()
}

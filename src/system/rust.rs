// Copyright (C) 2026 rezky_nightky
// SPDX-License-Identifier: GPL-3.0-only

//! Rust-for-Linux kernel support detection.

/// (CONFIG_RUST, CONFIG_RUST_IS_AVAILABLE) — each None when unknown.
pub fn rust_config(config: Option<&str>) -> (Option<bool>, Option<bool>) {
    let cfg = match config {
        Some(c) => c,
        None => return (None, None),
    };
    (
        super::config::config_is_set(cfg, "RUST"),
        super::config::config_is_set(cfg, "RUST_IS_AVAILABLE"),
    )
}

/// True when either CONFIG_RUST=y or CONFIG_RUST_IS_AVAILABLE=y.
pub fn rust_enabled(config: Option<&str>) -> bool {
    let (r, ra) = rust_config(config);
    r == Some(true) || ra == Some(true)
}

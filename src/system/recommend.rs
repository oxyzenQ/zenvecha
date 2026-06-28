// Copyright (C) 2026 rezky_nightky
// SPDX-License-Identifier: GPL-3.0-only

//! Actionable recommendation engine.
//!
//! Generates advice based only on failed checks.
//! Never suggests work already completed.

use super::config::ConfigValue;

/// Context for the recommendation engine.
pub struct RecCtx<'a> {
    pub rustc_installed: bool,
    pub bindgen_installed: bool,
    pub llvm_installed: bool,
    pub headers_available: bool,
    pub build_dir_present: bool,
    pub source_dir_present: bool,
    pub config_rust: ConfigValue,
    pub config_rust_available: ConfigValue,
    pub config_modules: ConfigValue,
    pub config_btf: ConfigValue,
    pub btf_available: bool,
    pub signing_required: bool,
    pub signing_enabled: bool,
    pub debugfs_ok: bool,
    pub tracefs_ok: bool,
    pub release: Option<&'a str>,
    pub headers_ver: Option<&'a str>,
}

/// Generate recommendations from the context.
pub fn generate(ctx: &RecCtx) -> Vec<String> {
    let mut recs: Vec<String> = Vec::new();

    // Toolchain
    if !ctx.rustc_installed {
        recs.push(
            "Install Rust: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh".into(),
        );
    }
    if !ctx.bindgen_installed {
        recs.push("Install bindgen: cargo install bindgen-cli".into());
    }
    if !ctx.llvm_installed {
        recs.push("Install LLVM/clang for kernel compilation".into());
    }

    // Headers
    if !ctx.headers_available {
        let mut needs_reboot = false;
        if let (Some(r), Some(h)) = (ctx.release, ctx.headers_ver)
            && r != h
        {
            recs.push(format!("Reboot into updated kernel ({h})"));
            needs_reboot = true;
        }
        // Don't suggest install if reboot will fix it
        if !needs_reboot {
            recs.push("Install kernel headers matching running kernel".into());
        }
    }

    // Build tree
    if !ctx.build_dir_present {
        recs.push("Install kernel headers to populate /lib/modules/$(uname -r)/build".into());
    }
    if !ctx.source_dir_present {
        recs.push(
            "Install kernel source or create symlink from /lib/modules/$(uname -r)/source".into(),
        );
    }

    // Rust-for-Linux
    if !ctx.config_rust.is_enabled() && !ctx.config_rust_available.is_enabled() {
        recs.push("Enable CONFIG_RUST=y in kernel configuration".into());
    }
    if ctx.config_rust_available.is_enabled() && !ctx.config_rust.is_enabled() {
        recs.push("Compile kernel with CONFIG_RUST=y (compiler is available)".into());
    }

    // Modules
    if !ctx.config_modules.is_enabled() {
        recs.push("Enable CONFIG_MODULES=y in kernel configuration".into());
    }
    if ctx.signing_required && !ctx.signing_enabled {
        recs.push("Set up module signing keys for kernel module development".into());
    }

    // Debug
    if !ctx.btf_available && !ctx.config_btf.is_enabled() {
        recs.push("Enable CONFIG_DEBUG_INFO_BTF=y for BTF support".into());
    }
    if !ctx.debugfs_ok {
        recs.push("Mount debugfs: sudo mount -t debugfs none /sys/kernel/debug".into());
    }
    if !ctx.tracefs_ok {
        recs.push("Mount tracefs: sudo mount -t tracefs none /sys/kernel/tracing".into());
    }

    recs
}

// Copyright (C) 2026 rezky_nightky
// SPDX-License-Identifier: GPL-3.0-only

//! Analyze command — kernel development readiness assessment.
//!
//! Deeper than `inspect`. Evaluates toolchain, build environment,
//! Rust-for-Linux compatibility, module development capability,
//! debug infrastructure, and filesystem layout.
//! Produces a compatibility report with a readiness percentage
//! and actionable recommendations.
//! Never modifies the system.

use std::io::{self, Write};

use crate::system::{btf, buildenv, config, fscheck, kallsyms, kernel, modules, rust, toolchain};

#[derive(Clone, Copy, PartialEq, Eq)]
enum RustCompat {
    Compatible,
    PartiallyCompatible,
    NotCompatible,
}

// ---- public API ------------------------------------------------------------

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let stdout = io::stdout();
    let mut out = stdout.lock();

    // Gather all data first
    let tools = toolchain::inspect_toolchain();
    let bld = buildenv::inspect_build_env();
    let release = kernel::kernel_release();
    let distro = kernel::detect_distro();
    let arch = kernel::architecture();
    let headers_ver = modules::inspect_modules(None).installed_header_version;

    let (config_text, _config_source) = config::read_kernel_config().unzip();
    let cfg = config_text.as_deref();

    let mod_info = modules::inspect_modules(cfg);
    let ks_info = kallsyms::inspect_kallsyms();
    let dbg = btf::inspect_debug();
    let (rust_cfg, rust_avail) = rust::rust_config(cfg);

    let fs_paths = {
        let rel = release.as_deref().unwrap_or("unknown");
        fscheck::check_paths(&[
            &format!("/lib/modules/{rel}"),
            "/usr/lib/modules",
            "/usr/src",
            "/usr/src/linux",
        ])
    };

    let debugfs_mounted = is_mount_point("/sys/kernel/debug");
    let tracefs_mounted = is_mount_point("/sys/kernel/tracing");

    // ---- Toolchain --------------------------------------------------------

    let _ = writeln!(out, "Zenvecha Analyze");
    let _ = writeln!(out);

    let _ = writeln!(out, "Toolchain");
    print_tool(&mut out, "rustc", &tools.rustc);
    print_tool(&mut out, "cargo", &tools.cargo);
    print_tool(&mut out, "rustfmt", &tools.rustfmt);
    print_tool(&mut out, "clippy", &tools.clippy);
    print_tool(&mut out, "bindgen", &tools.bindgen);
    print_tool(&mut out, "LLVM", &tools.llvm_version);
    let _ = writeln!(out);

    // ---- Kernel Build Environment -----------------------------------------

    let _ = writeln!(out, "Kernel Build Environment");
    print_kv(&mut out, "  Running kernel", release.as_deref());
    match (&release, &headers_ver) {
        (Some(r), Some(h)) if r == h => {
            let _ = writeln!(out, "  Installed headers : {h}");
        }
        (Some(_r), Some(h)) => {
            let _ = writeln!(out, "  Installed headers : {h} (mismatch — reboot needed)");
        }
        (Some(_r), None) if mod_info.headers_available => {
            let _ = writeln!(out, "  Installed headers : match running kernel");
        }
        _ => {
            print_kv(&mut out, "  Installed headers", headers_ver.as_deref());
        }
    }
    print_path_kv(&mut out, "Build directory", bld.build_dir.as_deref());
    print_path_kv(&mut out, "Source directory", bld.source_dir.as_deref());
    print_path_kv(&mut out, "Module.symvers", bld.module_symvers.as_deref());
    match bld.system_map.as_deref() {
        Some(p) => {
            let _ = writeln!(out, "System.map : {p}");
        }
        None => {
            let _ = writeln!(out, "System.map : Not installed (optional)");
        }
    }
    if bld.compile_commands {
        if let Some(ref d) = bld.build_dir {
            let _ = writeln!(out, "  compile_commands : {d}/compile_commands.json");
        }
    } else {
        let _ = writeln!(out, "  compile_commands : not found");
    }
    let _ = writeln!(out);

    // ---- Rust-for-Linux ---------------------------------------------------

    let _ = writeln!(out, "Rust-for-Linux");
    print_config(&mut out, "CONFIG_RUST", cfg, "RUST");
    print_config(
        &mut out,
        "CONFIG_RUST_IS_AVAILABLE",
        cfg,
        "RUST_IS_AVAILABLE",
    );

    if let Some(ref min_ver) = bld.kernel_rustc_min {
        let _ = writeln!(out, "  Kernel requires rustc : {min_ver}");
    } else {
        let _ = writeln!(out, "  Kernel requires rustc : Unknown");
    }

    if let Some(ref installed) = tools.rustc {
        let _ = writeln!(out, "  Installed rustc       : {installed}");
    } else {
        let _ = writeln!(out, "  Installed rustc       : Not installed");
    }

    let rust_level = match (rust_cfg, rust_avail) {
        (Some(true), Some(true)) => RustCompat::Compatible,
        (_, Some(true)) => RustCompat::PartiallyCompatible,
        _ => RustCompat::NotCompatible,
    };

    let _ = writeln!(
        out,
        "  Compatibility         : {}",
        match rust_level {
            RustCompat::Compatible => "Compatible",
            RustCompat::PartiallyCompatible =>
                "Partially compatible (compiler available, kernel not built with Rust)",
            RustCompat::NotCompatible => "Not compatible",
        }
    );

    let r4l_buildable =
        rust_level == RustCompat::Compatible && bld.build_dir.is_some() && tools.rustc.is_some();
    let r4l_msg = if r4l_buildable {
        "yes"
    } else if rust_level == RustCompat::PartiallyCompatible {
        "no (CONFIG_RUST not enabled)"
    } else {
        "no (see recommendations)"
    };
    let _ = writeln!(out, "  Rust modules buildable: {r4l_msg}");
    let _ = writeln!(out);

    // ---- Module Development -----------------------------------------------

    let _ = writeln!(out, "Module Development");
    print_config(&mut out, "CONFIG_MODULES", cfg, "MODULES");
    print_bool_opt(&mut out, "Module signing", mod_info.signing_enabled);
    print_bool(&mut out, "Signing required", mod_info.signing_required);

    // Module compression
    let compression = detect_module_compression(cfg);
    let _ = writeln!(out, "  Module compression  : {compression}");

    // Loadable module support
    let loadable = config_is(cfg, "MODULES") == Some(true) && mod_info.modules_dir.is_some();
    let _ = writeln!(
        out,
        "  Loadable modules    : {}",
        if loadable {
            "supported"
        } else {
            "not supported"
        }
    );

    print_kv(
        &mut out,
        "  Modules directory",
        mod_info.modules_dir.as_deref(),
    );
    let _ = writeln!(out);

    // ---- Debug Capability -------------------------------------------------

    let _ = writeln!(out, "Debug Capability");
    print_bool(&mut out, "BTF", dbg.btf_available);
    if ks_info.exists {
        let ks_label = if ks_info.readable {
            if ks_info.root_only {
                "readable (root only)"
            } else {
                "readable"
            }
        } else {
            "permission denied"
        };
        let _ = writeln!(out, "  Kallsyms : {ks_label}");
    } else {
        let _ = writeln!(out, "  Kallsyms : not found");
    }
    print_bool(&mut out, "DWARF", dbg.dwarf_available);
    if debugfs_mounted {
        let _ = writeln!(out, "  debugfs  : mounted (/sys/kernel/debug)");
    } else {
        let _ = writeln!(out, "  debugfs  : not mounted");
    }
    if tracefs_mounted {
        let _ = writeln!(out, "  tracefs  : mounted (/sys/kernel/tracing)");
    } else {
        let _ = writeln!(out, "  tracefs  : not mounted");
    }
    let _ = writeln!(out);

    // ---- Filesystem Checks ------------------------------------------------

    let _ = writeln!(out, "Filesystem Checks");
    for f in &fs_paths {
        let _ = writeln!(out, "  {} : {}", f.path, f.label());
    }
    let _ = writeln!(out);

    // ---- Compatibility Report ---------------------------------------------

    // Collect all boolean checks for scoring
    struct Check<'a> {
        category: &'a str,
        passed: bool,
    }

    let checks: Vec<Check> = vec![
        // Environment
        Check {
            category: "Environment",
            passed: release.is_some(),
        },
        Check {
            category: "Environment",
            passed: arch.is_some(),
        },
        Check {
            category: "Environment",
            passed: distro.is_some(),
        },
        Check {
            category: "Environment",
            passed: fs_paths.first().is_some_and(|f| f.passed()),
        },
        Check {
            category: "Environment",
            passed: fs_paths.get(2).is_some_and(|f| f.passed()),
        },
        // Toolchain
        Check {
            category: "Rust",
            passed: tools.rustc.is_some(),
        },
        Check {
            category: "Rust",
            passed: tools.cargo.is_some(),
        },
        Check {
            category: "Rust",
            passed: rust_cfg == Some(true) || rust_avail == Some(true),
        },
        Check {
            category: "Rust",
            passed: tools.bindgen.is_some(),
        },
        Check {
            category: "Rust",
            passed: rust_level == RustCompat::Compatible,
        },
        // Build
        Check {
            category: "Headers",
            passed: mod_info.headers_available,
        },
        Check {
            category: "Build tree",
            passed: bld.build_dir.is_some(),
        },
        Check {
            category: "Build tree",
            passed: bld.source_dir.is_some(),
        },
        Check {
            category: "Build tree",
            passed: bld.module_symvers.is_some(),
        },
        Check {
            category: "Build tree",
            passed: bld.compile_commands,
        },
        // Modules
        Check {
            category: "Modules",
            passed: config_is(cfg, "MODULES") == Some(true),
        },
        Check {
            category: "Modules",
            passed: mod_info.modules_dir.is_some(),
        },
        Check {
            category: "Modules",
            passed: loadable,
        },
        // Debug
        Check {
            category: "BTF",
            passed: dbg.btf_available,
        },
        Check {
            category: "Debug",
            passed: ks_info.exists && ks_info.readable,
        },
        Check {
            category: "Debug",
            passed: debugfs_mounted,
        },
    ];

    let total = checks.len() as f64;
    let passed = checks.iter().filter(|c| c.passed).count() as f64;
    let pct = (passed / total * 100.0).round() as u32;

    let _ = writeln!(out, "Compatibility Report");
    let _ = writeln!(out);

    let categories: Vec<(&str, bool)> = {
        let mut seen = std::collections::BTreeMap::new();
        for c in &checks {
            let entry = seen.entry(c.category).or_insert(true);
            *entry = *entry && c.passed;
        }
        seen.into_iter().collect()
    };

    for (cat, ok) in &categories {
        // Rust category uses three-state display
        if *cat == "Rust" {
            match rust_level {
                RustCompat::Compatible => {
                    let _ = writeln!(out, "  ✔ Rust");
                }
                RustCompat::PartiallyCompatible => {
                    let _ = writeln!(
                        out,
                        "  ~ Rust (partial — compiler available, kernel not built with Rust)"
                    );
                }
                RustCompat::NotCompatible => {
                    let _ = writeln!(out, "  ✘ Rust");
                }
            }
        } else {
            let mark = if *ok { "✔" } else { "✘" };
            let _ = writeln!(out, "  {mark} {cat}");
        }
    }
    let _ = writeln!(out);
    let _ = writeln!(out, "  Overall readiness : {pct}%");
    let _ = writeln!(out);

    // ---- Recommendations ----------------------------------------------

    let recs = {
        let ctx = RecCtx {
            tools: &tools,
            bld: &bld,
            cfg,
            mod_info: &mod_info,
            dbg: &dbg,
        };
        recommendations(
            &ctx,
            debugfs_mounted,
            tracefs_mounted,
            release.as_deref(),
            headers_ver.as_deref(),
        )
    };

    if !recs.is_empty() {
        let _ = writeln!(out, "Recommendations");
        let _ = writeln!(out);
        for (i, r) in recs.iter().enumerate() {
            let _ = writeln!(out, "  {}. {r}", i + 1);
        }
    }

    Ok(())
}

// ---- helpers ---------------------------------------------------------------

fn print_tool(out: &mut io::StdoutLock<'_>, name: &str, version: &Option<String>) {
    match version {
        Some(v) => {
            let _ = writeln!(out, "  {name:<8} : {v}");
        }
        None => {
            let _ = writeln!(out, "  {name:<8} : Not installed");
        }
    }
}

fn print_kv(out: &mut io::StdoutLock<'_>, label: &str, value: Option<&str>) {
    match value {
        Some(v) if !v.is_empty() => {
            let _ = writeln!(out, "{label} : {v}");
        }
        _ => {
            let _ = writeln!(out, "{label} : Unknown");
        }
    }
}

fn print_path_kv(out: &mut io::StdoutLock<'_>, label: &str, value: Option<&str>) {
    match value {
        Some(v) => {
            let _ = writeln!(out, "{label} : {v}");
        }
        None => {
            let _ = writeln!(out, "{label} : not found");
        }
    }
}

fn print_bool(out: &mut io::StdoutLock<'_>, label: &str, val: bool) {
    let _ = writeln!(
        out,
        "{label} : {}",
        if val { "present" } else { "not present" }
    );
}

fn print_bool_opt(out: &mut io::StdoutLock<'_>, label: &str, val: Option<bool>) {
    match val {
        Some(true) => {
            let _ = writeln!(out, "{label} : enabled");
        }
        Some(false) => {
            let _ = writeln!(out, "{label} : disabled");
        }
        None => {
            let _ = writeln!(out, "{label} : Unknown");
        }
    }
}

fn print_config(out: &mut io::StdoutLock<'_>, label: &str, cfg: Option<&str>, key: &str) {
    match cfg.and_then(|t| config::config_is_set(t, key)) {
        Some(true) => {
            let _ = writeln!(out, "  {label:<26} : y");
        }
        Some(false) => {
            let _ = writeln!(out, "  {label:<26} : not set");
        }
        None => {
            let _ = writeln!(out, "  {label:<26} : Unknown");
        }
    }
}

fn config_is(cfg: Option<&str>, key: &str) -> Option<bool> {
    cfg.and_then(|t| config::config_is_set(t, key))
}

/// Check whether a path is a mount point by consulting /proc/mounts.
fn is_mount_point(path: &str) -> bool {
    if let Ok(content) = std::fs::read_to_string("/proc/mounts") {
        for line in content.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 && parts[1] == path {
                return true;
            }
        }
    }
    false
}

fn detect_module_compression(cfg: Option<&str>) -> &str {
    let candidates = [
        ("MODULE_COMPRESS_ZSTD", "zstd"),
        ("MODULE_COMPRESS_XZ", "xz"),
        ("MODULE_COMPRESS_GZIP", "gzip"),
    ];
    for (key, name) in &candidates {
        if config_is(cfg, key) == Some(true) {
            return name;
        }
    }
    if config_is(cfg, "MODULE_COMPRESS_NONE") == Some(true)
        || config_is(cfg, "MODULE_COMPRESS") == Some(false)
    {
        return "none";
    }
    "Unknown"
}

struct RecCtx<'a> {
    tools: &'a toolchain::ToolchainInfo,
    bld: &'a buildenv::BuildEnvInfo,
    cfg: Option<&'a str>,
    mod_info: &'a modules::ModuleInfo,
    dbg: &'a btf::DebugInfo,
}

fn recommendations(
    ctx: &RecCtx,
    debugfs_ok: bool,
    tracefs_ok: bool,
    release: Option<&str>,
    headers_ver: Option<&str>,
) -> Vec<String> {
    let mut recs: Vec<String> = Vec::new();

    // Toolchain
    if ctx.tools.rustc.is_none() {
        recs.push(
            "Install Rust: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh".into(),
        );
    }
    if ctx.tools.bindgen.is_none() {
        recs.push("Install bindgen: cargo install bindgen-cli".into());
    }
    if ctx.tools.llvm_version.is_none() {
        recs.push("Install LLVM/clang for kernel compilation".into());
    }

    // Headers
    if !ctx.mod_info.headers_available {
        if let (Some(r), Some(h)) = (release, headers_ver)
            && r != h
        {
            recs.push(format!("Reboot into updated kernel ({h})"));
        }
        recs.push("Install kernel headers matching running kernel".into());
    }

    // Build tree
    if ctx.bld.build_dir.is_none() {
        recs.push("Install kernel headers to populate /lib/modules/$(uname -r)/build".into());
    }
    if ctx.bld.source_dir.is_none() {
        recs.push(
            "Install kernel source or create symlink from /lib/modules/$(uname -r)/source".into(),
        );
    }

    // Rust-for-Linux
    if config_is(ctx.cfg, "RUST") != Some(true)
        && config_is(ctx.cfg, "RUST_IS_AVAILABLE") != Some(true)
    {
        recs.push("Enable CONFIG_RUST=y in kernel configuration".into());
    }

    // Modules
    if config_is(ctx.cfg, "MODULES") != Some(true) {
        recs.push("Enable CONFIG_MODULES=y in kernel configuration".into());
    }
    if ctx.mod_info.signing_required && ctx.mod_info.signing_enabled != Some(true) {
        recs.push("Set up module signing keys for kernel module development".into());
    }

    // Debug
    if !ctx.dbg.btf_available && config_is(ctx.cfg, "DEBUG_INFO_BTF") != Some(true) {
        recs.push("Enable CONFIG_DEBUG_INFO_BTF=y for BTF support".into());
    }
    if !debugfs_ok {
        recs.push("Mount debugfs: sudo mount -t debugfs none /sys/kernel/debug".into());
    }
    if !tracefs_ok {
        recs.push("Mount tracefs: sudo mount -t tracefs none /sys/kernel/tracing".into());
    }

    recs
}

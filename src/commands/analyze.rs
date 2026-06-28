// Copyright (C) 2026 rezky_nightky
// SPDX-License-Identifier: GPL-3.0-only

//! Analyze command — kernel development readiness assessment.
//!
//! Uses scoring and recommend modules. Orchestration only.

use std::io::{self, Write};

use crate::system::{
    btf, buildenv, config, fscheck, kallsyms, kernel, modules, recommend, rust, scoring, toolchain,
};
use config::ConfigValue;

#[derive(Clone, Copy, PartialEq, Eq)]
enum RustLevel {
    Compatible,
    Partial,
    NotCompatible,
}

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let stdout = io::stdout();
    let mut out = stdout.lock();

    let tools = toolchain::inspect_toolchain();
    let bld = buildenv::inspect_build_env();
    let release = kernel::kernel_release();
    let _distro = kernel::detect_distro();
    let _arch = kernel::architecture();
    let headers_ver = modules::inspect_modules(None).installed_header_version;

    let (config_text, _config_source) = config::read_kernel_config().unzip();
    let cfg = config_text.as_deref();

    let mod_info = modules::inspect_modules(cfg);
    let ks_info = kallsyms::inspect_kallsyms();
    let dbg = btf::inspect_debug();
    let (rust_cfg, rust_avail) = rust::rust_config(cfg);

    let cv = |k: &str| cfg.map_or(ConfigValue::Missing, |t| config::config_value(t, k));

    let fs_paths = {
        let rel = release.as_deref().unwrap_or("unknown");
        fscheck::check_paths(&[
            &format!("/lib/modules/{rel}"),
            "/usr/lib/modules",
            "/usr/src",
            "/usr/src/linux",
        ])
    };

    let debugfs_mounted = mount_ok("/sys/kernel/debug");
    let tracefs_mounted = mount_ok("/sys/kernel/tracing");

    let rust_level = match (rust_cfg, rust_avail) {
        (ConfigValue::Yes, ConfigValue::Yes) => RustLevel::Compatible,
        (_, v) if v.is_enabled() => RustLevel::Partial,
        _ => RustLevel::NotCompatible,
    };

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
    let _ = writeln!(out, "  Header integrity    : {}", bld.header_status.label());
    // Clarify: header integrity ≠ full kernel source
    if bld.header_status.is_ready() && bld.source_dir.is_none() {
        let _ = writeln!(
            out,
            "  Kernel source       : not installed (header tree only)"
        );
    } else if !bld.header_status.is_ready() {
        let _ = writeln!(
            out,
            "  Kernel source       : {}",
            bld.source_dir.as_deref().unwrap_or("not found")
        );
    }
    print_path_kv(
        &mut out,
        "Module.symvers (source tree)",
        bld.module_symvers.as_deref(),
    );
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
    let _ = writeln!(
        out,
        "  CONFIG_RUST              : {}",
        rust_cfg.label(cfg.is_some())
    );
    let _ = writeln!(
        out,
        "  CONFIG_RUST_IS_AVAILABLE : {}",
        rust_avail.label(cfg.is_some())
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

    match rust_level {
        RustLevel::Compatible => {
            let _ = writeln!(out, "  Compatibility         : Compatible");
        }
        RustLevel::Partial => {
            let _ = writeln!(
                out,
                "  Compatibility         : Partially compatible (compiler available, kernel not built with Rust)"
            );
        }
        RustLevel::NotCompatible => {
            if rust_cfg == ConfigValue::Missing && rust_avail == ConfigValue::Missing {
                let _ = writeln!(
                    out,
                    "  Compatibility         : Not compatible — this kernel was not compiled with Rust support"
                );
            } else {
                let _ = writeln!(out, "  Compatibility         : Not compatible");
            }
        }
    }

    let r4l_buildable =
        rust_level == RustLevel::Compatible && bld.build_dir.is_some() && tools.rustc.is_some();
    let r4l_msg = if r4l_buildable {
        "yes"
    } else if rust_level == RustLevel::Partial {
        "no (CONFIG_RUST not enabled)"
    } else if rust_cfg == ConfigValue::Missing {
        "no — this kernel was not compiled with Rust support"
    } else {
        "no (see recommendations)"
    };
    let _ = writeln!(out, "  Rust modules buildable: {r4l_msg}");
    let _ = writeln!(out);

    // ---- Module Development -----------------------------------------------

    let _ = writeln!(out, "Module Development");
    let _ = writeln!(
        out,
        "  CONFIG_MODULES      : {}",
        cv("MODULES").label(cfg.is_some())
    );
    print_bool_opt(&mut out, "Module signing", mod_info.signing_enabled);
    print_bool(&mut out, "Signing required", mod_info.signing_required);

    let compression = detect_compression(cfg);
    let _ = writeln!(out, "  Module compression  : {compression}");

    let loadable = cv("MODULES").is_enabled() && mod_info.modules_dir.is_some();
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

    // ---- Star Score -------------------------------------------------------

    let scores = scoring::compute();
    let _ = writeln!(out, "Readiness Score");
    let _ = writeln!(out);
    for s in &scores {
        let _ = writeln!(out, "  {}  {}", s.render(), s.name);
    }
    let _ = writeln!(out);
    let _ = writeln!(out, "  Overall : {}", scoring::overall_rating(&scores));
    let _ = writeln!(out);

    // ---- Recommendations --------------------------------------------------

    let recs = recommend::generate(&recommend::RecCtx {
        rustc_installed: tools.rustc.is_some(),
        bindgen_installed: tools.bindgen.is_some(),
        llvm_installed: tools.llvm_version.is_some(),
        headers_available: mod_info.headers_available,
        build_dir_present: bld.build_dir.is_some(),
        source_dir_present: bld.source_dir.is_some(),
        config_rust: rust_cfg,
        config_rust_available: rust_avail,
        config_modules: cv("MODULES"),
        config_btf: cv("DEBUG_INFO_BTF"),
        btf_available: dbg.btf_available,
        signing_required: mod_info.signing_required,
        signing_enabled: mod_info.signing_enabled == Some(true),
        debugfs_ok: debugfs_mounted,
        tracefs_ok: tracefs_mounted,
        release: release.as_deref(),
        headers_ver: headers_ver.as_deref(),
    });

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

fn mount_ok(path: &str) -> bool {
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

fn detect_compression(cfg: Option<&str>) -> &str {
    let cv = |k: &str| cfg.map_or(ConfigValue::Missing, |t| config::config_value(t, k));
    if cv("MODULE_COMPRESS_ZSTD").is_enabled() {
        return "zstd";
    }
    if cv("MODULE_COMPRESS_XZ").is_enabled() {
        return "xz";
    }
    if cv("MODULE_COMPRESS_GZIP").is_enabled() {
        return "gzip";
    }
    if cv("MODULE_COMPRESS_NONE").is_enabled() || cv("MODULE_COMPRESS") == ConfigValue::No {
        return "none";
    }
    "Unknown"
}

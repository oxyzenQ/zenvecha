// Copyright (C) 2026 rezky_nightky
// SPDX-License-Identifier: GPL-3.0-only

//! Inspect command — read-only kernel capability discovery.
//!
//! Produces a structured report covering kernel identity, configuration,
//! module environment, debug info, symbol table, and Rust-for-Linux support.
//! Never modifies the system.

use std::io::{self, Write};

use crate::system::{btf, config, kallsyms, kernel, modules, rust};

// ---- public API ------------------------------------------------------------

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let stdout = io::stdout();
    let mut out = stdout.lock();

    let release = kernel::kernel_release();
    let arch = kernel::architecture();
    let compiler = kernel::compiler_version();
    let distro = kernel::detect_distro();

    let (config_text, config_source) = config::read_kernel_config().unzip();
    let config_text = config_text.as_deref();

    let mod_info = modules::inspect_modules(config_text);
    let ks_info = kallsyms::inspect_kallsyms();
    let debug = btf::inspect_debug();
    let (rust_cfg, rust_avail) = rust::rust_config(config_text);

    // ---- report -----------------------------------------------------------

    let _ = writeln!(out, "Zenvecha Inspect");
    let _ = writeln!(out);

    // Kernel identity
    let _ = writeln!(out, "Kernel");
    print_kv(&mut out, "  Version", release.as_deref());
    print_kv(&mut out, "  Architecture", arch.as_deref());
    print_kv(&mut out, "  Distribution", distro.as_deref());
    if let Some(ref c) = compiler {
        let _ = writeln!(out, "  Rust compiler : {c}");
    } else {
        let _ = writeln!(out, "  Rust compiler : not found");
    }
    let _ = writeln!(out);

    // Configuration
    let _ = writeln!(out, "Configuration");
    if let Some(ref src) = config_source {
        let _ = writeln!(out, "  Source : {src}");
    } else {
        let _ = writeln!(out, "  Source : not available");
    }
    let _ = writeln!(out);

    let cfg_keys = [
        "MODULES",
        "MODULE_SIG",
        "KALLSYMS",
        "KALLSYMS_ALL",
        "BPF",
        "DEBUG_INFO_BTF",
        "RUST",
        "RUST_IS_AVAILABLE",
        "LIVEPATCH",
    ];
    for key in &cfg_keys {
        let label = format!("  CONFIG_{key}");
        match config_text.and_then(|t| config::config_is_set(t, key)) {
            Some(true) => {
                let val = config_val_label(config_text, key);
                let _ = writeln!(out, "  {label:.<36} {val}");
            }
            Some(false) => {
                let _ = writeln!(out, "  {label:.<36} not set");
            }
            None => {
                let _ = writeln!(out, "  {label:.<36} Unknown");
            }
        }
    }
    let _ = writeln!(out);

    // Module environment
    let _ = writeln!(out, "Module Environment");
    print_kv(
        &mut out,
        "  Running kernel",
        mod_info.running_kernel.as_deref(),
    );
    print_kv(
        &mut out,
        "  Installed headers",
        mod_info.installed_header_version.as_deref(),
    );
    print_kv(
        &mut out,
        "  Modules directory",
        mod_info.modules_dir.as_deref(),
    );
    print_bool(&mut out, "  Build directory", mod_info.build_dir_present);
    print_bool(&mut out, "  Headers", mod_info.headers_available);
    print_bool_opt(&mut out, "  Module signing", mod_info.signing_enabled);
    print_bool(&mut out, "  Signing required", mod_info.signing_required);
    let _ = writeln!(out);

    // Debug information
    let _ = writeln!(out, "Debug Information");
    print_bool(&mut out, "  BTF", debug.btf_available);
    print_bool(&mut out, "  DWARF", debug.dwarf_available);
    let _ = writeln!(out);

    // Symbol information
    let _ = writeln!(out, "Symbol Information");
    if ks_info.exists {
        let label = "  /proc/kallsyms";
        match (ks_info.readable, ks_info.root_only) {
            (true, true) => {
                let _ = writeln!(out, "{label} : present, readable (root only)");
            }
            (true, false) => {
                let _ = writeln!(out, "{label} : present, readable");
            }
            (false, _) => {
                let _ = writeln!(out, "{label} : present, permission denied");
            }
        }
    } else {
        let _ = writeln!(out, "  /proc/kallsyms : not found");
    }
    let _ = writeln!(out);

    // ---- capability summary -----------------------------------------------

    let r4l_known = rust_cfg.is_some() || rust_avail.is_some();
    let r4l_ok = rust_cfg == Some(true) || rust_avail == Some(true);

    let modules_known = config_text
        .and_then(|t| config::config_is_set(t, "MODULES"))
        .is_some();
    let modules_ok = config_text
        .and_then(|t| config::config_is_set(t, "MODULES"))
        .unwrap_or(false)
        && mod_info.headers_available;

    let modsig_known = mod_info.signing_enabled.is_some();
    let modsig_ok = mod_info.signing_enabled == Some(true);

    let btf_known = config_text
        .and_then(|t| config::config_is_set(t, "DEBUG_INFO_BTF"))
        .is_some();
    let btf_ok = config_text
        .and_then(|t| config::config_is_set(t, "DEBUG_INFO_BTF"))
        .unwrap_or(false)
        && debug.btf_available;

    let livepatch_known = config_text
        .and_then(|t| config::config_is_set(t, "LIVEPATCH"))
        .is_some();
    let livepatch_ok = config_text
        .and_then(|t| config::config_is_set(t, "LIVEPATCH"))
        .unwrap_or(false)
        && modules_ok;

    let ks_ok = ks_info.exists && ks_info.readable;
    let ks_known = ks_info.exists;

    let _ = writeln!(out, "Kernel Capability Summary");
    let _ = writeln!(out);
    print_tri_check(&mut out, "Rust for Linux", r4l_ok, r4l_known);
    print_tri_check(&mut out, "Modules", modules_ok, modules_known);
    print_tri_check(&mut out, "Module Signing", modsig_ok, modsig_known);
    print_tri_check(&mut out, "BTF", btf_ok, btf_known);
    print_tri_check(&mut out, "Livepatch", livepatch_ok, livepatch_known);
    print_tri_check(&mut out, "Kallsyms", ks_ok, ks_known);
    let _ = writeln!(out);

    let _ = writeln!(out, "Suitable for:");
    print_check(
        &mut out,
        "module development",
        modules_ok && kernel::compiler_available(),
    );
    print_check(&mut out, "symbol analysis", ks_ok);
    print_check(
        &mut out,
        "live patching",
        livepatch_ok && ks_ok && modules_ok,
    );

    Ok(())
}

// ---- helpers ---------------------------------------------------------------

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

fn print_check(out: &mut io::StdoutLock<'_>, label: &str, ok: bool) {
    let mark = if ok { "✔" } else { "✘" };
    let _ = writeln!(out, "  {mark} {label}");
}

/// Three-state check: ✔ available, ✘ unavailable, ? unknown.
fn print_tri_check(out: &mut io::StdoutLock<'_>, label: &str, ok: bool, known: bool) {
    if !known {
        let _ = writeln!(out, "  ? {label}");
    } else if ok {
        let _ = writeln!(out, "  ✔ {label}");
    } else {
        let _ = writeln!(out, "  ✘ {label}");
    }
}

fn config_val_label<'a>(config_text: Option<&str>, key: &str) -> &'a str {
    match config_text.and_then(|t| config::config_is_set(t, key)) {
        Some(true) => {
            if let Some(t) = config_text {
                for line in t.lines() {
                    let trimmed = line.trim();
                    if trimmed == format!("CONFIG_{key}=m") {
                        return "m";
                    }
                    if trimmed == format!("CONFIG_{key}=y") {
                        return "y";
                    }
                }
            }
            "y"
        }
        _ => "y",
    }
}

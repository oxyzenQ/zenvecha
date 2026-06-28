// Copyright (C) 2026 rezky_nightky
// SPDX-License-Identifier: GPL-3.0-only

//! Inspect command — read-only kernel capability discovery.

use std::io::{self, Write};

use crate::system::{btf, config, kallsyms, kernel, modules, rust};

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

    // Kernel identity
    let _ = writeln!(out, "Zenvecha Inspect");
    let _ = writeln!(out);
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
        let val = config_text.map_or(config::ConfigValue::Missing, |t| {
            config::config_value(t, key)
        });
        let _ = writeln!(out, "  {label:.<36} {}", val.label());
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

    // Capability summary
    let r4l_known = rust_cfg.is_known() || rust_avail.is_known();
    let r4l_ok = rust_cfg.is_enabled() && rust_avail.is_enabled();

    let modsig_known = mod_info.signing_enabled.is_some();
    let modsig_ok = mod_info.signing_enabled == Some(true);

    let mod_known = config_text
        .map(|t| config::config_value(t, "MODULES").is_known())
        .unwrap_or(false);
    let mod_ok = config_text
        .map(|t| config::config_value(t, "MODULES").is_enabled())
        .unwrap_or(false)
        && mod_info.headers_available;

    let btf_known = config_text
        .map(|t| config::config_value(t, "DEBUG_INFO_BTF").is_known())
        .unwrap_or(false);
    let btf_ok = config_text
        .map(|t| config::config_value(t, "DEBUG_INFO_BTF").is_enabled())
        .unwrap_or(false)
        && debug.btf_available;

    let lp_known = config_text
        .map(|t| config::config_value(t, "LIVEPATCH").is_known())
        .unwrap_or(false);
    let lp_ok = config_text
        .map(|t| config::config_value(t, "LIVEPATCH").is_enabled())
        .unwrap_or(false)
        && mod_ok;

    let ks_ok = ks_info.exists && ks_info.readable;

    let _ = writeln!(out, "Kernel Capability Summary");
    let _ = writeln!(out);
    print_tri(&mut out, "Rust for Linux", r4l_ok, r4l_known);
    print_tri(&mut out, "Modules", mod_ok, mod_known);
    print_tri(&mut out, "Module Signing", modsig_ok, modsig_known);
    print_tri(&mut out, "BTF", btf_ok, btf_known);
    print_tri(&mut out, "Livepatch", lp_ok, lp_known);
    print_tri(&mut out, "Kallsyms", ks_ok, true);
    let _ = writeln!(out);

    let _ = writeln!(out, "Suitable for:");
    print_check(
        &mut out,
        "module development",
        mod_ok && kernel::compiler_available(),
    );
    print_check(&mut out, "symbol analysis", ks_ok);
    print_check(&mut out, "live patching", lp_ok && ks_ok && mod_ok);

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

fn print_tri(out: &mut io::StdoutLock<'_>, label: &str, ok: bool, known: bool) {
    if !known {
        let _ = writeln!(out, "  ? {label}");
    } else if ok {
        let _ = writeln!(out, "  ✔ {label}");
    } else {
        let _ = writeln!(out, "  ✘ {label}");
    }
}

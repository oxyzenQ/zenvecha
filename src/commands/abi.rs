// Copyright (C) 2026 rezky_nightky
// SPDX-License-Identifier: GPL-3.0-only

//! ABI command — kernel compatibility intelligence.
//!
//! Analyzes ABI, symbols, compiler compatibility, and module loader.
//! Read-only, streaming, never loads large files into memory.

use std::io::{self, Write};

use crate::system::{
    abi, compiler, config, kernel, moduleinfo, modules, recommend, scoring, symbols, toolchain,
};

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let stdout = io::stdout();
    let mut out = stdout.lock();

    let release = kernel::kernel_release();
    let tools = toolchain::inspect_toolchain();
    let (config_text, _) = config::read_kernel_config().unzip();
    let cfg = config_text.as_deref();

    let abi_info = abi::inspect_abi(cfg);
    let sym_info = symbols::inspect_symbols(release.as_deref());
    let comp_abi = compiler::compare_compilers(&tools.rustc);
    let mod_info = modules::inspect_modules(cfg);
    let loader = moduleinfo::inspect_loader(cfg);

    // ---- Kernel ABI -------------------------------------------------------

    let _ = writeln!(out, "Zenvecha ABI");
    let _ = writeln!(out);
    let _ = writeln!(out, "Kernel ABI");
    print_kv(&mut out, "  utsrelease", abi_info.utsrelease.as_deref());
    print_kv(&mut out, "  vermagic", abi_info.vermagic.as_deref());
    if let Some(ref layout) = abi_info.module_layout_version {
        let _ = writeln!(out, "  module layout version : present");
        let _ = writeln!(out, "    {layout}");
    } else {
        let _ = writeln!(out, "  module layout version : not available");
    }
    print_kv(
        &mut out,
        "  compiler string",
        abi_info.compiler_string.as_deref(),
    );
    let _ = writeln!(
        out,
        "  module compression  : {}",
        abi_info.module_compression
    );
    let _ = writeln!(out, "  module signing      : {}", abi_info.module_signing);
    let _ = writeln!(out);

    // ---- System.map -------------------------------------------------------

    let _ = writeln!(out, "System.map");
    match &sym_info.system_map_path {
        Some(p) => {
            let _ = writeln!(out, "  Path   : {p}");
            if let Some(sz) = sym_info.system_map_size {
                let _ = writeln!(out, "  Size   : {}", human_size(sz));
            }
            let _ = writeln!(out, "  Status : available");
        }
        None => {
            let _ = writeln!(out, "  Status : not installed (optional)");
        }
    }
    let _ = writeln!(out);

    // ---- Module.symvers ---------------------------------------------------

    let _ = writeln!(out, "Module.symvers");
    match &sym_info.module_symvers_path {
        Some(p) => {
            let _ = writeln!(out, "  Path          : {p}");
            if let Some(n) = sym_info.symvers_crc_count {
                let _ = writeln!(out, "  CRC count     : {n}");
            }
            if let Some(sz) = sym_info.symvers_size {
                let _ = writeln!(out, "  File size     : {}", human_size(sz));
            }
            if let Some(ref ts) = sym_info.symvers_modified {
                let _ = writeln!(out, "  Last modified : {ts}");
            }
        }
        None => {
            let _ = writeln!(out, "  Status : not found");
        }
    }
    let _ = writeln!(out);

    // ---- Kernel Symbols ---------------------------------------------------

    let _ = writeln!(out, "Kernel Symbols");
    let _ = writeln!(
        out,
        "  /proc/kallsyms : {}",
        sym_info.kallsyms_status.label()
    );
    if let Some(n) = sym_info.symbol_count {
        let _ = writeln!(out, "  Symbol count   : {n}");
    }
    let _ = writeln!(out);

    // ---- Module Loader ----------------------------------------------------

    let _ = writeln!(out, "Module Loader");
    let _ = writeln!(out, "  Loaded modules   : {}", loader.loaded_count);
    let _ = writeln!(
        out,
        "  Signed supported : {}",
        if loader.signed_supported { "yes" } else { "no" }
    );
    let _ = writeln!(out, "  Compression      : {}", loader.compression);
    let _ = writeln!(
        out,
        "  Livepatch        : {}",
        if loader.livepatch_enabled {
            "enabled"
        } else {
            "not enabled"
        }
    );
    let _ = writeln!(out);

    // ---- Compiler ABI -----------------------------------------------------

    let _ = writeln!(out, "Compiler ABI");
    print_kv(
        &mut out,
        "  Kernel compiler",
        comp_abi.kernel_compiler.as_deref(),
    );
    print_kv(
        &mut out,
        "  Installed gcc",
        comp_abi.installed_gcc.as_deref(),
    );
    print_kv(
        &mut out,
        "  Installed clang",
        comp_abi.installed_clang.as_deref(),
    );
    print_kv(
        &mut out,
        "  Installed rustc",
        comp_abi.installed_rustc.as_deref(),
    );
    let _ = writeln!(
        out,
        "  gcc compatibility   : {}",
        comp_abi.gcc_compat.label()
    );
    let _ = writeln!(
        out,
        "  clang compatibility : {}",
        comp_abi.clang_compat.label()
    );
    let _ = writeln!(
        out,
        "  rustc compatibility : {}",
        comp_abi.rustc_compat.label()
    );
    let _ = writeln!(out);

    // ---- Star Score -------------------------------------------------------

    let scores = scoring::compute();
    let _ = writeln!(out, "Compatibility Summary");
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
        build_dir_present: true, // ABI doesn't strictly need build dir
        source_dir_present: sym_info.system_map_path.is_some(),
        config_rust: cfg.map_or(config::ConfigValue::Missing, |t| {
            config::config_value(t, "RUST")
        }),
        config_rust_available: cfg.map_or(config::ConfigValue::Missing, |t| {
            config::config_value(t, "RUST_IS_AVAILABLE")
        }),
        config_modules: cfg.map_or(config::ConfigValue::Missing, |t| {
            config::config_value(t, "MODULES")
        }),
        config_btf: cfg.map_or(config::ConfigValue::Missing, |t| {
            config::config_value(t, "DEBUG_INFO_BTF")
        }),
        btf_available: true, // not critical for ABI
        signing_required: mod_info.signing_required,
        signing_enabled: mod_info.signing_enabled == Some(true),
        debugfs_ok: mount_ok("/sys/kernel/debug"),
        tracefs_ok: mount_ok("/sys/kernel/tracing"),
        release: release.as_deref(),
        headers_ver: mod_info.installed_header_version.as_deref(),
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

fn human_size(bytes: u64) -> String {
    if bytes >= 1_048_576 {
        format!("{:.1} MB", bytes as f64 / 1_048_576.0)
    } else if bytes >= 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{bytes} B")
    }
}

// Copyright (C) 2026 rezky_nightky
// SPDX-License-Identifier: GPL-3.0-only

//! Report context — single-pass inspection, shared across formatters.
//!
//! All expensive system probes run once here. Formatters receive
//! immutable references. No duplicated I/O.

use crate::system::{
    abi::AbiInfo, btf::DebugInfo, buildenv::BuildEnvInfo, compiler::CompilerAbi,
    config::ConfigValue, kallsyms::KallsymsInfo, moduleinfo::ModuleLoaderInfo, modules::ModuleInfo,
    scoring::CategoryScore, symbols::SymbolInfo, toolchain::ToolchainInfo,
};

/// All inspection results gathered in one pass.
pub struct ReportContext {
    pub release: Option<String>,
    pub arch: Option<String>,
    pub distro: Option<String>,
    pub tools: ToolchainInfo,
    pub bld: BuildEnvInfo,
    pub mod_info: ModuleInfo,
    pub ks_info: KallsymsInfo,
    pub dbg: DebugInfo,
    pub rust_cfg: ConfigValue,
    pub rust_avail: ConfigValue,
    pub config_text: Option<String>,
    pub sym_info: SymbolInfo,
    pub abi_info: AbiInfo,
    pub loader: ModuleLoaderInfo,
    pub comp_abi: CompilerAbi,
    pub scores: Vec<CategoryScore>,
    pub overall: &'static str,
    pub debugfs_ok: bool,
    pub tracefs_ok: bool,
}

/// Run all inspections once. Returns the immutable context.
pub fn gather() -> ReportContext {
    let release = crate::system::kernel::kernel_release();
    let arch = crate::system::kernel::architecture();
    let distro = crate::system::kernel::detect_distro();
    let tools = crate::system::toolchain::inspect_toolchain();
    let bld = crate::system::buildenv::inspect_build_env();
    let (config_text, _) = crate::system::config::read_kernel_config().unzip();
    let cfg = config_text.as_deref();
    let mod_info = crate::system::modules::inspect_modules(cfg);
    let ks_info = crate::system::kallsyms::inspect_kallsyms();
    let dbg = crate::system::btf::inspect_debug();
    let rust_cfg = cfg.map_or(ConfigValue::Missing, |t| {
        crate::system::config::config_value(t, "RUST")
    });
    let rust_avail = cfg.map_or(ConfigValue::Missing, |t| {
        crate::system::config::config_value(t, "RUST_IS_AVAILABLE")
    });
    let sym_info = crate::system::symbols::inspect_symbols(release.as_deref());
    let abi_info = crate::system::abi::inspect_abi(cfg);
    let loader = crate::system::moduleinfo::inspect_loader(cfg);
    let comp_abi = crate::system::compiler::compare_compilers(&tools.rustc);
    let scores = crate::system::scoring::compute();
    let overall = crate::system::scoring::overall_rating(&scores);
    let debugfs_ok = mount_ok("/sys/kernel/debug");
    let tracefs_ok = mount_ok("/sys/kernel/tracing");

    ReportContext {
        release,
        arch,
        distro,
        tools,
        bld,
        mod_info,
        ks_info,
        dbg,
        rust_cfg,
        rust_avail,
        config_text,
        sym_info,
        abi_info,
        loader,
        comp_abi,
        scores,
        overall,
        debugfs_ok,
        tracefs_ok,
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

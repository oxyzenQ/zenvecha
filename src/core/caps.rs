// Copyright (C) 2026 rezky_nightky
// SPDX-License-Identifier: GPL-3.0-only

//! Capability implementations.
//!
//! Each struct wraps an existing system/ probe. Capabilities never
//! print, score, or recommend — they only detect and return Evidence.
//!
//! To add a new capability:
//!   1. Define a struct
//!   2. Implement the `Capability` trait
//!   3. Register in `registry::register_all()`

use super::capability::Capability;
use super::evidence::{Confidence, Evidence, EvidenceValue, ProbeStatus};
use crate::system::{self, config::ConfigValue};

/* -------------------------------------------------------------------------- */
/*  Kernel identity                                                           */
/* -------------------------------------------------------------------------- */

pub struct KernelRelease;
impl Capability for KernelRelease {
    fn id(&self) -> &'static str {
        "kernel.release"
    }
    fn label(&self) -> &'static str {
        "Kernel Release"
    }
    fn probe(&self) -> Evidence {
        let r = system::kernel::kernel_release();
        match r {
            Some(v) => Evidence::present(self.id(), EvidenceValue::Text(Some(v))),
            None => Evidence::missing(self.id(), EvidenceValue::Text(None)),
        }
    }
}

pub struct KernelArchitecture;
impl Capability for KernelArchitecture {
    fn id(&self) -> &'static str {
        "kernel.arch"
    }
    fn label(&self) -> &'static str {
        "Architecture"
    }
    fn probe(&self) -> Evidence {
        let a = system::kernel::architecture();
        Evidence::present(self.id(), EvidenceValue::Text(a))
    }
}

pub struct KernelDistro;
impl Capability for KernelDistro {
    fn id(&self) -> &'static str {
        "kernel.distro"
    }
    fn label(&self) -> &'static str {
        "Distribution"
    }
    fn probe(&self) -> Evidence {
        let d = system::kernel::detect_distro();
        Evidence::present(self.id(), EvidenceValue::Text(d))
    }
}

pub struct CompilerVersion;
impl Capability for CompilerVersion {
    fn id(&self) -> &'static str {
        "kernel.compiler"
    }
    fn label(&self) -> &'static str {
        "Compiler"
    }
    fn probe(&self) -> Evidence {
        let c = system::kernel::compiler_version();
        if let Some(v) = c {
            Evidence::present(self.id(), EvidenceValue::Text(Some(v)))
        } else {
            Evidence::missing(self.id(), EvidenceValue::Text(None))
        }
    }
}

pub struct CompilerAbi;
impl Capability for CompilerAbi {
    fn id(&self) -> &'static str {
        "compiler.abi"
    }
    fn label(&self) -> &'static str {
        "Compiler ABI"
    }
    fn probe(&self) -> Evidence {
        let tools = system::toolchain::inspect_toolchain();
        let abi = system::compiler::compare_compilers(&tools.rustc);
        // Collect relevant fields
        let kernel_comp = abi.kernel_compiler.unwrap_or_else(|| "Unknown".into());
        let confidence = match abi.gcc_compat {
            system::compiler::CompilerCompat::Compatible => Confidence::High,
            system::compiler::CompilerCompat::Probably => Confidence::Medium,
            system::compiler::CompilerCompat::Unknown => Confidence::Low,
            system::compiler::CompilerCompat::NotCompatible => Confidence::Low,
        };
        Evidence::present(self.id(), EvidenceValue::Literal(kernel_comp))
            .with_confidence(confidence)
    }
}

/* -------------------------------------------------------------------------- */
/*  Configuration                                                             */
/* -------------------------------------------------------------------------- */

/// Helper — reads config once and caches. Capabilities using config
/// share the same read via lazy static-like pattern (each call reads
/// from /proc/config.gz or /boot/config-* — cheap, idempotent).
fn read_config() -> Option<String> {
    system::config::read_kernel_config().map(|(text, _)| text)
}

fn cfg_val(key: &str) -> ConfigValue {
    read_config().as_deref().map_or(ConfigValue::Missing, |t| {
        system::config::config_value(t, key)
    })
}

fn cfg_present() -> bool {
    read_config().is_some()
}

pub struct ConfigSource;
impl Capability for ConfigSource {
    fn id(&self) -> &'static str {
        "config.source"
    }
    fn label(&self) -> &'static str {
        "Config Source"
    }
    fn probe(&self) -> Evidence {
        let (_, src) = system::config::read_kernel_config().unzip();
        match src {
            Some(s) => Evidence::present(self.id(), EvidenceValue::Text(Some(s))),
            None => Evidence::missing(self.id(), EvidenceValue::Text(None)),
        }
    }
}

macro_rules! config_cap {
    ($name:ident, $id:literal, $label:literal, $key:literal) => {
        pub struct $name;
        impl Capability for $name {
            fn id(&self) -> &'static str {
                $id
            }
            fn label(&self) -> &'static str {
                $label
            }
            fn probe(&self) -> Evidence {
                let v = cfg_val($key);
                let status = if v == ConfigValue::Missing && !cfg_present() {
                    ProbeStatus::Missing
                } else {
                    ProbeStatus::Present
                };
                Evidence {
                    id: self.id(),
                    status,
                    confidence: Confidence::High,
                    value: EvidenceValue::Config(v),
                }
            }
        }
    };
}

config_cap!(ConfigModules, "config.MODULES", "CONFIG_MODULES", "MODULES");
config_cap!(
    ConfigModuleSig,
    "config.MODULE_SIG",
    "CONFIG_MODULE_SIG",
    "MODULE_SIG"
);
config_cap!(
    ConfigKallsyms,
    "config.KALLSYMS",
    "CONFIG_KALLSYMS",
    "KALLSYMS"
);
config_cap!(
    ConfigKallsymsAll,
    "config.KALLSYMS_ALL",
    "CONFIG_KALLSYMS_ALL",
    "KALLSYMS_ALL"
);
config_cap!(ConfigBpf, "config.BPF", "CONFIG_BPF", "BPF");
config_cap!(
    ConfigDebugInfoBtf,
    "config.DEBUG_INFO_BTF",
    "CONFIG_DEBUG_INFO_BTF",
    "DEBUG_INFO_BTF"
);
config_cap!(ConfigRust, "config.RUST", "CONFIG_RUST", "RUST");
config_cap!(
    ConfigRustAvailable,
    "config.RUST_IS_AVAILABLE",
    "CONFIG_RUST_IS_AVAILABLE",
    "RUST_IS_AVAILABLE"
);
config_cap!(
    ConfigLivepatch,
    "config.LIVEPATCH",
    "CONFIG_LIVEPATCH",
    "LIVEPATCH"
);

/* -------------------------------------------------------------------------- */
/*  Module environment                                                         */
/* -------------------------------------------------------------------------- */

pub struct ModuleSupport;
impl Capability for ModuleSupport {
    fn id(&self) -> &'static str {
        "modules.support"
    }
    fn label(&self) -> &'static str {
        "Module Support"
    }
    fn probe(&self) -> Evidence {
        let cfg = read_config();
        let info = system::modules::inspect_modules(cfg.as_deref());
        let has_mod = cfg_val("MODULES").is_enabled();
        let dev_ok =
            has_mod && info.headers_available && crate::system::kernel::compiler_available();
        // Return a literal that encodes both capability and dev readiness
        let desc = match (has_mod, dev_ok) {
            (true, true) => "supported with development environment",
            (true, false) => "supported, development not ready",
            (false, _) => "not supported",
        };
        Evidence::present(self.id(), EvidenceValue::Literal(desc.into()))
    }
}

pub struct ModuleSigning;
impl Capability for ModuleSigning {
    fn id(&self) -> &'static str {
        "modules.signing"
    }
    fn label(&self) -> &'static str {
        "Module Signing"
    }
    fn probe(&self) -> Evidence {
        let cfg = read_config();
        let info = system::modules::inspect_modules(cfg.as_deref());
        Evidence::present(
            self.id(),
            EvidenceValue::Bool(info.signing_enabled.unwrap_or(false)),
        )
    }
}

pub struct ModuleLoader;
impl Capability for ModuleLoader {
    fn id(&self) -> &'static str {
        "modules.loader"
    }
    fn label(&self) -> &'static str {
        "Module Loader"
    }
    fn probe(&self) -> Evidence {
        let cfg = read_config();
        let loader = system::moduleinfo::inspect_loader(cfg.as_deref());
        Evidence::present(
            self.id(),
            EvidenceValue::Literal(format!(
                "loaded={} signed={} compression={} livepatch={}",
                loader.loaded_count,
                if loader.signed_supported { "yes" } else { "no" },
                loader.compression,
                if loader.livepatch_enabled {
                    "yes"
                } else {
                    "no"
                },
            )),
        )
    }
}

/* -------------------------------------------------------------------------- */
/*  Symbols                                                                   */
/* -------------------------------------------------------------------------- */

pub struct KallsymsInfo;
impl Capability for KallsymsInfo {
    fn id(&self) -> &'static str {
        "symbols.kallsyms"
    }
    fn label(&self) -> &'static str {
        "Kallsyms"
    }
    fn probe(&self) -> Evidence {
        let ks = system::kallsyms::inspect_kallsyms();
        let status = if ks.exists && ks.readable {
            if ks.root_only {
                "readable (root)"
            } else {
                "readable"
            }
        } else if ks.exists {
            "permission denied"
        } else {
            "hidden"
        };
        Evidence::present(self.id(), EvidenceValue::Status(status))
    }
}

pub struct SymbolCount;
impl Capability for SymbolCount {
    fn id(&self) -> &'static str {
        "symbols.count"
    }
    fn label(&self) -> &'static str {
        "Symbol Count"
    }
    fn probe(&self) -> Evidence {
        let release = system::kernel::kernel_release();
        let info = system::symbols::inspect_symbols(release.as_deref());
        Evidence::present(
            self.id(),
            EvidenceValue::Count(info.symbol_count.unwrap_or(0)),
        )
    }
}

pub struct VmlinuxInfo;
impl Capability for VmlinuxInfo {
    fn id(&self) -> &'static str {
        "symbols.vmlinux"
    }
    fn label(&self) -> &'static str {
        "VMLinux"
    }
    fn probe(&self) -> Evidence {
        let release = system::kernel::kernel_release();
        let info = system::symbols::inspect_symbols(release.as_deref());
        match info.vmlinux_path {
            Some(p) => {
                let sz = info
                    .vmlinux_size
                    .map(|s| format!(" ({})", crate::system::capabilities::human_size(s)))
                    .unwrap_or_default();
                let bid = info
                    .vmlinux_build_id
                    .map(|b| format!(" BuildID={b}"))
                    .unwrap_or_default();
                Evidence::present(self.id(), EvidenceValue::Literal(format!("{p}{sz}{bid}")))
            }
            None => Evidence::missing(self.id(), EvidenceValue::Text(None)),
        }
    }
}

pub struct ModuleSymvers;
impl Capability for ModuleSymvers {
    fn id(&self) -> &'static str {
        "symbols.symvers"
    }
    fn label(&self) -> &'static str {
        "Module.symvers"
    }
    fn probe(&self) -> Evidence {
        let release = system::kernel::kernel_release();
        let info = system::symbols::inspect_symbols(release.as_deref());
        match info.module_symvers_path {
            Some(p) => Evidence::present(self.id(), EvidenceValue::Text(Some(p))),
            None => Evidence::missing(self.id(), EvidenceValue::Text(None)),
        }
    }
}

/* -------------------------------------------------------------------------- */
/*  Debug                                                                     */
/* -------------------------------------------------------------------------- */

pub struct DebugBtf;
impl Capability for DebugBtf {
    fn id(&self) -> &'static str {
        "debug.btf"
    }
    fn label(&self) -> &'static str {
        "BTF"
    }
    fn probe(&self) -> Evidence {
        let dbg = system::btf::inspect_debug();
        Evidence::present(self.id(), EvidenceValue::Bool(dbg.btf_available))
    }
}

pub struct DebugDwarf;
impl Capability for DebugDwarf {
    fn id(&self) -> &'static str {
        "debug.dwarf"
    }
    fn label(&self) -> &'static str {
        "DWARF"
    }
    fn probe(&self) -> Evidence {
        let dbg = system::btf::inspect_debug();
        Evidence::present(self.id(), EvidenceValue::Bool(dbg.dwarf_available))
    }
}

/* -------------------------------------------------------------------------- */
/*  ABI                                                                       */
/* -------------------------------------------------------------------------- */

pub struct AbiInfo;
impl Capability for AbiInfo {
    fn id(&self) -> &'static str {
        "abi.info"
    }
    fn label(&self) -> &'static str {
        "ABI Info"
    }
    fn probe(&self) -> Evidence {
        let cfg = read_config();
        let abi = system::abi::inspect_abi(cfg.as_deref());
        let value = format!(
            "utsrelease={} vermagic={} layout={} compression={}",
            abi.utsrelease.as_deref().unwrap_or("Unknown"),
            abi.vermagic.as_deref().unwrap_or("Unknown"),
            abi.module_layout_version.as_deref().unwrap_or("Unknown"),
            abi.module_compression,
        );
        Evidence::present(self.id(), EvidenceValue::Literal(value))
    }
}

/* -------------------------------------------------------------------------- */
/*  Toolchain                                                                 */
/* -------------------------------------------------------------------------- */

fn tool_field(tools: &system::toolchain::ToolchainInfo, field: &str) -> EvidenceValue {
    match field {
        "rustc" => EvidenceValue::Bool(tools.rustc.is_some()),
        "bindgen" => EvidenceValue::Bool(tools.bindgen.is_some()),
        "llvm" => EvidenceValue::Bool(tools.llvm_version.is_some()),
        _ => EvidenceValue::Bool(false),
    }
}

macro_rules! tool_cap {
    ($name:ident, $id:literal, $label:literal, $field:literal) => {
        pub struct $name;
        impl Capability for $name {
            fn id(&self) -> &'static str {
                $id
            }
            fn label(&self) -> &'static str {
                $label
            }
            fn probe(&self) -> Evidence {
                let tools = system::toolchain::inspect_toolchain();
                Evidence::present(self.id(), tool_field(&tools, $field))
            }
        }
    };
}

tool_cap!(RustcInstalled, "toolchain.rustc", "rustc", "rustc");
tool_cap!(BindgenInstalled, "toolchain.bindgen", "bindgen", "bindgen");
tool_cap!(LlvmInstalled, "toolchain.llvm", "llvm", "llvm");

pub struct MakeInstalled;
impl Capability for MakeInstalled {
    fn id(&self) -> &'static str {
        "toolchain.make"
    }
    fn label(&self) -> &'static str {
        "make"
    }
    fn probe(&self) -> Evidence {
        let ok = which("make");
        Evidence::present(self.id(), EvidenceValue::Bool(ok))
    }
}

pub struct GccInstalled;
impl Capability for GccInstalled {
    fn id(&self) -> &'static str {
        "toolchain.gcc"
    }
    fn label(&self) -> &'static str {
        "gcc"
    }
    fn probe(&self) -> Evidence {
        let ok = crate::system::kernel::compiler_available();
        Evidence::present(self.id(), EvidenceValue::Bool(ok))
    }
}

fn which(cmd: &str) -> bool {
    std::process::Command::new("which")
        .arg(cmd)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/* -------------------------------------------------------------------------- */
/*  Build environment                                                         */
/* -------------------------------------------------------------------------- */

pub struct HeaderIntegrity;
impl Capability for HeaderIntegrity {
    fn id(&self) -> &'static str {
        "build.headers"
    }
    fn label(&self) -> &'static str {
        "Header Integrity"
    }
    fn probe(&self) -> Evidence {
        let bld = system::buildenv::inspect_build_env();
        Evidence::present(self.id(), EvidenceValue::Status(bld.header_status.label()))
    }
}

pub struct BuildDirectory;
impl Capability for BuildDirectory {
    fn id(&self) -> &'static str {
        "build.dir"
    }
    fn label(&self) -> &'static str {
        "Build Directory"
    }
    fn probe(&self) -> Evidence {
        let bld = system::buildenv::inspect_build_env();
        Evidence::present(
            self.id(),
            EvidenceValue::Text(bld.build_dir.map(|d| d.to_string())),
        )
    }
}

pub struct SourceDirectory;
impl Capability for SourceDirectory {
    fn id(&self) -> &'static str {
        "build.source"
    }
    fn label(&self) -> &'static str {
        "Source Directory"
    }
    fn probe(&self) -> Evidence {
        let bld = system::buildenv::inspect_build_env();
        Evidence::present(
            self.id(),
            EvidenceValue::Text(bld.source_dir.map(|d| d.to_string())),
        )
    }
}

pub struct CompileCommands;
impl Capability for CompileCommands {
    fn id(&self) -> &'static str {
        "build.compile_commands"
    }
    fn label(&self) -> &'static str {
        "compile_commands.json"
    }
    fn probe(&self) -> Evidence {
        let bld = system::buildenv::inspect_build_env();
        Evidence::present(self.id(), EvidenceValue::Bool(bld.compile_commands))
    }
}

/* -------------------------------------------------------------------------- */
/*  Filesystems                                                               */
/* -------------------------------------------------------------------------- */

pub struct DebugfsMounted;
impl Capability for DebugfsMounted {
    fn id(&self) -> &'static str {
        "fs.debugfs"
    }
    fn label(&self) -> &'static str {
        "debugfs"
    }
    fn probe(&self) -> Evidence {
        Evidence::present(
            self.id(),
            EvidenceValue::Bool(mount_ok("/sys/kernel/debug")),
        )
    }
}

pub struct TracefsMounted;
impl Capability for TracefsMounted {
    fn id(&self) -> &'static str {
        "fs.tracefs"
    }
    fn label(&self) -> &'static str {
        "tracefs"
    }
    fn probe(&self) -> Evidence {
        Evidence::present(
            self.id(),
            EvidenceValue::Bool(mount_ok("/sys/kernel/tracing")),
        )
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

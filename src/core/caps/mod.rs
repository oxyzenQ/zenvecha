// Copyright (C) 2026 rezky_nightky
// SPDX-License-Identifier: GPL-3.0-only

//! Capability domain modules.
//!
//! Each submodule owns one domain of capability probes.
//! Capabilities never print, score, or recommend — they only detect
//! and return Evidence.
//!
//! To add a new capability:
//!   1. Implement in the correct domain module
//!   2. Re-export from this mod.rs
//!   3. Register in `capability::register_all()`

pub mod abi;
pub mod buildenv;
pub mod config;
pub mod debug;
pub mod fs;
pub mod kernel;
pub mod modules;
pub mod symbols;
pub mod toolchain;

// Re-export all capability structs for backward-compatible registration.
pub use abi::AbiInfo;
pub use buildenv::{BuildDirectory, CompileCommands, HeaderIntegrity, SourceDirectory};
pub use config::{
    ConfigBpf, ConfigDebugInfoBtf, ConfigKallsyms, ConfigKallsymsAll, ConfigLivepatch,
    ConfigModuleSig, ConfigModules, ConfigRust, ConfigRustAvailable, ConfigSource,
};
pub use debug::{DebugBtf, DebugDwarf};
pub use fs::{DebugfsMounted, TracefsMounted};
pub use kernel::{CompilerAbi, CompilerVersion, KernelArchitecture, KernelDistro, KernelRelease};
pub use modules::{ModuleLoader, ModuleSigning, ModuleSupport};
pub use symbols::{KallsymsInfo, ModuleSymvers, SymbolCount, VmlinuxInfo};
pub use toolchain::{BindgenInstalled, GccInstalled, LlvmInstalled, MakeInstalled, RustcInstalled};

// Shared utilities used across multiple capability domains.
use crate::system::config::ConfigValue;

pub(crate) fn read_config() -> Option<String> {
    crate::system::config::read_kernel_config().map(|(text, _)| text)
}

pub(crate) fn cfg_val(key: &str) -> ConfigValue {
    read_config().as_deref().map_or(ConfigValue::Missing, |t| {
        crate::system::config::config_value(t, key)
    })
}

pub(crate) fn cfg_present() -> bool {
    read_config().is_some()
}

pub(crate) fn which(cmd: &str) -> bool {
    std::process::Command::new("which")
        .arg(cmd)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

pub(crate) fn mount_ok(path: &str) -> bool {
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

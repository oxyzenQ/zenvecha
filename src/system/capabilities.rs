// Copyright (C) 2026 rezky_nightky
// SPDX-License-Identifier: GPL-3.0-only

//! Capability matrix, risks, and facts for the unified report.
//!
//! Pure functions — receive context, return structured data.

use crate::system::config::ConfigValue;
use crate::system::report::ReportContext;

/// One row in the capability matrix.
pub struct CapabilityRow {
    pub name: &'static str,
    pub status: CapabilityStatus,
    pub evidence: String,
}

#[derive(Clone, Copy)]
pub enum CapabilityStatus {
    Enabled,
    Disabled,
    Unknown,
}

impl CapabilityStatus {
    pub fn label(self) -> &'static str {
        match self {
            CapabilityStatus::Enabled => "Enabled",
            CapabilityStatus::Disabled => "Disabled",
            CapabilityStatus::Unknown => "Unknown",
        }
    }
}

/// Build the capability matrix from report context.
pub fn capability_matrix(ctx: &ReportContext) -> Vec<CapabilityRow> {
    let cv = |k: &str| {
        ctx.config_text
            .as_deref()
            .map_or(ConfigValue::Missing, |t| {
                crate::system::config::config_value(t, k)
            })
    };

    vec![
        CapabilityRow {
            name: "Modules",
            status: to_status(cv("MODULES")),
            evidence: evidence_config(cv("MODULES"), "CONFIG_MODULES"),
        },
        CapabilityRow {
            name: "Rust for Linux",
            status: to_status_rust(ctx.rust_cfg, ctx.rust_avail),
            evidence: evidence_rust(ctx.rust_cfg, ctx.rust_avail),
        },
        CapabilityRow {
            name: "BTF",
            status: to_status_btf(cv("DEBUG_INFO_BTF"), ctx.dbg.btf_available),
            evidence: evidence_btf(cv("DEBUG_INFO_BTF"), ctx.dbg.btf_available),
        },
        CapabilityRow {
            name: "Module Signing",
            status: match ctx.mod_info.signing_enabled {
                Some(true) => CapabilityStatus::Enabled,
                Some(false) => CapabilityStatus::Disabled,
                None => CapabilityStatus::Unknown,
            },
            evidence: match cv("MODULE_SIG") {
                ConfigValue::Yes => "CONFIG_MODULE_SIG=y".into(),
                ConfigValue::No => "CONFIG_MODULE_SIG not set".into(),
                _ => {
                    if ctx.mod_info.signing_enabled == Some(true) {
                        "modules signed in /sys/module".into()
                    } else {
                        "signing status unknown".into()
                    }
                }
            },
        },
        CapabilityRow {
            name: "Livepatch",
            status: to_status(cv("LIVEPATCH")),
            evidence: evidence_config(cv("LIVEPATCH"), "CONFIG_LIVEPATCH"),
        },
        CapabilityRow {
            name: "Kallsyms",
            status: if ctx.ks_info.exists && ctx.ks_info.readable {
                CapabilityStatus::Enabled
            } else if ctx.ks_info.exists {
                CapabilityStatus::Disabled
            } else {
                CapabilityStatus::Unknown
            },
            evidence: if ctx.ks_info.exists && ctx.ks_info.readable {
                "/proc/kallsyms readable".into()
            } else if ctx.ks_info.exists {
                "/proc/kallsyms exists but not readable".into()
            } else {
                "/proc/kallsyms not found".into()
            },
        },
    ]
}

/// Collect compatibility risks — only applicable ones.
pub fn collect_risks(ctx: &ReportContext) -> Vec<String> {
    let mut risks: Vec<String> = Vec::new();

    if !ctx.mod_info.headers_available {
        if let (Some(r), Some(h)) = (
            ctx.release.as_deref(),
            ctx.mod_info.installed_header_version.as_deref(),
        ) && r != h
        {
            risks.push(format!(
                "Running kernel ({r}) differs from installed headers ({h}) — reboot needed"
            ));
        }
        if ctx.bld.build_dir.is_none() {
            risks.push("Kernel headers not installed — module compilation impossible".into());
        }
    }

    if ctx.bld.build_dir.is_none() && ctx.bld.source_dir.is_none() {
        risks.push("Build tree incomplete — cannot compile out-of-tree modules".into());
    }

    if ctx.sym_info.module_symvers_path.is_none() {
        risks.push(
            "Module.symvers missing — CRC validation unavailable for external modules".into(),
        );
    }

    if !ctx.rust_cfg.is_enabled() && ctx.rust_avail == ConfigValue::Yes {
        risks.push("Rust compiler available but kernel not built with CONFIG_RUST=y".into());
    } else if ctx.rust_cfg == ConfigValue::Missing && ctx.rust_avail == ConfigValue::Missing {
        risks.push("Rust kernel support not available — cannot build Rust kernel modules".into());
    }

    if ctx.sym_info.kallsyms_status.label().contains("restricted") {
        risks.push("Kallsyms addresses hidden (kptr_restrict) — symbol analysis limited".into());
    }

    if !ctx.debugfs_ok && !ctx.tracefs_ok {
        risks.push(
            "debugfs and tracefs not mounted — debug/tracing capabilities unavailable".into(),
        );
    }

    if ctx.mod_info.signing_required && !ctx.loader.signed_supported {
        risks.push("Module signing required but no signed modules detected".into());
    }

    risks
}

/// Collect interesting environment facts.
pub fn collect_facts(ctx: &ReportContext) -> Vec<String> {
    let mut facts: Vec<String> = Vec::new();

    if let Some(n) = ctx.sym_info.symbol_count {
        facts.push(format!("{n} exported kernel symbols"));
    }

    if ctx.loader.loaded_count > 0 {
        facts.push(format!("{} modules loaded", ctx.loader.loaded_count));
    }

    if ctx.dbg.btf_available {
        facts.push("BTF type information available".into());
    }

    if ctx.abi_info.module_compression != "Unknown" && ctx.abi_info.module_compression != "none" {
        facts.push(format!(
            "Module compression: {}",
            ctx.abi_info.module_compression
        ));
    }

    if ctx.mod_info.signing_enabled == Some(true) {
        facts.push("Kernel module signing active".into());
    }

    if let Some(ref comp) = ctx.comp_abi.kernel_compiler {
        facts.push(format!("Kernel compiled with: {comp}"));
    }

    if let Some(ref vml) = ctx.sym_info.vmlinux_path {
        if let Some(sz) = ctx.sym_info.vmlinux_size {
            facts.push(format!("VMLinux: {vml} ({})", human_size(sz)));
        } else {
            facts.push(format!("VMLinux: {vml}"));
        }
        if let Some(ref bid) = ctx.sym_info.vmlinux_build_id {
            facts.push(format!("VMLinux Build ID: {bid}"));
        }
    }

    if let Some(ref arch) = ctx.arch {
        facts.push(format!("Architecture: {arch}"));
    }

    // Kernel vs headers status
    if let (Some(r), Some(h)) = (
        ctx.release.as_deref(),
        ctx.mod_info.installed_header_version.as_deref(),
    ) {
        if r == h {
            facts.push(format!("Running kernel matches installed headers ({r})"));
        } else {
            facts.push(format!("Running kernel: {r}"));
            facts.push(format!("Installed headers: {h} (mismatch)"));
        }
    }

    // Config source
    if let Some(ref cfg) = ctx.config_text
        && !cfg.is_empty()
    {
        facts.push("Kernel config available".into());
    }

    // Debug/Trace mount status
    if ctx.debugfs_ok {
        facts.push("debugfs mounted".into());
    }
    if ctx.tracefs_ok {
        facts.push("tracefs mounted".into());
    }

    // Module compression from config
    if ctx.abi_info.module_compression != "Unknown" {
        facts.push(format!(
            "Module compression: {}",
            ctx.abi_info.module_compression
        ));
    }

    facts
}

// ---- helpers ---------------------------------------------------------------

fn human_size(bytes: u64) -> String {
    if bytes >= 1_048_576 {
        format!("{:.1} MB", bytes as f64 / 1_048_576.0)
    } else if bytes >= 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{bytes} B")
    }
}

fn to_status(v: ConfigValue) -> CapabilityStatus {
    if v.is_enabled() {
        CapabilityStatus::Enabled
    } else if v == ConfigValue::No {
        CapabilityStatus::Disabled
    } else {
        CapabilityStatus::Unknown
    }
}

fn evidence_config(v: ConfigValue, name: &str) -> String {
    match v {
        ConfigValue::Yes => format!("{name}=y"),
        ConfigValue::Module => format!("{name}=m"),
        ConfigValue::No => format!("{name} not set"),
        ConfigValue::Missing => format!("{name} not found in config"),
    }
}

fn to_status_rust(rust: ConfigValue, avail: ConfigValue) -> CapabilityStatus {
    if rust.is_enabled() {
        CapabilityStatus::Enabled
    } else if avail.is_enabled() || rust == ConfigValue::No {
        CapabilityStatus::Disabled
    } else {
        CapabilityStatus::Unknown
    }
}

fn evidence_rust(rust: ConfigValue, avail: ConfigValue) -> String {
    match (rust, avail) {
        (ConfigValue::Yes, _) => "CONFIG_RUST=y — kernel built with Rust support".into(),
        (_, ConfigValue::Yes) => "CONFIG_RUST_IS_AVAILABLE=y but CONFIG_RUST missing".into(),
        (ConfigValue::No, _) => "CONFIG_RUST not set".into(),
        _ => "CONFIG_RUST status unknown".into(),
    }
}

fn to_status_btf(cfg: ConfigValue, available: bool) -> CapabilityStatus {
    if cfg.is_enabled() && available {
        CapabilityStatus::Enabled
    } else if (cfg.is_enabled() && !available) || cfg == ConfigValue::No {
        CapabilityStatus::Disabled
    } else {
        CapabilityStatus::Unknown
    }
}

fn evidence_btf(cfg: ConfigValue, available: bool) -> String {
    if cfg.is_enabled() && available {
        "CONFIG_DEBUG_INFO_BTF=y, /sys/kernel/btf/vmlinux present".into()
    } else if cfg.is_enabled() {
        "CONFIG_DEBUG_INFO_BTF=y but BTF data not found".into()
    } else if cfg == ConfigValue::No {
        "CONFIG_DEBUG_INFO_BTF not set".into()
    } else {
        "BTF status unknown".into()
    }
}

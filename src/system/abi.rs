// Copyright (C) 2026 rezky_nightky
// SPDX-License-Identifier: GPL-3.0-only

//! Kernel ABI inspection — utsrelease, vermagic, module layout, compiler.
//!
//! Extracts ABI identifiers without loading modules or parsing ELF.

/// Result of ABI inspection.
pub struct AbiInfo {
    pub utsrelease: Option<String>,
    pub vermagic: Option<String>,
    pub module_layout_version: Option<String>,
    pub compiler_string: Option<String>,
    pub module_compression: &'static str,
    pub module_signing: &'static str,
}

/// Inspect kernel ABI identifiers.
pub fn inspect_abi(cfg: Option<&str>) -> AbiInfo {
    let proc_version = std::fs::read_to_string("/proc/version").ok();
    let release = crate::system::kernel::kernel_release();

    let utsrelease = release
        .clone()
        .or_else(|| proc_version.as_deref().and_then(extract_version));

    let vermagic = build_vermagic(release.as_deref(), cfg, proc_version.as_deref());
    let compiler_string = proc_version.as_deref().and_then(extract_compiler);

    let module_layout_version = release.as_deref().and_then(|r| {
        let p = format!("/lib/modules/{r}/build/include/generated/utsrelease.h");
        std::fs::read_to_string(&p).ok().and_then(|s| {
            s.lines()
                .find(|l| l.contains("UTS_RELEASE"))
                .map(|l| l.to_string())
        })
    });

    let (compression, signing) = classify_modules(cfg);

    AbiInfo {
        utsrelease,
        vermagic,
        module_layout_version,
        compiler_string,
        module_compression: compression,
        module_signing: signing,
    }
}

fn extract_version(proc: &str) -> Option<String> {
    // "Linux version 6.18.35-1-cachyos-lts ..."
    let parts: Vec<&str> = proc.split_whitespace().collect();
    parts.get(2).map(|s| s.to_string())
}

fn extract_compiler(proc: &str) -> Option<String> {
    // Look for compiler string in parentheses
    // "... (gcc (GCC) 14.2.1, ...)" or "... (clang version 19.1.0, ...)"
    let start = proc.find('(')? + 1;
    let rest = &proc[start..];
    let end = rest.find(')')?;
    let compiler = &rest[..end];

    if compiler.len() > 120 {
        return Some(format!("{}…", &compiler[..117]));
    }
    Some(compiler.to_string())
}

fn build_vermagic(
    release: Option<&str>,
    cfg: Option<&str>,
    proc_version: Option<&str>,
) -> Option<String> {
    let rel = release?;
    let mut parts = vec![rel.to_string()];

    // SMP
    if proc_version.is_none_or(|p| p.contains("SMP")) {
        parts.push("SMP".into());
    }

    // Preempt model
    if let Some(cfg) = cfg {
        use crate::system::config::config_value;
        if config_value(cfg, "PREEMPT").is_enabled() {
            parts.push("PREEMPT".into());
        } else if config_value(cfg, "PREEMPT_DYNAMIC").is_enabled() {
            parts.push("PREEMPT_DYNAMIC".into());
        } else if config_value(cfg, "PREEMPT_RT").is_enabled() {
            parts.push("PREEMPT_RT".into());
        } else if config_value(cfg, "PREEMPT_VOLUNTARY").is_enabled() {
            parts.push("PREEMPT_VOLUNTARY".into());
        }
    } else if proc_version.is_some_and(|p| p.contains("PREEMPT")) {
        parts.push("PREEMPT".into());
    }

    parts.push("mod_unload".into());

    Some(parts.join(" "))
}

fn classify_modules(cfg: Option<&str>) -> (&'static str, &'static str) {
    use crate::system::config::ConfigValue;
    let cv = |k: &str| {
        cfg.map_or(ConfigValue::Missing, |t| {
            crate::system::config::config_value(t, k)
        })
    };

    let compression = if cv("MODULE_COMPRESS_ZSTD").is_enabled() {
        "zstd"
    } else if cv("MODULE_COMPRESS_XZ").is_enabled() {
        "xz"
    } else if cv("MODULE_COMPRESS_GZIP").is_enabled() {
        "gzip"
    } else if cv("MODULE_COMPRESS_NONE").is_enabled() || cv("MODULE_COMPRESS") == ConfigValue::No {
        "none"
    } else {
        "Unknown"
    };

    let signing = if cv("MODULE_SIG_FORCE").is_enabled() {
        "required"
    } else if cv("MODULE_SIG").is_enabled() {
        "enabled"
    } else {
        "disabled"
    };

    (compression, signing)
}

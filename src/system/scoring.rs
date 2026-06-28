// Copyright (C) 2026 rezky_nightky
// SPDX-License-Identifier: GPL-3.0-only

//! Category-based star scoring for kernel development readiness.
//!
//! Replaces arbitrary percentages with ★☆☆☆☆ style ratings
//! grouped by concern: Environment, Headers, Rust, Debug, Build tree, Modules.

/// A single category score — 0–5 stars.
pub struct CategoryScore {
    pub name: &'static str,
    pub stars: u8,
}

impl CategoryScore {
    fn new(name: &'static str, passed: usize, total: usize) -> Self {
        let stars = if total == 0 {
            0
        } else {
            ((passed as f64 / total as f64) * 5.0).round() as u8
        };
        Self { name, stars }
    }

    /// Render as stars, e.g. "★★★☆☆".
    pub fn render(&self) -> String {
        let filled = "★".repeat(self.stars as usize);
        let empty = "☆".repeat((5 - self.stars) as usize);
        format!("{filled}{empty}")
    }
}

/// Compute star scores for every readiness category.
pub fn compute() -> Vec<CategoryScore> {
    let arch = super::kernel::architecture();
    let release = super::kernel::kernel_release();
    let distro = super::kernel::detect_distro();
    let tools = super::toolchain::inspect_toolchain();
    let bld = super::buildenv::inspect_build_env();
    let (cfg_text, _) = super::config::read_kernel_config().unzip();
    let cfg = cfg_text.as_deref();
    let mod_info = super::modules::inspect_modules(cfg);
    let ks = super::kallsyms::inspect_kallsyms();
    let dbg = super::btf::inspect_debug();

    use super::config::ConfigValue;
    let cv = |k: &str| {
        cfg.map(|t| super::config::config_value(t, k))
            .unwrap_or(ConfigValue::Missing)
    };

    let debugfs_ok = mount_check("/sys/kernel/debug");
    let tracefs_ok = mount_check("/sys/kernel/tracing");

    // -- Environment (5 checks) --------------------------------------------
    let env_passed = [
        release.is_some(),
        arch.is_some(),
        distro.is_some(),
        std::path::Path::new(&format!(
            "/lib/modules/{}",
            release.as_deref().unwrap_or("")
        ))
        .is_dir(),
        std::path::Path::new("/usr/src").is_dir(),
    ]
    .iter()
    .filter(|&&p| p)
    .count();

    // -- Headers (4 checks) ------------------------------------------------
    let hdr_passed = [
        mod_info.headers_available,
        bld.build_dir.is_some(),
        bld.header_status.is_ready(),
        mod_info.installed_header_version.is_none() || mod_info.headers_available,
    ]
    .iter()
    .filter(|&&p| p)
    .count();

    // -- Rust (5 checks) ---------------------------------------------------
    let rust_enabled = cv("RUST").is_enabled();
    let rust_avail = cv("RUST_IS_AVAILABLE").is_enabled();
    let rust_passed = [
        tools.rustc.is_some(),
        tools.cargo.is_some(),
        rust_enabled,
        rust_avail,
        tools.bindgen.is_some(),
    ]
    .iter()
    .filter(|&&p| p)
    .count();

    // -- Debug (4 checks) --------------------------------------------------
    let debug_passed = [
        dbg.btf_available,
        ks.exists && ks.readable,
        debugfs_ok,
        tracefs_ok,
    ]
    .iter()
    .filter(|&&p| p)
    .count();

    // -- Build tree (4 checks) ---------------------------------------------
    let build_passed = [
        bld.build_dir.is_some(),
        bld.source_dir.is_some(),
        bld.module_symvers.is_some(),
        bld.compile_commands,
    ]
    .iter()
    .filter(|&&p| p)
    .count();

    // -- Modules (3 checks) ------------------------------------------------
    let mod_passed = [
        cv("MODULES").is_enabled(),
        mod_info.modules_dir.is_some(),
        cv("MODULES").is_enabled() && mod_info.modules_dir.is_some(),
    ]
    .iter()
    .filter(|&&p| p)
    .count();

    vec![
        CategoryScore::new("Environment", env_passed, 5),
        CategoryScore::new("Headers", hdr_passed, 4),
        CategoryScore::new("Rust", rust_passed, 5),
        CategoryScore::new("Debug", debug_passed, 4),
        CategoryScore::new("Build tree", build_passed, 4),
        CategoryScore::new("Modules", mod_passed, 3),
    ]
}

/// Overall readiness label from aggregated stars.
/// Strict criteria: requires high scores across all categories
/// since kernel development needs the full environment.
pub fn overall_rating(scores: &[CategoryScore]) -> &'static str {
    if scores.is_empty() {
        return "Unknown";
    }

    // Count how many categories are weak (≤2★)
    let weak = scores.iter().filter(|s| s.stars <= 2).count();
    // Count how many are good (≥4★)
    let good = scores.iter().filter(|s| s.stars >= 4).count();
    let total = scores.len();

    if good == total {
        "Ready"
    } else if weak == 0 && good >= total / 2 {
        "Mostly Ready"
    } else if weak <= 1 {
        "Needs Work"
    } else {
        "Needs Attention"
    }
}

/// Check whether a path is a mount point via /proc/mounts.
fn mount_check(path: &str) -> bool {
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

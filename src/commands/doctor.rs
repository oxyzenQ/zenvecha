// Copyright (C) 2026 rezky_nightky
// SPDX-License-Identifier: GPL-3.0-only

//! Doctor command — system readiness check.
//!
//! Verifies kernel version, architecture, compiler, headers, and
//! Rust-for-Linux support.  `--fix` mode shows actionable remediation.

use std::io::{self, Write};
use std::path::Path;

use crate::system::{kernel, rust};

// ---- public API ------------------------------------------------------------

pub struct Doctor {
    distro: Option<String>,
    kernel: Option<String>,
    architecture: Option<String>,
    checks: Vec<Check>,
    fix_commands: Vec<String>,
}

pub struct Check {
    pub name: &'static str,
    pub passed: bool,
    pub reason: Option<String>,
}

impl Doctor {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            distro: kernel::detect_distro(),
            kernel: kernel::kernel_version(),
            architecture: kernel::architecture(),
            checks: Vec::with_capacity(5),
            fix_commands: Vec::new(),
        }
    }

    pub fn run(mut self, fix_mode: bool) -> Result<(), Box<dyn std::error::Error>> {
        let kver = self.kernel.clone();
        let distro = self.distro.clone();
        let arch_val = self.architecture.clone();
        let kver_d: Option<&str> = kver.as_deref();
        let distro_d: Option<&str> = distro.as_deref();
        let arch_d: Option<&str> = arch_val.as_deref();

        let kernel_ok = self.check_kernel(kver_d);
        let arch_ok = self.check_architecture(arch_d);
        let rust_ok = self.check_rust();
        let headers_ok = self.check_headers(distro_d, kver_d);
        let r4l_ok = self.check_r4l(kver_d);

        let stdout = io::stdout();
        let mut out = stdout.lock();

        let _ = writeln!(out, "Zenvecha Doctor");
        let _ = writeln!(out);

        // Detected
        let _ = writeln!(out, "Detected");
        let _ = writeln!(out);
        if let Some(ref d) = self.distro {
            let _ = writeln!(out, " Distribution : {d}");
        }
        if let Some(ref k) = self.kernel {
            let _ = writeln!(out, " Kernel       : {k}");
        }
        if let Some(ref a) = self.architecture {
            let _ = writeln!(out, " Architecture : {a}");
        }
        let _ = writeln!(out);

        if fix_mode {
            self.print_fix_mode(&mut out);
            return Ok(());
        }

        // Checks
        let _ = writeln!(out, "Checks");
        let _ = writeln!(out);
        for c in &self.checks {
            let mark = if c.passed { "[+]" } else { "[-]" };
            let _ = writeln!(out, "{mark} {}", c.name);
            if !c.passed
                && let Some(ref reason) = c.reason
            {
                let _ = writeln!(out, "    {reason}");
            }
        }
        let _ = writeln!(out);

        // Suggested actions
        if !self.fix_commands.is_empty() {
            let _ = writeln!(out, "Suggested actions");
            let _ = writeln!(out);
            for (i, cmd) in self.fix_commands.iter().enumerate() {
                let _ = writeln!(out, " {}. {cmd}", i + 1);
                if cmd.contains("headers") || cmd.contains("kernel-devel") {
                    let _ = writeln!(out, "    Verify: ls /lib/modules/$(uname -r)/build");
                }
            }
            let _ = writeln!(out);
        }

        // Overall
        let passed = self.checks.iter().filter(|c| c.passed).count();
        let total = self.checks.len();
        let ready = kernel_ok && arch_ok && rust_ok && headers_ok && r4l_ok;

        let _ = writeln!(out, "Overall");
        let _ = writeln!(out);
        let _ = writeln!(out, " {passed} / {total}");
        if ready {
            let _ = writeln!(out, " READY");
        } else {
            let _ = writeln!(out, " NOT READY");
        }

        Ok(())
    }

    // -- fix mode output ----------------------------------------------------

    fn print_fix_mode(&self, out: &mut io::StdoutLock<'_>) {
        let failures: Vec<&Check> = self.checks.iter().filter(|c| !c.passed).collect();

        if failures.is_empty() {
            let _ = writeln!(out, "All checks passed. Nothing to fix.");
            return;
        }

        let _ = writeln!(out, "Detected issues");
        let _ = writeln!(out);
        for c in &failures {
            let _ = writeln!(out, " {}", c.name);
            let _ = writeln!(out);
            if let Some(ref reason) = c.reason {
                for line in reason.lines() {
                    let _ = writeln!(out, "   {line}");
                }
                let _ = writeln!(out);
            }
        }

        if !self.fix_commands.is_empty() {
            let _ = writeln!(out, "Automatic fixes");
            let _ = writeln!(out);
            for cmd in &self.fix_commands {
                let _ = writeln!(out, "  {cmd}");
            }
            let _ = writeln!(out);
            let _ = writeln!(out, "Run the commands above to apply automatically.");
            let _ = writeln!(out);
        }

        let manual: Vec<&str> = failures
            .iter()
            .filter_map(|c| manual_action_label(c))
            .collect();

        if !manual.is_empty() {
            if self.fix_commands.is_empty() {
                let _ = writeln!(out, "No automatic fixes available.");
                let _ = writeln!(out);
            }
            let _ = writeln!(out, "Manual actions");
            let _ = writeln!(out);
            for (i, label) in manual.iter().enumerate() {
                let _ = writeln!(out, " {}. {label}", i + 1);
            }
            let _ = writeln!(out);
            let _ = writeln!(out, "These require manual intervention.");
            let _ = writeln!(out, "No commands were executed.");
        }
    }

    // -- individual checks --------------------------------------------------

    fn check_kernel(&mut self, kver: Option<&str>) -> bool {
        match kver {
            Some(ver) => {
                let ok = ver.starts_with("6.");
                self.checks.push(Check {
                    name: "Kernel version",
                    passed: ok,
                    reason: if !ok {
                        Some("Zenvecha requires Linux 6.x.".into())
                    } else {
                        None
                    },
                });
                ok
            }
            None => {
                self.checks.push(Check {
                    name: "Kernel version",
                    passed: false,
                    reason: Some("Could not read /proc/version.".into()),
                });
                false
            }
        }
    }

    fn check_architecture(&mut self, arch_val: Option<&str>) -> bool {
        match arch_val {
            Some(a) => {
                let ok = a == "x86_64";
                self.checks.push(Check {
                    name: "CPU architecture",
                    passed: ok,
                    reason: if !ok {
                        Some("Zenvecha requires x86_64.".into())
                    } else {
                        None
                    },
                });
                ok
            }
            None => {
                self.checks.push(Check {
                    name: "CPU architecture",
                    passed: false,
                    reason: Some("uname -m failed.".into()),
                });
                false
            }
        }
    }

    fn check_rust(&mut self) -> bool {
        match kernel::compiler_version() {
            Some(_ver) => {
                self.checks.push(Check {
                    name: "Rust compiler",
                    passed: true,
                    reason: None,
                });
                true
            }
            None => {
                let cmd = "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh";
                self.fix_commands.push(cmd.to_string());
                self.checks.push(Check {
                    name: "Rust compiler",
                    passed: false,
                    reason: Some("rustc is not in PATH.".into()),
                });
                false
            }
        }
    }

    fn check_headers(&mut self, distro: Option<&str>, kver: Option<&str>) -> bool {
        let running = kver.unwrap_or("");

        let build_path = format!("/lib/modules/{running}/build");
        if Path::new(&build_path).exists() {
            self.checks.push(Check {
                name: "Kernel headers",
                passed: true,
                reason: None,
            });
            return true;
        }

        if let Some(installed_ver) = installed_header_version(running) {
            let reason = format!(
                "Running kernel: {}\n    Installed headers: {}\n    Action: reboot to boot into {installed_ver}.",
                running, installed_ver
            );
            self.checks.push(Check {
                name: "Kernel headers",
                passed: false,
                reason: Some(reason),
            });
            return false;
        }

        let pkg_cmd = header_package_command(distro, kver);
        self.fix_commands.push(pkg_cmd.clone());

        self.checks.push(Check {
            name: "Kernel headers",
            passed: false,
            reason: Some("Missing kernel headers.".into()),
        });
        false
    }

    fn check_r4l(&mut self, kver: Option<&str>) -> bool {
        // Use kver to read the matching config
        let config = kver.and_then(|_v| {
            // Read config via the shared system module
            let (content, _source) = crate::system::config::read_kernel_config()?;
            Some(content)
        });
        let detected =
            rust::rust_enabled(config.as_deref()) || kver.is_some_and(fallback_r4l_detect);

        if detected {
            self.checks.push(Check {
                name: "Rust-for-Linux",
                passed: true,
                reason: None,
            });
            return true;
        }

        self.checks.push(Check {
            name: "Rust-for-Linux",
            passed: false,
            reason: Some("CONFIG_RUST / CONFIG_RUST_IS_AVAILABLE not found.".into()),
        });
        false
    }
}

// ---- free functions (doctor helpers) ---------------------------------------

fn manual_action_label(c: &Check) -> Option<&'static str> {
    let reason = c.reason.as_deref()?;
    match c.name {
        "Kernel headers" if reason.contains("reboot") => Some("reboot into the installed kernel"),
        "Rust-for-Linux" => {
            Some("boot a kernel with CONFIG_RUST=y / CONFIG_RUST_IS_AVAILABLE=y enabled")
        }
        "Kernel version" => Some("install and boot Linux 6.x or later"),
        "CPU architecture" => Some("use an x86_64 system"),
        _ => None,
    }
}

/// Fallback Rust-for-Linux detection when config file is unavailable.
fn fallback_r4l_detect(version: &str) -> bool {
    use std::process::Command;

    let rust_configs = ["CONFIG_RUST=y", "CONFIG_RUST_IS_AVAILABLE=y"];

    for cfg in &rust_configs {
        if let Ok(output) = Command::new("zgrep")
            .args([*cfg, "/proc/config.gz"])
            .output()
            && output.status.success()
        {
            return true;
        }
    }

    let config_path = format!("/boot/config-{version}");
    if let Ok(content) = std::fs::read_to_string(&config_path) {
        for cfg in &rust_configs {
            if content.contains(cfg) {
                return true;
            }
        }
    }

    let rust_h = format!("/lib/modules/{version}/build/include/linux/rust.h");
    if Path::new(&rust_h).exists() {
        return true;
    }

    let rust_makefile = format!("/lib/modules/{version}/build/rust/Makefile");
    Path::new(&rust_makefile).exists()
}

fn installed_header_version(running: &str) -> Option<String> {
    let modules = Path::new("/lib/modules");
    if !modules.exists() {
        return None;
    }
    let entries = std::fs::read_dir(modules).ok()?;
    for entry in entries.filter_map(|e| e.ok()) {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if name == running {
            continue;
        }
        let build = entry.path().join("build");
        if build.exists() {
            return Some(name.to_string());
        }
    }
    None
}

fn kernel_variant(kver: Option<&str>) -> Option<String> {
    let ver = kver?;
    let parts: Vec<&str> = ver.split('-').collect();
    if parts.len() < 3 {
        return None;
    }
    let variant_parts = &parts[2..];
    if variant_parts.is_empty() {
        return None;
    }
    Some(variant_parts.join("-"))
}

fn header_package_command(distro: Option<&str>, kver: Option<&str>) -> String {
    let distro_id = distro.map(|d| d.to_lowercase()).unwrap_or_default();
    let variant = kernel_variant(kver);

    if distro_id.contains("cachyos")
        || distro_id.contains("arch")
        || distro_id.contains("endeavouros")
        || distro_id.contains("manjaro")
        || distro_id.contains("artix")
    {
        let pkg = arch_header_package(&variant);
        return format!("sudo pacman -S {pkg}");
    }

    if distro_id.contains("ubuntu")
        || distro_id.contains("debian")
        || distro_id.contains("linux mint")
        || distro_id.contains("pop")
    {
        return "sudo apt install linux-headers-$(uname -r)".into();
    }

    if distro_id.contains("fedora") {
        return "sudo dnf install kernel-devel kernel-headers".into();
    }

    if distro_id.contains("suse") || distro_id.contains("opensuse") {
        return "sudo zypper install kernel-default-devel".into();
    }

    if distro_id.contains("alpine") {
        let pkg = alpine_header_package(&variant);
        return format!("sudo apk add {pkg}");
    }

    "Install the kernel headers package for your distribution.".into()
}

fn arch_header_package(variant: &Option<String>) -> String {
    match variant.as_deref() {
        Some("cachyos-lts") => "linux-cachyos-lts-headers",
        Some("cachyos-hardened") => "linux-cachyos-hardened-headers",
        Some("cachyos-zen") => "linux-cachyos-zen-headers",
        Some(v) if v.starts_with("cachyos") => "linux-cachyos-headers",
        Some("zen") => "linux-zen-headers",
        Some("lts") => "linux-lts-headers",
        Some("hardened") => "linux-hardened-headers",
        Some("rt") | Some("rt-lts") => "linux-rt-headers",
        Some("arch") | Some("arch1") => "linux-headers",
        _ => "linux-headers",
    }
    .to_string()
}

fn alpine_header_package(variant: &Option<String>) -> String {
    match variant.as_deref() {
        Some("lts") => "linux-lts-dev",
        Some("virt") => "linux-virt-dev",
        _ => "linux-lts-dev",
    }
    .to_string()
}

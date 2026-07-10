// Copyright (C) 2026 rezky_nightky
// SPDX-License-Identifier: GPL-3.0-only

//! Compiler ABI comparison — kernel compiler vs installed toolchain.
//!
//! Never claims exact compatibility. Reports levels of confidence.

use std::process::Command;

/// Compatibility confidence level.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CompilerCompat {
    Compatible,
    Probably,
    Unknown,
    NotCompatible,
}

impl CompilerCompat {
    pub fn label(self) -> &'static str {
        match self {
            CompilerCompat::Compatible => "Compatible",
            CompilerCompat::Probably => "Probably compatible",
            CompilerCompat::Unknown => "Unknown",
            CompilerCompat::NotCompatible => "Not compatible",
        }
    }
}

/// Compiler ABI comparison result.
pub struct CompilerAbi {
    pub kernel_compiler: Option<String>,
    pub installed_gcc: Option<String>,
    pub installed_clang: Option<String>,
    pub installed_rustc: Option<String>,
    pub gcc_compat: CompilerCompat,
    pub clang_compat: CompilerCompat,
    pub rustc_compat: CompilerCompat,
}

/// Compare kernel compiler against installed toolchains.
pub fn compare_compilers(rustc_ver: &Option<String>) -> CompilerAbi {
    let kernel_compiler = kernel_compiler_string();
    let installed_gcc = gcc_version();
    let installed_clang = clang_version();
    let installed_rustc = rustc_ver.clone();

    let gcc_compat = compare_gcc(kernel_compiler.as_deref(), installed_gcc.as_deref());
    let clang_compat = compare_clang(kernel_compiler.as_deref(), installed_clang.as_deref());
    let rustc_compat = if installed_rustc.is_some() {
        CompilerCompat::Probably
    } else {
        CompilerCompat::NotCompatible
    };

    CompilerAbi {
        kernel_compiler,
        installed_gcc,
        installed_clang,
        installed_rustc,
        gcc_compat,
        clang_compat,
        rustc_compat,
    }
}

fn kernel_compiler_string() -> Option<String> {
    let raw = std::fs::read_to_string("/proc/version").ok()?;
    extract_compiler(&raw)
}

fn extract_compiler(proc: &str) -> Option<String> {
    let start = proc.find('(')? + 1;
    let rest = &proc[start..];
    let end = rest.find(')')?;
    let raw = &rest[..end];
    let lower = raw.to_lowercase();
    if lower.contains("gcc") || lower.contains("clang") || lower.contains("llvm") {
        Some(raw.to_string())
    } else {
        None
    }
}

fn gcc_version() -> Option<String> {
    let out = Command::new("gcc").args(["--version"]).output().ok()?;
    String::from_utf8(out.stdout)
        .ok()
        .and_then(|s| s.lines().next().map(|l| l.trim().to_string()))
}

fn clang_version() -> Option<String> {
    let out = Command::new("clang").args(["--version"]).output().ok()?;
    String::from_utf8(out.stdout)
        .ok()
        .and_then(|s| s.lines().next().map(|l| l.trim().to_string()))
}

fn compare_gcc(kernel: Option<&str>, installed: Option<&str>) -> CompilerCompat {
    let (k, i) = match (kernel, installed) {
        (Some(k), Some(i)) => (k, i),
        (Some(_), None) => return CompilerCompat::Unknown,
        (None, Some(_)) => return CompilerCompat::Probably,
        (None, None) => return CompilerCompat::Unknown,
    };

    let k_major = extract_major_gcc(k);
    let i_major = extract_major_gcc(i);

    match (k_major, i_major) {
        (Some(km), Some(im)) if km == im => CompilerCompat::Compatible,
        (Some(_), Some(_)) => CompilerCompat::Probably,
        _ => CompilerCompat::Unknown,
    }
}

fn compare_clang(kernel: Option<&str>, installed: Option<&str>) -> CompilerCompat {
    let installed = match installed {
        Some(i) => i,
        None => return CompilerCompat::Unknown,
    };

    let _ = kernel;
    // If kernel was built with clang, exact match is important
    if let Some(k) = kernel
        && k.starts_with("clang")
    {
        let k_ver = extract_clang_version(k);
        let i_ver = extract_clang_version(installed);
        if let (Some(kv), Some(iv)) = (k_ver, i_ver)
            && kv == iv
        {
            return CompilerCompat::Compatible;
        }
        return CompilerCompat::Probably;
    }

    // Kernel built with GCC, clang is extra — probably works for analysis
    CompilerCompat::Probably
}

fn extract_major_gcc(s: &str) -> Option<u32> {
    // "gcc (GCC) 14.2.1 ..." → 14
    // Find the first number in the string
    let version_part = s
        .split_whitespace()
        .find(|w| w.chars().next().is_some_and(|c| c.is_ascii_digit()))?;
    version_part.split('.').next()?.parse().ok()
}

fn extract_clang_version(s: &str) -> Option<String> {
    // "clang version 19.1.0" → "19.1"
    let parts: Vec<&str> = s.split_whitespace().collect();
    let idx = parts.iter().position(|&p| p == "version")?;
    let ver = parts.get(idx + 1)?;
    let segments: Vec<&str> = ver.split('.').collect();
    if segments.len() >= 2 {
        Some(format!("{}.{}", segments[0], segments[1]))
    } else {
        Some(ver.to_string())
    }
}

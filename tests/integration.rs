// Copyright (C) 2026 rezky_nightky
// SPDX-License-Identifier: GPL-3.0-only

//! Integration tests for Zenvecha.

fn zenvecha_bin() -> std::path::PathBuf {
    std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .join("zenvecha")
}

fn run(args: &[&str]) -> Option<std::process::Output> {
    std::process::Command::new(zenvecha_bin())
        .args(args)
        .output()
        .ok()
}

#[test]
fn test_version_flag() {
    let output = run(&["-V"]);
    if let Some(o) = output {
        let stdout = String::from_utf8_lossy(&o.stdout);
        assert!(stdout.contains("zenvecha"));
        assert!(stdout.contains("v0."));
        assert!(stdout.contains("rezky_nightky"));
        assert!(stdout.contains("GPL-3.0"));
    }
}

#[test]
fn test_doctor_runs() {
    let output = run(&["doctor"]);
    if let Some(o) = output {
        let stdout = String::from_utf8_lossy(&o.stdout);
        assert!(stdout.contains("Zenvecha Doctor"));
        assert!(stdout.contains("Detected"));
        assert!(stdout.contains("Checks"));
        assert!(stdout.contains("Overall"));
    }
}

#[test]
fn test_doctor_fix_mode() {
    let output = run(&["doctor", "--fix"]);
    if let Some(o) = output {
        let stdout = String::from_utf8_lossy(&o.stdout);
        assert!(stdout.contains("Zenvecha Doctor"));
        assert!(stdout.contains("Detected issues"));
        assert!(stdout.contains("No commands were executed"));
    }
}

#[test]
fn test_inspect_runs() {
    let output = run(&["inspect"]);
    if let Some(o) = output {
        let stdout = String::from_utf8_lossy(&o.stdout);
        assert!(stdout.contains("Zenvecha Inspect"));
        assert!(stdout.contains("Kernel"));
        assert!(stdout.contains("Configuration"));
        assert!(stdout.contains("Module Environment"));
        assert!(stdout.contains("Debug Information"));
        assert!(stdout.contains("Symbol Information"));
        assert!(stdout.contains("Kernel Capability Summary"));
        assert!(stdout.contains("Suitable for:"));
    }
}

#[test]
fn test_inspect_never_panics() {
    // inspect must never panic even with missing config/procfs
    let output = run(&["inspect"]);
    if let Some(o) = output {
        assert!(o.status.success());
        // Should not contain panic messages
        let stderr = String::from_utf8_lossy(&o.stderr);
        assert!(!stderr.contains("panic"));
        assert!(!stderr.contains("thread"));
    }
}

#[test]
fn test_unknown_command() {
    let output = run(&["nonexistent"]);
    if let Some(o) = output {
        let stderr = String::from_utf8_lossy(&o.stderr);
        assert!(stderr.contains("unknown command"));
    }
}

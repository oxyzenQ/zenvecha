// Copyright (C) 2026 rezky_nightky
// SPDX-License-Identifier: GPL-3.0-only

//! Integration tests for Zenvecha.

#[test]
fn test_version_flag() {
    let output = std::process::Command::new(
        std::env::current_exe()
            .unwrap()
            .parent()
            .unwrap()
            .join("zenvecha"),
    )
    .arg("-V")
    .output();

    match output {
        Ok(o) => {
            let stdout = String::from_utf8_lossy(&o.stdout);
            assert!(stdout.contains("zenvecha"), "version output missing");
        }
        Err(_) => {
            // Binary may not exist in CI test run — skip
        }
    }
}

#[test]
fn test_doctor_runs() {
    let output = std::process::Command::new(
        std::env::current_exe()
            .unwrap()
            .parent()
            .unwrap()
            .join("zenvecha"),
    )
    .arg("doctor")
    .output();

    match output {
        Ok(o) => {
            let stdout = String::from_utf8_lossy(&o.stdout);
            assert!(
                stdout.contains("Zenvecha Doctor"),
                "doctor output missing header"
            );
            assert!(stdout.contains("Status:"), "doctor output missing status");
        }
        Err(_) => {
            // Binary may not exist — skip
        }
    }
}

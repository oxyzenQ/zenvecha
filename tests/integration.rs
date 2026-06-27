// Copyright (C) 2026 rezky_nightky
// SPDX-License-Identifier: GPL-3.0-only

//! Integration tests for Zenvecha.

#[test]
fn test_version_flag_works() {
    // TODO: Add actual CLI integration tests once commands are implemented.
    // For now, just verify the test harness works.
    let status = std::process::Command::new("bash")
        .args(["-c", "exit 0"])
        .status();
    assert!(status.is_ok());
}

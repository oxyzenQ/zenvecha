// Copyright (C) 2026 rezky_nightky
// SPDX-License-Identifier: GPL-3.0-only

fn main() {
    // Embed the current git commit hash for --version output
    let hash = std::process::Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_else(|| "unknown".to_string());

    println!("cargo:rustc-env=ZENVECHA_COMMIT_HASH={}", hash.trim());
}

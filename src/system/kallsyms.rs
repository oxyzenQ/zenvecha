// Copyright (C) 2026 rezky_nightky
// SPDX-License-Identifier: GPL-3.0-only

//! Kernel symbol table inspection.
//!
//! Only detects existence and readability — never resolves symbols.

use std::path::Path;

/// Result of /proc/kallsyms inspection.
pub struct KallsymsInfo {
    pub exists: bool,
    pub readable: bool,
    pub root_only: bool,
}

/// Inspect /proc/kallsyms accessibility.
pub fn inspect_kallsyms() -> KallsymsInfo {
    let path = Path::new("/proc/kallsyms");

    if !path.exists() {
        return KallsymsInfo {
            exists: false,
            readable: false,
            root_only: false,
        };
    }

    // Try to open for reading
    let readable = std::fs::File::open(path).is_ok();

    // Check if the file is only readable by root.
    // kallsyms is typically 0 bytes reported by stat when not root,
    // or the open fails with EACCES when KPTI is active.
    let root_only = !readable || file_zero_bytes(path);

    KallsymsInfo {
        exists: true,
        readable,
        root_only,
    }
}

/// kallsyms often reports 0 size for non-root readers.
fn file_zero_bytes(path: &Path) -> bool {
    std::fs::metadata(path)
        .map(|m| m.len() == 0)
        .unwrap_or(false)
}

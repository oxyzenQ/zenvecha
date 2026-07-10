// Copyright (C) 2026 rezky_nightky
// SPDX-License-Identifier: GPL-3.0-only

//! Filesystem path verification.
//!
//! Check common kernel-development paths for presence, absence,
//! or permission issues. Read-only, never creates directories.

use std::path::Path;

/// Status of a checked filesystem path.
pub enum FsStatus {
    Present,
    Missing,
    PermissionDenied,
}

/// Result for a single path check.
pub struct FsCheck {
    pub path: String,
    pub status: FsStatus,
}

impl FsCheck {
    pub fn label(&self) -> &str {
        match self.status {
            FsStatus::Present => "Present",
            FsStatus::Missing => "Missing",
            FsStatus::PermissionDenied => "Permission denied",
        }
    }

    pub fn passed(&self) -> bool {
        matches!(self.status, FsStatus::Present)
    }
}

/// Check a set of paths.  Missing paths are not errors —
/// they simply report `Missing`.
pub fn check_paths(paths: &[&str]) -> Vec<FsCheck> {
    paths.iter().map(|p| check_one(p)).collect()
}

fn check_one(raw: &str) -> FsCheck {
    let path = Path::new(raw);
    if path.exists() {
        return FsCheck {
            path: raw.into(),
            status: FsStatus::Present,
        };
    }

    // Walk up to find the deepest existing parent to check perms
    let mut current = path;
    loop {
        match current.parent() {
            Some(parent) if parent.as_os_str().is_empty() => {
                // Hit root — truly missing
                return FsCheck {
                    path: raw.into(),
                    status: FsStatus::Missing,
                };
            }
            Some(parent) => {
                if parent.exists() {
                    // Parent exists — check if we can read it
                    if std::fs::read_dir(parent).is_err() {
                        return FsCheck {
                            path: raw.into(),
                            status: FsStatus::PermissionDenied,
                        };
                    }
                    return FsCheck {
                        path: raw.into(),
                        status: FsStatus::Missing,
                    };
                }
                current = parent;
            }
            None => {
                return FsCheck {
                    path: raw.into(),
                    status: FsStatus::Missing,
                };
            }
        }
    }
}

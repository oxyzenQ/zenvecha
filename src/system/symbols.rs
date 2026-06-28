// Copyright (C) 2026 rezky_nightky
// SPDX-License-Identifier: GPL-3.0-only

//! Symbol inspection — /proc/kallsyms, System.map, Module.symvers, vmlinux.

use std::fs;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub enum KallsymsStatus {
    Available,
    Restricted,
    PermissionDenied,
    Hidden,
}

impl KallsymsStatus {
    pub fn label(&self) -> &'static str {
        match self {
            KallsymsStatus::Available => "available",
            KallsymsStatus::Restricted => "restricted (addresses hidden — root required)",
            KallsymsStatus::PermissionDenied => "permission denied",
            KallsymsStatus::Hidden => "not available (/proc/sys/kernel/kptr_restrict)",
        }
    }
}

pub struct SymbolInfo {
    pub kallsyms_status: KallsymsStatus,
    pub symbol_count: Option<u64>,
    pub system_map_path: Option<String>,
    pub system_map_size: Option<u64>,
    pub module_symvers_path: Option<String>,
    pub symvers_crc_count: Option<u64>,
    pub symvers_size: Option<u64>,
    pub symvers_modified: Option<String>,
    pub vmlinux_path: Option<String>,
    pub vmlinux_size: Option<u64>,
    pub vmlinux_build_id: Option<String>,
}

pub fn inspect_symbols(release: Option<&str>) -> SymbolInfo {
    let (kallsyms_status, symbol_count) = inspect_kallsyms();
    let system_map_path = find_system_map(release);
    let system_map_size = system_map_path
        .as_deref()
        .and_then(|p| fs::metadata(p).ok())
        .map(|m| m.len());
    let symvers = find_module_symvers(release);
    let symvers_crc_count = symvers.as_deref().and_then(count_symvers_crcs);
    let symvers_size = symvers
        .as_deref()
        .and_then(|p| fs::metadata(p).ok())
        .map(|m| m.len());
    let symvers_modified = symvers
        .as_deref()
        .and_then(|p| fs::metadata(p).ok())
        .and_then(|m| m.modified().ok())
        .map(|t| format!("{:?}", t));

    let vmlinux_path = find_vmlinux(release);
    let vmlinux_size = vmlinux_path
        .as_deref()
        .and_then(|p| fs::metadata(p).ok())
        .map(|m| m.len());
    let vmlinux_build_id = vmlinux_path.as_deref().and_then(extract_build_id);

    SymbolInfo {
        kallsyms_status,
        symbol_count,
        system_map_path,
        system_map_size,
        module_symvers_path: symvers,
        symvers_crc_count,
        symvers_size,
        symvers_modified,
        vmlinux_path,
        vmlinux_size,
        vmlinux_build_id,
    }
}

fn inspect_kallsyms() -> (KallsymsStatus, Option<u64>) {
    let path = "/proc/kallsyms";
    if !Path::new(path).exists() {
        return (KallsymsStatus::Hidden, None);
    }
    let file = match fs::File::open(path) {
        Ok(f) => f,
        Err(e) => {
            let status = if e.kind() == std::io::ErrorKind::PermissionDenied {
                KallsymsStatus::PermissionDenied
            } else {
                KallsymsStatus::Hidden
            };
            return (status, None);
        }
    };
    let mut reader = BufReader::new(&file);
    let mut first = String::new();
    let restricted = match reader.read_line(&mut first) {
        Ok(n) if n > 0 => first.starts_with("0000000000000000"),
        _ => false,
    };
    let mut count: u64 = if first.is_empty() { 0 } else { 1 };
    let mut buf = String::new();
    while reader.read_line(&mut buf).unwrap_or(0) > 0 {
        count += 1;
        buf.clear();
    }
    let status = if restricted {
        KallsymsStatus::Restricted
    } else {
        KallsymsStatus::Available
    };
    (status, Some(count))
}

fn find_system_map(release: Option<&str>) -> Option<String> {
    let candidates: Vec<String> = if let Some(r) = release {
        vec![
            format!("/boot/System.map-{r}"),
            format!("/usr/lib/modules/{r}/System.map"),
            "/boot/System.map".into(),
        ]
    } else {
        vec!["/boot/System.map".into()]
    };
    candidates.into_iter().find(|p| Path::new(p).exists())
}

fn find_module_symvers(release: Option<&str>) -> Option<String> {
    let r = release?;
    let candidates = [
        format!("/lib/modules/{r}/build/Module.symvers"),
        format!("/usr/lib/modules/{r}/build/Module.symvers"),
        format!("/lib/modules/{r}/source/Module.symvers"),
    ];
    candidates.into_iter().find(|p| Path::new(p).exists())
}

fn count_symvers_crcs(path: &str) -> Option<u64> {
    let file = fs::File::open(path).ok()?;
    let reader = BufReader::new(file);
    let mut count: u64 = 0;
    for line in reader.lines() {
        if line.is_ok() {
            count += 1;
        }
    }
    (count > 0).then_some(count)
}

fn find_vmlinux(release: Option<&str>) -> Option<String> {
    let r = release?;
    let candidates = [
        format!("/usr/lib/debug/lib/modules/{r}/vmlinux"),
        format!("/usr/lib/debug/boot/vmlinux-{r}"),
        format!("/boot/vmlinux-{r}"),
        format!("/usr/lib/modules/{r}/build/vmlinux"),
    ];
    candidates.into_iter().find(|p| Path::new(p).exists())
}

fn extract_build_id(path: &str) -> Option<String> {
    use std::process::Command;
    let output = Command::new("readelf").args(["-n", path]).output().ok()?;
    let text = String::from_utf8(output.stdout).ok()?;
    for line in text.lines() {
        if let Some(id) = line.strip_prefix("    Build ID: ") {
            return Some(id.trim().to_string());
        }
    }
    None
}

// SPDX-License-Identifier: GPL-2.0-only
// Copyright (C) 2026 rezky_nightky

//! Structured kernel capability types.
//!
//! ## Architecture
//!
//! Two layers:
//!   1. Compile-time contract: `CapabilityDescriptor` — defines what each
//!      probe promises to discover (used by the skeletal architecture).
//!   2. Runtime data: `ProbeResult` — what was actually discovered at
//!      runtime (used by real kernel modules with dynamic allocation).
//!
//! ## Domain Models
//!
//! Each capability domain owns its types. The Symbol Discovery domain
//! is the reference implementation — future providers (tracepoints,
//! kprobes, eBPF, livepatch) follow the same pattern:
//!   - One domain model struct (e.g., KernelSymbol)
//!   - One collection struct (e.g., KernelSymbolSet)
//!   - One status enum
//!   - One visibility/category enum as needed
//!
//! ## Design rules:
//!   - No rendering
//!   - No scoring
//!   - No CLI awareness
//!   - No human-readable formatting
//!   - One type per capability domain

use alloc::string::String;
use alloc::vec::Vec;

// ============================================================================
//  Generic Capability Infrastructure
// ============================================================================

/// Compile-time capability descriptor — defines what a probe discovers.
/// Used by the architectural skeleton. At runtime, this becomes a
/// `ProbeResult` with dynamically-allocated values.
pub struct CapabilityDescriptor {
    pub key: &'static str,
    pub value: &'static str,
}

/// Result of a single capability probe at runtime.
#[derive(Clone, Debug)]
pub struct ProbeResult {
    pub key: String,
    pub value: String,
    pub status: ProbeStatus,
    pub error: Option<String>,
}

/// Outcome of a capability probe.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProbeStatus {
    Success,
    Unavailable,
    Partial,
    Failed,
}

impl ProbeStatus {
    pub fn label(self) -> &'static str {
        match self {
            ProbeStatus::Success => "success",
            ProbeStatus::Unavailable => "unavailable",
            ProbeStatus::Partial => "partial",
            ProbeStatus::Failed => "failed",
        }
    }
}

/// Complete set of probe results from kernel module discovery.
pub struct ProbeResultSet {
    pub results: Vec<ProbeResult>,
}

// ============================================================================
//  Symbol Discovery Domain — Reference Implementation
// ============================================================================

/// A single kernel symbol with full ownership and visibility metadata.
///
/// Discovered at runtime via kallsyms. Each symbol carries:
///   - Its name and optional address
///   - Export status (normal, GPL-only, or internal)
///   - Module ownership (None = built-in / vmlinux)
///   - Namespace (EXPORT_SYMBOL_NS)
///
/// This is the reference domain model — all future providers follow
/// the same pattern: one struct per entity, rich metadata, no rendering.
pub struct KernelSymbol {
    /// Symbol name as it appears in kallsyms.
    pub name: String,
    /// Memory address (available only when CONFIG_KALLSYMS_ALL=y and
    /// /proc/sys/kernel/kptr_restrict permits).
    pub address: Option<usize>,
    /// Which module owns this symbol. None = built into vmlinux.
    pub owner: Option<String>,
    /// Export visibility level.
    pub visibility: SymbolVisibility,
    /// EXPORT_SYMBOL_NS namespace (e.g., "VFS", "BPF", "NET").
    pub namespace: Option<String>,
    /// Whether the symbol address is readable from userspace.
    pub address_available: bool,
    /// Module association metadata.
    pub metadata: SymbolMetadata,
}

/// Export visibility of a kernel symbol.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SymbolVisibility {
    /// EXPORT_SYMBOL — available to all GPL-compatible modules.
    Exported,
    /// EXPORT_SYMBOL_GPL — available to GPL-licensed modules only.
    GplOnly,
    /// Not exported — internal kernel use only.
    Internal,
    /// Visibility could not be determined (e.g., kallsyms without type info).
    Unknown,
}

impl SymbolVisibility {
    pub fn label(self) -> &'static str {
        match self {
            SymbolVisibility::Exported => "exported",
            SymbolVisibility::GplOnly => "gpl_only",
            SymbolVisibility::Internal => "internal",
            SymbolVisibility::Unknown => "unknown",
        }
    }
}

/// Per-symbol metadata collected during discovery.
pub struct SymbolMetadata {
    /// ELF section the symbol belongs to (e.g., ".text", ".data", ".bss").
    pub section: Option<String>,
    /// CRC checksum if CONFIG_MODVERSIONS is enabled.
    pub crc: Option<u32>,
    /// Symbol type ('t'=text, 'd'=data, 'b'=bss, 'r'=rodata, etc.).
    pub symbol_type: Option<char>,
}

/// Collection of kernel symbols with statistics and status.
///
/// Produced by the Symbol Discovery probe. Userspace reads the
/// aggregate statistics via /proc/zenvecha/symbols/ and individual
/// symbols via iteration (future: /proc/zenvecha/symbols/list).
pub struct KernelSymbolSet {
    /// Total symbols in kallsyms (including internal).
    pub total_symbols: u64,
    /// Symbols exported via EXPORT_SYMBOL or EXPORT_SYMBOL_GPL.
    pub exported_symbols: u64,
    /// Symbols exported via EXPORT_SYMBOL_GPL only.
    pub gpl_only_symbols: u64,
    /// Symbols owned by loadable kernel modules.
    pub module_owned_symbols: u64,
    /// Symbols using EXPORT_SYMBOL_NS (namespaced exports).
    pub namespaced_symbols: u64,
    /// Symbols built into vmlinux (not module-owned).
    pub vmlinux_symbols: u64,
    /// Status of the collection operation.
    pub collection_status: SymbolCollectionStatus,
    /// Which symbol capabilities are available on this kernel.
    pub capabilities: Vec<String>,
}

/// Outcome of symbol discovery.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SymbolCollectionStatus {
    /// All symbols enumerated with full metadata.
    Complete,
    /// Symbols enumerated but non-exported symbols hidden (CONFIG_KALLSYMS_ALL=n).
    ExportedOnly,
    /// Only visible symbols counted (kptr_restrict hides addresses).
    AddressesHidden,
    /// kallsyms not available (CONFIG_KALLSYMS=n).
    Unavailable,
}

impl SymbolCollectionStatus {
    pub fn label(self) -> &'static str {
        match self {
            SymbolCollectionStatus::Complete => "complete",
            SymbolCollectionStatus::ExportedOnly => "exported_only",
            SymbolCollectionStatus::AddressesHidden => "addresses_hidden",
            SymbolCollectionStatus::Unavailable => "unavailable",
        }
    }
}

// ============================================================================
//  Capability Categories
// ============================================================================

/// Capability categories the kernel module can discover.
pub enum CapabilityKind {
    Version,
    Symbols,
    Kallsyms,
    Btf,
    ModuleLoader,
    Tracing,
    Livepatch,
    RustSupport,
    Architecture,
    /// Reserved for future providers.
    Custom(&'static str),
}

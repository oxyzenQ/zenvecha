// Copyright (C) 2026 rezky_nightky
// SPDX-License-Identifier: GPL-3.0-only

//! Knowledge database — immutable Linux kernel intelligence.
//!
//! Centralized domain knowledge about kernel versions, configuration options,
//! Rust for Linux evolution, features, and subsystem capabilities.
//!
//! To add support for a new kernel version, add rules here.
//! No engine code changes required.

use super::rules::{
    ConfigExpectation, ConfigRule, FeatureRule, KernelRule, KnowledgeCategory, RuleImpact, RustRule,
};

/// The complete knowledge base.
pub struct KnowledgeBase {
    pub kernel_rules: Vec<KernelRule>,
    pub config_rules: Vec<ConfigRule>,
    pub rust_rules: Vec<RustRule>,
    pub feature_rules: Vec<FeatureRule>,
}

impl KnowledgeBase {
    /// Load the built-in knowledge base.
    pub fn load() -> Self {
        KnowledgeBase {
            kernel_rules: Self::kernel_rules(),
            config_rules: Self::config_rules(),
            rust_rules: Self::rust_rules(),
            feature_rules: Self::feature_rules(),
        }
    }

    // ========================================================================
    //  Kernel Version Rules
    // ========================================================================

    fn kernel_rules() -> Vec<KernelRule> {
        vec![
            KernelRule {
                id: "kver-6.18-rust-improved",
                min_version_major: 6,
                min_version_minor: 18,
                category: KnowledgeCategory::Rust,
                description: "Rust for Linux subsystem significantly improved",
                implications: &[
                    "Rust module development becomes practical",
                    "More kernel APIs available to Rust modules",
                    "Better compiler integration",
                ],
                impact: RuleImpact::Important,
            },
            KernelRule {
                id: "kver-6.20-ftrace-v2",
                min_version_major: 6,
                min_version_minor: 20,
                category: KnowledgeCategory::Feature,
                description: "ftrace API v2 introduced with improved hooking",
                implications: &[
                    "New function hooking API available",
                    "Better performance for tracepoints",
                    "Existing ftrace v1 hooks need migration",
                ],
                impact: RuleImpact::Important,
            },
            KernelRule {
                id: "kver-6.12-bpf-improvements",
                min_version_major: 6,
                min_version_minor: 12,
                category: KnowledgeCategory::Feature,
                description: "BPF verifier and JIT improvements",
                implications: &[
                    "More complex BPF programs supported",
                    "Better BPF CO-RE compatibility",
                ],
                impact: RuleImpact::Notable,
            },
            KernelRule {
                id: "kver-6.6-lts",
                min_version_major: 6,
                min_version_minor: 6,
                category: KnowledgeCategory::Kernel,
                description: "Linux 6.6 LTS — long-term support kernel",
                implications: &[
                    "Extended security support until Dec 2026",
                    "Stable API for enterprise deployments",
                    "Recommended for production systems",
                ],
                impact: RuleImpact::Important,
            },
            KernelRule {
                id: "kver-6.1-lts",
                min_version_major: 6,
                min_version_minor: 1,
                category: KnowledgeCategory::Kernel,
                description: "Linux 6.1 LTS — previous long-term support kernel",
                implications: &[
                    "Still receiving security updates",
                    "Consider upgrading to 6.6+ for Rust support",
                ],
                impact: RuleImpact::Notable,
            },
            KernelRule {
                id: "kver-6.0-rust-initial",
                min_version_major: 6,
                min_version_minor: 0,
                category: KnowledgeCategory::Rust,
                description: "Initial Rust for Linux merge (experimental)",
                implications: &[
                    "Rust support is experimental",
                    "Limited driver APIs only",
                    "Upgrade to 6.18+ for production Rust module development",
                ],
                impact: RuleImpact::Important,
            },
            KernelRule {
                id: "kver-5.15-lts",
                min_version_major: 5,
                min_version_minor: 15,
                category: KnowledgeCategory::Kernel,
                description: "Linux 5.15 LTS — legacy long-term support",
                implications: &[
                    "No Rust support available",
                    "Consider upgrading for modern features",
                ],
                impact: RuleImpact::Notable,
            },
            KernelRule {
                id: "kver-legacy",
                min_version_major: 0,
                min_version_minor: 0,
                category: KnowledgeCategory::Deprecation,
                description: "Pre-5.15 kernel — no Rust for Linux, limited BPF",
                implications: &[
                    "Rust kernel modules not supported",
                    "Limited BPF capabilities",
                    "Strongly recommend upgrading to 6.6+",
                ],
                impact: RuleImpact::Critical,
            },
        ]
    }

    // ========================================================================
    //  Configuration Rules
    // ========================================================================

    fn config_rules() -> Vec<ConfigRule> {
        vec![
            ConfigRule {
                id: "cfg-modules-required",
                config_key: "MODULES",
                expected: ConfigExpectation::Enabled,
                description: "CONFIG_MODULES — required for kernel module loading",
                implications: &[
                    "Without MODULES, no external kernel code can be loaded",
                    "Required for any kernel development workflow",
                ],
                impact: RuleImpact::Critical,
            },
            ConfigRule {
                id: "cfg-btf-co-re",
                config_key: "DEBUG_INFO_BTF",
                expected: ConfigExpectation::Enabled,
                description: "CONFIG_DEBUG_INFO_BTF — enables BPF CO-RE and type introspection",
                implications: &[
                    "BPF CO-RE programs can run across kernel versions",
                    "Type-aware kernel debugging becomes possible",
                    "Required for modern tracing tools",
                ],
                impact: RuleImpact::Important,
            },
            ConfigRule {
                id: "cfg-rust-enabled",
                config_key: "RUST",
                expected: ConfigExpectation::Enabled,
                description: "CONFIG_RUST — enables Rust kernel module support",
                implications: &[
                    "Rust kernel modules can be compiled and loaded",
                    "Memory-safe kernel development becomes possible",
                ],
                impact: RuleImpact::Important,
            },
            ConfigRule {
                id: "cfg-module-sig-security",
                config_key: "MODULE_SIG",
                expected: ConfigExpectation::Enabled,
                description: "CONFIG_MODULE_SIG — cryptographically verifies module integrity",
                implications: &[
                    "Modules must be signed to load on secure boot systems",
                    "Protects against malicious module injection",
                ],
                impact: RuleImpact::Notable,
            },
            ConfigRule {
                id: "cfg-kallsyms-debug",
                config_key: "KALLSYMS",
                expected: ConfigExpectation::Enabled,
                description: "CONFIG_KALLSYMS — exports kernel symbol table",
                implications: &[
                    "Required for symbol resolution and function hooking",
                    "Essential for kernel debugging and tracing",
                ],
                impact: RuleImpact::Important,
            },
            ConfigRule {
                id: "cfg-livepatch-capability",
                config_key: "LIVEPATCH",
                expected: ConfigExpectation::Enabled,
                description: "CONFIG_LIVEPATCH — enables runtime kernel patching",
                implications: &[
                    "Security fixes can be applied without reboot",
                    "Critical for zero-downtime maintenance",
                    "Zenvecha's core capability depends on this",
                ],
                impact: RuleImpact::Important,
            },
            ConfigRule {
                id: "cfg-bpf-required",
                config_key: "BPF",
                expected: ConfigExpectation::Enabled,
                description: "CONFIG_BPF — enables Berkeley Packet Filter subsystem",
                implications: &[
                    "Required for modern tracing (bpftrace, bcc)",
                    "Required for BPF CO-RE",
                    "Foundation for many kernel observability tools",
                ],
                impact: RuleImpact::Important,
            },
        ]
    }

    // ========================================================================
    //  Rust for Linux Rules
    // ========================================================================

    fn rust_rules() -> Vec<RustRule> {
        vec![
            RustRule {
                id: "rust-6.18-production",
                min_version_major: 6,
                min_version_minor: 18,
                description: "Rust for Linux reaches production readiness",
                implications: &[
                    "Full driver API surface available",
                    "Stable Rust ABI for kernel modules",
                    "Community-adopted best practices established",
                ],
                impact: RuleImpact::Important,
            },
            RustRule {
                id: "rust-6.12-expanded",
                min_version_major: 6,
                min_version_minor: 12,
                description: "Rust kernel abstractions significantly expanded",
                implications: &[
                    "More kernel subsystems expose Rust APIs",
                    "Bindgen tooling matured",
                ],
                impact: RuleImpact::Notable,
            },
            RustRule {
                id: "rust-6.1-experimental",
                min_version_major: 6,
                min_version_minor: 1,
                description: "Rust for Linux available as experimental feature",
                implications: &[
                    "CONFIG_RUST=m may be available",
                    "Limited API surface",
                    "May require specific rustc version",
                ],
                impact: RuleImpact::Informational,
            },
        ]
    }

    // ========================================================================
    //  Feature Rules
    // ========================================================================

    fn feature_rules() -> Vec<FeatureRule> {
        vec![
            FeatureRule {
                id: "feat-bpf-core",
                name: "BPF CO-RE",
                min_version_major: 5,
                min_version_minor: 4,
                requires_config: Some("DEBUG_INFO_BTF"),
                category: KnowledgeCategory::Feature,
                description: "Compile Once, Run Everywhere BPF — portable BPF programs",
                implications: &[
                    "BPF programs work across kernel versions without recompilation",
                    "Required for portable tracing tools",
                ],
                impact: RuleImpact::Important,
            },
            FeatureRule {
                id: "feat-io-uring",
                name: "io_uring",
                min_version_major: 5,
                min_version_minor: 1,
                requires_config: None,
                category: KnowledgeCategory::Feature,
                description: "High-performance async I/O interface",
                implications: &[
                    "Significantly faster disk and network I/O",
                    "Available on most modern kernels",
                ],
                impact: RuleImpact::Informational,
            },
            FeatureRule {
                id: "feat-ebpf-trampoline",
                name: "eBPF Trampoline",
                min_version_major: 5,
                min_version_minor: 11,
                requires_config: Some("BPF"),
                category: KnowledgeCategory::Feature,
                description: "eBPF trampoline — efficient function hooking mechanism",
                implications: &[
                    "Lower overhead function hooking vs kprobes",
                    "Enables faster tracing and live patching",
                ],
                impact: RuleImpact::Notable,
            },
            FeatureRule {
                id: "feat-lsm-bpf",
                name: "BPF LSM",
                min_version_major: 5,
                min_version_minor: 7,
                requires_config: Some("BPF"),
                category: KnowledgeCategory::Security,
                description: "Linux Security Module hooks via BPF",
                implications: &[
                    "Custom security policies without kernel rebuild",
                    "Runtime security monitoring capability",
                ],
                impact: RuleImpact::Notable,
            },
            FeatureRule {
                id: "feat-btf-type-format",
                name: "BTF Type Format",
                min_version_major: 5,
                min_version_minor: 2,
                requires_config: Some("DEBUG_INFO_BTF"),
                category: KnowledgeCategory::Feature,
                description: "BPF Type Format — compact type information for BPF",
                implications: &[
                    "Enables type-aware kernel introspection",
                    "Foundation for BPF CO-RE",
                ],
                impact: RuleImpact::Notable,
            },
        ]
    }
}

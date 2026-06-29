// Copyright (C) 2026 rezky_nightky
// SPDX-License-Identifier: GPL-3.0-only

//! Capability trait and Registry.
//!
//! Capabilities detect kernel facts. They never print, score, or recommend.
//! The Registry owns capability execution and collects Evidence.

use super::evidence::Evidence;

/// A kernel capability probe.
///
/// Each implementation detects one aspect of the kernel environment.
/// Capabilities MUST NOT:
/// - Print to stdout/stderr
/// - Compute scores
/// - Generate recommendations
/// - Modify the system
///
/// Capabilities SHOULD:
/// - Be idempotent
/// - Return quickly (no blocking I/O beyond filesystem reads)
/// - Be testable in isolation
pub trait Capability {
    /// Unique identifier (e.g., "kernel.release").
    fn id(&self) -> &'static str;

    /// Human-readable label for display.
    fn label(&self) -> &'static str;

    /// Execute the probe. Returns Evidence.
    fn probe(&self) -> Evidence;
}

/// Registry of all capabilities.
///
/// Commands select which capabilities to run. The registry
/// owns execution and collects results.
pub struct Registry {
    capabilities: Vec<Box<dyn Capability>>,
}

impl Registry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Registry {
            capabilities: Vec::new(),
        }
    }

    /// Register a capability.
    pub fn register(&mut self, cap: Box<dyn Capability>) {
        self.capabilities.push(cap);
    }

    /// Run all registered capabilities and return evidence.
    pub fn run_all(&self) -> Vec<Evidence> {
        self.capabilities.iter().map(|c| c.probe()).collect()
    }

    /// Run a subset of capabilities by ID prefix.
    ///
    /// Example: `run("config.")` runs all config-related capabilities.
    pub fn run(&self, prefix: &str) -> Vec<Evidence> {
        self.capabilities
            .iter()
            .filter(|c| c.id().starts_with(prefix))
            .map(|c| c.probe())
            .collect()
    }

    /// Look up a single capability by exact ID.
    pub fn get(&self, id: &str) -> Option<&dyn Capability> {
        self.capabilities
            .iter()
            .find(|c| c.id() == id)
            .map(|c| c.as_ref())
    }
}

impl Default for Registry {
    fn default() -> Self {
        let mut reg = Registry::new();
        register_all(&mut reg);
        reg
    }
}

/// Register every known capability.
///
/// This is the single place where new capabilities are wired in.
/// To add a capability:
///   1. Implement the `Capability` trait
///   2. Call `reg.register(Box::new(YourCap))` here
///      Nothing else needs to change.
pub fn register_all(reg: &mut Registry) {
    use super::caps;

    // Kernel identity
    reg.register(Box::new(caps::KernelRelease));
    reg.register(Box::new(caps::KernelArchitecture));
    reg.register(Box::new(caps::KernelDistro));
    reg.register(Box::new(caps::CompilerVersion));
    reg.register(Box::new(caps::CompilerAbi));

    // Configuration
    reg.register(Box::new(caps::ConfigSource));
    reg.register(Box::new(caps::ConfigModules));
    reg.register(Box::new(caps::ConfigModuleSig));
    reg.register(Box::new(caps::ConfigKallsyms));
    reg.register(Box::new(caps::ConfigKallsymsAll));
    reg.register(Box::new(caps::ConfigBpf));
    reg.register(Box::new(caps::ConfigDebugInfoBtf));
    reg.register(Box::new(caps::ConfigRust));
    reg.register(Box::new(caps::ConfigRustAvailable));
    reg.register(Box::new(caps::ConfigLivepatch));

    // Module environment
    reg.register(Box::new(caps::ModuleSupport));
    reg.register(Box::new(caps::ModuleSigning));
    reg.register(Box::new(caps::ModuleLoader));

    // Symbols
    reg.register(Box::new(caps::KallsymsInfo));
    reg.register(Box::new(caps::SymbolCount));
    reg.register(Box::new(caps::VmlinuxInfo));
    reg.register(Box::new(caps::ModuleSymvers));

    // Debug
    reg.register(Box::new(caps::DebugBtf));
    reg.register(Box::new(caps::DebugDwarf));

    // ABI
    reg.register(Box::new(caps::AbiInfo));

    // Toolchain
    reg.register(Box::new(caps::RustcInstalled));
    reg.register(Box::new(caps::BindgenInstalled));
    reg.register(Box::new(caps::LlvmInstalled));
    reg.register(Box::new(caps::MakeInstalled));
    reg.register(Box::new(caps::GccInstalled));

    // Build environment
    reg.register(Box::new(caps::HeaderIntegrity));
    reg.register(Box::new(caps::BuildDirectory));
    reg.register(Box::new(caps::SourceDirectory));
    reg.register(Box::new(caps::CompileCommands));

    // Filesystem
    reg.register(Box::new(caps::DebugfsMounted));
    reg.register(Box::new(caps::TracefsMounted));

    // Kernel module capability bridge (Phase 7)
    reg.register(Box::new(caps::kernel_cap::KernelModuleStatus));
    reg.register(Box::new(caps::kernel_cap::KernelVersionFromModule));

    // Symbol Discovery — reference runtime provider
    reg.register(Box::new(caps::kernel_cap::KernelSymbolTotal));
    reg.register(Box::new(caps::kernel_cap::KernelSymbolExported));
    reg.register(Box::new(caps::kernel_cap::KernelSymbolGplOnly));
    reg.register(Box::new(caps::kernel_cap::KernelSymbolInternal));
    reg.register(Box::new(caps::kernel_cap::KernelSymbolModuleOwned));
    reg.register(Box::new(caps::kernel_cap::KernelSymbolVmlinux));
    reg.register(Box::new(caps::kernel_cap::KernelSymbolNamespaced));
    reg.register(Box::new(caps::kernel_cap::KernelSymbolKallsyms));
    reg.register(Box::new(caps::kernel_cap::KernelSymbolKallsymsAll));
    reg.register(Box::new(caps::kernel_cap::KernelSymbolKptrRestrict));
    reg.register(Box::new(caps::kernel_cap::KernelSymbolCollection));

    reg.register(Box::new(caps::kernel_cap::KernelBtfStatus));
    reg.register(Box::new(caps::kernel_cap::KernelModuleLoader));
    reg.register(Box::new(caps::kernel_cap::KernelTracingFtrace));
    reg.register(Box::new(caps::kernel_cap::KernelTracingKprobes));

    // Security (Phase 7 Milestone E)
    reg.register(Box::new(caps::kernel_cap::KernelLockdown));
    reg.register(Box::new(caps::kernel_cap::KernelActiveLsms));
    reg.register(Box::new(caps::kernel_cap::KernelKaslr));

    // Scheduler (Phase 7 Milestone E)
    reg.register(Box::new(caps::kernel_cap::KernelSchedulerClasses));
    reg.register(Box::new(caps::kernel_cap::KernelPreemption));

    // Memory (Phase 7 Milestone E)
    reg.register(Box::new(caps::kernel_cap::KernelPageSize));
    reg.register(Box::new(caps::kernel_cap::KernelHugePages));
    reg.register(Box::new(caps::kernel_cap::KernelMemoryModel));

    // Tracepoints (Phase 7 Milestone E)
    reg.register(Box::new(caps::kernel_cap::KernelTracepointCount));
    reg.register(Box::new(caps::kernel_cap::KernelTracepointSubsystems));
}

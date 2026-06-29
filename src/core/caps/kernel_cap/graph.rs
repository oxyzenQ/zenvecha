// Copyright (C) 2026 rezky_nightky
// SPDX-License-Identifier: GPL-3.0-only

//! Kernel Capability Graph — structural dependency declarations.
//!
//! Describes capability relationships as a directed graph.
//! This is purely declarative — it never evaluates availability,
//! computes compatibility, or makes recommendations.
//!
//! ## What the graph IS
//!
//!   - A static declaration of what depends on what
//!   - Queryable: "what does Livepatch need?" → [Modules, Kallsyms, ftrace]
//!   - Queryable: "what breaks if Modules are missing?" → [Livepatch, BTF-loading]
//!   - Basis for future impact analysis engines (not yet built)
//!
//! ## What the graph is NOT
//!
//!   - NOT a runtime evaluator (that's Compatibility Engine's job)
//!   - NOT a decision maker (that's Decision Engine's job)
//!   - NOT a renderer (never formats output)
//!   - NOT a replacement for Evidence or Pipeline
//!
//! ## Architecture Position
//!
//!   Capability Providers → Evidence → Pipeline → Analysis → ...
//!   Capability Graph     → (consumed by future engines, not Phase 6)
//!
//! ## Adding a capability node
//!
//!   Add one entry in `known_graph()` — 3-5 lines per capability.
//!   Zero existing code changes required.

use std::collections::HashMap;

// ============================================================================
//  Domain Models
// ============================================================================

/// A dependency from one capability to another.
#[derive(Clone, Debug)]
pub struct CapabilityDependency {
    /// The capability ID this node depends on.
    pub target_id: &'static str,
    /// The kind of dependency.
    pub kind: DependencyKind,
    /// Human-readable explanation of why this dependency exists.
    pub reason: &'static str,
}

/// How strongly one capability depends on another.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DependencyKind {
    /// Hard requirement — capability is non-functional without this.
    Required,
    /// Soft requirement — capability is degraded without this.
    Optional,
    /// Enhancement — capability gains additional features with this.
    Enhances,
}

impl DependencyKind {
    pub fn label(self) -> &'static str {
        match self {
            DependencyKind::Required => "required",
            DependencyKind::Optional => "optional",
            DependencyKind::Enhances => "enhances",
        }
    }
}

/// A node in the capability dependency graph.
#[derive(Clone, Debug)]
pub struct CapabilityNode {
    /// Unique capability ID (matches Evidence id).
    pub id: &'static str,
    /// Human-readable label.
    pub label: &'static str,
    /// Capabilities this node depends on.
    pub depends_on: Vec<CapabilityDependency>,
    /// Category for grouping.
    pub category: CapabilityCategory,
}

/// High-level category of a capability.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CapabilityCategory {
    /// Kernel configuration options.
    Config,
    /// Kernel infrastructure (kallsyms, modules, BTF).
    Infrastructure,
    /// Tracing and debugging frameworks.
    Tracing,
    /// Security features.
    Security,
    /// Userspace toolchain.
    Toolchain,
    /// Build environment.
    BuildEnv,
    /// Kernel module (zenvecha) capability.
    KernelModule,
}

impl CapabilityCategory {
    pub fn label(self) -> &'static str {
        match self {
            CapabilityCategory::Config => "config",
            CapabilityCategory::Infrastructure => "infrastructure",
            CapabilityCategory::Tracing => "tracing",
            CapabilityCategory::Security => "security",
            CapabilityCategory::Toolchain => "toolchain",
            CapabilityCategory::BuildEnv => "build_env",
            CapabilityCategory::KernelModule => "kernel_module",
        }
    }
}

/// The complete capability dependency graph.
///
/// Build once via `CapabilityGraph::known()`, query repeatedly.
/// Immutable after construction.
#[derive(Clone, Debug)]
pub struct CapabilityGraph {
    nodes: Vec<CapabilityNode>,
    index: HashMap<&'static str, usize>,
}

impl CapabilityGraph {
    /// Build the known capability graph with all declared dependencies.
    ///
    /// This is the single source of truth for capability relationships.
    /// Adding a new capability = add one entry here.
    pub fn known() -> Self {
        let nodes = known_graph();
        let mut index = HashMap::with_capacity(nodes.len());
        for (i, node) in nodes.iter().enumerate() {
            index.insert(node.id, i);
        }
        CapabilityGraph { nodes, index }
    }

    /// Find a node by capability ID.
    pub fn node(&self, id: &str) -> Option<&CapabilityNode> {
        self.index.get(id).map(|&i| &self.nodes[i])
    }

    /// List all capabilities that `id` depends on.
    pub fn dependencies_of(&self, id: &str) -> Vec<&CapabilityDependency> {
        self.node(id)
            .map(|n| n.depends_on.iter().collect())
            .unwrap_or_default()
    }

    /// List all capabilities that depend on `id` (reverse lookup).
    pub fn dependents_of(&self, id: &str) -> Vec<&CapabilityNode> {
        self.nodes
            .iter()
            .filter(|n| n.depends_on.iter().any(|d| d.target_id == id))
            .collect()
    }

    /// Walk the full dependency chain for a capability (recursive).
    /// Returns dependencies in topological order (prerequisites first).
    pub fn dependency_chain(&self, id: &str) -> Vec<&CapabilityNode> {
        let mut visited = Vec::new();
        let mut stack: Vec<&str> = vec![id];
        while let Some(current) = stack.pop() {
            if visited.contains(&current) {
                continue;
            }
            if let Some(node) = self.node(current) {
                visited.push(node.id);
                for dep in node.depends_on.iter().rev() {
                    if !visited.contains(&dep.target_id) {
                        stack.push(dep.target_id);
                    }
                }
            }
        }
        visited.reverse();
        visited.iter().filter_map(|id| self.node(id)).collect()
    }

    /// Return all nodes in the graph.
    pub fn all_nodes(&self) -> &[CapabilityNode] {
        &self.nodes
    }

    /// Number of nodes in the graph.
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Whether the graph is empty.
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }
}

// ============================================================================
//  Known Capability Graph — the single source of truth
// ============================================================================

/// Declare every known capability and its dependencies.
///
/// Structure:
///   CapabilityNode::new("id", "label", Category)
///       .requires("dep_id", "reason")
///       .optional("dep_id", "reason")
///       .enhanced_by("dep_id", "reason")
///
/// Add one entry per capability. Zero existing code changes.
fn known_graph() -> Vec<CapabilityNode> {
    use CapabilityCategory::*;
    use DependencyKind::*;

    vec![
        // ── Config (leaf nodes — no dependencies) ──
        node("config.MODULES", "Module Support", Config),
        node("config.MODULE_SIG", "Module Signing", Config),
        node("config.KALLSYMS", "kallsyms", Config),
        node("config.KALLSYMS_ALL", "kallsyms All Symbols", Config),
        node("config.DEBUG_INFO_BTF", "BTF Debug Info", Config),
        node("config.BPF", "BPF Support", Config),
        node("config.RUST", "Rust for Linux", Config),
        node("config.LIVEPATCH", "Livepatch Support", Config),
        node("config.FUNCTION_TRACER", "Function Tracer", Config),
        node("config.TRACEPOINTS", "Tracepoints", Config),
        node("config.KPROBES", "kprobes", Config),
        node("config.UPROBES", "uprobes", Config),
        // ── Infrastructure ──
        node("symbols.kallsyms", "kallsyms Available", Infrastructure).dep(
            Required,
            "config.KALLSYMS",
            "Needs CONFIG_KALLSYMS=y",
        ),
        node(
            "symbols.kallsyms_all",
            "kallsyms All Symbols",
            Infrastructure,
        )
        .dep(
            Required,
            "config.KALLSYMS_ALL",
            "Needs CONFIG_KALLSYMS_ALL=y",
        )
        .dep(
            Required,
            "config.KALLSYMS",
            "kallsyms must be enabled first",
        ),
        node("kernel.modules", "Module Loader", Infrastructure).dep(
            Required,
            "config.MODULES",
            "Needs CONFIG_MODULES=y",
        ),
        node("btf.available", "BTF Available", Infrastructure).dep(
            Required,
            "config.DEBUG_INFO_BTF",
            "Needs CONFIG_DEBUG_INFO_BTF=y",
        ),
        node("bpf.available", "BPF Available", Infrastructure)
            .dep(Required, "config.BPF", "Needs CONFIG_BPF=y")
            .dep(Enhances, "btf.available", "BTF enables BPF CO-RE"),
        node("modules.signing", "Module Signing", Infrastructure)
            .dep(Required, "config.MODULE_SIG", "Needs CONFIG_MODULE_SIG=y")
            .dep(Required, "config.MODULES", "Modules must be enabled"),
        // ── Tracing ──
        node("tracing.ftrace", "Function Tracer", Tracing).dep(
            Required,
            "config.FUNCTION_TRACER",
            "Needs CONFIG_FUNCTION_TRACER=y",
        ),
        node("tracing.tracepoints", "Tracepoints", Tracing).dep(
            Required,
            "config.TRACEPOINTS",
            "Needs CONFIG_TRACEPOINTS=y",
        ),
        node("tracing.kprobes", "kprobes", Tracing).dep(
            Required,
            "config.KPROBES",
            "Needs CONFIG_KPROBES=y",
        ),
        node("tracing.kretprobes", "kretprobes", Tracing).dep(
            Required,
            "config.KPROBES",
            "Needs CONFIG_KPROBES=y for kretprobe support",
        ),
        node("tracing.uprobes", "uprobes", Tracing).dep(
            Required,
            "config.UPROBES",
            "Needs CONFIG_UPROBES=y",
        ),
        // ── Security ──
        node("security.livepatch", "Livepatch", Security)
            .dep(Required, "config.LIVEPATCH", "Needs CONFIG_LIVEPATCH=y")
            .dep(
                Required,
                "config.MODULES",
                "Livepatch uses module infrastructure",
            )
            .dep(
                Required,
                "tracing.ftrace",
                "Livepatch uses ftrace for function redirection",
            )
            .dep(
                Optional,
                "symbols.kallsyms_all",
                "Full kallsyms improves patch reliability",
            ),
        // ── Kernel Module Capabilities (Phase 7) ──
        node("kernel.module_loaded", "Zenvecha Module", KernelModule).dep(
            Required,
            "config.MODULES",
            "Kernel module loader must be enabled",
        ),
        node("kernel.symbols.total", "Symbol Discovery", KernelModule)
            .dep(
                Required,
                "kernel.module_loaded",
                "Requires Zenvecha kernel module",
            )
            .dep(
                Required,
                "config.KALLSYMS",
                "Symbol iteration needs kallsyms",
            ),
        node("kernel.btf.module", "BTF Discovery", KernelModule)
            .dep(
                Required,
                "kernel.module_loaded",
                "Requires Zenvecha kernel module",
            )
            .dep(Required, "btf.available", "BTF vmlinux must be available"),
        node("kernel.tracing.ftrace", "ftrace Discovery", KernelModule)
            .dep(
                Required,
                "kernel.module_loaded",
                "Requires Zenvecha kernel module",
            )
            .dep(
                Required,
                "tracing.ftrace",
                "ftrace infrastructure must be available",
            ),
        node("kernel.tracing.kprobes", "kprobes Discovery", KernelModule)
            .dep(
                Required,
                "kernel.module_loaded",
                "Requires Zenvecha kernel module",
            )
            .dep(
                Required,
                "tracing.kprobes",
                "kprobes infrastructure must be available",
            ),
        // ── Rust for Linux ──
        node("rust.available", "Rust for Linux Available", Toolchain).dep(
            Required,
            "config.RUST",
            "Needs CONFIG_RUST=y",
        ),
        node("rust.bindgen", "bindgen Available", Toolchain).dep(
            Optional,
            "rust.available",
            "bindgen is most useful with Rust support",
        ),
        // ── Build Environment ──
        node("build.headers", "Kernel Headers", BuildEnv),
        node("build.toolchain.gcc", "C Compiler", BuildEnv),
        node("build.toolchain.rustc", "Rust Compiler", BuildEnv).dep(
            Enhances,
            "rust.available",
            "Rust compiler enables Rust module builds",
        ),
    ]
}

// ============================================================================
//  Builder helpers
// ============================================================================

fn node(id: &'static str, label: &'static str, category: CapabilityCategory) -> CapabilityNode {
    CapabilityNode {
        id,
        label,
        depends_on: Vec::new(),
        category,
    }
}

impl CapabilityNode {
    fn dep(mut self, kind: DependencyKind, target_id: &'static str, reason: &'static str) -> Self {
        self.depends_on.push(CapabilityDependency {
            target_id,
            kind,
            reason,
        });
        self
    }

    #[allow(dead_code)]
    fn enhanced_by(self, target_id: &'static str, reason: &'static str) -> Self {
        self.dep(DependencyKind::Enhances, target_id, reason)
    }
}

// ============================================================================
//  Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graph_construction() {
        let g = CapabilityGraph::known();
        assert!(g.len() > 10, "graph should have multiple nodes");
    }

    #[test]
    fn test_livepatch_dependencies() {
        let g = CapabilityGraph::known();
        let deps = g.dependencies_of("security.livepatch");
        let dep_ids: Vec<&str> = deps.iter().map(|d| d.target_id).collect();

        // Livepatch must depend on config.LIVEPATCH
        assert!(
            dep_ids.contains(&"config.LIVEPATCH"),
            "livepatch must depend on CONFIG_LIVEPATCH"
        );
        // Livepatch must depend on modules
        assert!(
            dep_ids.contains(&"config.MODULES"),
            "livepatch must depend on module infrastructure"
        );
        // Livepatch must depend on ftrace
        assert!(
            dep_ids.contains(&"tracing.ftrace"),
            "livepatch must depend on ftrace"
        );
    }

    #[test]
    fn test_dependents_of_modules() {
        let g = CapabilityGraph::known();
        let dependents = g.dependents_of("config.MODULES");

        // Modules should have multiple dependents
        assert!(
            !dependents.is_empty(),
            "many capabilities should depend on modules"
        );

        let dep_ids: Vec<&str> = dependents.iter().map(|n| n.id).collect();
        assert!(
            dep_ids.contains(&"kernel.modules"),
            "module loader depends on CONFIG_MODULES"
        );
    }

    #[test]
    fn test_dependency_chain() {
        let g = CapabilityGraph::known();
        let chain = g.dependency_chain("kernel.tracing.ftrace");

        // Chain should include: config.FUNCTION_TRACER → tracing.ftrace → kernel.tracing.ftrace
        let ids: Vec<&str> = chain.iter().map(|n| n.id).collect();
        assert!(
            ids.contains(&"config.FUNCTION_TRACER"),
            "chain missing ftrace config"
        );
        assert!(
            ids.contains(&"tracing.ftrace"),
            "chain missing ftrace infra"
        );
        assert!(
            ids.contains(&"kernel.tracing.ftrace"),
            "chain missing kernel module probe"
        );
    }

    #[test]
    fn test_btf_dependency_chain() {
        let g = CapabilityGraph::known();
        let chain = g.dependency_chain("kernel.btf.module");

        let ids: Vec<&str> = chain.iter().map(|n| n.id).collect();
        assert!(
            ids.contains(&"config.DEBUG_INFO_BTF"),
            "BTF requires CONFIG_DEBUG_INFO_BTF"
        );
        assert!(ids.contains(&"btf.available"), "BTF module needs BTF infra");
        assert!(
            ids.contains(&"kernel.btf.module"),
            "BTF module probe itself"
        );
    }
}

// Copyright (C) 2026 rezky_nightky
// SPDX-License-Identifier: GPL-3.0-only

//! Kernel identity capabilities — release, architecture, distro, compiler.

use crate::core::capability::Capability;
use crate::core::evidence::{Confidence, Evidence, EvidenceValue};
use crate::system;

pub struct KernelRelease;
impl Capability for KernelRelease {
    fn id(&self) -> &'static str {
        "kernel.release"
    }
    fn label(&self) -> &'static str {
        "Kernel Release"
    }
    fn probe(&self) -> Evidence {
        let r = system::kernel::kernel_release();
        match r {
            Some(v) => Evidence::present(self.id(), EvidenceValue::Text(Some(v))),
            None => Evidence::missing(self.id(), EvidenceValue::Text(None)),
        }
    }
}

pub struct KernelArchitecture;
impl Capability for KernelArchitecture {
    fn id(&self) -> &'static str {
        "kernel.arch"
    }
    fn label(&self) -> &'static str {
        "Architecture"
    }
    fn probe(&self) -> Evidence {
        let a = system::kernel::architecture();
        Evidence::present(self.id(), EvidenceValue::Text(a))
    }
}

pub struct KernelDistro;
impl Capability for KernelDistro {
    fn id(&self) -> &'static str {
        "kernel.distro"
    }
    fn label(&self) -> &'static str {
        "Distribution"
    }
    fn probe(&self) -> Evidence {
        let d = system::kernel::detect_distro();
        Evidence::present(self.id(), EvidenceValue::Text(d))
    }
}

pub struct CompilerVersion;
impl Capability for CompilerVersion {
    fn id(&self) -> &'static str {
        "kernel.compiler"
    }
    fn label(&self) -> &'static str {
        "Compiler"
    }
    fn probe(&self) -> Evidence {
        let c = system::kernel::compiler_version();
        if let Some(v) = c {
            Evidence::present(self.id(), EvidenceValue::Text(Some(v)))
        } else {
            Evidence::missing(self.id(), EvidenceValue::Text(None))
        }
    }
}

pub struct CompilerAbi;
impl Capability for CompilerAbi {
    fn id(&self) -> &'static str {
        "compiler.abi"
    }
    fn label(&self) -> &'static str {
        "Compiler ABI"
    }
    fn probe(&self) -> Evidence {
        let tools = system::toolchain::inspect_toolchain();
        let abi = system::compiler::compare_compilers(&tools.rustc);
        let kernel_comp = abi.kernel_compiler.unwrap_or_else(|| "Unknown".into());
        let confidence = match abi.gcc_compat {
            system::compiler::CompilerCompat::Compatible => Confidence::High,
            system::compiler::CompilerCompat::Probably => Confidence::Medium,
            system::compiler::CompilerCompat::Unknown => Confidence::Low,
            system::compiler::CompilerCompat::NotCompatible => Confidence::Low,
        };
        Evidence::present(self.id(), EvidenceValue::Literal(kernel_comp))
            .with_confidence(confidence)
    }
}

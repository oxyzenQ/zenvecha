// Copyright (C) 2026 rezky_nightky
// SPDX-License-Identifier: GPL-3.0-only

//! Toolchain capabilities — rustc, bindgen, llvm, make, gcc.

use crate::core::capability::Capability;
use crate::core::caps::which;
use crate::core::evidence::{Evidence, EvidenceValue};
use crate::system;

fn tool_field(tools: &system::toolchain::ToolchainInfo, field: &str) -> EvidenceValue {
    match field {
        "rustc" => EvidenceValue::Bool(tools.rustc.is_some()),
        "bindgen" => EvidenceValue::Bool(tools.bindgen.is_some()),
        "llvm" => EvidenceValue::Bool(tools.llvm_version.is_some()),
        _ => EvidenceValue::Bool(false),
    }
}

macro_rules! tool_cap {
    ($name:ident, $id:literal, $label:literal, $field:literal) => {
        pub struct $name;
        impl Capability for $name {
            fn id(&self) -> &'static str {
                $id
            }
            fn label(&self) -> &'static str {
                $label
            }
            fn probe(&self) -> Evidence {
                let tools = system::toolchain::inspect_toolchain();
                Evidence::present(self.id(), tool_field(&tools, $field))
            }
        }
    };
}

tool_cap!(RustcInstalled, "toolchain.rustc", "rustc", "rustc");
tool_cap!(BindgenInstalled, "toolchain.bindgen", "bindgen", "bindgen");
tool_cap!(LlvmInstalled, "toolchain.llvm", "llvm", "llvm");

pub struct MakeInstalled;
impl Capability for MakeInstalled {
    fn id(&self) -> &'static str {
        "toolchain.make"
    }
    fn label(&self) -> &'static str {
        "make"
    }
    fn probe(&self) -> Evidence {
        let ok = which("make");
        Evidence::present(self.id(), EvidenceValue::Bool(ok))
    }
}

pub struct GccInstalled;
impl Capability for GccInstalled {
    fn id(&self) -> &'static str {
        "toolchain.gcc"
    }
    fn label(&self) -> &'static str {
        "gcc"
    }
    fn probe(&self) -> Evidence {
        let ok = system::kernel::compiler_available();
        Evidence::present(self.id(), EvidenceValue::Bool(ok))
    }
}

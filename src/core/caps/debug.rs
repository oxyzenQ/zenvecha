// Copyright (C) 2026 rezky_nightky
// SPDX-License-Identifier: GPL-3.0-only

//! Debug capabilities — BTF, DWARF.

use crate::core::capability::Capability;
use crate::core::evidence::{Evidence, EvidenceValue};
use crate::system;

pub struct DebugBtf;
impl Capability for DebugBtf {
    fn id(&self) -> &'static str {
        "debug.btf"
    }
    fn label(&self) -> &'static str {
        "BTF"
    }
    fn probe(&self) -> Evidence {
        let dbg = system::btf::inspect_debug();
        Evidence::present(self.id(), EvidenceValue::Bool(dbg.btf_available))
    }
}

pub struct DebugDwarf;
impl Capability for DebugDwarf {
    fn id(&self) -> &'static str {
        "debug.dwarf"
    }
    fn label(&self) -> &'static str {
        "DWARF"
    }
    fn probe(&self) -> Evidence {
        let dbg = system::btf::inspect_debug();
        Evidence::present(self.id(), EvidenceValue::Bool(dbg.dwarf_available))
    }
}

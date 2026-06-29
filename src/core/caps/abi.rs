// Copyright (C) 2026 rezky_nightky
// SPDX-License-Identifier: GPL-3.0-only

//! ABI capability — kernel ABI information.

use crate::core::capability::Capability;
use crate::core::caps::read_config;
use crate::core::evidence::{Evidence, EvidenceValue};
use crate::system;

pub struct AbiInfo;
impl Capability for AbiInfo {
    fn id(&self) -> &'static str {
        "abi.info"
    }
    fn label(&self) -> &'static str {
        "ABI Info"
    }
    fn probe(&self) -> Evidence {
        let cfg = read_config();
        let abi = system::abi::inspect_abi(cfg.as_deref());
        let value = format!(
            "utsrelease={} vermagic={} layout={} compression={}",
            abi.utsrelease.as_deref().unwrap_or("Unknown"),
            abi.vermagic.as_deref().unwrap_or("Unknown"),
            abi.module_layout_version.as_deref().unwrap_or("Unknown"),
            abi.module_compression,
        );
        Evidence::present(self.id(), EvidenceValue::Literal(value))
    }
}

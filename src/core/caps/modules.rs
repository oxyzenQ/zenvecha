// Copyright (C) 2026 rezky_nightky
// SPDX-License-Identifier: GPL-3.0-only

//! Module environment capabilities — support, signing, loader.

use crate::core::capability::Capability;
use crate::core::caps::{cfg_val, read_config};
use crate::core::evidence::{Evidence, EvidenceValue};
use crate::system;

pub struct ModuleSupport;
impl Capability for ModuleSupport {
    fn id(&self) -> &'static str {
        "modules.support"
    }
    fn label(&self) -> &'static str {
        "Module Support"
    }
    fn probe(&self) -> Evidence {
        let cfg = read_config();
        let info = system::modules::inspect_modules(cfg.as_deref());
        let has_mod = cfg_val("MODULES").is_enabled();
        let dev_ok = has_mod && info.headers_available && system::kernel::compiler_available();
        let desc = match (has_mod, dev_ok) {
            (true, true) => "supported with development environment",
            (true, false) => "supported, development not ready",
            (false, _) => "not supported",
        };
        Evidence::present(self.id(), EvidenceValue::Literal(desc.into()))
    }
}

pub struct ModuleSigning;
impl Capability for ModuleSigning {
    fn id(&self) -> &'static str {
        "modules.signing"
    }
    fn label(&self) -> &'static str {
        "Module Signing"
    }
    fn probe(&self) -> Evidence {
        let cfg = read_config();
        let info = system::modules::inspect_modules(cfg.as_deref());
        Evidence::present(
            self.id(),
            EvidenceValue::Bool(info.signing_enabled.unwrap_or(false)),
        )
    }
}

pub struct ModuleLoader;
impl Capability for ModuleLoader {
    fn id(&self) -> &'static str {
        "modules.loader"
    }
    fn label(&self) -> &'static str {
        "Module Loader"
    }
    fn probe(&self) -> Evidence {
        let cfg = read_config();
        let loader = system::moduleinfo::inspect_loader(cfg.as_deref());
        Evidence::present(
            self.id(),
            EvidenceValue::Literal(format!(
                "loaded={} signed={} compression={} livepatch={}",
                loader.loaded_count,
                if loader.signed_supported { "yes" } else { "no" },
                loader.compression,
                if loader.livepatch_enabled {
                    "yes"
                } else {
                    "no"
                },
            )),
        )
    }
}

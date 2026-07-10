// Copyright (C) 2026 rezky_nightky
// SPDX-License-Identifier: GPL-3.0-only

//! Kernel configuration capabilities — CONFIG_* values.

use crate::core::capability::Capability;
use crate::core::caps::{cfg_present, cfg_val};
use crate::core::evidence::{Confidence, Evidence, EvidenceValue, ProbeStatus};
use crate::system::config::ConfigValue;

pub struct ConfigSource;
impl Capability for ConfigSource {
    fn id(&self) -> &'static str {
        "config.source"
    }
    fn label(&self) -> &'static str {
        "Config Source"
    }
    fn probe(&self) -> Evidence {
        let (_, src) = crate::system::config::read_kernel_config().unzip();
        match src {
            Some(s) => Evidence::present(self.id(), EvidenceValue::Text(Some(s))),
            None => Evidence::missing(self.id(), EvidenceValue::Text(None)),
        }
    }
}

macro_rules! config_cap {
    ($name:ident, $id:literal, $label:literal, $key:literal) => {
        pub struct $name;
        impl Capability for $name {
            fn id(&self) -> &'static str {
                $id
            }
            fn label(&self) -> &'static str {
                $label
            }
            fn probe(&self) -> Evidence {
                let v = cfg_val($key);
                let status = if v == ConfigValue::Missing && !cfg_present() {
                    ProbeStatus::Missing
                } else {
                    ProbeStatus::Present
                };
                Evidence {
                    id: self.id(),
                    status,
                    confidence: Confidence::High,
                    value: EvidenceValue::Config(v),
                }
            }
        }
    };
}

config_cap!(ConfigModules, "config.MODULES", "CONFIG_MODULES", "MODULES");
config_cap!(
    ConfigModuleSig,
    "config.MODULE_SIG",
    "CONFIG_MODULE_SIG",
    "MODULE_SIG"
);
config_cap!(
    ConfigKallsyms,
    "config.KALLSYMS",
    "CONFIG_KALLSYMS",
    "KALLSYMS"
);
config_cap!(
    ConfigKallsymsAll,
    "config.KALLSYMS_ALL",
    "CONFIG_KALLSYMS_ALL",
    "KALLSYMS_ALL"
);
config_cap!(ConfigBpf, "config.BPF", "CONFIG_BPF", "BPF");
config_cap!(
    ConfigDebugInfoBtf,
    "config.DEBUG_INFO_BTF",
    "CONFIG_DEBUG_INFO_BTF",
    "DEBUG_INFO_BTF"
);
config_cap!(ConfigRust, "config.RUST", "CONFIG_RUST", "RUST");
config_cap!(
    ConfigRustAvailable,
    "config.RUST_IS_AVAILABLE",
    "CONFIG_RUST_IS_AVAILABLE",
    "RUST_IS_AVAILABLE"
);
config_cap!(
    ConfigLivepatch,
    "config.LIVEPATCH",
    "CONFIG_LIVEPATCH",
    "LIVEPATCH"
);

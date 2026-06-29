// Copyright (C) 2026 rezky_nightky
// SPDX-License-Identifier: GPL-3.0-only

//! Build environment capabilities — headers, directories, compile_commands.

use crate::core::capability::Capability;
use crate::core::evidence::{Evidence, EvidenceValue};
use crate::system;

pub struct HeaderIntegrity;
impl Capability for HeaderIntegrity {
    fn id(&self) -> &'static str {
        "build.headers"
    }
    fn label(&self) -> &'static str {
        "Header Integrity"
    }
    fn probe(&self) -> Evidence {
        let bld = system::buildenv::inspect_build_env();
        Evidence::present(self.id(), EvidenceValue::Status(bld.header_status.label()))
    }
}

pub struct BuildDirectory;
impl Capability for BuildDirectory {
    fn id(&self) -> &'static str {
        "build.dir"
    }
    fn label(&self) -> &'static str {
        "Build Directory"
    }
    fn probe(&self) -> Evidence {
        let bld = system::buildenv::inspect_build_env();
        Evidence::present(
            self.id(),
            EvidenceValue::Text(bld.build_dir.map(|d| d.to_string())),
        )
    }
}

pub struct SourceDirectory;
impl Capability for SourceDirectory {
    fn id(&self) -> &'static str {
        "build.source"
    }
    fn label(&self) -> &'static str {
        "Source Directory"
    }
    fn probe(&self) -> Evidence {
        let bld = system::buildenv::inspect_build_env();
        Evidence::present(
            self.id(),
            EvidenceValue::Text(bld.source_dir.map(|d| d.to_string())),
        )
    }
}

pub struct CompileCommands;
impl Capability for CompileCommands {
    fn id(&self) -> &'static str {
        "build.compile_commands"
    }
    fn label(&self) -> &'static str {
        "compile_commands.json"
    }
    fn probe(&self) -> Evidence {
        let bld = system::buildenv::inspect_build_env();
        Evidence::present(self.id(), EvidenceValue::Bool(bld.compile_commands))
    }
}

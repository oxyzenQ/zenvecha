// Copyright (C) 2026 rezky_nightky
// SPDX-License-Identifier: GPL-3.0-only

//! Read-only system inspection modules.
//!
//! Each submodule inspects one subsystem. No mutation, no side effects.

pub mod abi;
pub mod btf;
pub mod buildenv;
pub mod capabilities;
pub mod compiler;
pub mod config;
pub mod formatter;
pub mod fscheck;
pub mod json;
pub mod kallsyms;
pub mod kernel;
pub mod moduleinfo;
pub mod modules;
pub mod recommend;
pub mod report;
pub mod rust;
pub mod scoring;
pub mod symbols;
pub mod toolchain;

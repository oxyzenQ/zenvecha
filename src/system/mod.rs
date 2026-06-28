// Copyright (C) 2026 rezky_nightky
// SPDX-License-Identifier: GPL-3.0-only

//! Read-only system inspection modules.
//!
//! Each submodule inspects one subsystem. No mutation, no side effects.

pub mod btf;
pub mod buildenv;
pub mod config;
pub mod fscheck;
pub mod kallsyms;
pub mod kernel;
pub mod modules;
pub mod rust;
pub mod toolchain;

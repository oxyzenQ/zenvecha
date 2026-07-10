// Copyright (C) 2026 rezky_nightky
// SPDX-License-Identifier: GPL-3.0-only

//! Wolfzenix renderer layer — formatting only.
//!
//! Each renderer accepts already-computed models. Renderers never inspect
//! the system, never collect evidence, never compute scores. Only formatting.
//!
//! Architecture:
//!   abi.rs     → ABI command output
//!   analyze.rs → analyze command output
//!   doctor.rs  → doctor command output
//!   inspect.rs → inspect command output
//!   report.rs  → human / compact / JSON report renderers

pub mod abi;
pub mod analyze;
pub mod doctor;
pub mod inspect;
pub mod report;

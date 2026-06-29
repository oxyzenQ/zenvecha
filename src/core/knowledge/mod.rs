// Copyright (C) 2026 rezky_nightky
// SPDX-License-Identifier: GPL-3.0-only

//! Knowledge Engine — persistent Linux kernel intelligence.
//!
//! Centralizes domain knowledge about kernel versions, configuration
//! options, Rust for Linux evolution, features, and subsystem capabilities.
//!
//! Architecture:
//!   rules.rs     → rule type definitions
//!   database.rs  → immutable knowledge base
//!   resolver.rs  → matches rules against Evidence

pub mod database;
pub mod resolver;
pub mod rules;

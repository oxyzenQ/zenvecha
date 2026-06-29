// SPDX-License-Identifier: GPL-2.0-only
// Copyright (C) 2026 rezky_nightky

//! Livepatch kernel module — safe execution without reboot.
//!
//! This module exposes three proc interfaces:
//!   /proc/zenvecha/livepatch/apply   — write patch payload
//!   /proc/zenvecha/livepatch/status  — read last application result
//!   /proc/zenvecha/livepatch/verify  — read verification state
//!
//! The module NEVER decides what to patch — userspace owns the decision.
//! The module ONLY provides atomic execution and status reporting.

pub mod dummy_target;
pub mod executor;
pub mod guard;

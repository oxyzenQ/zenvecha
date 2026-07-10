// Copyright (C) 2026 rezky_nightky
// SPDX-License-Identifier: GPL-3.0-only

//! Safe Livepatch Execution Engine — the "without reboot" core.
//!
//! ## Architecture
//!
//! Userspace owns the decision; kernel owns atomic execution.
//!
//!   Decision Engine → LivepatchRequest
//!       ↓
//!   LivepatchEngine.validate()
//!       ├── CapabilityGraph checks (4 required capabilities)
//!       ├── Dependency chain (3 livepatch dependencies)
//!       ├── Semantic must-pass (2 required states)
//!       └── Semantic forbidden (2 forbidden states)
//!       ↓
//!   Rejected? → RejectionReason (structured, actionable)
//!   Approved? → kernel/livepatch/executor.rs (atomic apply)
//!       ↓
//!   Post-patch verification
//!       ↓
//!   LivepatchResult
//!
//! ## Safety Contract
//!
//! Patch is ONLY applied when ALL of these hold:
//!   1. Livepatch capability exists (CONFIG_LIVEPATCH=y)
//!   2. Modules enabled (CONFIG_MODULES=y)  
//!   3. ftrace available (CONFIG_FUNCTION_TRACER=y)
//!   4. Runtime risk is LOW (not MEDIUM, HIGH, or CRITICAL)
//!   5. System stability is NOT Unstable
//!
//! Any ONE failure → REJECT with structured reason.
//!
//! ## Files
//!
//!   model.rs     → LivepatchRequest, LivepatchResult, RejectionReason,
//!                  ValidationContext, LivepatchSafetyPolicy
//!   validator.rs → Consumes Graph + Semantic, produces validation
//!   engine.rs    → Orchestrates validate → execute → verify

pub mod engine;
pub mod model;
pub mod validator;

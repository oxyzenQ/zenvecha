// SPDX-License-Identifier: GPL-2.0-only
// Copyright (C) 2026 rezky_nightky

//! Zenvecha Kernel Module — Shared Declarations.
//!
//! Wolfzenix architecture contract:
//!   - Kernel module ONLY discovers facts and executes atomic ops.
//!   - Userspace owns all decisions, scoring, and rendering.
//!   - Communication channel = /proc/zenvecha/* (flat dotted filenames).

#ifndef _ZENVECHA_H
#define _ZENVECHA_H

#include <linux/proc_fs.h>
#include <linux/seq_file.h>

// ── Proc root ──────────────────────────────────────────────────────────
//
// All capability entries live directly under /proc/zenvecha/ with dotted
// filenames matching the userspace reader contract:
//
//   /proc/zenvecha/version.release        /proc/zenvecha/symbols.total
//   /proc/zenvecha/security.lockdown      /proc/zenvecha/scheduler.classes
//   /proc/zenvecha/architecture.name      /proc/zenvecha/btf.available
//
// The livepatch interface lives under /proc/zenvecha/livepatch/ as a
// nested directory (apply, status, verify, revert) — matches the
// userspace writer/reader paths in src/core/livepatch/engine.rs.

extern struct proc_dir_entry *zenvecha_proc_root;

// ── Capability Probe Interface ─────────────────────────────────────────
//
// Each probe exports a discover() function returning a static array of
// key=value descriptors. The capability layer iterates the registry and
// creates one flat proc entry per descriptor.
//
// Adding a new probe:
//   1. Create probes/{domain}.c with a {domain}_probe_discover() function
//   2. Add entry to the probes[] table in capability.c
//   3. Zero modifications to existing probes

struct capability_descriptor {
	const char *key;    /* dotted filename under /proc/zenvecha/ */
	const char *value;  /* static value string */
};

struct capability_probe {
	const char *name;
	const struct capability_descriptor *(*discover)(void);
	size_t count;
};

extern const struct capability_probe *const zenvecha_probes[];
extern const size_t zenvecha_probes_count;

// ── Livepatch Interface ────────────────────────────────────────────────
//
// /proc/zenvecha/livepatch/apply    (write)  parse symbol/target/new payload
// /proc/zenvecha/livepatch/status   (read)   "applied" | "reverted" | "rejected: ..."
// /proc/zenvecha/livepatch/verify   (read)   "verified redirect_observed ..."
// /proc/zenvecha/livepatch/revert   (write)  "revert\n" triggers rollback

int zenvecha_livepatch_init(void);
void zenvecha_livepatch_exit(void);

// ── Guard Layer (preflight + target validation + atomic exec) ──────────

struct preflight_result {
	bool ok;
	const char *fatal_check;   /* NULL when ok=true */
	const char *fatal_reason;
};

struct preflight_result zenvecha_preflight(void);

int zenvecha_validate_target(const char *symbol_name);
int zenvecha_guarded_apply(const char *symbol_name);
int zenvecha_guarded_revert(void);

bool zenvecha_patch_is_active(void);

// ── Dummy Target ───────────────────────────────────────────────────────
//
// zenvecha_dummy_func() is the only safe target for the PoC. It returns
// DUMMY_ORIGINAL_VALUE (42) when unpatched, DUMMY_PATCHED_VALUE (99)
// when the atomic flag has been flipped by the executor.

#define ZENVECHA_DUMMY_ORIGINAL_VALUE  42ULL
#define ZENVECHA_DUMMY_PATCHED_VALUE   99ULL

u64 zenvecha_dummy_func(void);
const char *zenvecha_dummy_target_name(void);

// ── Semantic Bridge (optional kernel-side RuntimeRisk gate) ────────────
//
// /proc/zenvecha/semantic.runtime_risk (read+write)
//   Default value: "low"
//   Userspace may write "low" | "medium" | "high" | "critical"
//   Kernel checks this in the apply gate — non-"low" rejects the patch.

extern char zenvecha_runtime_risk[16];

bool zenvecha_runtime_risk_is_low(void);

#endif /* _ZENVECHA_H */

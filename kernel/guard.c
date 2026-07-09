// SPDX-License-Identifier: GPL-2.0-only
// Copyright (C) 2026 rezky_nightky

//! Kernel livepatch safety guards — prevent kernel panics.
//!
//! Every operation that touches kernel execution flow must pass
//! through a guard. Guards check preconditions, wrap dangerous
//! operations in error-handling, and return structured errors.
//!
//! ## Safety Principles
//!
//!   1. Check BEFORE acting — never act and then discover failure
//!   2. Return errors, never panic — userspace handles rejection
//!   3. Isolate dangerous code — stop_machine lives in a guarded scope
//!   4. Recover gracefully — any unexpected state → structured error
//!
//! ## Six-Gate Safety Protocol
//!
//! Every patch must pass six gates before it is applied:
//!
//!   Gate 1: CONFIG_LIVEPATCH compiled in (compile-time check)
//!   Gate 2: CONFIG_FUNCTION_TRACER compiled in (compile-time)
//!   Gate 3: CONFIG_MODULES compiled in (compile-time)
//!   Gate 4: RuntimeRisk = low (read /proc/zenvecha/semantic.runtime_risk)
//!   Gate 5: No active patch on this symbol (check internal state)
//!   Gate 6: Symbol exists in our target registry (kallsyms-equivalent)
//!
//! Gates 1-3 are evaluated at module init (preflight). If any fails,
//! the module refuses to load entirely.
//! Gates 4-6 are evaluated at every apply request. If any fails,
//! the apply is rejected with a structured error message written
//! to /proc/zenvecha/livepatch/status.

#define pr_fmt(fmt) "zenvecha: " fmt

#include <linux/kernel.h>
#include <linux/string.h>
#include <linux/module.h>
#include <linux/atomic.h>
#include <linux/stop_machine.h>

#include "zenvecha.h"

/* Internal state — single-slot patch (one active patch at a time).
 *
 * For the PoC skeleton, only one patch is allowed at any time.
 * Production multi-patch support would require a hash table of
 * (symbol → patch_state) entries. */
static atomic_t zenvecha_patch_active = ATOMIC_INIT(0);
static char zenvecha_active_symbol[64];

bool zenvecha_patch_is_active(void)
{
	return atomic_read(&zenvecha_patch_active) != 0;
}

static bool symbol_is_known(const char *name)
{
	if (!name)
		return false;

	/* For the PoC skeleton, only zenvecha_dummy_func is a known
	 * patchable target. Production code would call kallsyms_lookup_name
	 * here — but that symbol is unexported since 5.7. The proper
	 * production approach is the kprobe-trick (register a kprobe with
	 * .symbol_name, read .addr, unregister). For now, match our own
	 * module symbol. */
	return strcmp(name, zenvecha_dummy_target_name()) == 0;
}

// ── Pre-flight Guard — runs at module init ─────────────────────────────

struct preflight_result zenvecha_preflight(void)
{
	struct preflight_result r = { .ok = true };

	/* Gate 1: CONFIG_LIVEPATCH */
#if !IS_ENABLED(CONFIG_LIVEPATCH)
	r.ok = false;
	r.fatal_check = "CONFIG_LIVEPATCH";
	r.fatal_reason = "Livepatch support not compiled into this kernel";
	return r;
#endif

	/* Gate 2: CONFIG_FUNCTION_TRACER */
#if !IS_ENABLED(CONFIG_FUNCTION_TRACER)
	r.ok = false;
	r.fatal_check = "CONFIG_FUNCTION_TRACER";
	r.fatal_reason = "Function tracer support not compiled into this kernel";
	return r;
#endif

	/* Gate 3: CONFIG_MODULES */
#if !IS_ENABLED(CONFIG_MODULES)
	r.ok = false;
	r.fatal_check = "CONFIG_MODULES";
	r.fatal_reason = "Module loader not compiled into this kernel";
	return r;
#endif

	/* All compile-time gates passed. */
	pr_info("preflight: CONFIG_LIVEPATCH + FUNCTION_TRACER + MODULES ok\n");
	return r;
}

// ── Target Validation Guard — runs before every patch ──────────────────

int zenvecha_validate_target(const char *symbol_name)
{
	/* Gate 4: RuntimeRisk must be "low" */
	if (!zenvecha_runtime_risk_is_low()) {
		pr_warn("gate 4 FAIL: runtime_risk=%s (must be 'low')\n",
			zenvecha_runtime_risk);
		return -EPERM;
	}

	/* Gate 5: No active patch */
	if (zenvecha_patch_is_active()) {
		pr_warn("gate 5 FAIL: active patch on '%s'\n",
			zenvecha_active_symbol);
		return -EBUSY;
	}

	/* Gate 6: Symbol exists in our target registry */
	if (!symbol_is_known(symbol_name)) {
		pr_warn("gate 6 FAIL: unknown symbol '%s'\n", symbol_name);
		return -ENOENT;
	}

	return 0;
}

// ── Atomic Execution Guard — stop_machine callback ─────────────────────

struct patch_payload {
	const char *symbol_name;
	int error; /* 0 = success, negative errno on failure */
};

static int apply_patch_atomic(void *data)
{
	struct patch_payload *p = data;

	/* All other CPUs are halted by stop_machine. We are now in the
	 * single critical section where it is safe to modify execution
	 * state.
	 *
	 * Skeleton: flip the dummy function's atomic flag.
	 * Production: install ftrace handler at the target address. */
	zenvecha_dummy_set_patched(true);
	p->error = 0;
	return 0;
}

static int revert_patch_atomic(void *data)
{
	struct patch_payload *p = data;

	zenvecha_dummy_set_patched(false);
	p->error = 0;
	return 0;
}

// ── Guarded apply / revert entry points ────────────────────────────────

int zenvecha_guarded_apply(const char *symbol_name)
{
	struct patch_payload payload = { .symbol_name = symbol_name };
	int ret;

	/* Run gates 4-6 */
	ret = zenvecha_validate_target(symbol_name);
	if (ret)
		return ret;

	/* Atomic apply via stop_machine — halts ALL other CPUs */
	ret = stop_machine(apply_patch_atomic, &payload, NULL);
	if (ret) {
		pr_err("stop_machine(apply) failed: %d\n", ret);
		return ret;
	}
	if (payload.error) {
		pr_err("apply callback failed: %d\n", payload.error);
		return payload.error;
	}

	/* Commit state */
	atomic_set(&zenvecha_patch_active, 1);
	strncpy(zenvecha_active_symbol, symbol_name,
		sizeof(zenvecha_active_symbol) - 1);
	zenvecha_active_symbol[sizeof(zenvecha_active_symbol) - 1] = '\0';

	pr_info("patch applied: %s (42 → 99)\n", symbol_name);
	return 0;
}

int zenvecha_guarded_revert(void)
{
	struct patch_payload payload = { .symbol_name = NULL };
	int ret;

	if (!zenvecha_patch_is_active()) {
		pr_info("revert: no active patch\n");
		return 0;
	}

	ret = stop_machine(revert_patch_atomic, &payload, NULL);
	if (ret) {
		pr_err("stop_machine(revert) failed: %d\n", ret);
		return ret;
	}

	pr_info("patch reverted: %s (99 → 42)\n", zenvecha_active_symbol);
	atomic_set(&zenvecha_patch_active, 0);
	zenvecha_active_symbol[0] = '\0';

	return 0;
}

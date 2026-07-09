// SPDX-License-Identifier: GPL-2.0-only
// Copyright (C) 2026 rezky_nightky

//! Dummy kernel target for safe livepatch testing.
//!
//! Provides a simple, isolated function that can be safely patched.
//! Zero risk to the core kernel — this function only returns a known
//! value. The patch flips an atomic flag (via stop_machine) that
//! switches the return value.
//!
//! ## Skeleton vs Production
//!
//! Skeleton (current):
//!   - zenvecha_dummy_func() checks atomic flag, returns 42 or 99
//!   - "Patching" = stop_machine + atomic_set(flag, 1)
//!   - "Reverting" = stop_machine + atomic_set(flag, 0)
//!   - Proves the atomicity model without modifying kernel text
//!
//! Production (future):
//!   - zenvecha_dummy_func marked __visible + NOTRACE
//!   - Patch installs ftrace handler at the function entry
//!   - Calls are redirected to zenvecha_dummy_patched()
//!   - stop_machine ensures no CPU is mid-execution during the swap

#include <linux/kernel.h>
#include <linux/atomic.h>
#include <linux/string.h>
#include "zenvecha.h"

static atomic_t zenvecha_dummy_patched = ATOMIC_INIT(0);

const char *zenvecha_dummy_target_name(void)
{
	return "zenvecha_dummy_func";
}

/* The dummy target function.
 *
 * Marked noinline to prevent the compiler from inlining it into
 * callers (which would break the patching contract). The notrace
 * annotation prevents ftrace from instrumenting this function —
 * essential for production livepatch, harmless in skeleton mode.
 */
__attribute__((noinline, used))
u64 zenvecha_dummy_func(void)
{
	if (atomic_read(&zenvecha_dummy_patched))
		return ZENVECHA_DUMMY_PATCHED_VALUE;
	return ZENVECHA_DUMMY_ORIGINAL_VALUE;
}

/* Internal state setters — called only from the executor inside
 * stop_machine. */
void zenvecha_dummy_set_patched(bool patched)
{
	atomic_set(&zenvecha_dummy_patched, patched ? 1 : 0);
}

bool zenvecha_dummy_is_patched(void)
{
	return atomic_read(&zenvecha_dummy_patched) != 0;
}

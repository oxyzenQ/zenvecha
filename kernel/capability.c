// SPDX-License-Identifier: GPL-2.0-only
// Copyright (C) 2026 rezky_nightky

//! Capability probe registry.
//!
//! Each probe is a small C file under probes/ that exports two functions:
//!   - {domain}_probe_discover()  → returns static descriptor array
//!   - {domain}_probe_count()     → returns array length
//!
//! This file declares the probe struct instances and the registry array.
//! We use explicit struct initialization (no macros) for clear compiler
//! diagnostics — kernel module build errors through macros are hard to
//! read, and clang's "expected a field designator" through a #define
//! expansion is a classic example.
//!
//! Adding a new probe:
//!   1. Create probes/{domain}.c with {domain}_probe_discover() + count()
//!   2. Add forward declarations below
//!   3. Add a struct instance + array entry
//!   4. Zero modifications to existing probes

#include <linux/kernel.h>
#include <linux/string.h>
#include "zenvecha.h"

/* Forward declarations — one per probe file */
const struct capability_descriptor *version_probe_discover(void);
size_t version_probe_count(void);
const struct capability_descriptor *symbols_probe_discover(void);
size_t symbols_probe_count(void);
const struct capability_descriptor *kallsyms_probe_discover(void);
size_t kallsyms_probe_count(void);
const struct capability_descriptor *btf_probe_discover(void);
size_t btf_probe_count(void);
const struct capability_descriptor *modules_probe_discover(void);
size_t modules_probe_count(void);
const struct capability_descriptor *tracing_probe_discover(void);
size_t tracing_probe_count(void);
const struct capability_descriptor *arch_probe_discover(void);
size_t arch_probe_count(void);
const struct capability_descriptor *security_probe_discover(void);
size_t security_probe_count(void);
const struct capability_descriptor *scheduler_probe_discover(void);
size_t scheduler_probe_count(void);
const struct capability_descriptor *memory_probe_discover(void);
size_t memory_probe_count(void);
const struct capability_descriptor *tracepoints_probe_discover(void);
size_t tracepoints_probe_count(void);

/* Probe struct instances — explicit initialization, no macros.
 *
 * .count is a function POINTER (address, no parentheses). The struct
 * field type is `size_t (*count)(void)`. Storing the function address
 * is a compile-time constant — valid in a static const initializer.
 * Calling the function (with parens) would NOT be a constant expression
 * and is rejected by C. */
static const struct capability_probe version_probe = {
	.name = "version",
	.discover = version_probe_discover,
	.count = version_probe_count,
};

static const struct capability_probe symbols_probe = {
	.name = "symbols",
	.discover = symbols_probe_discover,
	.count = symbols_probe_count,
};

static const struct capability_probe kallsyms_probe = {
	.name = "kallsyms",
	.discover = kallsyms_probe_discover,
	.count = kallsyms_probe_count,
};

static const struct capability_probe btf_probe = {
	.name = "btf",
	.discover = btf_probe_discover,
	.count = btf_probe_count,
};

static const struct capability_probe modules_probe = {
	.name = "modules",
	.discover = modules_probe_discover,
	.count = modules_probe_count,
};

static const struct capability_probe tracing_probe = {
	.name = "tracing",
	.discover = tracing_probe_discover,
	.count = tracing_probe_count,
};

static const struct capability_probe arch_probe = {
	.name = "architecture",
	.discover = arch_probe_discover,
	.count = arch_probe_count,
};

static const struct capability_probe security_probe = {
	.name = "security",
	.discover = security_probe_discover,
	.count = security_probe_count,
};

static const struct capability_probe scheduler_probe = {
	.name = "scheduler",
	.discover = scheduler_probe_discover,
	.count = scheduler_probe_count,
};

static const struct capability_probe memory_probe = {
	.name = "memory",
	.discover = memory_probe_discover,
	.count = memory_probe_count,
};

static const struct capability_probe tracepoints_probe = {
	.name = "tracepoints",
	.discover = tracepoints_probe_discover,
	.count = tracepoints_probe_count,
};

const struct capability_probe *const zenvecha_probes[] = {
	&version_probe,
	&symbols_probe,
	&kallsyms_probe,
	&btf_probe,
	&modules_probe,
	&tracing_probe,
	&arch_probe,
	&security_probe,
	&scheduler_probe,
	&memory_probe,
	&tracepoints_probe,
};

const size_t zenvecha_probes_count = ARRAY_SIZE(zenvecha_probes);

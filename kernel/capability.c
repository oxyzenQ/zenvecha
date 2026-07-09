// SPDX-License-Identifier: GPL-2.0-only
// Copyright (C) 2026 rezky_nightky

//! Capability probe registry.
//!
//! Each probe is a small C file under probes/ that exports a
//! `{domain}_probe_discover()` function returning a static array
//! of (key, value) descriptors.
//!
//! Adding a new probe:
//!   1. Create probes/{domain}.c with {domain}_probe_discover()
//!   2. Add entry to zenvecha_probes[] below
//!   3. Bump zenvecha_probes_count if needed
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

/* Static probe wrappers — bridge discover() + count() into the
 * capability_probe struct expected by the module. */
#define PROBE(name, lower)                                                  \
	static const struct capability_probe lower##_probe = {              \
		.name = name,                                               \
		.discover = lower##_probe_discover,                         \
		.count = lower##_probe_count(),                             \
	}

PROBE("version", version);
PROBE("symbols", symbols);
PROBE("kallsyms", kallsyms);
PROBE("btf", btf);
PROBE("modules", modules);
PROBE("tracing", tracing);
PROBE("architecture", arch);
PROBE("security", security);
PROBE("scheduler", scheduler);
PROBE("memory", memory);
PROBE("tracepoints", tracepoints);

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

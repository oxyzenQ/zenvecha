// SPDX-License-Identifier: GPL-2.0-only
// Copyright (C) 2026 rezky_nightky

//! Tracing infrastructure probe.
//!
//! Discovered facts:
//!   tracing.ftrace       — "available" | "unavailable"
//!   tracing.kprobes      — "available" | "unavailable"
//!   tracing.kretprobes   — "available" | "unavailable"
//!   tracing.tracepoints  — "available" | "unavailable"
//!   tracing.uprobes      — "available" | "unavailable"

#include <linux/kernel.h>
#include <linux/string.h>
#include "zenvecha.h"

static char ftrace_buf[16] = "available";
static char kprobes_buf[16] = "available";
static char kretprobes_buf[16] = "available";
static char tracepoints_buf[16] = "available";
static char uprobes_buf[16] = "available";

static const struct capability_descriptor descriptors[] = {
	{ .key = "tracing.ftrace",      .value = ftrace_buf      },
	{ .key = "tracing.kprobes",     .value = kprobes_buf     },
	{ .key = "tracing.kretprobes",  .value = kretprobes_buf  },
	{ .key = "tracing.tracepoints", .value = tracepoints_buf },
	{ .key = "tracing.uprobes",     .value = uprobes_buf     },
};

const struct capability_descriptor *tracing_probe_discover(void)
{
#ifdef CONFIG_FUNCTION_TRACER
	snprintf(ftrace_buf, sizeof(ftrace_buf), "available");
#else
	snprintf(ftrace_buf, sizeof(ftrace_buf), "unavailable");
#endif

#ifdef CONFIG_KPROBES
	snprintf(kprobes_buf, sizeof(kprobes_buf), "available");
#else
	snprintf(kprobes_buf, sizeof(kprobes_buf), "unavailable");
#endif

#if defined(CONFIG_KRETPROBES) && defined(CONFIG_KPROBES)
	snprintf(kretprobes_buf, sizeof(kretprobes_buf), "available");
#else
	snprintf(kretprobes_buf, sizeof(kretprobes_buf), "unavailable");
#endif

#ifdef CONFIG_TRACEPOINTS
	snprintf(tracepoints_buf, sizeof(tracepoints_buf), "available");
#else
	snprintf(tracepoints_buf, sizeof(tracepoints_buf), "unavailable");
#endif

#ifdef CONFIG_UPROBES
	snprintf(uprobes_buf, sizeof(uprobes_buf), "available");
#else
	snprintf(uprobes_buf, sizeof(uprobes_buf), "unavailable");
#endif

	return descriptors;
}

size_t tracing_probe_count(void)
{
	return ARRAY_SIZE(descriptors);
}

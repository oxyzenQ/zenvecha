// SPDX-License-Identifier: GPL-2.0-only
// Copyright (C) 2026 rezky_nightky

//! Tracepoints probe.
//!
//! Discovered facts:
//!   tracepoints.count       — number of registered tracepoints
//!   tracepoints.subsystems  — comma-separated subsystem names
//!                             (best-effort static list)

#include <linux/kernel.h>
#include <linux/string.h>
#include <linux/tracepoint.h>
#include "zenvecha.h"

static char count_buf[16];
static char subsystems_buf[128] = "sched,block,net,irq,syscalls,fs,mm";

static const struct capability_descriptor descriptors[] = {
	{ .key = "tracepoints.count",      .value = count_buf      },
	{ .key = "tracepoints.subsystems", .value = subsystems_buf },
};

static u64 tracepoint_count;

static void count_tracepoint(struct tracepoint *tp, void *priv)
{
	(void)tp;
	(void)priv;
	tracepoint_count++;
}

const struct capability_descriptor *tracepoints_probe_discover(void)
{
	tracepoint_count = 0;

#ifdef CONFIG_TRACEPOINTS
	for_each_kernel_tracepoint(count_tracepoint, NULL);
#endif

	snprintf(count_buf, sizeof(count_buf), "%llu", tracepoint_count);

	return descriptors;
}

size_t tracepoints_probe_count(void)
{
	return ARRAY_SIZE(descriptors);
}

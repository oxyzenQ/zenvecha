// SPDX-License-Identifier: GPL-2.0-only
// Copyright (C) 2026 rezky_nightky

//! Scheduler capability probe.
//!
//! Discovered facts:
//!   scheduler.classes     — "cfs,rt,deadline" (active sched classes)
//!   scheduler.preemption  — "none" | "voluntary" | "full"

#include <linux/kernel.h>
#include <linux/string.h>
#include <linux/sched.h>
#include "zenvecha.h"

static char classes_buf[32] = "cfs,rt,deadline";
static char preemption_buf[16] = "voluntary";

static const struct capability_descriptor descriptors[] = {
	{ .key = "scheduler.classes",    .value = classes_buf    },
	{ .key = "scheduler.preemption", .value = preemption_buf },
};

const struct capability_descriptor *scheduler_probe_discover(void)
{
	/* Scheduler classes — on x86_64 Linux 6.x the default set is
	 * stop, dl, rt, fair, idle. Expose the user-relevant subset. */
#if defined(CONFIG_SCHED_DEBUG) || defined(CONFIG_FAIR_GROUP_SCHED)
	snprintf(classes_buf, sizeof(classes_buf), "cfs,rt,deadline");
#else
	snprintf(classes_buf, sizeof(classes_buf), "cfs,rt");
#endif

	/* Preemption model */
#if defined(CONFIG_PREEMPT_NONE)
	snprintf(preemption_buf, sizeof(preemption_buf), "none");
#elif defined(CONFIG_PREEMPT_VOLUNTARY)
	snprintf(preemption_buf, sizeof(preemption_buf), "voluntary");
#elif defined(CONFIG_PREEMPT) || defined(CONFIG_PREEMPT_RT)
	snprintf(preemption_buf, sizeof(preemption_buf), "full");
#else
	snprintf(preemption_buf, sizeof(preemption_buf), "voluntary");
#endif

	return descriptors;
}

size_t scheduler_probe_count(void)
{
	return ARRAY_SIZE(descriptors);
}

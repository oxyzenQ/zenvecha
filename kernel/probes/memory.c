// SPDX-License-Identifier: GPL-2.0-only
// Copyright (C) 2026 rezky_nightky

//! Memory model probe.
//!
//! Discovered facts:
//!   memory.hugepages — supported hugepage sizes, comma-separated
//!                      (e.g. "2M,1G" on x86_64)

#include <linux/kernel.h>
#include <linux/string.h>
#include <linux/hugetlb.h>
#include <asm/page.h>
#include "zenvecha.h"

static char hugepages_buf[32] = "2M,1G";

static const struct capability_descriptor descriptors[] = {
	{ .key = "memory.hugepages", .value = hugepages_buf },
};

const struct capability_descriptor *memory_probe_discover(void)
{
	/* On x86_64 the standard hugepage sizes are 2M and 1G.
	 * On aarch64 with 64k pages, also 512M and 16G.
	 * We expose the compile-time-supported set. */
#if defined(CONFIG_X86_64)
	snprintf(hugepages_buf, sizeof(hugepages_buf), "2M,1G");
#elif defined(CONFIG_ARM64_64K_PAGES)
	snprintf(hugepages_buf, sizeof(hugepages_buf), "512M,16G");
#elif defined(CONFIG_ARM64)
	snprintf(hugepages_buf, sizeof(hugepages_buf), "2M,1G");
#else
	snprintf(hugepages_buf, sizeof(hugepages_buf), "2M");
#endif

	return descriptors;
}

size_t memory_probe_count(void)
{
	return ARRAY_SIZE(descriptors);
}

// SPDX-License-Identifier: GPL-2.0-only
// Copyright (C) 2026 rezky_nightky

//! kallsyms availability probe.
//!
//! Discovered facts:
//!   kallsyms.available      — "yes" | "no"
//!   kallsyms.all_symbols    — "yes" | "no" (CONFIG_KALLSYMS_ALL)
//!   kallsyms.base_address   — page-aligned kernel text base (heuristic)

#include <linux/kernel.h>
#include <linux/string.h>
#include <linux/kallsyms.h>
#include "zenvecha.h"

static char available_buf[8] = "yes";
static char all_symbols_buf[8] = "yes";
static char base_address_buf[32] = "0x0";

static const struct capability_descriptor descriptors[] = {
	{ .key = "kallsyms.available",     .value = available_buf    },
	{ .key = "kallsyms.all_symbols",   .value = all_symbols_buf  },
	{ .key = "kallsyms.base_address",  .value = base_address_buf },
};

const struct capability_descriptor *kallsyms_probe_discover(void)
{
#ifdef CONFIG_KALLSYMS
	snprintf(available_buf, sizeof(available_buf), "yes");
#else
	snprintf(available_buf, sizeof(available_buf), "no");
#endif

#ifdef CONFIG_KALLSYMS_ALL
	snprintf(all_symbols_buf, sizeof(all_symbols_buf), "yes");
#else
	snprintf(all_symbols_buf, sizeof(all_symbols_buf), "no");
#endif

	/* Kernel text base — use kallsyms_lookup_name("do_one_initcall")
	 * or similar universally-present symbol. If unavailable, fall back
	 * to 0x0 (userspace can still query via /proc/kallsyms directly). */
	{
		unsigned long addr = 0;

		addr = kallsyms_lookup_name("do_one_initcall");
		if (!addr)
			addr = kallsyms_lookup_name("printk");
		if (!addr)
			addr = kallsyms_lookup_name("init_task");
		snprintf(base_address_buf, sizeof(base_address_buf),
			 "0x%lx", addr);
	}

	return descriptors;
}

size_t kallsyms_probe_count(void)
{
	return ARRAY_SIZE(descriptors);
}

// SPDX-License-Identifier: GPL-2.0-only
// Copyright (C) 2026 rezky_nightky

//! CPU architecture probe.
//!
//! Discovered facts:
//!   architecture.name      — e.g. "x86_64", "aarch64"
//!   architecture.bits      — "32" | "64"
//!   architecture.endian    — "little" | "big"
//!   architecture.page_size — page size in bytes

#include <linux/kernel.h>
#include <linux/string.h>
#include <linux/utsname.h>
#include <linux/mm.h>
#include <asm/page.h>
#include <asm/byteorder.h>
#include "zenvecha.h"

static char name_buf[16];
static char bits_buf[8];
static char endian_buf[8];
static char page_size_buf[16];

static const struct capability_descriptor descriptors[] = {
	{ .key = "architecture.name",      .value = name_buf      },
	{ .key = "architecture.bits",      .value = bits_buf      },
	{ .key = "architecture.endian",    .value = endian_buf    },
	{ .key = "architecture.page_size", .value = page_size_buf },
};

const struct capability_descriptor *arch_probe_discover(void)
{
	const char *machine = init_utsname()->machine;

	snprintf(name_buf, sizeof(name_buf), "%s", machine);

#if BITS_PER_LONG == 64
	snprintf(bits_buf, sizeof(bits_buf), "64");
#else
	snprintf(bits_buf, sizeof(bits_buf), "32");
#endif

#ifdef __BIG_ENDIAN
	snprintf(endian_buf, sizeof(endian_buf), "big");
#else
	snprintf(endian_buf, sizeof(endian_buf), "little");
#endif

	snprintf(page_size_buf, sizeof(page_size_buf), "%lu",
		 (unsigned long)PAGE_SIZE);

	return descriptors;
}

size_t arch_probe_count(void)
{
	return ARRAY_SIZE(descriptors);
}

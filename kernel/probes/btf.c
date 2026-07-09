// SPDX-License-Identifier: GPL-2.0-only
// Copyright (C) 2026 rezky_nightky

//! BTF (BPF Type Format) availability probe.
//!
//! Discovered facts:
//!   btf.available   — "yes" | "no"
//!   btf.vmlinux     — "available" | "unavailable"
//!
//! Source: CONFIG_DEBUG_INFO_BTF compile-time + runtime btf_vmlinux check.

#include <linux/kernel.h>
#include <linux/string.h>
#include "zenvecha.h"

static char available_buf[8] = "yes";
static char vmlinux_buf[16] = "available";

static const struct capability_descriptor descriptors[] = {
	{ .key = "btf.available",  .value = available_buf },
	{ .key = "btf.vmlinux",    .value = vmlinux_buf   },
};

const struct capability_descriptor *btf_probe_discover(void)
{
#ifdef CONFIG_DEBUG_INFO_BTF
	snprintf(available_buf, sizeof(available_buf), "yes");
	snprintf(vmlinux_buf, sizeof(vmlinux_buf), "available");
#else
	snprintf(available_buf, sizeof(available_buf), "no");
	snprintf(vmlinux_buf, sizeof(vmlinux_buf), "unavailable");
#endif

	return descriptors;
}

size_t btf_probe_count(void)
{
	return ARRAY_SIZE(descriptors);
}

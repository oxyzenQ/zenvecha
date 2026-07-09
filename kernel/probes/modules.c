// SPDX-License-Identifier: GPL-2.0-only
// Copyright (C) 2026 rezky_nightky

//! Kernel module loader status probe.
//!
//! Discovered facts:
//!   modules.loader        — "enabled" | "disabled"
//!   modules.active_count  — number of currently-loaded modules
//!   modules.vermagic      — vermagic string (module version magic)
//!
//! Sources:
//!   - CONFIG_MODULES (compile-time)
//!   - /proc/modules line count (runtime)
//!   - init_utsname().release (vermagic base)

#include <linux/kernel.h>
#include <linux/string.h>
#include <linux/module.h>
#include <linux/fs.h>
#include <linux/file.h>
#include <linux/uaccess.h>
#include <linux/utsname.h>
#include "zenvecha.h"

static char loader_buf[12] = "enabled";
static char active_count_buf[16];
static char vermagic_buf[128] = "unknown";

static const struct capability_descriptor descriptors[] = {
	{ .key = "modules.loader",       .value = loader_buf       },
	{ .key = "modules.active_count", .value = active_count_buf },
	{ .key = "modules.vermagic",     .value = vermagic_buf     },
};

/* Count active modules by reading /proc/modules line count.
 * The &modules list head is not exported to loadable modules, so we
 * use the /proc filesystem interface instead — portable across all
 * kernel versions and configs. */
static unsigned int count_loaded_modules(void)
{
	struct file *f;
	char buf[4096];
	loff_t pos = 0;
	ssize_t n;
	unsigned int count = 0;

	f = filp_open("/proc/modules", O_RDONLY, 0);
	if (IS_ERR(f))
		return 0;

	while ((n = kernel_read(f, buf, sizeof(buf), &pos)) > 0) {
		ssize_t i;

		for (i = 0; i < n; i++)
			if (buf[i] == '\n')
				count++;
	}
	filp_close(f, NULL);
	return count;
}

const struct capability_descriptor *modules_probe_discover(void)
{
#ifdef CONFIG_MODULES
	unsigned int count;

	snprintf(loader_buf, sizeof(loader_buf), "enabled");
	count = count_loaded_modules();
	snprintf(active_count_buf, sizeof(active_count_buf), "%u", count);
#else
	snprintf(loader_buf, sizeof(loader_buf), "disabled");
	snprintf(active_count_buf, sizeof(active_count_buf), "0");
#endif

	/* vermagic — UTS release + SMP + mod_unload markers.
	 * The canonical vermagic also includes preempt flag and module
	 * version, but the basic form (release + SMP + mod_unload) is
	 * sufficient for userspace compatibility checks. */
	snprintf(vermagic_buf, sizeof(vermagic_buf), "%s SMP mod_unload",
		 init_utsname()->release);

	return descriptors;
}

size_t modules_probe_count(void)
{
	return ARRAY_SIZE(descriptors);
}

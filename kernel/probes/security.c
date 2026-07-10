// SPDX-License-Identifier: GPL-2.0-only
// Copyright (C) 2026 rezky_nightky

//! Kernel security probe — lockdown, LSMs, KASLR.
//!
//! Discovered facts:
//!   security.lockdown  — "none" | "integrity" | "confidentiality"
//!   security.lsms      — comma-separated active LSMs
//!   security.kaslr     — "yes" | "no"
//!
//! Sources:
//!   - /sys/kernel/security/lockdown (read at probe time)
//!   - /sys/kernel/security/lsm (LSM stack list)
//!   - CONFIG_RANDOMIZE_BASE + boot params

#include <linux/kernel.h>
#include <linux/string.h>
#include <linux/fs.h>
#include <linux/file.h>
#include <linux/uaccess.h>
#include "zenvecha.h"

static char lockdown_buf[24] = "none";
static char lsms_buf[128] = "unknown";
static char kaslr_buf[8] = "yes";

static const struct capability_descriptor descriptors[] = {
	{ .key = "security.lockdown", .value = lockdown_buf },
	{ .key = "security.lsms",     .value = lsms_buf     },
	{ .key = "security.kaslr",    .value = kaslr_buf    },
};

static void read_file_into(const char *path, char *buf, size_t len)
{
	struct file *f;
	loff_t pos = 0;
	ssize_t n;

	f = filp_open(path, O_RDONLY, 0);
	if (IS_ERR(f))
		return;
	n = kernel_read(f, buf, len - 1, &pos);
	filp_close(f, NULL);
	if (n > 0) {
		buf[n] = '\0';
		/* strip trailing newline */
		if (buf[n - 1] == '\n')
			buf[n - 1] = '\0';
	} else {
		buf[0] = '\0';
	}
}

const struct capability_descriptor *security_probe_discover(void)
{
	char tmp[128];

	/* Lockdown — /sys/kernel/security/lockdown format:
	 *   "[integrity] confidentiality none"
	 * The bracketed value is the active mode. */
	read_file_into("/sys/kernel/security/lockdown", tmp, sizeof(tmp));
	if (tmp[0] != '\0') {
		char *start = strchr(tmp, '[');
		char *end = strchr(tmp, ']');

		if (start && end && end > start) {
			size_t mode_len = end - start - 1;

			if (mode_len < sizeof(lockdown_buf)) {
				memcpy(lockdown_buf, start + 1, mode_len);
				lockdown_buf[mode_len] = '\0';
			}
		} else {
			snprintf(lockdown_buf, sizeof(lockdown_buf), "none");
		}
	}

	/* Active LSMs — /sys/kernel/security/lsm format:
	 *   "lockdown,yama,integrity,apparmor,bpf" */
	read_file_into("/sys/kernel/security/lsm", lsms_buf, sizeof(lsms_buf));

	/* KASLR — kernel randomize_base boot param */
#ifdef CONFIG_RANDOMIZE_BASE
	snprintf(kaslr_buf, sizeof(kaslr_buf), "yes");
#else
	snprintf(kaslr_buf, sizeof(kaslr_buf), "no");
#endif

	return descriptors;
}

size_t security_probe_count(void)
{
	return ARRAY_SIZE(descriptors);
}

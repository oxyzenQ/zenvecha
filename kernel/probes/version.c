// SPDX-License-Identifier: GPL-2.0-only
// Copyright (C) 2026 rezky_nightky

//! Kernel version probe.
//!
//! Discovered facts:
//!   version.release  — full kernel release (e.g. "6.18.0-arch1-1")
//!   version.major    — major version number
//!   version.minor    — minor version number
//!   version.patch    — patch number (best-effort parse)
//!
//! Source: init_uts_ns().name.release (UTS namespace).

#include <linux/utsname.h>
#include <linux/string.h>
#include "zenvecha.h"

static char release_buf[__NEW_UTS_LEN + 1];
static char major_buf[8];
static char minor_buf[8];
static char patch_buf[8];

static const struct capability_descriptor descriptors[] = {
	{ .key = "version.release", .value = release_buf },
	{ .key = "version.major",   .value = major_buf   },
	{ .key = "version.minor",   .value = minor_buf   },
	{ .key = "version.patch",   .value = patch_buf   },
};

static void parse_release(const char *release)
{
	int major = 0, minor = 0, patch = 0;
	const char *p = release;

	while (*p >= '0' && *p <= '9') {
		major = major * 10 + (*p - '0');
		p++;
	}
	if (*p == '.') p++;
	while (*p >= '0' && *p <= '9') {
		minor = minor * 10 + (*p - '0');
		p++;
	}
	if (*p == '.') p++;
	while (*p >= '0' && *p <= '9') {
		patch = patch * 10 + (*p - '0');
		p++;
	}

	snprintf(major_buf, sizeof(major_buf), "%d", major);
	snprintf(minor_buf, sizeof(minor_buf), "%d", minor);
	snprintf(patch_buf, sizeof(patch_buf), "%d", patch);
}

const struct capability_descriptor *version_probe_discover(void)
{
	const char *release = init_utsname()->release;

	strncpy(release_buf, release, sizeof(release_buf) - 1);
	release_buf[sizeof(release_buf) - 1] = '\0';
	parse_release(release_buf);

	return descriptors;
}

size_t version_probe_count(void)
{
	return ARRAY_SIZE(descriptors);
}

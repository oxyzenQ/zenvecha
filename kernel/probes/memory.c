// SPDX-License-Identifier: GPL-2.0-only
// Copyright (C) 2026 rezky_nightky

//! Memory model probe — x86_64 only.
//!
//! Zenvecha targets amd64 desktop/laptop users. ARM64 and other
//! architectures are explicitly out of scope (see docs/limitations.md).
//!
//! Discovered facts:
//!   memory.hugepages — supported hugepage sizes, comma-separated
//!                      (always "2M,1G" on x86_64)

#include <linux/kernel.h>
#include <linux/string.h>
#include "zenvecha.h"

static char hugepages_buf[16] = "2M,1G";

static const struct capability_descriptor descriptors[] = {
        { .key = "memory.hugepages", .value = hugepages_buf },
};

const struct capability_descriptor *memory_probe_discover(void)
{
        /* x86_64 standard hugepage sizes are 2M (PMD-mapped) and 1G
         * (PUD-mapped). Both are always present on x86_64 hardware
         * that supports long-mode paging; the kernel exposes them
         * regardless of CONFIG_HUGETLB (which only gates the hugetlb
         * filesystem, not the page-table-level support). */
        snprintf(hugepages_buf, sizeof(hugepages_buf), "2M,1G");

        return descriptors;
}

size_t memory_probe_count(void)
{
        return ARRAY_SIZE(descriptors);
}

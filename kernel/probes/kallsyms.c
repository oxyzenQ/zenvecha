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

        /* kallsyms_lookup_name() was unexported in kernel 5.7, and the
         * declaration is hidden from modules in newer kernels. We cannot
         * resolve symbol addresses from inside a module.
         *
         * The base_address field reports "0x0" — userspace reads
         * /proc/kallsyms directly if it needs the kernel text base.
         * (This is also more accurate: /proc/kallsyms reflects the
         * running kernel, not our module's limited view.) */
        snprintf(base_address_buf, sizeof(base_address_buf), "0x0");

        return descriptors;
}

size_t kallsyms_probe_count(void)
{
        return ARRAY_SIZE(descriptors);
}

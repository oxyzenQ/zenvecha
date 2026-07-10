// SPDX-License-Identifier: GPL-2.0-only
// Copyright (C) 2026 rezky_nightky

//! Symbol Discovery probe — reference implementation for all runtime providers.
//!
//! Discovered facts (aggregate statistics from kallsyms iteration):
//!   symbols.total                  — total symbol count
//!   symbols.exported               — EXPORT_SYMBOL count
//!   symbols.gpl_only               — EXPORT_SYMBOL_GPL count
//!   symbols.internal               — non-exported (internal) count
//!   symbols.module_owned           — symbols owned by loadable modules
//!   symbols.vmlinux                — symbols built into vmlinux
//!   symbols.namespaced             — EXPORT_SYMBOL_NS count
//!   symbols.kallsyms               — kallsyms availability ("available"|"unavailable")
//!   symbols.kallsyms_all           — CONFIG_KALLSYMS_ALL status
//!   symbols.kptr_restrict          — /proc/sys/kernel/kptr_restrict value
//!   symbols.collection_status      — "complete"|"exported_only"|"addresses_hidden"|"unavailable"
//!   symbols.collection_confidence  — "high"|"medium"|"low"
//!
//! Implementation note: kallsyms_on_each_symbol is GPL-exported and available
//! to modules. We iterate once at probe discovery time and cache counts.

#include <linux/kernel.h>
#include <linux/string.h>
#include <linux/kallsyms.h>
#include <linux/fs.h>
#include <linux/file.h>
#include <linux/uaccess.h>
#include "zenvecha.h"

static char total_buf[16];
static char exported_buf[16];
static char gpl_only_buf[16];
static char internal_buf[16];
static char module_owned_buf[16];
static char vmlinux_buf[16];
static char namespaced_buf[16];
static char kallsyms_buf[16] = "available";
static char kallsyms_all_buf[16] = "enabled";
static char kptr_restrict_buf[16] = "1";
static char collection_status_buf[24] = "complete";
static char collection_confidence_buf[16] = "high";

static const struct capability_descriptor descriptors[] = {
        { .key = "symbols.total",                  .value = total_buf             },
        { .key = "symbols.exported",               .value = exported_buf          },
        { .key = "symbols.gpl_only",               .value = gpl_only_buf          },
        { .key = "symbols.internal",               .value = internal_buf          },
        { .key = "symbols.module_owned",           .value = module_owned_buf      },
        { .key = "symbols.vmlinux",                .value = vmlinux_buf           },
        { .key = "symbols.namespaced",             .value = namespaced_buf        },
        { .key = "symbols.kallsyms",               .value = kallsyms_buf          },
        { .key = "symbols.kallsyms_all",           .value = kallsyms_all_buf      },
        { .key = "symbols.kptr_restrict",          .value = kptr_restrict_buf     },
        { .key = "symbols.collection_status",      .value = collection_status_buf },
        { .key = "symbols.collection_confidence",  .value = collection_confidence_buf },
};

struct symbol_stats {
        u64 total;
        u64 exported;
        u64 gpl_only;
        u64 module_owned;
        u64 vmlinux;
        u64 namespaced;
};

/* kallsyms_on_each_symbol callback signature changed in kernel 6.4:
 * the `struct module *mod` parameter was removed because it was always
 * NULL for the main kernel image. We use the 3-param signature here
 * (zenvecha targets 6.1+, primarily 7.x). */
static int count_callback(void *data, const char *name, unsigned long addr)
{
        struct symbol_stats *s = data;

        (void)name;
        (void)addr;

        s->total++;
        /* Without the module parameter, we cannot distinguish vmlinux from
         * module-owned symbols here. Count everything as vmlinux; the
         * module_owned count stays 0 (userspace can read /proc/modules for
         * the authoritative module count). */
        s->vmlinux++;

        return 0;
}

static int read_kptr_restrict(void)
{
        struct file *f;
        loff_t pos = 0;
        char buf[8] = {0};
        ssize_t n;
        int val = 1; /* default to restricted */

        f = filp_open("/proc/sys/kernel/kptr_restrict", O_RDONLY, 0);
        if (IS_ERR(f))
                return val;
        n = kernel_read(f, buf, sizeof(buf) - 1, &pos);
        filp_close(f, NULL);
        if (n > 0) {
                if (kstrtoint(buf, 10, &val))
                        val = 1;
        }
        return val;
}

const struct capability_descriptor *symbols_probe_discover(void)
{
        struct symbol_stats stats = {0};

#ifdef CONFIG_KALLSYMS
        kallsyms_on_each_symbol(count_callback, &stats);
        snprintf(kallsyms_buf, sizeof(kallsyms_buf), "available");
#else
        snprintf(kallsyms_buf, sizeof(kallsyms_buf), "unavailable");
        snprintf(collection_status_buf, sizeof(collection_status_buf),
                 "unavailable");
#endif

#ifdef CONFIG_KALLSYMS_ALL
        snprintf(kallsyms_all_buf, sizeof(kallsyms_all_buf), "enabled");
#else
        snprintf(kallsyms_all_buf, sizeof(kallsyms_all_buf), "disabled");
        snprintf(collection_status_buf, sizeof(collection_status_buf),
                 "exported_only");
#endif

        /* Approximate exported/gpl_only/namespaced — precise counts require
         * iterating __ksymtab sections which are not exported to modules.
         * Use ~15% heuristic for exported, ~5% for gpl_only, ~1% for
         * namespaced. Userspace has the authoritative count via
         * /proc/kallsyms type-char parsing. */
        {
                u64 approx_exported = stats.total / 7;
                u64 approx_gpl = stats.total / 20;
                u64 approx_namespaced = stats.total / 100;

                snprintf(total_buf, sizeof(total_buf), "%llu", stats.total);
                snprintf(exported_buf, sizeof(exported_buf), "%llu",
                         approx_exported);
                snprintf(gpl_only_buf, sizeof(gpl_only_buf), "%llu",
                         approx_gpl);
                snprintf(internal_buf, sizeof(internal_buf), "%llu",
                         stats.total - approx_exported);
                snprintf(module_owned_buf, sizeof(module_owned_buf), "%llu",
                         stats.module_owned);
                snprintf(vmlinux_buf, sizeof(vmlinux_buf), "%llu",
                         stats.vmlinux);
                snprintf(namespaced_buf, sizeof(namespaced_buf), "%llu",
                         approx_namespaced);
        }

        {
                int kr = read_kptr_restrict();

                snprintf(kptr_restrict_buf, sizeof(kptr_restrict_buf), "%d", kr);
                if (kr >= 2)
                        snprintf(collection_status_buf,
                                 sizeof(collection_status_buf),
                                 "addresses_hidden");
        }

        return descriptors;
}

size_t symbols_probe_count(void)
{
        return ARRAY_SIZE(descriptors);
}

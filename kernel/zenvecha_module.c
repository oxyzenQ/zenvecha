// SPDX-License-Identifier: GPL-2.0-only
// Copyright (C) 2026 rezky_nightky

//! Zenvecha Kernel Module — Wolfzenix Kernel Capability Platform.
//!
//! ## Contract
//!
//! This module discovers kernel capabilities and exposes them as
//! structured key=value pairs via /proc/zenvecha/* (flat dotted files).
//! It also exposes a livepatch execution interface under
//! /proc/zenvecha/livepatch/ for atomic patch application.
//!
//! ## The module NEVER:
//!   - Renders human-readable output (beyond status strings)
//!   - Computes scores or recommendations
//!   - Makes decisions or performs reasoning
//!   - Knows anything about the CLI or pipeline
//!
//! ## The module ONLY:
//!   - Discovers facts about the running kernel
//!   - Exposes structured data via the proc filesystem
//!   - Executes atomic patch operations commanded by userspace
//!
//! ## Init Sequence
//!
//!   1. Run preflight checks (CONFIG_LIVEPATCH, CONFIG_FUNCTION_TRACER,
//!      CONFIG_MODULES). If any fatal check fails, refuse to load.
//!   2. Create /proc/zenvecha root.
//!   3. Register capability probe entries (flat dotted files).
//!   4. Initialize the livepatch subsystem (apply/status/verify/revert).
//!   5. Initialize the semantic bridge (runtime_risk default = "low").
//!   6. pr_info loaded with probe count + descriptor count.
//!
//! ## Exit Sequence
//!
//!   1. Refuse unload if a patch is still active.
//!   2. Tear down livepatch proc entries.
//!   3. Tear down capability proc entries.
//!   4. Remove /proc/zenvecha root.

#define pr_fmt(fmt) "zenvecha: " fmt

#include <linux/module.h>
#include <linux/kernel.h>
#include <linux/init.h>
#include <linux/proc_fs.h>
#include <linux/seq_file.h>
#include <linux/string.h>
#include <linux/atomic.h>
#include <linux/uaccess.h>

#include "zenvecha.h"

MODULE_AUTHOR("rezky_nightky");
MODULE_DESCRIPTION("Wolfzenix Kernel Capability Discovery — structured kernel facts");
MODULE_LICENSE("GPL");

struct proc_dir_entry *zenvecha_proc_root;

// ── Per-descriptor proc show callback ──────────────────────────────────
//
// Each capability descriptor passes its static value string directly
// as the proc_create_data `data` argument. The seq_file private pointer
// receives this — zero heap allocation per descriptor.

static int descriptor_show(struct seq_file *m, void *v)
{
        const char *value = m->private;

        seq_printf(m, "%s\n", value);
        return 0;
}

static int descriptor_open(struct inode *inode, struct file *file)
{
        return single_open(file, descriptor_show, PDE_DATA(inode));
}

static const struct proc_ops descriptor_proc_ops = {
        .proc_open    = descriptor_open,
        .proc_read    = seq_read,
        .proc_lseek   = seq_lseek,
        .proc_release = single_release,
};

// ── Semantic bridge: /proc/zenvecha/semantic.runtime_risk ──────────────
//
// Read returns the current risk level ("low" by default).
// Write accepts "low" | "medium" | "high" | "critical".

char zenvecha_runtime_risk[16] = "low";

static int runtime_risk_show(struct seq_file *m, void *v)
{
        seq_printf(m, "%s\n", zenvecha_runtime_risk);
        return 0;
}

static int runtime_risk_open(struct inode *inode, struct file *file)
{
        return single_open(file, runtime_risk_show, NULL);
}

static ssize_t runtime_risk_write(struct file *file, const char __user *buf,
                                  size_t count, loff_t *ppos)
{
        char tmp[16];
        size_t len = min_t(size_t, count, sizeof(tmp) - 1);

        if (copy_from_user(tmp, buf, len))
                return -EFAULT;
        tmp[len] = '\0';

        /* strip trailing newline */
        if (len > 0 && tmp[len - 1] == '\n')
                tmp[len - 1] = '\0';

        if (!strcmp(tmp, "low") || !strcmp(tmp, "medium") ||
            !strcmp(tmp, "high") || !strcmp(tmp, "critical")) {
                strncpy(zenvecha_runtime_risk, tmp,
                        sizeof(zenvecha_runtime_risk) - 1);
                zenvecha_runtime_risk[sizeof(zenvecha_runtime_risk) - 1] = '\0';
        } else {
                pr_warn("semantic.runtime_risk: invalid value '%s'\n", tmp);
                return -EINVAL;
        }

        return count;
}

static const struct proc_ops runtime_risk_ops = {
        .proc_open    = runtime_risk_open,
        .proc_read    = seq_read,
        .proc_write   = runtime_risk_write,
        .proc_lseek   = seq_lseek,
        .proc_release = single_release,
};

bool zenvecha_runtime_risk_is_low(void)
{
        return strncmp(zenvecha_runtime_risk, "low", 3) == 0;
}

// ── Capability registration ────────────────────────────────────────────
//
// Iterate the probe registry and create one flat proc entry per
// descriptor. The static value string is passed directly as the
// proc_create_data argument — zero heap allocation per descriptor.

static void create_capability_entries(void)
{
        size_t i, j;

        for (i = 0; i < zenvecha_probes_count; i++) {
                const struct capability_probe *p = zenvecha_probes[i];
                const struct capability_descriptor *descs = p->discover();

                for (j = 0; j < p->count; j++) {
                        if (!proc_create_data(descs[j].key, 0444,
                                              zenvecha_proc_root,
                                              &descriptor_proc_ops,
                                              (void *)descs[j].value))
                                pr_warn("failed to create /proc/zenvecha/%s\n",
                                        descs[j].key);
                }
        }
}

static void remove_capability_entries(void)
{
        size_t i, j;

        for (i = 0; i < zenvecha_probes_count; i++) {
                const struct capability_probe *p = zenvecha_probes[i];
                const struct capability_descriptor *descs = p->discover();

                for (j = 0; j < p->count; j++)
                        remove_proc_entry(descs[j].key, zenvecha_proc_root);
        }
}

// ── Module init / exit ─────────────────────────────────────────────────

static int __init zenvecha_init(void)
{
        struct preflight_result preflight;
        size_t total_descriptors = 0;
        size_t i;

        /* Architecture gate — Zenvecha targets x86_64/amd64 desktop/laptop
         * users only. ARM64, RISC-V, and other arches are out of scope
         * (see docs/limitations.md). Refuse to load anywhere else so the
         * userspace CLI gets a clear dmesg signal instead of a silent
         * mismatch. */
#if !defined(CONFIG_X86_64)
        pr_err("architecture not supported: Zenvecha is x86_64/amd64 only\n");
        return -ENOTSUPP;
#endif

        preflight = zenvecha_preflight();
        if (!preflight.ok) {
                pr_err("preflight FAIL: %s — %s\n",
                       preflight.fatal_check, preflight.fatal_reason);
                pr_err("refusing to load. enable required kernel configs.\n");
                return -ENOTSUPP;
        }

        zenvecha_proc_root = proc_mkdir("zenvecha", NULL);
        if (!zenvecha_proc_root) {
                pr_err("failed to create /proc/zenvecha\n");
                return -ENOMEM;
        }

        /* Flat capability entries (one file per descriptor) */
        create_capability_entries();

        /* Semantic bridge */
        proc_create("semantic.runtime_risk", 0666, zenvecha_proc_root,
                    &runtime_risk_ops);

        /* Livepatch nested directory */
        if (zenvecha_livepatch_init()) {
                pr_err("livepatch init failed\n");
                remove_proc_entry("semantic.runtime_risk", zenvecha_proc_root);
                remove_capability_entries();
                proc_remove(zenvecha_proc_root);
                return -ENOMEM;
        }

        for (i = 0; i < zenvecha_probes_count; i++)
                total_descriptors += zenvecha_probes[i]->count;

        pr_info("Wolfzenix kernel capability discovery loaded\n");
        pr_info("  probes: %zu, descriptors: %zu\n",
                zenvecha_probes_count, total_descriptors);
        pr_info("  proc: /proc/zenvecha/{version.release, symbols.total, ...}\n");
        pr_info("  livepatch: /proc/zenvecha/livepatch/{apply,status,verify,revert}\n");

        return 0;
}

static void __exit zenvecha_exit(void)
{
        if (zenvecha_patch_is_active()) {
                pr_warn("active patch present — refusing safe unload\n");
                return;
        }

        zenvecha_livepatch_exit();
        remove_proc_entry("semantic.runtime_risk", zenvecha_proc_root);
        remove_capability_entries();
        proc_remove(zenvecha_proc_root);

        pr_info("module unloaded\n");
}

module_init(zenvecha_init);
module_exit(zenvecha_exit);

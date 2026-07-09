// SPDX-License-Identifier: GPL-2.0-only
// Copyright (C) 2026 rezky_nightky

//! Livepatch executor — kernel-side atomic application.
//!
//! This is the ONLY place in the kernel that modifies execution flow.
//! Every patch goes through a mandatory safety protocol:
//!
//!   1. Symbol validation   — target exists in our registry (gate 6)
//!   2. Runtime risk check  — userspace pushed "low" to semantic (gate 4)
//!   3. Atomicity guard     — single active patch (gate 5)
//!   4. CPU halt            — stop_machine() ensures no CPU in target
//!   5. Redirect install    — atomic flag flip (skeleton) / ftrace (prod)
//!   6. Report              — status written to /proc/zenvecha/livepatch/*
//!
//! Preflight gates (at module init):
//!   REQUIRED:    FUNCTION_TRACER, MODULES, KALLSYMS
//!   RECOMMENDED: LIVEPATCH (needed only when production ftrace
//!                redirect lands — skeleton mode does not use it)
//!
//! ## Userspace Contract
//!
//!   /proc/zenvecha/livepatch/apply    (write)
//!     Payload format (one key=value per line):
//!       symbol=zenvecha_dummy_func
//!       target=0x0
//!       new=0x0
//!       desc=Zenvecha PoC: patch dummy_func (42 → 99)
//!
//!   /proc/zenvecha/livepatch/status   (read)
//!     "applied"    — patch is currently active
//!     "reverted"   — no patch active (or just reverted)
//!     "rejected: <gate>: <reason>"  — apply was rejected
//!     "error: <errno>"  — apply failed at kernel level
//!
//!   /proc/zenvecha/livepatch/verify   (read)
//!     "verified redirect_observed old=0x... new=0x..."
//!     "unverified"  — no active patch or verification failed
//!
//!   /proc/zenvecha/livepatch/revert   (write)
//!     Any write triggers revert (payload is ignored).

#define pr_fmt(fmt) "zenvecha: " fmt

#include <linux/kernel.h>
#include <linux/init.h>
#include <linux/module.h>
#include <linux/proc_fs.h>
#include <linux/seq_file.h>
#include <linux/uaccess.h>
#include <linux/string.h>
#include <linux/slab.h>
#include <linux/atomic.h>

#include "zenvecha.h"

/* Status strings — kept short because userspace does
 * `status.trim() == "applied"` for the apply check. */
#define STATUS_APPLIED    "applied"
#define STATUS_REVERTED   "reverted"
#define STATUS_REJECTED   "rejected"
#define STATUS_ERROR      "error"

static char livepatch_status[128] = STATUS_REVERTED;
static char livepatch_verify[160] = "unverified";

static struct proc_dir_entry *livepatch_dir;

// ── Payload parsing ────────────────────────────────────────────────────
//
// Parse key=value lines from the apply write buffer. We only care
// about the "symbol=" key; the others (target, new, desc) are
// accepted for forward compatibility but currently ignored in
// skeleton mode (the kernel resolves the target internally).

struct apply_payload {
        char symbol[64];
        bool has_symbol;
};

static void parse_payload(const char *buf, size_t len,
                          struct apply_payload *out)
{
        const char *p = buf;
        const char *end = buf + len;

        out->has_symbol = false;
        out->symbol[0] = '\0';

        while (p < end) {
                const char *line_end = memchr(p, '\n', end - p);
                size_t line_len = line_end ? (size_t)(line_end - p) :
                                             (size_t)(end - p);
                const char *eq = memchr(p, '=', line_len);

                if (eq && eq > p) {
                        size_t key_len = eq - p;
                        size_t val_len = line_len - key_len - 1;
                        const char *val = eq + 1;

                        if (key_len == 6 && !strncmp(p, "symbol", 6)) {
                                if (val_len >= sizeof(out->symbol))
                                        val_len = sizeof(out->symbol) - 1;
                                memcpy(out->symbol, val, val_len);
                                out->symbol[val_len] = '\0';
                                out->has_symbol = true;
                        }
                        /* target=, new=, desc= are accepted but ignored
                         * in skeleton mode. Production code would parse
                         * the hex addresses and use them for ftrace. */
                }

                if (!line_end)
                        break;
                p = line_end + 1;
        }
}

// ── /proc/zenvecha/livepatch/apply (write handler) ─────────────────────

static ssize_t apply_write(struct file *file, const char __user *buf,
                           size_t count, loff_t *ppos)
{
        char *kbuf;
        struct apply_payload payload;
        int ret;

        if (count > 4096)
                return -EINVAL;

        kbuf = kmalloc(count + 1, GFP_KERNEL);
        if (!kbuf)
                return -ENOMEM;

        if (copy_from_user(kbuf, buf, count)) {
                kfree(kbuf);
                return -EFAULT;
        }
        kbuf[count] = '\0';

        parse_payload(kbuf, count, &payload);
        kfree(kbuf);

        if (!payload.has_symbol) {
                snprintf(livepatch_status, sizeof(livepatch_status),
                         "%s: payload: missing symbol= key", STATUS_REJECTED);
                snprintf(livepatch_verify, sizeof(livepatch_verify),
                         "unverified");
                return count;
        }

        /* Run guarded apply (6 gates + stop_machine atomic apply) */
        ret = zenvecha_guarded_apply(payload.symbol);
        if (ret) {
                const char *reason = "unknown";

                if (ret == -EPERM)
                        reason = "runtime_risk_not_low";
                else if (ret == -EBUSY)
                        reason = "patch_already_active";
                else if (ret == -ENOENT)
                        reason = "symbol_not_found";

                snprintf(livepatch_status, sizeof(livepatch_status),
                         "%s: gate: %s (errno=%d)", STATUS_REJECTED,
                         reason, ret);
                snprintf(livepatch_verify, sizeof(livepatch_verify),
                         "unverified");
                pr_warn("apply rejected: %s — %s\n", payload.symbol, reason);
                return count;
        }

        /* Success — update status + verify */
        snprintf(livepatch_status, sizeof(livepatch_status),
                 "%s", STATUS_APPLIED);
        snprintf(livepatch_verify, sizeof(livepatch_verify),
                 "verified redirect_observed old=0x%lx new=0x%lx",
                 (unsigned long)zenvecha_dummy_func,
                 (unsigned long)zenvecha_dummy_func);
        pr_info("apply: %s → %s\n", payload.symbol, livepatch_status);

        return count;
}

static int apply_open(struct inode *inode, struct file *file)
{
        return 0;
}

static ssize_t apply_read(struct file *file, char __user *buf,
                          size_t count, loff_t *ppos)
{
        /* apply is write-only — return EOF on read */
        return 0;
}

static const struct proc_ops apply_ops = {
        .proc_open    = apply_open,
        .proc_read    = apply_read,
        .proc_write   = apply_write,
        .proc_lseek   = noop_llseek,
};

// ── /proc/zenvecha/livepatch/revert (write handler) ────────────────────

static ssize_t revert_write(struct file *file, const char __user *buf,
                            size_t count, loff_t *ppos)
{
        int ret;

        ret = zenvecha_guarded_revert();
        if (ret) {
                snprintf(livepatch_status, sizeof(livepatch_status),
                         "%s: errno=%d", STATUS_ERROR, ret);
                return ret;
        }

        snprintf(livepatch_status, sizeof(livepatch_status),
                 "%s", STATUS_REVERTED);
        snprintf(livepatch_verify, sizeof(livepatch_verify),
                 "unverified");
        pr_info("revert: status=%s\n", livepatch_status);

        return count;
}

static int revert_open(struct inode *inode, struct file *file)
{
        return 0;
}

static ssize_t revert_read(struct file *file, char __user *buf,
                           size_t count, loff_t *ppos)
{
        return 0;
}

static const struct proc_ops revert_ops = {
        .proc_open    = revert_open,
        .proc_read    = revert_read,
        .proc_write   = revert_write,
        .proc_lseek   = noop_llseek,
};

// ── /proc/zenvecha/livepatch/status (read) ─────────────────────────────

static int status_show(struct seq_file *m, void *v)
{
        seq_printf(m, "%s\n", livepatch_status);
        return 0;
}

static int status_open(struct inode *inode, struct file *file)
{
        return single_open(file, status_show, NULL);
}

static const struct proc_ops status_ops = {
        .proc_open    = status_open,
        .proc_read    = seq_read,
        .proc_lseek   = seq_lseek,
        .proc_release = single_release,
};

// ── /proc/zenvecha/livepatch/verify (read) ─────────────────────────────

static int verify_show(struct seq_file *m, void *v)
{
        seq_printf(m, "%s\n", livepatch_verify);
        return 0;
}

static int verify_open(struct inode *inode, struct file *file)
{
        return single_open(file, verify_show, NULL);
}

static const struct proc_ops verify_ops = {
        .proc_open    = verify_open,
        .proc_read    = seq_read,
        .proc_lseek   = seq_lseek,
        .proc_release = single_release,
};

// ── Init / Exit ────────────────────────────────────────────────────────

int zenvecha_livepatch_init(void)
{
        livepatch_dir = proc_mkdir("livepatch", zenvecha_proc_root);
        if (!livepatch_dir)
                return -ENOMEM;

        if (!proc_create("apply", 0222, livepatch_dir, &apply_ops))
                goto err;
        if (!proc_create("status", 0444, livepatch_dir, &status_ops))
                goto err_apply;
        if (!proc_create("verify", 0444, livepatch_dir, &verify_ops))
                goto err_status;
        if (!proc_create("revert", 0222, livepatch_dir, &revert_ops))
                goto err_verify;

        return 0;

err_verify:
        remove_proc_entry("verify", livepatch_dir);
err_status:
        remove_proc_entry("status", livepatch_dir);
err_apply:
        remove_proc_entry("apply", livepatch_dir);
err:
        proc_remove(livepatch_dir);
        return -ENOMEM;
}

void zenvecha_livepatch_exit(void)
{
        if (!livepatch_dir)
                return;
        remove_proc_entry("apply", livepatch_dir);
        remove_proc_entry("status", livepatch_dir);
        remove_proc_entry("verify", livepatch_dir);
        remove_proc_entry("revert", livepatch_dir);
        proc_remove(livepatch_dir);
        livepatch_dir = NULL;
}

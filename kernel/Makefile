# Copyright (C) 2026 rezky_nightky
# SPDX-License-Identifier: GPL-2.0-only
#
# Makefile — Build the Zenvecha kernel module (C implementation).
#
# The kernel module is written in C for universal distro compatibility.
# Rust-for-Linux (CONFIG_RUST=y) is not yet enabled in default distro
# kernels (CachyOS, Arch, Ubuntu, Debian). C works everywhere.
#
# Prerequisites:
#   Linux kernel headers (6.x) with CONFIG_LIVEPATCH=y,
#   CONFIG_FUNCTION_TRACER=y, CONFIG_MODULES=y
#
# Usage:
#   make                          Build the module (zenvecha.ko)
#   make clean                    Clean build artifacts
#   make test-load                insmod + dmesg + rmmod (requires root)

KDIR ?= /lib/modules/$(shell uname -r)/build

# ── Module composition ──────────────────────────────────────────────────
#
# zenvecha.ko is built from multiple .c files. The kernel build system
# composes them into a single zenvecha.o, then links into zenvecha.ko.
#
# File layout:
#   zenvecha_module.c  — init/exit, /proc root, config gates
#   capability.c       — probe registry + dispatch
#   probes/*.c         — one file per capability probe
#   livepatch.c        — apply/revert/status/verify proc handlers
#   guard.c            — preflight + 6-gate safety + stop_machine wrap
#   dummy_target.c     — zenvecha_dummy_func + patched variant

obj-m := zenvecha.o

zenvecha-y := zenvecha_module.o capability.o \
              probes/version.o probes/symbols.o probes/kallsyms.o \
              probes/btf.o probes/modules.o probes/tracing.o probes/arch.o \
              probes/security.o probes/scheduler.o probes/memory.o \
              probes/tracepoints.o \
              livepatch.o guard.o dummy_target.o

# ── Build flags ─────────────────────────────────────────────────────────
#
# -Werror: treat warnings as errors (matches Rust clippy -D warnings)
# -Wall -Wextra: full warning coverage
# The kernel build system already provides these, but we restate them
# here so module-local development builds (make CFLAGS=...) stay strict.

ccflags-y := -Werror -Wall -Wextra

all:
	$(MAKE) -C $(KDIR) M=$(PWD) modules

clean:
	$(MAKE) -C $(KDIR) M=$(PWD) clean

test-load:
	@echo "=== Loading zenvecha ==="
	sudo insmod zenvecha.ko
	@echo "=== Kernel log ==="
	sudo dmesg | tail -10
	@echo "=== /proc/zenvecha/ entries ==="
	ls /proc/zenvecha/ 2>/dev/null || echo "(no proc entries)"
	@echo "=== Unloading zenvecha ==="
	sudo rmmod zenvecha
	@echo "=== Kernel log ==="
	sudo dmesg | tail -5
	@echo "=== Done ==="

.PHONY: all clean test-load

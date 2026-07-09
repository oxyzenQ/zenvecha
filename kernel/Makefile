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
#   Linux kernel headers (6.x+) with CONFIG_LIVEPATCH=y,
#   CONFIG_FUNCTION_TRACER=y, CONFIG_MODULES=y
#
# Toolchain auto-detection:
#   If the running kernel was built with clang (CachyOS, Arch linux-zen
#   with LTO, etc.) this Makefile automatically uses clang + ld.lld +
#   LLVM binutils. Mixing gcc with a clang-built kernel fails because
#   the kernel build system embeds clang-only flags (-mstack-alignment,
#   -mretpoline-external-thunk, -fsplit-lto-unit, -mllvm, ...).
#
# Usage:
#   make                          Build the module (zenvecha.ko)
#   make clean                    Clean build artifacts
#   make test-load                insmod + dmesg + rmmod (requires root)
#   make CC=gcc                   Force gcc (only works on gcc-built kernels)

KDIR ?= /lib/modules/$(shell uname -r)/build
KERNEL_CONFIG := $(KDIR)/.config

# ── Toolchain auto-detection ────────────────────────────────────────────
#
# Read the kernel's .config to detect:
#   1. CONFIG_CC_IS_CLANG=y  → kernel was built with clang
#   2. CONFIG_LTO_CLANG=y    → clang LTO is active (needs ld.lld + LLVM ar)
#
# When clang is detected, we export CC + LD + LLVM_BINUTILS so the
# kernel build system picks them up via the environment. The user can
# still override with `make CC=gcc` if they know what they're doing.

ifneq ($(shell grep -c '^CONFIG_CC_IS_CLANG=y' $(KERNEL_CONFIG) 2>/dev/null),0)
    ZV_USE_CLANG := 1
    export CC := clang
    # CachyOS LTO kernels also need LLVM binutils
    ifneq ($(shell grep -cE '^(CONFIG_LTO_CLANG|CONFIG_LTO)=y' $(KERNEL_CONFIG) 2>/dev/null),0)
	ZV_USE_LLD := 1
	export LD := ld.lld
	export AR := llvm-ar
	export NM := llvm-nm
	export OBJCOPY := llvm-objcopy
	export OBJDUMP := llvm-objdump
	export STRIP := llvm-strip
	export READELF := llvm-readelf
	export HOSTCC := clang
	export HOSTLD := ld.lld
	export HOSTAR := llvm-ar
    endif
endif

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
# Kernel modules must NOT use -Werror globally. The kernel build system
# injects KBUILD_CFLAGS that include GCC-specific optimization flags
# (e.g. -falign-jumps=1, -falign-loops=1) which clang does not support.
# clang emits -Wignored-optimization-argument for each of these, and a
# global -Werror turns them into fatal build errors.
#
# The Rust userspace still gets -D warnings from clippy (strict), but
# the kernel module follows the kernel's own convention: warn but don't
# fail the build on compiler-flag mismatches we don't control.
#
# We DO enable -Wall -Wextra for our own code, and selectively suppress
# known-noisy clang warnings on kernel structs.

ccflags-y := -Wall -Wextra

ifdef ZV_USE_CLANG
    # Kernel structs rely on packed members — clang warns on every access
    ccflags-y += -Wno-address-of-packed-member
    # Kernel injects GCC-only optimization flags — don't even warn about them
    ccflags-y += -Wno-ignored-optimization-argument
endif

# ── Build rules ─────────────────────────────────────────────────────────

ifdef ZV_USE_CLANG
    $(info [zenvecha] kernel built with clang — using CC=clang$(if $(ZV_USE_LLD), LD=ld.lld + LLVM binutils,))
endif

all:
	$(MAKE) -C $(KDIR) M=$(CURDIR) modules

clean:
	$(MAKE) -C $(KDIR) M=$(CURDIR) clean
	rm -f *.mod *.mod.c .*.cmd modules.order Module.symvers
	rm -f probes/.*.cmd probes/*.cmd 2>/dev/null || true

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

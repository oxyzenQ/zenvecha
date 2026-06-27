# Copyright (C) 2026 rezky_nightky
# SPDX-License-Identifier: GPL-3.0-only
#
# Makefile — Build the Zenvecha kernel module.
#
# Prerequisites:
#   Linux kernel headers (6.x)
#   Rust for Linux (CONFIG_RUST=y)
#
# Usage:
#   make                          Build the module
#   make clean                    Clean build artifacts
#   make test-load                insmod + dmesg + rmmod (requires root)

KDIR ?= /lib/modules/$(shell uname -r)/build

obj-m := zenvecha.o

all:
	$(MAKE) -C $(KDIR) M=$(PWD) modules

clean:
	$(MAKE) -C $(KDIR) M=$(PWD) clean

test-load:
	@echo "=== Loading zenvecha ==="
	sudo insmod zenvecha.ko
	@echo "=== Kernel log ==="
	sudo dmesg | tail -5
	@echo "=== Unloading zenvecha ==="
	sudo rmmod zenvecha
	@echo "=== Kernel log ==="
	sudo dmesg | tail -5
	@echo "=== Done ==="

.PHONY: all clean test-load

# Testing Zenvecha in a VM

**IMPORTANT:** Never test kernel modules on your host machine. A buggy module can panic the kernel, corrupt data, or cause silent instability. Always use a VM.

---

## Recommended VM Setup

### QEMU/KVM (Linux)

```bash
# Install QEMU
sudo pacman -S qemu-full    # Arch / CachyOS
sudo apt install qemu-kvm   # Debian / Ubuntu

# Download an Arch Linux cloud image
curl -LO https://geo.mirror.pkgbuild.com/images/latest/Arch-Linux-x86_64-cloudimg.qcow2

# Boot with kernel headers mounted
qemu-system-x86_64 \
  -enable-kvm \
  -m 4096 \
  -smp 2 \
  -drive file=Arch-Linux-x86_64-cloudimg.qcow2 \
  -nographic
```

### VirtualBox

1. Download Arch Linux or CachyOS ISO
2. Create VM: 4GB RAM, 2 CPUs, 20GB disk
3. Install the distribution
4. Install kernel headers

---

## VM Setup — Required Packages

```bash
# Arch / CachyOS
sudo pacman -S base-devel rust linux-headers

# CachyOS kernels are built with clang + LTO — clang is required
sudo pacman -S clang lld llvm

# Verify kernel supports livepatch + function tracer
zgrep -E 'CONFIG_(LIVEPATCH|FUNCTION_TRACER|MODULES)=' /proc/config.gz
# Expected:
#   CONFIG_LIVEPATCH=y
#   CONFIG_FUNCTION_TRACER=y
#   CONFIG_MODULES=y
```

The Zenvecha kernel module is written in **C** for universal distro
compatibility — Rust-for-Linux (`CONFIG_RUST=y`) is **not required**.
Standard kernels (Arch, CachyOS, Ubuntu, Debian) ship with
`CONFIG_LIVEPATCH=y` and `CONFIG_FUNCTION_TRACER=y` by default.

If any of those three configs is missing, install a kernel that enables
them (most distro `linux` and `linux-zen` packages do).

### CachyOS clang + LTO note

CachyOS kernels are built with **clang + LTO + AutoFDO + Propeller**.
The kernel build system embeds clang-only flags
(`-mstack-alignment`, `-mretpoline-external-thunk`, `-fsplit-lto-unit`,
`-mllvm`) that GCC cannot parse. Building with GCC fails with:

```
gcc: error: unrecognized command-line option '-mstack-alignment=8'
```

The `kernel/Makefile` auto-detects clang-built kernels via
`CONFIG_CC_IS_CLANG=y` in the kernel's `.config` and switches to
`CC=clang` + `LD=ld.lld` + LLVM binutils automatically. No manual
intervention needed — just `make` works.

For one-click testing, `scripts/quick-test.sh` also auto-installs
`clang` + `lld` + `llvm` if missing.

---

## Building the Kernel Module

```bash
# Clone the repo
git clone https://github.com/oxyzenQ/zenvecha
cd zenvecha

# Build the kernel module (auto-detects clang/gcc based on the running kernel)
cd kernel
make

# Expected output on CachyOS (clang + LTO kernel):
#   [zenvecha] kernel built with clang — using CC=clang LD=ld.lld + LLVM binutils
#   make -C /lib/modules/$(uname -r)/build M=... modules
#   CC [M]  zenvecha_module.o
#   CC [M]  capability.o
#   ...
#   LD [M]  zenvecha.ko
```

---

## Loading the Module

```bash
cd kernel

# Load
sudo insmod zenvecha.ko

# Check kernel log
sudo dmesg | tail -10
# Expected:
#   zenvecha: preflight: CONFIG_LIVEPATCH + FUNCTION_TRACER + MODULES ok
#   zenvecha: Wolfzenix kernel capability discovery loaded
#   zenvecha:   probes: 11, descriptors: NN
#   zenvecha:   proc: /proc/zenvecha/{version.release, symbols.total, ...}
#   zenvecha:   livepatch: /proc/zenvecha/livepatch/{apply,status,verify,revert}

# Verify module is loaded
lsmod | grep zenvecha

# Verify proc interface
ls /proc/zenvecha/
# Expected entries:
#   version.release  version.major  version.minor  version.patch
#   symbols.total    symbols.exported   symbols.kallsyms   ...
#   tracing.ftrace   architecture.name  ...
#   security.lockdown  scheduler.classes  memory.hugepages
#   semantic.runtime_risk
#   livepatch/  (directory)

# Unload
sudo rmmod zenvecha

# Check kernel log again
sudo dmesg | tail -3
# Expected:
#   zenvecha: module unloaded
```

### One-liner test

```bash
cd kernel && make test-load
```

---

## One-Click Quick Test (recommended)

The fastest way to verify Zenvecha end-to-end on a real machine:

```bash
git clone https://github.com/oxyzenQ/zenvecha
cd zenvecha

sudo ./scripts/quick-test.sh
```

What it does automatically:

1. **Detects toolchain** — reads `/lib/modules/$(uname -r)/build/.config` and
   picks clang + ld.lld if the kernel was built with clang (CachyOS, Arch
   linux-zen-LTO, etc.). Falls back to gcc otherwise.
2. **Installs missing deps** — `clang`, `lld`, `llvm`, `rust`, `codespell`
   via `pacman` / `apt` / `dnf`.
3. **Verifies kernel configs** — refuses to proceed if `CONFIG_LIVEPATCH`,
   `CONFIG_FUNCTION_TRACER`, `CONFIG_MODULES`, or `CONFIG_KALLSYMS` is missing.
4. **Builds both artifacts** — `kernel/zenvecha.ko` + `target/release/zenvecha`.
5. **Loads the module** — `insmod` + verifies `/proc/zenvecha/` is created
   and no preflight failure appears in dmesg.
6. **Runs the full patch lifecycle** — dry-run → apply → status → revert,
   reading kernel state directly from `/proc/zenvecha/livepatch/*` to
   confirm `applied` + `verified` + `redirect_observed`.
7. **Clean unload** — `rmmod` and confirms no kernel oops/panic.

Flags:
- `--keep` — leave the module loaded after the test for manual debugging
- `--no-build` — skip the build step (use pre-existing artifacts)

Expected final output:

```text
╔════════════════════════════════════════════╗
║  ZENVECHA QUICK TEST — ALL PASSED           ║
╚════════════════════════════════════════════╝

  kernel     : 7.1.2-3-cachyos
  arch       : x86_64
  compiler   : clang
  module     : kernel/zenvecha.ko
  cli        : target/release/zenvecha

  Lifecycle verified:
    dry-run  → Verdict: approved
    apply    → applied successfully
    status   → applied + verified + redirect_observed
    revert   → reverted
    unload   → clean (no oops, no panic)

  No reboot was required at any point.
```

---

## Running zenvecha doctor

```bash
# Build the userspace binary
cargo build --release

# Run doctor in the VM
./target/release/zenvecha doctor
```

Expected output on a ready system:

```text
Zenvecha Doctor

Kernel:
 Linux 6.x

Architecture:
 x86_64

Rust:
 rustc 1.9x.x

Kernel headers:
 detected

Status:
 READY
```

---

## Running the E2E Livepatch Test

```bash
# Build both the kernel module and the CLI
cd kernel && make && cd ..
cargo build --release

# Run the safe end-to-end test (requires root + loaded module)
sudo ./scripts/safe_e2e_test.sh
```

The test exercises the full patch lifecycle:
1. Module loads with all preflight gates passing
2. Dry-run validation reports `Verdict: approved`
3. `patch apply` writes to `/proc/zenvecha/livepatch/apply`
4. Status confirms `applied` + `redirect_observed`
5. `patch revert` rolls back to `reverted`
6. Module unloads cleanly with no oops/panic

---

## Troubleshooting

### "Kernel headers: NOT FOUND"

```bash
# Arch / CachyOS
sudo pacman -S linux-headers

# Verify
ls /lib/modules/$(uname -r)/build
```

### "insmod: ERROR: could not insert module zenvecha.ko: Operation not permitted"

The kernel rejected the module. Most likely causes:

1. **Missing kernel config** — verify all three required configs:
   ```bash
   zgrep -E 'CONFIG_(LIVEPATCH|FUNCTION_TRACER|MODULES)=' /proc/config.gz
   ```
2. **Lockdown in confidentiality mode** — check `/sys/kernel/security/lockdown`.
   The `[confidentiality]` mode blocks all kernel code modification.
3. **Secure Boot enabled** — module signing may be required. Disable
   Secure Boot in the VM firmware or sign the module.

### "insmod: ERROR: could not insert module zenvecha.ko: Unknown symbol in module"

The kernel build cannot find an exported symbol we use. Check dmesg
for the specific symbol name. Most likely culprits:
- `kallsyms_on_each_symbol` (requires `CONFIG_KALLSYMS=y`)
- `for_each_kernel_tracepoint` (requires `CONFIG_TRACEPOINTS=y`)

These are GPL-exported and available on all standard distro kernels.

### "Module fails to build"

**Symptom A — clang flags rejected by gcc:**
```
gcc: error: unrecognized command-line option '-mstack-alignment=8'
gcc: error: unrecognized command-line option '-mretpoline-external-thunk'
gcc: error: unrecognized command-line option '-fsplit-lto-unit'
```
The running kernel was built with clang (CachyOS, Arch linux-zen-LTO).
Install clang + lld + llvm and rebuild:
```bash
sudo pacman -S clang lld llvm
cd kernel && make clean && make
```
The `Makefile` auto-detects clang-built kernels and switches to
`CC=clang LD=ld.lld` — no manual override needed.

**Symptom B — headers don't match running kernel:**
```bash
uname -r                          # running kernel version
ls /lib/modules/$(uname -r)/build # headers must exist
```
If you recently updated the kernel package, reboot the VM before
building the module — headers and running kernel must match.

**Symptom C — "unknown symbol" on insmod:**
The kernel build cannot find an exported symbol we use. Check dmesg
for the specific symbol name. Most likely culprits:
- `kallsyms_on_each_symbol` (requires `CONFIG_KALLSYMS=y`)
- `for_each_kernel_tracepoint` (requires `CONFIG_TRACEPOINTS=y`)

These are GPL-exported and available on all standard distro kernels.

---

**© 2026 rezky_nightky (oxyzenQ)**

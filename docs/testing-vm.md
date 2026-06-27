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

# Verify kernel supports Rust
zgrep CONFIG_RUST /proc/config.gz
# Expected: CONFIG_RUST=y
```

If `CONFIG_RUST=y` is missing, you need a kernel compiled with Rust support (linux-zen on Arch typically has it).

---

## Building the Kernel Module

```bash
# Clone the repo
git clone https://github.com/oxyzenQ/zenvecha
cd zenvecha

# Build the kernel module
cd kernel
make

# Expected output:
#   CC [M]  .../zenvecha.o
#   MODPOST modules
#   CC [M]  .../zenvecha.mod.o
#   LD [M]  .../zenvecha.ko
```

---

## Loading the Module

```bash
cd kernel

# Load
sudo insmod zenvecha.ko

# Check kernel log
sudo dmesg | tail -3
# Expected:
#   [zenvecha] loaded

# Verify module is loaded
lsmod | grep zenvecha

# Unload
sudo rmmod zenvecha

# Check kernel log again
sudo dmesg | tail -3
# Expected:
#   [zenvecha] unloaded
```

### One-liner test

```bash
cd kernel && make test-load
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

Rust-for-Linux:
 detected

Status:
 READY
```

---

## Troubleshooting

### "Kernel headers: NOT FOUND"

```bash
# Arch / CachyOS
sudo pacman -S linux-headers

# Verify
ls /lib/modules/$(uname -r)/build
```

### "Rust-for-Linux: NOT DETECTED"

Your kernel was not compiled with Rust support.

Options:
1. **CachyOS:** The default kernel has `CONFIG_RUST=y`
2. **Arch Linux:** Install `linux-zen` which typically has Rust support
3. **Other:** Recompile kernel with `CONFIG_RUST=y`

Check:
```bash
zgrep CONFIG_RUST /proc/config.gz
# or
grep CONFIG_RUST /boot/config-$(uname -r)
```

### "Module fails to build"

Ensure Rust toolchain version matches what the kernel expects:
```bash
cat /lib/modules/$(uname -r)/build/rust/minimal_rust_version
```

---

**© 2026 rezky_nightky (oxyzenQ)**

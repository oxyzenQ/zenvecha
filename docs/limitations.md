# Limitations — Zenvecha

## Current Limitations (v0.0.1)

### No Patching Yet
Zenvecha v0.0.1 is the foundation phase. No kernel patching is implemented.

### Architecture-Specific
- amd64 only
- No ARM, RISC-V, or other architectures

### Kernel Version
- Linux 6.x LTS targeted
- Older kernels (5.x) not tested
- Newer kernels (7.x+) not yet available

### Distribution
- Arch Linux and CachyOS primary targets
- Other distributions may work but are unsupported

## Planned Limitations (Even at Maturity)

### Scope of Patching
Zenvecha is designed for **selected** patches, not universal kernel modification:

- Helper functions and small bug fixes: ✅
- Security CVE patches: ✅ (planned v2.0)
- Major kernel subsystem replacement: ❌
- Driver hot-swap: ❌
- Full kernel component replacement: ❌ (research only, v3.0+)

### Reboots May Still Be Required
Zenvecha **reduces** the need for reboots, it does not eliminate them:

- Major kernel version upgrades: reboot required
- Some kernel config changes: reboot required
- Hardware driver changes: reboot required
- Memory layout changes: reboot required

### Safety Constraints
- Patches that modify kernel data structures accessed concurrently are high-risk
- Patches affecting interrupt handlers require careful design
- Patches to scheduler internals are extremely high-risk

### Performance
- Hook overhead exists (minimal for ftrace, but non-zero)
- Hot-path functions should not be patched unless necessary

## Explicit Non-Goals

- ❌ Universal "no reboot" Linux
- ❌ Binary-only closed-source patches
- ❌ Cross-architecture kernel patching
- ❌ Real-time kernel support (PREEMPT_RT)
- ❌ Android or embedded kernel support
- ❌ Mainline kernel replacement
- ❌ Bypassing kernel security mechanisms (SELinux, KASLR, etc.)

---

**© 2026 rezky_nightky (oxyzenQ)**

# 🌌 Zenvecha Roadmap

> **Vision:** Reboot only when you choose, not when a routine kernel fix forces you to.

---

## Phase 1 — System Readiness

### v0.1.0 "Doctor"
- [x] Repository structure
- [x] Rust workspace
- [x] CI/CD pipelines
- [x] Documentation framework
- [x] Build system
- [x] `zenvecha doctor` — system readiness check
- [x] `zenvecha doctor --fix` — actionable remediation
- [x] Gatekeeper: fmt, clippy, build, test, codespell, audit, deny

### v0.2.0 "Inspect"
- [x] `zenvecha inspect` — kernel capability discovery
- [x] Kernel version, architecture, compiler detection
- [x] CONFIG_* parsing from /boot/config-* and /proc/config.gz
- [x] Module environment (signing, headers, build dir)
- [x] Debug information (BTF, DWARF)
- [x] Symbol table accessibility (/proc/kallsyms)
- [x] Rust-for-Linux support detection
- [x] Capability summary with ✔/✘ indicators
- [x] Modular architecture: commands/, system/
- [x] No unsafe code
- [x] Zero new crate dependencies

---

## Phase 2 — Soft Live Patching

### v0.3 "Symbol Discovery"
- [ ] Runtime symbol lookup (kallsyms)
- [ ] Discover symbols: `schedule`, `do_exit`, `printk`
- [ ] Read-only — no modifications

### v0.4 "Safe Inspection"
- [ ] Read symbol metadata
- [ ] Read kernel version info
- [ ] Verify kernel ABI compatibility
- [ ] NO modification of any kernel state

---

## Phase 3 — First Runtime Change

### v0.5 "First Hook"
- [ ] Hook a dummy function
- [ ] Redirect to replacement
- [ ] Verify behavior change
- [ ] **Claim:** Runtime Function Redirection (Experimental)

### v0.6 "Safety Net"
- [ ] Manual rollback command
- [ ] Patch validation
- [ ] Checksum verification

### v0.7 "Patch Format"
- [ ] `.zenv` patch package format
- [ ] Patch metadata (target, version, checksum)
- [ ] Patch loading/parsing

---

## Phase 3 — Soft Live Patching

### v0.8 "Soft Live Patch"
- [ ] Patch helper functions
- [ ] Patch small bug fixes
- [ ] Patch simple logic
- [ ] **Claim:** Reduce unnecessary reboots (experimental)

### v0.5 "Safety Hardening"
- [ ] Dependency validation
- [ ] Kernel ABI check on patch load
- [ ] Pre-patch verification

### v0.6 "Auto Rollback"
- [ ] Automatic rollback on patch failure
- [ ] Health monitoring
- [ ] Recovery logging

---

## Phase 4 — Desktop Edition

### v1.0 "Desktop Support"
- [ ] Arch Linux official support
- [ ] CachyOS official support
- [ ] Kernel 6.x LTS
- [ ] x86_64 only

### v1.1 "Stress Testing"
- [ ] Firefox runtime test
- [ ] Docker runtime test
- [ ] QEMU runtime test
- [ ] Gaming workload test
- [ ] Rust compile workload
- [ ] Suspend/resume test
- [ ] Target: 7-day uptime

### v1.2 "Community Preview"
- [ ] Public experimental release
- [ ] Community testing
- [ ] Bug reports and feedback

---

## Phase 5 — Zenvecha 1.x

### v1.5 "Stable Desktop"
- [ ] **Claim:** Safe runtime patching for selected Linux desktop kernel fixes without requiring an immediate reboot
- [ ] Stable API
- [ ] Comprehensive tests
- [ ] Full documentation
- [ ] Packaging for Arch/CachyOS

---

## Phase 6 — Security

### v1.5 "Extended Support"
- [ ] More kernel functions
- [ ] Patch bundles
- [ ] Broader kernel version support

### v2.0 "Live Security"
- [ ] Kernel CVE patching
- [ ] Zero-downtime security fixes
- [ ] Deferred reboot support
- [ ] **Claim:** Live security patching for desktop Linux

---

## Phase 7 — Advanced

### v3.0 "Component Replacement"
- [ ] Live kernel component research
- [ ] Module hot-swap (experimental)
- [ ] Extended safety guarantees

---

## Phase 8 — Future

### Zenvecha X
- [ ] No-reboot kernel evolution research
- [ ] Broader kernel modification support
- [ ] Multi-year research effort

---

## Claim Progression

| Version | Claim |
|---------|-------|
| v0.1    | System readiness check (doctor) |
| v0.2    | Kernel capability discovery (inspect) |
| v0.5    | Runtime function redirection |
| v0.8    | Experimental live patching |
| v1.0    | Desktop support |
| v1.5    | Safe runtime desktop kernel patching |
| v2.0    | Live security patching |
| v3.0+   | Towards reboot-less desktop maintenance |

**Never claim "Linux without reboot" until proven across all scenarios.**

---

**© 2026 rezky_nightky (oxyzenQ)**

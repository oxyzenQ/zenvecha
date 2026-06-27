# 🌌 Zenvecha Roadmap

> **Vision:** Reboot only when you choose, not when a routine kernel fix forces you to.

---

## Phase 0 — Foundation

### v0.0.1 "Genesis"
- [x] Repository structure
- [x] Rust workspace
- [x] CI/CD pipelines
- [x] Documentation framework
- [x] Build system
- [ ] Architecture design docs
- [ ] No patching yet

### v0.0.2 "Kernel Hello"
- [ ] Rust kernel module (`insmod zenvecha.ko`)
- [ ] Clean module unload
- [ ] Kernel logging integration

### v0.0.3 "Symbol Discovery"
- [ ] Runtime symbol lookup (kallsyms)
- [ ] Discover symbols: `schedule`, `do_exit`, `printk`
- [ ] Read-only — no modifications

### v0.0.4 "Safe Inspection"
- [ ] Read symbol metadata
- [ ] Read kernel version info
- [ ] Verify kernel ABI compatibility
- [ ] NO modification of any kernel state

---

## Phase 1 — First Runtime Change

### v0.1.0 "First Hook"
- [ ] Hook a dummy function
- [ ] Redirect to replacement
- [ ] Verify behavior change
- [ ] **Claim:** Runtime Function Redirection (Experimental)

### v0.2 "Safety Net"
- [ ] Manual rollback command
- [ ] Patch validation
- [ ] Checksum verification

### v0.3 "Patch Format"
- [ ] `.zenv` patch package format
- [ ] Patch metadata (target, version, checksum)
- [ ] Patch loading/parsing

---

## Phase 2 — Soft Live Patching

### v0.4 "Soft Live Patch"
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

## Phase 3 — Desktop Edition

### v0.7 "Desktop Support"
- [ ] Arch Linux official support
- [ ] CachyOS official support
- [ ] Kernel 6.x LTS
- [ ] x86_64 only

### v0.8 "Stress Testing"
- [ ] Firefox runtime test
- [ ] Docker runtime test
- [ ] QEMU runtime test
- [ ] Gaming workload test
- [ ] Rust compile workload
- [ ] Suspend/resume test
- [ ] Target: 7-day uptime

### v0.9 "Community Preview"
- [ ] Public experimental release
- [ ] Community testing
- [ ] Bug reports and feedback

---

## Phase 4 — Zenvecha 1.0

### v1.0 "Stable Desktop"
- [ ] **Claim:** Safe runtime patching for selected Linux desktop kernel fixes without requiring an immediate reboot
- [ ] Stable API
- [ ] Comprehensive tests
- [ ] Full documentation
- [ ] Packaging for Arch/CachyOS

---

## Phase 5 — Security

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

## Phase 6 — Advanced

### v3.0 "Component Replacement"
- [ ] Live kernel component research
- [ ] Module hot-swap (experimental)
- [ ] Extended safety guarantees

---

## Phase 7 — Future

### Zenvecha X
- [ ] No-reboot kernel evolution research
- [ ] Broader kernel modification support
- [ ] Multi-year research effort

---

## Claim Progression

| Version | Claim |
|---------|-------|
| v0.1    | Runtime function redirection |
| v0.4    | Experimental live patching |
| v0.7    | Reduce unnecessary reboots |
| v1.0    | Safe runtime desktop kernel patching |
| v2.0    | Live security patching |
| v3.0+   | Towards reboot-less desktop maintenance |

**Never claim "Linux without reboot" until proven across all scenarios.**

---

**© 2026 rezky_nightky (oxyzenQ)**

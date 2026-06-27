# 🌌 Zenvecha Roadmap

> **Vision:** Reboot only when you choose, not when a routine kernel fix forces you to.

See [ROADMAP.md](../ROADMAP.md) for the full roadmap. This document contains technical implementation details.

---

## Current: Phase 0 — Foundation (v0.0.1 "Genesis")

**Status:** In Progress

### Deliverables
- [x] Rust workspace with Cargo.toml
- [x] CI/CD (ci.yml, release.yml)
- [x] Documentation framework
- [x] Build gatekeeper script
- [x] Install/uninstall scripts
- [x] Project governance docs (RULES, SECURITY, SUPPORT)
- [ ] Architecture decision records
- [ ] Working binary (`zenvecha -V`)

### Technical Decisions
- Language: Rust (stable, edition 2024)
- Kernel module approach: To be decided in ADR-0002
- Patching mechanism: To be decided in ADR-0002

---

## Next: v0.0.2 "Kernel Hello"

**Goal:** First kernel module — load, print message, unload cleanly.

### Technical Requirements
- Rust kernel module skeleton
- Makefile or build.rs for kernel build
- `insmod zenvecha.ko` → "Zenvecha loaded."
- `rmmod zenvecha` → clean unload

### Dependencies
- Linux kernel headers (6.x)
- Rust for Linux toolchain (or raw C FFI)

---

## Upcoming Milestones

| Version | Milestone | Technical Key |
|---------|-----------|---------------|
| v0.0.3  | Symbol Discovery | kallsyms lookup |
| v0.0.4  | Safe Inspection | Read-only kernel data |
| v0.1.0  | First Hook | ftrace/kprobe function redirection |
| v0.2    | Safety Net | Rollback, checksums |
| v0.4    | Soft Live Patch | Real function patching |
| v0.7    | Desktop Support | Arch/CachyOS packaging |
| v1.0    | Stable | Production-ready desktop patching |

---

**© 2026 rezky_nightky (oxyzenQ)**

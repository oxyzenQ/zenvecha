# ADR-0001: Initial Scope & Target Platform

**Status:** Accepted
**Date:** 2026-06-28
**Author:** rezky_nightky (oxyzenQ)

## Context

Zenvecha is an experimental runtime kernel patching research project. Scope must be deliberately narrow to ensure safety, auditability, and realistic progress.

## Decision

**Initially target: x86_64, Linux desktop, kernel 6.x, Arch Linux / CachyOS.**

## Rationale

### x86_64 only

- Dominant desktop architecture
- Livepatch/ftrace infrastructure is mature on x86_64
- Single architecture reduces validation surface
- ARM64, RISC-V can be added later once core engine is stable

### Linux Desktop (not server, not embedded)

- Desktop users benefit most from reduced reboots
- Desktop workloads are more tolerant of experimental patching
- Server environments demand production-grade stability (premature for Zenvecha)
- Embedded systems have different kernel configs and constraints

### Kernel 6.x

- Current LTS series with wide adoption
- `CONFIG_LIVEPATCH` is well-supported
- Avoids fragmentation across kernel versions during research phase

### Arch Linux / CachyOS

- Rolling release = latest kernels available
- CachyOS specifically targets performance-optimized desktop kernels
- Both ship with `CONFIG_LIVEPATCH=y`
- Single distribution focus reduces packaging and testing overhead

## Consequences

- Users on other distributions, architectures, or kernel versions are explicitly unsupported
- Scope may broaden in later versions once safety guarantees are proven
- Forces disciplined API boundaries that will ease future platform expansion

## Alternatives Considered

### Multi-architecture from start
**Rejected.** Increases validation surface exponentially. Delays v1.0.

### Server-first
**Rejected.** Server environments require production stability. Desktop is a safer proving ground.

### All Linux distributions
**Rejected.** Packaging, testing, and support burden is too high for a small research project.

---

**© 2026 rezky_nightky (oxyzenQ)**

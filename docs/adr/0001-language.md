# ADR-0001: Language Selection

**Status:** Accepted
**Date:** 2026-06-28
**Author:** rezky_nightky (oxyzenQ)

## Context

Zenvecha requires a systems programming language for:
1. Kernel module development (ring 0)
2. Userspace CLI and tooling
3. Patch validation and processing

## Decision

**Use Rust as the sole implementation language.**

## Alternatives Considered

### C
- **Pros:** Universal kernel language, mature ecosystem, all kernel APIs available
- **Cons:** Memory safety bugs (use-after-free, buffer overflow) are critical in kernel space; manual safety review burden

### Rust
- **Pros:** Memory safety at compile time (no use-after-free, no data races), zero-cost abstractions, modern tooling (cargo, clippy, rustfmt), growing kernel support
- **Cons:** Kernel Rust support is maturing, some kernel APIs require unsafe blocks, steeper learning curve for contributors

### Zig
- **Pros:** C interop, comptime, no hidden allocations
- **Cons:** Smaller ecosystem, less mature kernel support, fewer contributors

## Rationale

1. **Safety:** Kernel-level bugs are catastrophic. Rust eliminates entire classes of memory safety bugs at compile time.
2. **Modern Tooling:** `cargo fmt`, `cargo clippy`, `cargo audit` provide built-in quality gates.
3. **Growing Ecosystem:** Rust for Linux (R4L) is actively developed and supported.
4. **Consistency:** Single language for both kernel module and userspace reduces context switching.

## Consequences

- Kernel modules may require `unsafe` blocks for raw kernel API calls — these must be carefully reviewed and minimized
- Contributors must know Rust
- Build system uses Cargo (with Makefile for kernel module integration if needed)

---

**© 2026 rezky_nightky (oxyzenQ)**

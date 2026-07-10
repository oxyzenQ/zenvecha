# ADR-0003: Symbol Resolution Strategy

**Status:** Proposed
**Date:** 2026-06-28
**Author:** rezky_nightky (oxyzenQ)

## Context

Zenvecha must locate kernel symbols at runtime to apply patches. Symbols include:
- Function addresses (target for hooking)
- Data structures (for validation)

## Decision (Draft)

**Use `/proc/kallsyms` as the primary symbol source, with fallback to kernel module exported symbols.**

## Alternatives

### /proc/kallsyms
- **Pros:** Complete symbol table, works without kernel headers, always available
- **Cons:** Requires root, some symbols may be hidden (kptr_restrict), kernel addresses visible

### Kernel Module Symbols
- **Pros:** Clean API, type-safe via Rust bindings
- **Cons:** Limited to exported symbols only, requires build-time header matching

### System.map
- **Pros:** Static, offline lookup
- **Cons:** Distribution-specific location, may not match running kernel after updates

### BPF / BTF
- **Pros:** Type information preserved, modern approach
- **Cons:** Requires BTF-enabled kernel, more complex implementation

## Rationale

1. **Availability:** `/proc/kallsyms` is present on all Linux systems
2. **Accuracy:** Reflects the actual running kernel, not a build-time snapshot
3. **Simplicity:** Parse text output, no complex API dependencies
4. **Completeness:** Includes non-exported symbols needed for patching

## Implementation Notes

- Parse `/proc/kallsyms` in the kernel module (not userspace) to avoid address exposure
- Cache resolved symbols in module memory
- Validate symbol addresses against expected ranges
- Handle `kptr_restrict=1` gracefully

## Consequences

- Requires `CAP_SYSLOG` or root for kallsyms access
- Symbol addresses change per kernel build — must resolve at runtime
- Hidden symbols (kptr_restrict=2) are inaccessible

---

**© 2026 rezky_nightky (oxyzenQ)**

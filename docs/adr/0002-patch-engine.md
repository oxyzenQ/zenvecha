# ADR-0002: Patch Engine Approach

**Status:** Proposed
**Date:** 2026-06-28
**Author:** rezky_nightky (oxyzenQ)

## Context

Zenvecha needs a mechanism to redirect kernel function calls at runtime. Options include:

1. **ftrace-based hooking** — Linux kernel's built-in function tracing infrastructure
2. **kprobe-based hooking** — Dynamic instrumentation
3. **Direct memory patching** — Overwriting function prologues
4. **Livepatch framework** — kernel's `CONFIG_LIVEPATCH`

## Decision (Draft)

**Use the kernel's livepatch framework (`CONFIG_LIVEPATCH`) as the primary mechanism.**

## Alternatives

### ftrace-based
- **Pros:** Mature, stable API, handles concurrency (stop_machine)
- **Cons:** Limited to functions in ftrace-able sections, overhead on hot paths

### kprobe-based
- **Pros:** Flexible, can hook almost any address
- **Cons:** Higher overhead, not designed for permanent hooks

### Direct Memory Patching
- **Pros:** Minimal overhead
- **Cons:** Extremely fragile, kernel-version specific, high risk of crashes

### Livepatch Framework (CONFIG_LIVEPATCH)
- **Pros:** Purpose-built for live patching, atomic patching, consistency model, upstream support
- **Cons:** Requires kernel built with CONFIG_LIVEPATCH, limited to function-level replacement

## Rationale

1. **Upstream Support:** Livepatch is the official kernel mechanism for live patching
2. **Safety:** Consistency model ensures safe patching of running functions
3. **Stack Checking:** Livepatch checks call stacks before completing patching
4. **Integration:** Works with existing kernel infrastructure (ftrace underneath)

## Consequences

- Requires kernels built with `CONFIG_LIVEPATCH=y` (Arch/CachyOS have this)
- Module format must follow livepatch conventions
- Limited to architectures supported by livepatch (x86_64 is supported)

---

**© 2026 rezky_nightky (oxyzenQ)**

# Threat Model — Zenvecha

## Why Runtime Kernel Patching Is Dangerous

Modifying kernel behavior at runtime is one of the most inherently risky operations in systems programming:

1. **Ring 0 Execution** — Any bug in patch logic runs with full kernel privileges. There is no supervisor to catch a mistake.
2. **Concurrency** — Kernel functions may be executing on multiple CPUs simultaneously when a patch is applied. Race conditions can corrupt kernel state.
3. **No Isolation** — Unlike userspace processes, kernel memory is shared globally. A corrupted data structure affects the entire system.
4. **Silent Corruption** — Kernel bugs often manifest as mysterious crashes minutes or hours after the root cause, making debugging extremely difficult.
5. **ABI Fragility** — Kernel internal ABIs change between versions (and sometimes between configs of the same version). A patch validated on one kernel may crash another.

**This is why Zenvecha prioritizes safety over features, and correctness over performance.**

---

## Unsupported Scenarios

Zenvecha explicitly does NOT support:

- Patching interrupt handlers or NMI context
- Patching during suspend/resume transitions
- Patching scheduler internals while real-time tasks are running
- Concurrent patching from multiple Zenvecha instances
- Patching functions that modify kernel page tables
- Patching on kernels with `kptr_restrict=2`
- Production server environments

---

## Design Philosophy

```
Read-Only First → Validate → Apply → Verify → Monitor → Rollback
```

Every patch operation follows this sequence. No step is skipped. No optimization that bypasses safety.

## Trust Boundaries

```
┌──────────────────────────────────────┐
│           Untrusted Zone              │
│  (user input, .zenv files, network)   │
└──────────────┬───────────────────────┘
               │ Validation boundary
               ▼
┌──────────────────────────────────────┐
│           Trusted Zone                │
│  (parser, validator, checksum)        │
└──────────────┬───────────────────────┘
               │ Kernel boundary (syscall)
               ▼
┌──────────────────────────────────────┐
│           Ring 0                      │
│  (zenvecha.ko, hook engine)           │
└──────────────────────────────────────┘
```

**Rule:** No data crosses a trust boundary without validation.

## Supply Chain Philosophy

- **Minimal dependencies** — Every crate is a potential attack vector
- **No auto-updated dependencies** — Manual review for kernel-level project
- **Pinned versions** — `Cargo.lock` is committed and verified
- **`cargo audit` on every build** — Advisory warnings are non-blocking but reviewed
- **No build-time code generation from external sources** — All code is in-repo

## Assets Under Protection

| Asset | Criticality | Threat |
|-------|-------------|--------|
| Kernel memory integrity | Critical | Malicious patch, buggy hook |
| System stability | Critical | Race condition, ABI mismatch |
| Patch authenticity | High | Tampered `.zenv` file |
| Symbol table integrity | High | Symbol hijacking |
| Build pipeline | High | Compromised dependency |

## Attack Vectors

### T1 — Malicious Patch Injection
**Risk:** Critical. A crafted `.zenv` file could inject arbitrary kernel code.
**Mitigation:** Cryptographic signatures (planned), checksum verification, manual review.

### T2 — Supply Chain Compromise
**Risk:** High. A compromised Rust dependency builds malicious kernel code.
**Mitigation:** Minimal deps, cargo audit, pinned versions, manual review.

### T3 — Race Condition Exploitation
**Risk:** High. Concurrent kernel execution during patch apply/remove.
**Mitigation:** Use kernel livepatch consistency model, `stop_machine()` where needed.

### T4 — ABI Drift Attack
**Risk:** Medium. Patch validated against wrong kernel version/configuration.
**Mitigation:** Kernel ABI fingerprint, version locking, pre-patch verification.

### T5 — Rollback Interception
**Risk:** Medium. Attacker prevents rollback, leaving system in patched state.
**Mitigation:** Health monitoring, watchdog-based auto-rollback.

---

**© 2026 rezky_nightky (oxyzenQ)**

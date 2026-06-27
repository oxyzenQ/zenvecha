# Threat Model — Zenvecha

## Scope

This document analyzes security threats specific to Zenvecha's operation as a kernel-level live patching system.

## Assets

| Asset | Criticality | Description |
|-------|-------------|-------------|
| Patch files (`.zenv`) | High | Tampered patches can inject arbitrary kernel code |
| Kernel symbol table | High | Symbol hijacking risk |
| Kernel module (zenvecha.ko) | Critical | Runs in ring 0 |
| Patch metadata/checksums | High | Integrity verification foundation |

## Threat Actors

| Actor | Motivation | Capability |
|-------|-----------|------------|
| Malicious patch author | Inject kernel malware | Can craft `.zenv` files |
| Local unprivileged user | Escalate privileges | Limited by Linux DAC/MAC |
| Compromised dependency | Supply chain attack | Rust crate with backdoor |
| Network attacker | MITM on update check | Can spoof version info |

## Threats

### T1 — Malicious Patch Injection
- **Vector:** Attacker provides a crafted `.zenv` patch file
- **Impact:** Arbitrary code execution in kernel space (ring 0)
- **Mitigation:**
  - Cryptographic signature verification (planned v0.2)
  - Checksum validation before load
  - Manual review requirement for all patches
  - Source verification before application

### T2 — Symbol Hijacking
- **Vector:** Attacker replaces or redirects kernel symbols
- **Impact:** Patch applied to wrong target, unpredictable behavior
- **Mitigation:**
  - Symbol address verification against kallsyms
  - ABI compatibility check
  - Pre-patch snapshot for rollback

### T3 — Supply Chain Compromise
- **Vector:** Malicious code in a Rust dependency
- **Impact:** Compromised binary with kernel privileges
- **Mitigation:**
  - Minimal dependency policy
  - `cargo audit` on every build
  - Manual dependency review
  - Pinned dependency versions in `Cargo.lock`

### T4 — Rollback Failure
- **Vector:** Patch removal fails, leaving system in undefined state
- **Impact:** Kernel instability, possible panic
- **Mitigation:**
  - Atomic hook operations (where supported by kernel)
  - Pre-patch state preservation
  - Health monitoring with auto-rollback trigger
  - Fallback to safe state on any failure

### T5 — Race Condition in Hook
- **Vector:** Concurrent access to hooked function during patch apply/remove
- **Impact:** Kernel crash, data corruption
- **Mitigation:**
  - Use kernel's ftrace with stop_machine() where available
  - Proper synchronization primitives
  - Short critical sections

### T6 — Information Leak via Logs
- **Vector:** Sensitive kernel data exposed in Zenvecha logs
- **Impact:** Information disclosure to unprivileged users
- **Mitigation:**
  - Sanitize log output
  - Kernel address filtering (`%pK`)
  - Configurable log level

## Risk Matrix

| Threat | Likelihood | Impact | Risk |
|--------|-----------|--------|------|
| T1 — Malicious patch | Low | Critical | Medium |
| T2 — Symbol hijacking | Low | High | Medium |
| T3 — Supply chain | Medium | Critical | High |
| T4 — Rollback failure | Medium | High | High |
| T5 — Race condition | Low | Critical | Medium |
| T6 — Info leak | Medium | Low | Low |

## Security Boundaries

```
User Space  │  Kernel Space
            │
  CLI ──────┼──→ zenvecha.ko
            │        │
  .zenv ────┼──→ Validator → Hook Engine
            │        │
            │        ▼
            │   Target Function
```

All user space ↔ kernel space transitions are security boundaries. Every input crossing this boundary must be validated.

---

**© 2026 rezky_nightky (oxyzenQ)**

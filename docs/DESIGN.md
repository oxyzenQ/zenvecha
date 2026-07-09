# 🏗️ Zenvecha Architecture Design

## High-Level Overview

```
┌─────────────────────────────────────┐
│              zenvecha CLI            │
│         (userspace binary)           │
└──────────────┬──────────────────────┘
               │
               ▼
┌─────────────────────────────────────┐
│          Patch Validator             │
│    (checksum, ABI, dependency)       │
└──────────────┬──────────────────────┘
               │
               ▼
┌─────────────────────────────────────┐
│         Kernel Module (zenvecha.ko)  │
│    (symbol lookup, function hook)    │
└──────────────┬──────────────────────┘
               │
               ▼
┌─────────────────────────────────────┐
│         Target Kernel Function       │
└─────────────────────────────────────┘
```

## Component Architecture

### 1. CLI Layer (`src/cli.rs` → `userspace/`)

- Command parsing (Clap)
- `-V` / `--version` output
- `--check-update` release check
- Patch loading and management commands
- Rollback command

### 2. Core Library (`src/`)

- Shared types and traits
- Patch format parsing (`.zenv`)
- Checksum verification
- Version compatibility

### 3. Kernel Module (`kernel/`)

- C kernel module (universal distro compatibility — no `CONFIG_RUST=y` needed)
- Symbol discovery via kallsyms + `/proc` filesystem reads
- Atomic patch execution via `stop_machine()` + ftrace hooks
- 6-gate safety protocol (3 compile-time + 3 runtime gates)
- Structured `key=value` proc interface under `/proc/zenvecha/*`
- Clean module init/exit with preflight + unload guards

### 4. Userspace Tools (`userspace/`)

- Patch preparation
- Module management (insmod/rmmod wrapper)
- Logging and diagnostics
- Health monitoring

## Data Flow

```
Patch File (.zenv)
    │
    ▼
Parse & Validate
    │
    ▼
Kernel ABI Check
    │
    ▼
Load Kernel Module
    │
    ▼
Resolve Target Symbol
    │
    ▼
Apply Hook
    │
    ▼
Verify (checksum/behavior)
    │
    ▼
Monitor Health
```

## Design Principles

1. **Read-Only First** — Always inspect before modifying
2. **Fail-Safe** — Automatic rollback on any failure
3. **Minimal Surface** — Smallest possible kernel footprint
4. **Checksum Everything** — Every patch verified before and after
5. **Auditability** — Every operation logged

## Module Boundaries

| Module | Responsibility | LOC Target |
|--------|---------------|------------|
| `src/cli.rs` | CLI dispatch | <400 |
| `src/lib.rs` | Core types, traits | <300 |
| `userspace/` | Patch tools, validation | <600/file |
| `kernel/` | Kernel module, hooks | <600/file |

---

**© 2026 rezky_nightky (oxyzenQ)**

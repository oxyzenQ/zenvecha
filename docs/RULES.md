# 🚀 PROJECT MASTERCLASS RULES — Zenvecha

This document is the absolute source of truth. AI agents MUST strictly follow these rules without exception.

## Rule #0 — Core Philosophy

```
Correctness > Features.
Stability > Performance.
Auditability > Cleverness.
Small patches > Large rewrites.
Dependencies are a last resort.
```

---

## 🔄 1. WORKFLOW: TEST-DRIVEN INTERACTION

1. **Test First:** Send ONLY the test command and expected result.
2. **Review:** Wait for output.
3. **Fix (If Fail):** Targeted fix prompt.
4. **Proceed (If Pass):** Next step.

**RULE:** NEVER send a long coding prompt immediately after a test command.

---

## 🏗️ 2. ARCHITECTURE & CODE QUALITY

- **Per-file LOC Limit:** <600 LOC per file. Hard warning at 800 LOC.
- **Modular `main.rs`:** Bootstrap and wiring only. Target: <150 LOC.
- **Module Structure:**
  ```
  kernel/          # Kernel-space Rust modules
  userspace/       # Userspace tools and CLI
  src/             # Core library (cli, types, services)
  ```
- **CLI Definitions:** Clap-based, defined in `src/cli.rs`.
- **Version & Update Command:** Consistent `-V` / `--version` + `--check-update`.
- **Release Profile:** Efficiency and stability focused (`[profile.release]` in `Cargo.toml`).

---

## 🛠️ 3. LOCAL TOOLING

- **Gatekeeper:** `./scripts/build.sh --check-all` runs before every commit:
  - `cargo fmt -- --check`
  - `cargo clippy`
  - `cargo test`
  - `codespell`
  - `cargo audit` (warning only, non-blocking)
- **Version Bumper:** `./scripts/version-to.sh vX.Y.Z` — single source of truth.

---

## ⚙️ 4. CI/CD

- **CI Filtering:** Run only on `*.rs`, `*.toml`, `*.lock`, workflow, script changes. Ignore `*.md`, `*.txt`, `docs/`.
- **Node.js v24:** `FORCE_JAVASCRIPT_ACTIONS_TO_NODE24=true`.
- **No Dependabot.**
- **Weekly Dep Update:** Monday 07:00, author `github-actions[bot]`, direct commit to `main`.
- **Lint Workflows:** `actionlint` + `yamllint`.

---

## 🎨 5. BRANDING & DOCS

- **Author:** `rezky_nightky (oxyzenQ)` — exact casing.
- **Repository:** `github.com/oxyzenQ` (Capital Q).
- **Contact:** `with dot rezky at gmail dot com`.
- **File Headers:** All `*.rs` files must have:
  ```rust
  // Copyright (C) 2026 rezky_nightky
  // SPDX-License-Identifier: GPL-3.0-only
  ```
- **Logo:** 260px width.
- **Badges:** Ko-fi (`ko-fi/rezky`) only.
- **Project Name:** `zenvecha` (lowercase).

---

## 🗑️ 6. GIT HYGIENE

- **Lean `.gitignore`** — strictly relevant.
- **Ignore AI artifacts:** `worklog.md`, `codex/`, `agent/`.

---

## 📦 7. DEPENDENCY POLICY

- **Minimal deps** to reduce supply chain risk.
- **Manual review** for dependency updates (no auto-merge for kernel project).
- Remove unused/burden dependencies aggressively.

---

## 📁 8. REQUIRED DOCS

- `RULES.md` — this file
- `TRADEMARK.md`
- `SECURITY.md`
- `SUPPORT.md`
- `DESIGN.md`
- `docs/architecture.md`
- `docs/threat-model.md`
- `docs/limitations.md`
- `docs/adr/` — Architecture Decision Records

---

## 🔒 9. PATH SECURITY — WHITELIST POLICY

Zenvecha's state I/O must NEVER read or write dangerous system paths
(e.g. `/etc/shadow`, `~/.ssh/`, `~/.gnupg/`) unless explicitly required
by the feature. This is a **whitelist** policy, not a blacklist — only
specific, named paths are allowed; everything else is blocked by default.

### Current State (v0.5.x — Early Dev)

Zenvecha currently has **no user-controlled state I/O**. All filesystem
reads are from hardcoded kernel/system paths:

- `/proc/version`, `/proc/mounts`, `/proc/kallsyms`, `/proc/config.gz`
- `/proc/sys/kernel/*`, `/proc/modules`
- `/boot/config-*`, `/etc/os-release`
- `/proc/zenvecha/livepatch/*` (kernel module procfs interface)

No `$HOME`-derived paths, no `$XDG_*` environment variables, no config
files under user home. Therefore **no pathguard module is needed yet**.

### Future Requirement

When zenvecha adds user-controlled state I/O (config file, cache, PID
file, audit log, patch storage), a `pathguard` module MUST be implemented
following the same pattern used in zelynic/zylaxion:

1. **`resolve_state_dir()`** — validates `$HOME` / `$XDG_RUNTIME_DIR`
   before deriving state paths. Falls back to `/tmp` when the env var
   points to a dangerous location.

2. **`is_dangerous()`** — rejects:
   - System paths: `/etc`, `/usr`, `/var`, `/bin`, `/sbin`, `/lib`,
     `/lib64`, `/boot`, `/root`, `/proc`, `/sys`, `/dev`
   - User credential stores: `~/.ssh`, `~/.gnupg`, `~/.kwallet`,
     `~/.local/share/keyrings`
   - Path traversal: `/tmp/../etc` (defeated via lexical normalization)

3. **Default-deny** — only allowlisted directories are valid for state.
   The tool must run (fall back to `/tmp`) rather than refuse to start,
   but it must NEVER write state into protected directories.

4. **Wiring** — every `fs::write()`, `File::create()`, or path-deriving
   function that uses user-controlled env vars MUST go through pathguard
   before touching the filesystem.

### Never Gated (BY DESIGN)

- Kernel procfs writes to `/proc/zenvecha/*` — hardcoded kernel module
  interface, not user-controlled via environment variables
- Read-only kernel/system path reads (`/proc/*`, `/boot/*`, `/etc/os-release`)
  — these are system inspection, not state I/O

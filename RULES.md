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
- `ROADMAP.md`
- `DESIGN.md`
- `docs/architecture.md`
- `docs/threat-model.md`
- `docs/limitations.md`
- `docs/adr/` — Architecture Decision Records

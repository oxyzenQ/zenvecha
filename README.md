<p align="center">
  <img src="assets/zenvecha-logo-master.png" alt="zenvecha logo" width="260">
</p>

<h1 align="center">zenvecha</h1>

<p align="center">
  <strong>Safe runtime patching for selected Linux desktop kernel fixes without requiring an immediate reboot.</strong>
</p>

<p align="center">
  <a href="https://ko-fi.com/rezky">
    <img src="https://img.shields.io/badge/Ko--fi-support-7C3AED?style=flat-square&logo=kofi&logoColor=white&labelColor=111827" alt="Support on Ko-fi">
  </a>
</p>

---

## Vision

> **Reboot only when you choose, not when a routine kernel fix forces you to.**

zenvecha is an experimental Rust-based live kernel patching engine. It aims to reduce unnecessary reboots on Linux desktop systems by applying safe, verified runtime function patches.

**zenvecha does NOT claim to eliminate reboots.** It targets selected, well-understood kernel fixes — starting small and expanding as safety guarantees improve.

---

## Status

**Phase 0 — Foundation (v0.0.1 "Genesis")**

Repository structure, CI, docs, and build system. No patching yet.

See [ROADMAP.md](ROADMAP.md) for the full development plan.

---

## Quick Start

```bash
# Build
cargo build --release --locked

# Run gatekeeper checks (before commit)
./scripts/build.sh --check-all

# Install
./scripts/install.sh           # User install (~/.local/bin)
./scripts/install.sh --system  # System install (/usr/local/bin)

# Verify
zenvecha --version
```

---

## Supported Platforms

| Platform    | Status        |
|-------------|---------------|
| Arch Linux  | ✅ Supported  |
| CachyOS     | ✅ Supported  |
| Kernel 6.x  | ✅ Supported  |
| x86_64      | ✅ Supported  |

See [SUPPORT.md](SUPPORT.md) for details.

---

## Architecture

```
zenvecha CLI → Patch Validator → Kernel Module → Target Function
```

- **Kernel module** (`kernel/`) — Symbol discovery, function hooking via livepatch/ftrace
- **Userspace** (`userspace/`) — Patch loading, validation, rollback
- **CLI** (`src/`) — Command interface, version management

See [DESIGN.md](DESIGN.md) and [docs/architecture.md](docs/architecture.md) for details.

---

## Documentation

| Document | Description |
|----------|-------------|
| [RULES.md](RULES.md) | Project engineering rules |
| [ROADMAP.md](ROADMAP.md) | Development roadmap |
| [DESIGN.md](DESIGN.md) | Architecture design |
| [SECURITY.md](SECURITY.md) | Security policy |
| [SUPPORT.md](SUPPORT.md) | Platform support |
| [trademark.md](trademark.md) | Trademark & IP |
| [docs/architecture.md](docs/architecture.md) | Detailed architecture |
| [docs/threat-model.md](docs/threat-model.md) | Threat model |
| [docs/limitations.md](docs/limitations.md) | Known limitations |
| [docs/adr/](docs/adr/) | Architecture Decision Records |

---

## License

GPL-3.0-only. See [LICENSE](LICENSE).

---

**© 2026 rezky_nightky (oxyzenQ)**

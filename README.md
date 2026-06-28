<p align="center">
  <img src="assets/zenvecha-logo-master.png" alt="zenvecha logo" width="260">
</p>

<h1 align="center">zenvecha</h1>

<p align="center">
  <strong>Experimental runtime Linux kernel patching research.</strong>
</p>

<p align="center">
  <a href="https://ko-fi.com/rezky">
    <img src="https://img.shields.io/badge/Ko--fi-support-7C3AED?style=flat-square&logo=kofi&logoColor=white&labelColor=111827" alt="Support on Ko-fi">
  </a>
</p>

---

## Status

**Phase 2 — Kernel Capability Discovery (v0.2.0)**

Read-only kernel inspection. No patching, no modification.

```bash
zenvecha doctor   # Check system readiness
zenvecha inspect  # Kernel capability discovery
```

---

## Mission

zenvecha researches safe, verifiable methods for applying runtime patches to the Linux kernel on desktop systems. The long-term goal is to reduce unnecessary reboots — not eliminate them.

### Goals

- Reduce unnecessary desktop reboots
- Safety first — never compromise system stability
- Open source (GPL-3.0-only)
- Desktop-first (Arch Linux / CachyOS, x86_64, kernel 6.x)

### What zenvecha is NOT

- ❌ A universal "no reboot" solution for Linux
- ❌ A production server patching tool
- ❌ A replacement for proper kernel updates
- ❌ A bypass for kernel security mechanisms

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
| ARM64       | ❌ Not yet    |
| Windows     | ❌ Never      |
| macOS       | ❌ Never      |

See [SUPPORT.md](SUPPORT.md) for details.

---

## Documentation

| Document | Description |
|----------|-------------|
| [RULES.md](RULES.md) | Engineering rules & philosophy |
| [ROADMAP.md](ROADMAP.md) | Development milestones |
| [DESIGN.md](DESIGN.md) | Architecture overview |
| [SECURITY.md](SECURITY.md) | Security policy & reporting |
| [SUPPORT.md](SUPPORT.md) | Platform support matrix |
| [TRADEMARK.md](TRADEMARK.md) | Trademark & IP |
| [docs/architecture.md](docs/architecture.md) | Detailed architecture |
| [docs/threat-model.md](docs/threat-model.md) | Threat analysis |
| [docs/limitations.md](docs/limitations.md) | Known limitations |
| [docs/adr/](docs/adr/) | Architecture Decision Records |

---

## License

GPL-3.0-only. See [LICENSE](LICENSE).

---

**© 2026 rezky_nightky (oxyzenQ)**

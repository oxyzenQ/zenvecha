# Security Policy

## Supported Versions

Zenvecha is in active development. Only the latest release receives security updates.

| Version | Supported          |
| ------- | ------------------ |
| latest  | ✅                 |
| < latest| ❌                 |

## Reporting a Vulnerability

**Do not open a public issue.**

Email: with dot rezky at gmail dot com

Please include:
- Affected version
- Steps to reproduce
- Impact assessment
- Suggested fix (if any)

Response time: within 72 hours.

## Security Model

Zenvecha operates at the kernel level. Security is the highest priority.

### Principles

1. **Least Privilege** — Zenvecha modules operate with the minimum required capabilities.
2. **Read-Only by Default** — Inspection before modification.
3. **Automatic Rollback** — Failed patches revert immediately.
4. **Checksum Verification** — Every patch is validated before application.
5. **Audit Trail** — All operations are logged.

### Threat Model

See `docs/threat-model.md` for the full threat model.

## Disclosure Policy

- Vulnerabilities will be disclosed 90 days after the fix is released.
- Critical vulnerabilities may have expedited disclosure.

---

**© 2026 rezky_nightky (oxyzenQ)**

# Zenvecha Architecture

See [DESIGN.md](DESIGN.md) for the high-level architecture overview.

## Detailed Architecture

### Patch Lifecycle

```
1. Load        → Parse .zenv file
2. Validate    → Checksum, ABI, dependencies
3. Prepare     → Resolve symbols, allocate resources
4. Apply       → Install hook into kernel
5. Verify      → Confirm hook is active and correct
6. Monitor     → Health checks during runtime
7. Rollback    → Remove hook, restore original
```

### Safety Layers

```
Layer 1: Static validation  (build time)
Layer 2: ABI checking       (load time)
Layer 3: Runtime checks     (apply time)
Layer 4: Health monitoring  (continuous)
Layer 5: Auto-rollback      (failure)
```

### Kernel Interaction Model

Zenvecha uses the Linux kernel's existing live-patching infrastructure (ftrace/kprobes) rather than implementing custom hooking mechanisms. This ensures:

- Kernel ABI compatibility
- Safe concurrency with other kernel subsystems
- Proper handling of preemption and interrupts

### Supported Kernel Versions

- Linux 6.x LTS (primary target)
- x86_64 architecture

---

**© 2026 rezky_nightky (oxyzenQ)**

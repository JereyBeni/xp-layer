# xp-layer

**Windows XP API translation / compatibility layer written in Rust**

`xp-layer` is a compatibility layer that aims to run Windows XP-era applications on modern operating systems by translating Win32 and related APIs into equivalent host functionality. It is **not** a full system emulator (it does not emulate x86 hardware or boot a real Windows XP installation). Instead, it focuses on loading PE binaries and providing a sufficient subset of the XP-era API surface.

This approach is conceptually similar to projects such as Wine, but deliberately scoped toward Windows XP behaviour and constraints.

---

## Goals

- Load and execute 32-bit Windows XP-era PE executables and DLLs.
- Reimplement (or accurately stub) a useful subset of the Win32 API and supporting components that XP applications expect.
- Preserve XP-era behavioural quirks where they matter for compatibility.
- Provide a constrained view of system resources (especially memory) so that older applications behave as they would on period-appropriate hardware.
- Keep the implementation in idiomatic, safe Rust where practical, with clear module boundaries so the project can grow incrementally.

---

## Memory Reporting Policy

Many Windows XP applications were written with the assumption of relatively small amounts of physical RAM. To improve compatibility, `xp-layer` reports a deliberately reduced memory size to guest applications according to the following mapping from host RAM:

| Host RAM | Reported to XP environment |
|----------|----------------------------|
| 16 GB    | 2 GB                       |
| 8 GB     | 1 GB                       |
| 4 GB     | 768 MB                     |
| 2 GB     | 512 MB                     |
| 1 GB     | 128 MB                     |

This value is intended to be used by memory-status APIs (`GlobalMemoryStatus`, `GlobalMemoryStatusEx`, related `NtQuerySystemInformation` queries, etc.) and by the layer’s internal accounting so that applications cannot freely allocate far beyond what would have been realistic on XP-era hardware.

---

## Current State

**Status: Project initialised — very early stage**

- Repository created and basic README established.
- No executable code, PE loader, or API stubs have been implemented yet.
- Project structure, build configuration, and core modules are still to be added.
- The memory-mapping policy has been defined and will be encoded in configuration once the first memory-related APIs are written.

This is currently a green-field project. Expect rapid early commits that establish the skeleton (Cargo workspace / crate layout, PE parsing foundations, configuration, and the first handful of kernel32 stubs).

---

## Planned High-Level Structure (subject to change)

```
src/
  main.rs / lib.rs          # Entry point / library root
  config.rs                 # Host detection + XP-visible limits (including memory policy)
  pe/                       # PE/COFF loading and relocation
  api/
    kernel32/               # Process, thread, memory, file, etc.
    user32/                 # Windowing and messaging (later)
    gdi32/                  # Graphics (later)
    ntdll/                  # Lower-level NT APIs as needed
  memory/                   # Virtual memory tracking and enforcement of reported limits
  ...
```

---

## Building (once code exists)

```bash
cargo build
cargo run -- [path-to-xp-era-exe]
```

Exact command-line interface and feature flags will be documented as they are introduced.

---

## Contributing / Development Notes

The project is in its earliest phase. Contributions of structure, design discussion, and carefully scoped initial implementations (especially PE loading and the first memory-status APIs) are welcome once the skeleton is in place.

Please keep the focus on XP-era behaviour and on keeping the reported environment intentionally constrained where it aids compatibility.

---

## Licence

Licence still to be decided. A permissive open-source licence (MIT or Apache-2.0) is the current preference.

---

*Last updated to reflect project state at initial README expansion.*

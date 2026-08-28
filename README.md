# xp-layer

**Windows XP API translation / compatibility layer written in Rust**

*(Repo made by Jeremiah, Assistant is Grok)*

`xp-layer` is a compatibility layer that aims to run Windows XP-era applications on modern operating systems by translating Win32 and related APIs into equivalent host functionality. It is **not** a full system emulator (it does not emulate x86 hardware or boot a real Windows XP installation). Instead, it focuses on loading PE binaries and providing a sufficient subset of the XP-era API surface.

This approach is conceptually similar to projects such as Wine, but deliberately scoped toward Windows XP behaviour and constraints. The user interface follows a workflow inspired by touchHLE.

**Target platforms:** Windows and Linux. On Linux, future use of KVM (where available) is expected to improve performance of the translation layer compared with a pure software path on Windows.

---

## Goals (current focus)

- Load and execute 32-bit Windows XP-era PE executables and DLLs.
- Reimplement (or accurately stub) a useful subset of the Win32 API and supporting components that XP applications expect.
- Preserve XP-era behavioural quirks where they matter for compatibility.
- Provide a constrained view of system resources (especially memory) so that older applications behave as they would on period-appropriate hardware.
- Keep the implementation in idiomatic, safe Rust where practical, with clear module boundaries so the project can grow incrementally.

---

## Future Roadmap

After a solid Windows XP baseline is in place, the project intends to expand support in two directions:

### Windows 2000 compatibility
- Support for Windows 2000-era applications.
- Coverage of the NT 5.0 API surface and behavioural differences relative to Windows XP.
- Adjusted resource limits and compatibility quirks appropriate to the Windows 2000 environment.

### Windows Vista compatibility
- Support for Windows Vista-era applications.
- Extension of the translation layer to cover Vista-specific APIs, security model changes, and related system components.
- Optional profiles that present a Vista-like environment alongside the classic XP profile.

These are longer-term goals. The immediate priority remains a usable Windows XP translation layer.

---

## Usage (Graphical App Picker)

The workflow is intentionally similar to touchHLE:

1. Place Windows XP-era `.exe` files (and any required companion DLLs) into the `apps/` directory.
2. Run the program:
   ```bash
   cargo run
   ```
3. A window appears listing every `.exe` found in `apps/`.
4. Select an application and click **Run selected application**.
5. The translation layer will start for that binary (currently a placeholder message is shown; PE loading and API translation are still under development).

You can click **Refresh** at any time to rescan the directory.

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

This value is intended to be used by memory-status APIs (`GlobalMemoryStatus`, `GlobalMemoryStatusEx`, related `NtQuerySystemInformation` queries, etc.) and by the layer's internal accounting so that applications cannot freely allocate far beyond what would have been realistic on XP-era hardware.

The mapping is already implemented in `src/config.rs` as `reported_xp_memory_mb` and is shown in the UI status area.

---

## Current State

**Status: Early skeleton + graphical app picker**

- Graphical user interface (egui/eframe) that scans the `apps/` directory and lists `.exe` files.
- TouchHLE-style workflow: drop applications into `apps/`, select, and run.
- Memory-reporting policy implemented and visible in the UI.
- `src/config.rs` contains the mapping function with unit tests.
- Continuous integration builds on both Ubuntu and Windows (and uploads release binaries as artifacts).
- No PE loader, no real API stubs, and no host-memory detection yet.
- Selecting "Run" currently shows a status message only.

Next planned work: real host memory detection, a minimal PE loader, and the first kernel32 memory-status stubs that feed into the running layer.

---

## Planned High-Level Structure (subject to change)

```
src/
  main.rs                   # GUI entry point + app picker
  config.rs                 # Host detection + XP-visible limits (including memory policy)
  pe/                       # PE/COFF loading and relocation
  api/
    kernel32/               # Process, thread, memory, file, etc.
    user32/                 # Windowing and messaging (later)
    gdi32/                  # Graphics (later)
    ntdll/                  # Lower-level NT APIs as needed
  memory/                   # Virtual memory tracking and enforcement of reported limits
  ...
apps/                       # Place XP-era .exe files here
```

---

## Building

```bash
git clone https://github.com/JereyBeni/xp-layer.git
cd xp-layer
cargo run
```

Unit tests (currently covering the memory mapping):

```bash
cargo test
```

CI status and downloadable binaries: https://github.com/JereyBeni/xp-layer/actions

---

## Contributing / Development Notes

The project is in its earliest phase. Contributions of structure, design discussion, and carefully scoped initial implementations (especially PE loading and the first memory-status APIs) are welcome.

Please keep the focus on XP-era behaviour for the present, while keeping the architecture open enough for the planned Windows 2000 and Windows Vista expansions.

---

## Licence

Licence still to be decided. A permissive open-source licence (MIT or Apache-2.0) is the current preference.

---

*Last updated with corrected credit (Repo made by Jeremiah, Assistant is Grok).*

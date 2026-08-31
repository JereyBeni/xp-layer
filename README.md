# xp-layer

**Windows XP API translation / compatibility layer written in Rust**

*(Repo made by Jeremiah, Assistant is Grok)*

*(Made for Linux and Windows — you Apple fuckers and glazers)*

`xp-layer` is a compatibility layer that aims to run Windows XP-era applications on modern operating systems by translating Win32 and related APIs into equivalent host functionality. It is **not** a full system emulator (it does not emulate x86 hardware or boot a real Windows XP installation). Instead, it focuses on loading PE binaries and providing a sufficient subset of the XP-era API surface.

This approach is conceptually similar to projects such as Wine, but deliberately scoped toward Windows XP behaviour and constraints. The user interface follows a workflow inspired by touchHLE.

**Target platforms:** Linux and Windows only. On Linux, future use of KVM (where available) is expected to improve performance of the translation layer compared with a pure software path on Windows. macOS is not a goal.

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
5. The PE is loaded, imports are listed, and memory-status values the guest would see are shown.

You can click **Refresh** at any time to rescan the directory. Use **Options** for profile, memory override, apps path, fullscreen, and debug logging.

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

This value is used by the implemented `GlobalMemoryStatus` / `GlobalMemoryStatusEx` stubs and will be used by related queries later.

Host memory is detected at startup via the `sysinfo` crate; the mapping is applied automatically (or overridden in Options).

---

## Current State

**Status: PE loader + import parsing + early API surface**

- Graphical app picker (egui/eframe) with Options panel.
- Real host memory detection and constrained XP memory reporting.
- kernel32 stubs: `GlobalMemoryStatus`, `GlobalMemoryStatusEx`.
- Minimal 32-bit PE loader (DOS + PE headers, section mapping).
- Import table parsing (DLL names and function names/ordinals).
- CI builds on Ubuntu and Windows; release binaries uploaded as artifacts.
- No CPU execution, import resolution into stubs, or USER32/GDI32 yet.

Next planned work: import resolver wired to stubs, then more kernel32 / user32 surface.

---

## Planned High-Level Structure (subject to change)

```
src/
  main.rs                   # GUI entry point + app picker
  config.rs                 # Host detection + XP-visible limits
  pe/                       # PE/COFF loading, imports, (later) relocations
  api/
    kernel32/               # Process, thread, memory, file, etc.
    user32/                 # Windowing and messaging (later)
    gdi32/                  # Graphics (later)
    ntdll/                  # Lower-level NT APIs as needed
  memory/                   # Virtual memory tracking (later)
apps/                       # Place XP-era .exe files here
assets/                     # logo.png used as window icon
```

---

## Building

```bash
git clone https://github.com/JereyBeni/xp-layer.git
cd xp-layer
cargo run
```

Unit tests:

```bash
cargo test
```

CI status and downloadable binaries: https://github.com/JereyBeni/xp-layer/actions

---

## Contributing / Development Notes

The project is in its earliest phase. Contributions of structure, design discussion, and carefully scoped initial implementations are welcome.

Please keep the focus on XP-era behaviour for the present, while keeping the architecture open enough for the planned Windows 2000 and Windows Vista expansions. Target platforms remain Linux and Windows.

---

## Licence

Licence still to be decided. A permissive open-source licence (MIT or Apache-2.0) is the current preference.

---

*Last updated with platform note and current PE/import status.*

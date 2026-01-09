# Console Diagnostics

This note captures the structure and intended use of the kernel console, along with guidance for the diagnostics macros that write through it.

## Scope

The console provides the kernel's primary human-visible output path once firmware hands off control. It renders text into the linear framebuffer, timestamps every line, and retains a bounded history so recent output survives screen scrollback. The implementation lives in [kernel/src/console/mod.rs](kernel/src/console/mod.rs).

## Storage and Flow

- `ConsoleStorage` reserves a ring of 128 `LineSlot` records so the console can keep recent lines even after they leave the visible display. Each slot records the rendered bytes and their capture timestamp. See [kernel/src/console/mod.rs#L10-L139](kernel/src/console/mod.rs#L10-L139).
- Memory for `ConsoleStorage` comes from early physical reservations during memory bring-up. The loader hands the kernel a framebuffer; the kernel allocates backing storage before runtime allocators exist, then hands it into `console::init` during foundational setup.
- `console::write` is the single sink for formatted text. Macros emit `core::format_args!` payloads; the console sanitizes bytes, injects timestamp prefixes, appends to the on-screen buffer, and writes into history.

## Macro Surface

All macros live in [kernel/src/console/mod.rs#L340-L413](kernel/src/console/mod.rs#L340-L413) and funnel into `console::write`.

- `print!` / `println!`: Unconditional output. Use for messages that must always appear (panic banners, fatal errors).
- `diag!` / `diagln!`: Guarded by `options::diagnostics_enabled()`, which resolves to `debug_enabled() && !quiet_enabled()` in [kernel/src/options.rs#L1-L32](kernel/src/options.rs#L1-L32). Prefer these for routine bring-up tracepoints and status messages that are valuable during normal debugging but should respect the user's quiet flag.
- `debug!` / `debugln!`: Guarded solely by `options::debug_enabled()`. These are for high-volume or niche traces you only want when explicitly opting into full debug verbosity (for example, per-iteration scheduler breadcrumbs).
- `debug_structured!`: When `debug_enabled()` is true, prints a headline followed by key/value pairs on indented lines. This is intended for structured dumps (allocator plans, capability inventories, etc.) where pairing labels with values improves scanability.

All macros drop their writes if the relevant option returns `false`, so callers do not need to branch manually.

## Usage Guidance

1. **Baseline telemetry**: prefer `diag!`/`diagln!`. They honor `quiet` while still surfacing helpful state during development builds.
2. **Verbose debugging**: use `debug!`, `debugln!`, or `debug_structured!`. Reserve these for data that would overwhelm normal diagnostics.
3. **Critical failures**: fall back to `print!`/`println!` (or fatal paths that call them) so the message is never filtered out.

Avoid mixing raw `console::write` calls with macros; the macros centralize option gating and formatting rules. When adding new diagnostics, decide which audience the message serves (always-on, default debugging, deep tracing) and choose the macro that matches that intent.

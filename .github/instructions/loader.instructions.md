---
applyTo: 'loader/**'
---

# Oxide Loader — Purpose & Rules

## Purpose

The loader exists to bootstrap the system using UEFI and then get out of the way.

It is responsible for:
- acquiring the minimal platform information the kernel needs to start (e.g., memory map, framebuffer)
- preparing a compact handoff structure for the kernel
- terminating UEFI Boot Services cleanly
- transferring control to the kernel entry point

The loader is disposable scaffolding. Treat it like a launch vehicle: necessary at liftoff, irrelevant in orbit.

## Non-Goals

The loader must not become a “mini kernel.”

Specifically, the loader must not implement:
- scheduling, tasks, threads, IPC
- virtual memory management beyond what is required for transition
- device drivers intended to persist beyond boot
- filesystem stacks as a permanent subsystem (only enough to load the kernel/config if needed)
- rich UI systems (fonts, image decoding, animations, layout engines)
- long-lived abstractions that the kernel “depends on”

If code feels too important to delete, it probably does not belong in the loader.

## Firmware Boundary Rules

- UEFI is used only while Boot Services are active.
- After `ExitBootServices`, UEFI types, globals, protocols, and helpers are forbidden.
- Any data required after firmware exit must be copied into memory owned by the OS and described by the handoff struct.

The loader must make the firmware boundary explicit and auditable.

## Output & Debugging

- Prefer framebuffer-based output for anything meant to survive firmware exit.
- UEFI text console output is acceptable only as an early bring-up aid and should not become a dependency.
- Avoid building a “nice” UI in the loader. Visual output should be minimal and primarily diagnostic.

## Ownership & Interfaces

- The loader may know about the kernel.
- The kernel must never know about the loader.
- The only contract is the handoff ABI (a well-defined struct layout and entry signature).
- The handoff struct should avoid UEFI-specific enums/structs; use project-owned types.

## Design Heuristics

- Minimize loader code size and complexity.
- Prefer clarity over cleverness.
- Keep all work in the loader directly tied to: discovery → handoff → exit → jump.
- If a change increases long-term maintenance burden, it likely belongs in the kernel instead.
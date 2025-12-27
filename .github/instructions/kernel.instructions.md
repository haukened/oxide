---
applyTo: 'kernel/**'
---

# Oxide Kernel — Purpose & Rules

## Purpose

The kernel is the operating system.

It is responsible for owning the machine after firmware exit, including:
- permanent graphics output (framebuffer or later GPU stack)
- physical and virtual memory management
- interrupts/exceptions and CPU feature management
- scheduling, isolation, IPC, drivers, and higher-level services

The kernel is not allowed to rely on UEFI. The kernel runs after UEFI is gone.

## Firmware Independence

- The kernel must not depend on the `uefi` crate or any firmware-provided types or protocols.
- The kernel must not call or reference UEFI Boot Services or Runtime Services.
- All firmware-derived data must arrive through an explicit handoff struct.

Assume the firmware is dead; the kernel owns the hardware.

## Entry Contract

- The kernel starts at a freestanding entry point (`_start` or equivalent).
- The kernel receives a pointer/reference to a handoff struct provided by the loader.
- The kernel must validate critical handoff fields before using them (bounds, formats, sizes).

The handoff is the kernel’s first ABI. Treat changes to it as breaking changes.

## Memory & Safety Expectations

- `no_std` is expected.
- Unsafe is expected in low-level code, but must be localized and justified.
- Prefer building small safe wrappers around unsafe primitives.
- Avoid implicit global state unless it is explicitly part of the kernel’s design (and documented).

## Output & Debugging

- Early kernel debugging should work without firmware help.
- Framebuffer output is a primary early debugging channel.
- Serial/logging can be added, but should not be the only debugging surface.

A kernel that cannot communicate failures is not debuggable.

## Architectural Principles

- Prefer explicit ownership and invariants over “magic” convenience.
- Keep subsystems decoupled: memory, interrupts, scheduler, drivers, etc.
- Resist premature abstraction. Build minimal primitives, then compose.
- Avoid bringing loader constraints into the kernel. The kernel defines the platform going forward.

## Non-Goals

The kernel is not required to:
- maximize compatibility with legacy platforms
- support BIOS, real mode, or VGA text mode
- preserve firmware assumptions

Oxide targets modern x86_64 UEFI systems and leans into that constraint.
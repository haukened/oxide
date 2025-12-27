# Oxide OS

Oxide is an experimental operating system written in Rust for modern x86_64 machines that boot exclusively via UEFI. The project explores how far a small, explicit, and debuggable kernel can go when it treats firmware as disposable scaffolding and owns the machine as early as possible.

## Project Status

- Target hardware: x86_64 with UEFI 2.x firmware, GOP framebuffer, and working boot-services memory map.
- Current focus: Designing the loader→kernel handoff ABI (`BootInfo`) and standing up the early kernel epochs.
- Stability: Pre-epoch-1; architectural contracts are still settling and breaking changes are expected.

## Core Principles

Oxide’s design is governed by the ADRs under `docs/adr/` and the vision documents in `docs/vision/`. Key non-negotiables:

- Firmware is a bootstrap helper only; ownership transfers to the kernel immediately after `ExitBootServices`.
- Responsibilities are split cleanly: a disposable UEFI loader and a freestanding kernel (ADR 0004).
- All authority is explicit. Capabilities guard every boundary; there is no ambient privilege.
- Epochs make time explicit (ADR 0001). Code must declare which guarantees it depends on.
- Drivers and services default to user space; kernel code remains minimal and policy-free.

## Architecture Snapshot

The `docs/architecture/` tree captures the current architecture:

- `00-overview.md` — structural decomposition (minimal kernel, user-space services, capability system).
- `01-boundaries.md` — trust and privilege boundaries that must never be crossed implicitly.
- `02-epochs.md` — ordered boot/runtime epochs with their guarantees and constraints.
- `03-boot-flow.md` — concrete boot sequencing mapped to epoch transitions.

The loader→kernel ABI policy lives in `docs/adr/0005-loader-kernel-boundary-abi.md`; the concrete `BootInfo` layout is being defined in a forthcoming spec.

## Repository Layout

- `loader/` — UEFI application responsible for discovery, `BootInfo` construction, and handing off to the kernel.
- `kernel/` — Firmware-independent kernel crate that takes ownership after `ExitBootServices`.
- `docs/` — ADRs, architectural references, vision, and working notes.
- `scripts/` — Utility scripts (e.g., flashing helpers).

## Development Workflow

1. Read all relevant ADRs before proposing design or code changes; ADRs override other docs when conflicts arise.
2. Keep firmware-specific logic inside `loader/`. Kernel code must not depend on `uefi` crates or firmware-centric types.
3. Treat the loader→kernel boundary as an ABI contract. Any change to `BootInfo` requires a version bump and synchronized updates.
4. Maintain epoch discipline. Do not assume guarantees from later epochs; validate handoff data on entry.
5. Document new architectural decisions with ADRs before merging significant changes.

## Getting Started

Prerequisites:

- Rust nightly toolchain (for `#![no_std]` bare-metal support).
- A UEFI-aware emulator such as QEMU with OVMF firmware for testing.

Typical workflow:

```bash
# Build loader and kernel artifacts
cargo build --package loader
cargo build --package kernel

# (Optional) run via QEMU once build and boot images are scripted
scripts/flash.sh # see script for arguments and environment expectations
```

The boot pipeline is under active development; expect manual steps while the `BootInfo` ABI solidifies.

## Contributing

- Follow the architectural boundaries and capability model laid out in the docs.
- Keep commits small, explicit, and traceable to ADR-backed decisions.
- Prefer Rust (and minimal `unsafe`) for kernel-space code; justify any divergence in reviews.
- File issues or drafts if proposed changes affect architecture or epoch guarantees.

## Further Reading

- `docs/vision/` — purpose, principles, and non-goals that frame every decision.
- `docs/notes/kernel-design.md` — research notes informing the hybrid kernel and capability focus.
- `docs/adr/` — authoritative history of architectural decisions.

Oxide is pre-alpha and not intended for production use. The value today is in refining the architecture, validating assumptions on real hardware, and learning how explicit contracts shape a modern kernel.

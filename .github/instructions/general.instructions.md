---
applyTo: '**'
---

# Oxide OS — Project Purpose & Design Intent

## Must follow ADRs

- Regardless of the high-level intent described here, all specific architectural decisions are governed by the project's Architecture Decision Records (ADRs) located in `docs/adr/**`.
- In case of any conflict between this document and the ADRs, the ADRs take precedence.
- You must read the ADRs before making any design or implementation decisions.

## What Oxide Is

Oxide is an experimental operating system written in Rust, targeting modern x86_64 systems using UEFI firmware exclusively.

The project exists to explore OS design under the assumption that:
	•	legacy BIOS is obsolete
	•	firmware is a temporary bootstrap mechanism
	•	the operating system must explicitly and permanently own the machine

Oxide is not a hobby bootloader, a Linux clone, or a firmware extension. It is an attempt to build a clean, minimal OS architecture starting from modern constraints rather than historical baggage.

⸻

## Core Philosophy

Oxide is guided by a small number of non-negotiable principles:

1. Firmware Is Not the OS

UEFI is used only to:
	•	enter 64-bit long mode
	•	access hardware in a standardized way early
	•	discover memory, graphics, and platform configuration

UEFI is exited as early as possible.
After that point, no firmware services, types, globals, or assumptions are allowed to remain.

Firmware is scaffolding, not a foundation.

⸻

2. Explicit Boundaries Over Convenience

All major transitions are treated as explicit contracts, not implicit magic:
	•	loader → kernel
	•	firmware → owned hardware
	•	physical memory → virtual memory
	•	early output → permanent output

Data passed across boundaries is:
	•	minimal
	•	well-defined
	•	owned by the receiver

This makes architectural mistakes visible early and prevents dependency leakage.

⸻

3. Separation of Responsibility

Oxide is intentionally structured into distinct roles:
	•	Loader: firmware-facing, temporary, disposable
	•	Kernel: firmware-independent, long-lived, authoritative

The loader exists to die.
The kernel exists to run indefinitely.

Any code that feels “too important to delete” does not belong in the loader.

⸻

4. Modern Assumptions, Fewer Apologies

Oxide assumes:
	•	x86_64
	•	UEFI
	•	linear framebuffers
	•	APIC-based interrupt models
	•	modern CPUs with standard features

Oxide does not aim to support:
	•	legacy BIOS
	•	real mode
	•	VGA text mode
	•	32-bit systems

Reducing compatibility scope is considered a feature, not a limitation.

⸻

5. Visual Output Is a Debugging Primitive

Early graphics output is treated as a first-class debugging channel, not a luxury.

The system is designed so that:
	•	graphical output works before and after firmware exit
	•	no firmware-dependent text console is required
	•	the kernel can always render something to the screen

This reduces reliance on serial-only debugging and accelerates development on real hardware.

⸻

## What Oxide Is Not

Oxide is not:
	•	a production OS
	•	a drop-in Linux replacement
	•	a UEFI application that never exits firmware
	•	an experiment in maximum abstraction
	•	a collection of clever tricks

Clarity, control, and correctness are valued more than novelty.

⸻

## Long-Term Intent

Oxide is intended to serve as:
	•	a testbed for kernel architecture decisions
	•	a platform for exploring memory management, scheduling, and isolation
	•	a vehicle for understanding modern boot flows deeply
	•	a codebase where architectural decisions are deliberate and documented

Whether Oxide grows into a usable system is secondary to whether its design remains coherent.

⸻

## Decision-Making Heuristics

When making design decisions in this project:
	•	Prefer explicit control over implicit convenience
	•	Prefer deleting code later over entangling it now
	•	Prefer correctness and observability over premature optimization
	•	Prefer architectural cleanliness over short-term speed

If a choice introduces long-lived dependency on firmware, tooling, or environment quirks, it is almost certainly the wrong choice.

⸻

## Audience

This project is written for:
	•	developers comfortable with systems programming
	•	readers who value architectural clarity
	•	tools (including AI assistants) that need high-level intent to guide local decisions

It is not written for beginners or tutorial readers.
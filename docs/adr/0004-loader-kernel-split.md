# ADR 0004: Loader / Kernel Responsibility Split

## Status
Accepted

## Context

Oxide uses UEFI as a bootstrap environment and intends to exit firmware services
early. This creates two distinct phases with fundamentally different
constraints:

1. **Firmware phase**: UEFI Boot Services are available; firmware protocols may
   be used for discovery and loading.
2. **OS phase**: Firmware services are unavailable or treated as unusable; the
   operating system must own and manage hardware resources directly.

Mixing responsibilities across these phases leads to:
- Firmware concepts leaking into long-lived kernel design
- Hidden dependencies on UEFI types and lifetimes
- Increased difficulty reasoning about ownership after `ExitBootServices`
- Hard-to-delete early boot code that becomes permanent technical debt

The project requires a clear boundary that makes the transition explicit,
auditable, and hard to violate accidentally.

## Decision

Oxide will enforce a strict split between a **UEFI loader** and a **freestanding
kernel**.

### Loader responsibilities

The loader is firmware-facing and disposable. It is responsible for:
- Initial execution as a UEFI application
- Platform discovery needed for kernel bring-up (e.g., framebuffer, memory map)
- Loading the kernel image (and optional configuration)
- Constructing a compact handoff structure (`BootInfo`)
- Calling `ExitBootServices` correctly
- Transferring control to the kernel entry point

The loader may use UEFI protocols and types internally, but must not expose them
across the handoff boundary.

### Kernel responsibilities

The kernel is firmware-independent and long-lived. It is responsible for:
- Owning the machine after firmware exit
- Using only the data provided in the handoff structure
- Establishing memory management, interrupts, scheduling, and drivers
- Providing persistent debugging/output mechanisms independent of firmware

The kernel must not depend on the `uefi` crate or any UEFI-provided types,
protocols, services, or globals.

### Boundary rule

The only interface between loader and kernel is an explicit ABI contract:
- A stable entry signature
- A project-owned handoff struct layout (`BootInfo`)

The loader may know about the kernel; the kernel must never know about the
loader.

## Consequences

- Boot code becomes simpler to reason about and delete.
- Firmware dependencies are contained and cannot leak into kernel design.
- The `BootInfo` handoff becomes a critical contract and must be treated as an
  ABI boundary.
- Additional capabilities (paging, interrupts, device drivers) are developed in
  the kernel, not in the loader.
- Debugging output must be available in the kernel without firmware support.

## Future Considerations

The split supports future evolution:
- The loader can change or be replaced without altering kernel design, as long
  as the handoff contract is maintained.
- Additional boot paths (e.g., alternative loaders or future architectures) can
  be introduced by implementing the same handoff contract.
- If stable external interfaces emerge (e.g., userspace ABI), they can be
  versioned independently of the loader.

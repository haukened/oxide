# ADR 0005: Loader–Kernel Handoff ABI Policy

## Status
Accepted

## Context

Oxide enforces a strict responsibility split between the firmware-facing loader
and the firmware-independent kernel (see ADR 0004). The transition from loader
to kernel is the point at which:

- UEFI Boot Services are terminated (`ExitBootServices`)
- Firmware protocols and helpers become unavailable and must not be relied upon
- The kernel assumes ownership of hardware resources and system state

This transition must be explicit, auditable, and difficult to violate
accidentally. Without an explicit contract, the loader and kernel can drift via
implicit assumptions, ad-hoc globals, or firmware-leaking types.

Oxide therefore requires a single, well-defined ABI boundary for control
transfer and initial state handoff.

## Decision

Oxide will define a **single loader→kernel handoff ABI** consisting of:

1. A **kernel entry contract** (calling convention + parameter semantics)
2. A **project-owned handoff structure** (commonly referred to as `BootInfo`)
3. A **compatibility check** performed by the kernel at entry

This ADR defines policy and constraints for the handoff ABI. The concrete
structure layout and field-level specification live in the canonical contract
documentation (e.g., `docs/architecture/bootinfo.md`) and may evolve within the
bounds of this policy.

## Handoff ABI Policy

### Single Interface

- The handoff structure is the **only** supported interface between loader and
  kernel at boot.
- The loader may know about the kernel; the kernel must never depend on loader
  internals.
- No UEFI types, protocol handles, or firmware-managed lifetimes may appear in
  the handoff structure.

### Versioning

- The handoff structure must include an explicit **layout/version identifier**
  (e.g., `u32 version` or a magic+version pair).
- Version identifiers exist to detect incompatible handoff layouts, not to
  promise backward compatibility.
- Changes that invalidate interpretation of existing fields require a version
  bump and coordinated loader+kernel updates.

### Ownership and Lifetime

- All pointers in the handoff must refer to memory **owned by the OS after
  firmware exit**.
- The loader must not pass pointers to firmware-owned buffers or protocol
  objects.
- The kernel may assume that handoff-referenced buffers remain valid until the
  kernel explicitly reclaims or repurposes them.

### Address Semantics

- All addresses passed in the handoff must declare whether they are **physical**
  or **virtual** addresses.
- Until paging is enabled and a stable virtual map is defined, the default
  expectation is that addresses are **physical**.
- The handoff contract must avoid ambiguous “pointer” fields without a stated
  address space.

### Alignment and Layout Stability

- The handoff structure must define required alignment for itself and any
  pointed-to buffers.
- The loader must allocate and align buffers accordingly.
- The kernel must not assume Rust compiler layout defaults unless the contract
  explicitly requires a stable representation (e.g., `repr(C)`).

### Required Compatibility Check

At kernel entry, the kernel must validate at minimum:

- The handoff magic/version is recognized
- Required pointers are non-null and correctly aligned
- Buffer sizes are sane (non-zero, within expected bounds)
- Enumerated values are within the defined range (e.g., pixel format)
- Any “length/count” metadata matches the buffer extents

On failure, the kernel must halt safely (or enter a minimal debug path) without
invoking firmware services.

## Consequences

- The loader→kernel transition is explicit and reviewable.
- Firmware dependencies are prevented from leaking into kernel design.
- The handoff contract becomes a first-class architectural artifact.
- Kernel bring-up becomes easier to debug because invalid handoff states fail
  early and deterministically.
- The project can evolve the handoff within epochs without treating every field
  change as an ADR-level decision, as long as policy constraints are respected.

## Future Considerations

- As Oxide’s memory model evolves (paging, virtual address map), the handoff
  documentation must state address-space semantics explicitly.
- If stable external ABIs emerge (userspace ABI, module ABI), they may adopt
  their own versioning strategies independently.
- If additional boot paths or architectures are introduced in future epochs,
  they must implement this handoff ABI policy (or introduce a new epoch with a
  deliberate policy change).

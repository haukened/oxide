# Architectural Boundaries

This document defines the **non-negotiable boundaries** of the system. These boundaries are architectural law. They exist to constrain design, prevent privilege creep, and make security, correctness, and reasoning tractable over time.

If a change violates a boundary defined here, the change is incorrect unless this document is updated first.

This file describes *what must not cross*, not *how things are implemented*.

---

## Boundary Principles

All boundaries in the system follow these principles:

- No implicit authority
- No ambient access
- No invisible control flow
- No shared mutable state across trust boundaries without explicit design
- Every boundary crossing is explicit, intentional, and auditable

Boundaries exist to **reduce blast radius**.

---

## Primary Boundaries

### Kernel ↔ User Space

**Type:** Privilege boundary  
**Enforced by:** Hardware (MMU, CPU privilege levels), kernel

The kernel and user space are strictly separated.

Kernel guarantees:
- User processes cannot execute privileged instructions
- User processes cannot read or write kernel memory
- User processes cannot influence kernel control flow except through defined interfaces

User space guarantees:
- All input to the kernel is treated as untrusted
- No assumptions are made about kernel internal state

Allowed crossings:
- System calls
- IPC endpoints explicitly exposed by the kernel
- Faults and exceptions handled by the kernel

Prohibited:
- Direct memory access without mediation
- Implicit control transfer
- Shared writable memory without explicit kernel-managed mechanisms

---

### Kernel ↔ Drivers

**Type:** Privilege and trust boundary  
**Enforced by:** Kernel policy, capability system

Drivers are not trusted by default.

Rules:
- Drivers run in user space unless explicitly designated kernel-resident
- Drivers receive only the minimum capabilities required for their device
- Drivers cannot allocate arbitrary kernel memory
- Drivers cannot execute arbitrary kernel code

Kernel-resident drivers:
- Are exceptions, not the norm
- Require explicit justification
- Are subject to stricter review and reduced interface surface

Driver failure must not:
- Panic the kernel
- Corrupt kernel memory
- Compromise unrelated subsystems

---

### Service ↔ Service

**Type:** Isolation boundary  
**Enforced by:** Address spaces, capability system

Services are isolated from one another.

Rules:
- No service may directly access another service’s memory
- No service may observe another service’s existence without an explicit channel
- All service-to-service interaction occurs via IPC

Capabilities:
- Define exactly what operations are permitted
- Are explicit and transferable only through IPC
- Cannot be guessed or forged

There is no global service registry implicitly visible to all services.

---

### Process ↔ Process

**Type:** Isolation boundary  
**Enforced by:** Address spaces, scheduler, kernel

Processes are isolated execution units.

Rules:
- One process cannot read or write another’s memory
- One process cannot signal another without explicit permission
- Scheduling does not imply authority

Shared memory:
- Is opt-in
- Is explicitly established
- Has well-defined ownership and lifetime

---

## Resource Boundaries

### Memory

- Each process owns its address space
- Kernel memory is never directly accessible
- Shared memory regions are explicitly created and capability-guarded
- Memory ownership and lifetime must be explicit

No component may assume memory persistence across restarts unless explicitly documented.

---

### CPU and Scheduling

- Scheduling priority does not imply trust
- No component may assume guaranteed CPU time without explicit contract
- Preemption is always possible unless explicitly disabled by design

Real-time guarantees, if any, must be documented as contracts, not assumptions.

---

### I/O and Devices

- All device access is capability-mediated
- No implicit access to global device namespaces
- DMA is mediated by the kernel and hardware (e.g. IOMMU)

Devices are not globally visible. Visibility is explicitly granted.

---

## Information Boundaries

### Visibility

- Components cannot observe:
  - Other components’ memory
  - Other components’ scheduling state
  - Other components’ failures
- Unless explicitly permitted

Side channels are acknowledged as a risk but must not be relied upon for functionality.

---

### Error Propagation

- Errors do not propagate across boundaries implicitly
- Failure in one component must not cause cascading failure
- Recovery is local unless explicitly coordinated

---

## Boundary Enforcement

Boundaries are enforced by:
- Hardware mechanisms
- Kernel invariants
- Capability checks
- IPC contracts

Logging, debugging, or observability mechanisms must not violate boundaries.

---

## What This Document Is Not

This document does **not**:
- Describe IPC message formats
- Define system call semantics
- Specify ABI stability guarantees
- Provide implementation details

Those belong in interface and ABI documentation.

---

## Related Documents

- `overview.md` — system structure
- `epochs.md` — temporal boundaries
- `boot-flow.md` — trust establishment and early execution
- `abi/` — interface and compatibility contracts
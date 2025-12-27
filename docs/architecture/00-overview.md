# Architecture Overview

This document describes the **structural shape** of the system: the major components, their responsibilities, and the boundaries between them. It intentionally avoids historical context, justification, or trade‑off discussion. Those belong in design rationale documents.

This file defines *what exists*, not *why it exists*.

---

## System Model

The system is composed of a **minimal privileged kernel** and a set of **isolated user‑space services**. Privilege, trust, and responsibility are explicitly partitioned.

At a high level:

- The kernel provides:
  - CPU scheduling
  - Virtual memory management
  - Capability enforcement
  - IPC primitives
  - Minimal hardware abstraction

- All policy, drivers, and services are externalized where feasible.

No subsystem has implicit authority. All interaction is explicit.

---

## Major Components

### Kernel Core

**Privilege:** Highest  
**Address Space:** Kernel  
**Trust Level:** Trusted Computing Base (TCB)

Responsibilities:
- Thread and process management
- Virtual memory and address space isolation
- Capability creation, transfer, and revocation
- Interrupt and exception handling
- Low-level IPC mechanisms
- Bootstrapping and early system bring‑up

The kernel core does **not**:
- Parse complex or user-controlled data formats
- Implement device policy
- Perform filesystem logic
- Contain UI, networking stacks, or service logic

---

### User-Space Services

**Privilege:** Unprivileged  
**Address Space:** Per-process  
**Trust Level:** Untrusted by default

Services include (non-exhaustive):
- Device drivers
- Filesystems
- Networking stacks
- Init / service management
- Logging and diagnostics
- User session management

Each service:
- Runs in its own address space
- Holds only explicitly granted capabilities
- Communicates solely via IPC

Failure of a service must not compromise kernel integrity.

---

### Drivers

Drivers are treated as **services**, not extensions of the kernel.

Characteristics:
- Run in user space by default
- Use IPC and shared memory for performance-critical paths
- Access hardware through kernel-mediated capabilities
- Are restartable and isolatable

Exception cases (kernel-resident drivers) must be explicitly justified and reviewed.

---

### Inter-Process Communication (IPC)

IPC is a first-class kernel primitive.

Properties:
- Explicit endpoints
- Capability-guarded access
- Support for synchronous and asynchronous messaging
- Shared memory for bulk data transfer

No implicit global namespaces or broadcast mechanisms exist.

---

### Capability System

All authority is represented by **capabilities**.

Capabilities:
- Are unforgeable
- Are explicit objects
- Encode rights and scope
- Can be transferred only through IPC

There is no ambient authority. Possession of a capability is the sole mechanism for access.

---

## Trust Boundaries

The system enforces the following primary boundaries:

- Kernel ↔ User space
- Service ↔ Service
- Driver ↔ Kernel
- Process ↔ Process

Crossing a boundary always requires:
- An explicit interface
- A defined capability
- A documented contract

---

## Execution Environment

### Address Spaces
- One address space per process
- Kernel address space isolated and protected
- No shared writable memory across trust boundaries without explicit intent

### Scheduling
- Preemptive, priority-aware scheduling
- Policy configurable; mechanism stable
- No service may assume scheduling guarantees without explicit contract

---

## What This Document Is Not

This document does **not**:
- Argue architectural trade-offs
- Compare against other operating systems
- Describe boot-time sequencing in detail
- Define ABI or syscall stability guarantees

Those concerns are covered elsewhere.

---

## Related Documents

- `boundaries.md` — explicit trust and privilege boundaries
- `epochs.md` — system lifecycle phases and temporal constraints
- `boot-flow.md` — boot and initialization sequencing
- `abi/` — user-space and kernel interface contracts
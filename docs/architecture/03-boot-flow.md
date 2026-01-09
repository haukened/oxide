# Boot Flow

This document describes the **concrete boot and initialization sequence** of the system. It maps execution steps to epoch transitions and defines when guarantees become valid.

Boot flow is not narrative. It is a sequence of **irreversible transitions** that establish trust, authority, and capability.

If a step assumes guarantees from a later epoch, it is incorrect.

---

## Boot Flow Principles

- Boot proceeds through explicit epoch transitions
- Each step establishes new guarantees and never revokes prior ones
- Trust is accumulated monotonically
- Failure before steady-state is fatal unless explicitly handled
- No step may rely on user-space availability unless explicitly stated

Boot flow exists to eliminate ambiguity during system start-up.

---

## Phase 0: Firmware Execution  
**Epoch:** Genesis (Epoch 0)

### Responsibilities
- Hardware initialization
- Platform configuration
- Discovery of bootable media
- Transfer of control to bootloader

### Assumptions
- Firmware behavior is minimally trusted
- Firmware input is not considered secure
- Firmware correctness is not guaranteed beyond handoff

### Transition
- Control transferred to bootloader entry point

No kernel guarantees exist in this phase.

---

## Phase 1: Bootloader  
**Epoch:** Genesis → Spark (Epoch 0 → Epoch 1)

### Responsibilities
- Load kernel image into memory
- Load initial metadata (memory map, CPU state)
- Establish initial execution context
- Transfer control to kernel entry point

### Constraints
- No virtual memory
- No concurrency
- No dynamic allocation beyond bootloader-managed memory

### Transition
- Enter kernel entry point
- Transition to Spark

---

## Phase 2: Early Kernel Entry  
**Epoch:** Spark (Epoch 1)

### Responsibilities
- Establish kernel stack
- Assert CPU privilege level
- Disable or mask interrupts
- Validate boot-time metadata

### Constraints
- Single-threaded execution
- No blocking
- No scheduler
- No IPC
- No logging guarantees

### Transition
- Kernel-controlled execution environment established

Failure in this phase halts the system.

---

## Phase 3: Memory Bring-Up  
**Epoch:** Spark → Foundation (Epoch 1 → Epoch 2)

### Responsibilities
- Initialize physical memory management
- Establish virtual memory
- Map kernel address space
- Enable memory protection

### Guarantees After Transition
- Virtual memory active (identity mapped regions)
- Kernel memory protected
- Allocators available

### Transition
- Transition to Foundation

Errors in this phase are fatal.

---

## Phase 4: Kernel Core Initialization  
**Epoch:** Foundation → Awakening (Epoch 2 → Epoch 3)

### Responsibilities
- Initialize scheduler
- Enable interrupts
- Initialize timers
- Initialize IPC primitives
- Initialize capability system

### Guarantees After Transition
- Preemptive scheduling available
- Concurrency permitted
- IPC endpoints may be created

### Transition
- Transition to Awakening

Kernel subsystems must not depend on user-space availability.

---

## Phase 5: Initial User Space Launch  
**Epoch:** Awakening → First Light (Epoch 3 → Epoch 4)

### Responsibilities
- Create first user-space process (init)
- Establish initial capability set
- Establish kernel ↔ user IPC channel

### Guarantees After Transition
- User-space execution available
- Capability transfer possible

### Transition
- Transition to First Light

Failure of init is fatal.

---

## Phase 6: Core Service Initialization  
**Epoch:** First Light → Perception (Epoch 4 → Epoch 5)

### Responsibilities
- Launch essential services
- Launch driver services
- Establish hardware access capabilities
- Configure interrupt routing

### Guarantees After Transition
- Device discovery possible
- Hardware interaction available via drivers

### Transition
- Transition to Perception

Services must tolerate partial device availability.

---

## Phase 7: Steady-State Establishment  
**Epoch:** Perception → Equilibrium (Epoch 5 → Epoch 6)

### Responsibilities
- Complete service bring-up
- Validate system readiness
- Enable persistent storage semantics
- Enable networking (if configured)

### Guarantees After Transition
- Stable scheduling and timing
- Full service availability
- ABI guarantees in effect

### Transition
- Transition to Equilibrium

This marks normal system operation.

---

## Phase 8: Fault Handling and Recovery  
**Epoch:** Any → Resilience (Epoch 7)

### Responsibilities
- Detect service or device failure
- Restart or isolate failed components
- Preserve kernel authority

### Constraints
- No new trust granted
- No boundary violations permitted
- Recovery is bounded and explicit

Resilience may overlap temporally with other epochs.

---

## Phase 9: Shutdown or Reboot  
**Epoch:** Any → Quiescence (Epoch 8)

### Responsibilities
- Revoke capabilities
- Stop accepting new work
- Tear down services
- Prepare for power-off or reboot

### Guarantees
- Best-effort orderly shutdown

Shutdown is a controlled loss of state, not a rollback.

---

## Invalid Transitions

The following are explicitly invalid:
- Skipping epochs
- Re-entering completed epochs
- Assuming steady-state guarantees early
- Granting new trust during shutdown

Invalid transitions are bugs.

---

## What This Document Is Not

This document does **not**:
- Specify bootloader implementation details
- Define ABI or syscall formats
- Describe recovery algorithms
- Provide performance guarantees

Those belong in interface or implementation documentation.

---

## Related Documents

- `overview.md` — system structure
- `boundaries.md` — trust and isolation boundaries
- `epochs.md` — temporal guarantees and constraints
- `abi/` — interface and compatibility contracts
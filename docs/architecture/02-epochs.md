# System Epochs

This document defines the **temporal structure** of the system: the major phases (“epochs”) the system passes through during its lifetime, and the constraints that apply in each phase.

Epochs exist to make **time explicit**. Many kernel failures arise not from incorrect logic, but from code executing under assumptions that are not yet valid.

If code relies on behavior not guaranteed in the current epoch, it is incorrect.

This document describes *when assumptions become valid*, not *how they are implemented*.

---

## Epoch Principles

- Epochs are strictly ordered
- Earlier epochs have fewer guarantees than later ones
- Assumptions only accumulate; they are never retroactively valid
- Code must explicitly declare which epoch it assumes
- Transitions between epochs are explicit and irreversible
- No guarantees exist for an epoch until it is completed
- Completed epochs define immutable system state

Epoch boundaries are as important as privilege boundaries.

---

## Epoch 0: Firmware / Pre-Kernel
**Codename:** Genesis

**Entry:** System reset or power-on  
**Exit:** Control transferred to kernel entry point

### Guarantees
- CPU is executing in a known reset state
- Firmware-provided memory map is available (format is firmware-specific)
- One hardware thread is active

### Non-Guarantees
- No virtual memory
- No interrupts
- No scheduler
- No heap
- No stack beyond what firmware provides
- No trust in firmware correctness beyond minimal handoff

### Constraints
- No dynamic allocation
- No blocking
- No concurrency assumptions

This epoch is outside kernel control but defines initial trust assumptions.

---

## Epoch 1: Early Kernel Bring-Up
**Codename:** Spark

**Entry:** Kernel entry point  
**Exit:** Kernel has established its own execution environment

### Guarantees
- Kernel code and data are resident
- A minimal stack exists
- CPU mode and privilege level are under kernel control

### Non-Guarantees
- No full virtual memory yet
- No scheduler
- No IPC
- No drivers
- No logging guarantees

### Constraints
- Single-threaded execution
- No preemption
- No blocking
- Only static or bump allocation

Only code explicitly marked as early-init may execute here.

---

## Epoch 2: Memory Initialization
**Codename:** Foundation

**Entry:** Kernel-controlled execution environment established  
**Exit:** Virtual memory and core allocators initialized

### Guarantees
- Virtual memory enabled
- Kernel address space fully mapped
- Basic physical and virtual allocators available

### Non-Guarantees
- No user processes
- No IPC
- No drivers
- No stable timing guarantees

### Constraints
- Limited concurrency
- Allocation permitted, but must be bounded
- Errors are fatal

This is the first epoch where memory safety mechanisms are fully active.

---

## Epoch 3: Kernel Core Online
**Codename:** Awakening

**Entry:** Memory and core subsystems initialized  
**Exit:** First user-space services launched

### Guarantees
- Scheduler active
- Interrupts enabled
- Timers available
- IPC primitives available
- Capability system initialized

### Non-Guarantees
- No user-space services yet
- No drivers
- No persistent storage

### Constraints
- Kernel subsystems must be self-contained
- No dependency on user-space availability
- Blocking allowed only on kernel-managed primitives

This is the earliest point at which concurrency is fully supported.

---

## Epoch 4: Initial User Space
**Codename:** First Light

**Entry:** First user-space process launched (init)  
**Exit:** Core services available

### Guarantees
- User processes can execute
- IPC between kernel and services is operational
- Capability transfer is supported

### Non-Guarantees
- No full device availability
- No network
- No persistent filesystem guarantees

### Constraints
- Services must tolerate partial system availability
- Failure of a service must not halt system progress

This epoch is characterized by gradual capability discovery.

---

## Epoch 5: Driver Bring-Up
**Codename:** Perception

**Entry:** Driver services launched  
**Exit:** Essential devices operational

### Guarantees
- Device drivers executing in user space
- Hardware access mediated by capabilities
- Interrupt routing stable

### Non-Guarantees
- Not all devices are present
- Performance characteristics may be unstable
- Optional hardware may be absent

### Constraints
- Drivers must be restartable
- Driver failure must not cascade
- No assumptions about device ordering

---

## Epoch 6: Service Steady-State
**Codename:** Equilibrium

**Entry:** Core services and drivers operational  
**Exit:** System shutdown or reboot

### Guarantees
- All declared services are available
- Scheduling and timing guarantees are stable
- Persistent storage semantics are valid
- Networking may be available

### Non-Guarantees
- Hardware permanence (hotplug may occur)
- Service permanence (services may restart)

### Constraints
- All boundaries defined in `boundaries.md` apply
- ABI stability guarantees apply
- Observability must respect isolation rules

This is the normal operating state of the system.

---

## Epoch 7: Degradation and Recovery
**Codename:** Resilience

**Entry:** Fault detected (service failure, device removal, resource exhaustion)  
**Exit:** Return to steady-state or shutdown

### Guarantees
- Kernel remains authoritative
- Boundaries remain enforced
- Recovery mechanisms may execute

### Non-Guarantees
- Service availability
- Performance guarantees

### Constraints
- No new trust is granted
- Recovery must be explicit and bounded
- Failures must not escalate privilege

This epoch may overlap logically with others but represents a change in assumptions.

---

## Epoch 8: Shutdown
**Codename:** Quiescence

**Entry:** Shutdown or reboot requested  
**Exit:** Control returned to firmware or power-off

### Guarantees
- Orderly teardown where possible
- Explicit revocation of capabilities

### Non-Guarantees
- Completion of all cleanup
- Persistence of in-flight state

### Constraints
- No new work accepted
- Best-effort cleanup only

Shutdown is not a rollback; it is a controlled loss of state.

---

## Epoch Transitions

- Epoch transitions are explicit
- Code must not assume later-epoch guarantees early
- Backward transitions do not exist

Epoch violations are bugs, not edge cases.

---

## What This Document Is Not

This document does **not**:
- Specify bootloader behavior
- Define IPC or ABI formats
- Describe recovery algorithms
- Guarantee specific performance characteristics

Those concerns belong in implementation and interface documents.

---

## Related Documents

- `overview.md` — structural system model
- `boundaries.md` — trust and isolation boundaries
- `boot-flow.md` — concrete boot and initialization sequence
- `abi/` — interface and compatibility contracts
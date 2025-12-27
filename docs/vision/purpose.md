# Purpose

Oxide exists to explore and validate operating system architecture under modern
assumptions.

The project’s purpose is to design and build a small, coherent kernel that:
- boots on contemporary hardware
- establishes clear ownership of machine resources
- makes architectural boundaries explicit
- prioritizes correctness, debuggability, and security over legacy compatibility

Oxide is not driven by feature completeness or production readiness. Its value
comes from making system design decisions *visible*, *deliberate*, and
*reversible* while the system is still small enough to reason about as a whole.

## Design Exploration

Oxide is intentionally informed by decades of operating system design experience
(Unix/Linux, Windows NT, BSD, Mach, and modern research kernels), but it is not an
attempt to reimplement any single lineage.

Instead, the project explores questions such as:
- how to balance performance with isolation and fault tolerance
- how to minimize the amount of code that must run with full privilege
- how to design clean handoff boundaries between system phases
- how to scale across modern multi-core hardware without relying on global state
- how to make security properties structural rather than bolted on

These questions are treated as first-class design concerns, not afterthoughts.

## Security as a Structural Property

Oxide treats security as an architectural property rather than a collection of
mitigations. The system is designed to:
- follow the principle of least privilege
- isolate components wherever feasible
- prefer explicit capabilities and ownership over ambient authority
- reduce the kernel’s trusted computing base over time

The use of memory-safe implementation techniques (where practical) is intended
to eliminate entire classes of bugs rather than merely detecting them.

Advanced security mechanisms (randomization, formal verification, hardened
isolation) are intentionally deferred until the system’s core invariants are
well understood.

## Performance and Scalability

Performance remains a first-order concern. Oxide aims to:
- keep hot paths efficient and low-overhead
- scale cleanly across multiple cores
- avoid global bottlenecks and unnecessary serialization
- support asynchronous and event-driven execution models

Rather than choosing dogmatically between monolithic or microkernel designs,
Oxide treats kernel structure as a spectrum, favoring pragmatic trade-offs that
reflect modern hardware capabilities and workloads.

## Adaptability and Longevity

Oxide is designed to evolve through clearly defined architectural epochs.
Decisions that materially change the system’s invariants are made explicitly
and recorded, allowing the system to change shape without losing coherence.

By prioritizing explicit contracts, clear boundaries, and deletable early code,
the project aims to remain adaptable as requirements, hardware, and threat
models change.

At its core, Oxide is a vehicle for understanding how a modern operating system
can be:
- secure by design
- performant in practice
- modular without being fragmented
- and understandable by the humans who build it
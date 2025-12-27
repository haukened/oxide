# Principles

The following principles guide all design and implementation decisions in
Oxide. They are intended to be stable across epochs unless explicitly revised.

## Explicit Ownership

Every resource—memory, CPU state, devices, data structures—must have a clearly
defined owner at every stage of execution.

Implicit ownership, hidden global state, and “temporary” responsibility sharing
are treated as design errors.

## Clear Boundaries

Architectural boundaries must be explicit and enforceable:
- loader vs kernel
- firmware vs OS
- physical vs virtual memory
- policy vs mechanism

Boundaries exist to reduce cognitive load and prevent accidental coupling.

## Modern Assumptions

Oxide assumes modern hardware and firmware:
- x86_64 architecture (for current epochs)
- UEFI firmware
- linear framebuffers
- contemporary CPU features

Legacy compatibility is not a goal unless explicitly stated.

## Pragmatic Structure

Oxide does not commit prematurely to a single kernel structure model.
Monolithic, microkernel, hybrid, and message-passing designs are treated as
points in a design space rather than identities to defend.

Structural choices are evaluated based on concrete trade-offs in performance,
isolation, debuggability, and complexity, not architectural dogma.

## Minimalism with Intent

Minimalism is a means, not an aesthetic.
Code should be as small as possible while still making system behavior clear.

Abstraction is introduced only when it reduces complexity rather than hiding it.

## Debuggability First

The system must be able to explain itself when something goes wrong.

Early graphical output, deterministic behavior, and fail-fast validation are
preferred over silent recovery or implicit assumptions.

Observability and explainability are considered prerequisites for correctness.
When trade-offs exist between raw performance and the ability to reason about
system behavior, Oxide favors designs that fail visibly and early.

## Policy over Mechanism

Architectural decisions are recorded as policy.
Implementation details are expected to evolve.

This distinction allows the system to change shape without losing coherence.

## Deletability

Early code should be written with the expectation that it will be deleted.

If code feels too important to remove, it likely belongs at a different layer.

# Non-Goals

The following items are explicitly *not* goals of the Oxide project. Stating
these up front is intended to prevent scope creep and misplaced optimization.

## Legacy Platform Support

Oxide does not aim to support:
- BIOS-based booting
- real mode or protected mode execution
- VGA text mode
- 32-bit x86 systems

These constraints are considered historical artifacts rather than requirements.

## Production Readiness

Oxide is not currently intended to be:
- a general-purpose OS
- a Linux or BSD replacement
- suitable for end users
- stable across updates

Correctness and clarity take precedence over stability guarantees.

## Feature Parity

Oxide does not aim to replicate the feature sets of existing operating systems.

Features are added only when they serve architectural exploration or validate
design assumptions.

## Premature Portability

Supporting multiple architectures, platforms, or boot environments is not a
goal during early epochs.

Portability is addressed only after core invariants are proven on the initial
target platform.

This includes deferring commitments to specific compatibility APIs (e.g.,
POSIX or Linux syscall emulation) until core kernel invariants are established.

## Security Hardening (Early Epochs)

Advanced security features such as:
- address randomization
- sandboxing
- formal isolation guarantees
- exploit mitigations

are intentionally deferred until a concrete threat model exists.

## Accidental Complexity

Oxide avoids:
- unnecessary abstraction layers
- framework-driven design
- configuration-heavy systems
- solving problems “just in case”

Complexity must earn its place.

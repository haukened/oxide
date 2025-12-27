# ADR 0000: Architectural Decision Process and Authority

**Status:** Accepted  

## Context

The project did not initially define how architectural decisions are proposed, evaluated, approved, or overturned. This omission creates ambiguity about authority, escalation, and final responsibility—particularly as the system grows in complexity and contributors.

This document establishes a lightweight, explicit model for architectural decision-making. It is intentionally pragmatic, minimally bureaucratic, and slightly self-aware.

## Decision Authority

This project follows a **BDFL-style model**.

### BDFL (Benevolent Dictator For Life)

The project has a single final architectural authority:

> **The Architect**

The Architect:
- Has final say on architectural decisions
- Is responsible for maintaining conceptual integrity
- Is accountable for long-term coherence, not short-term consensus
- May delegate decisions, but retains override authority

This role is not ceremonial. Responsibility is inseparable from authority.

## What the Architect Is (and Is Not)

The Architect **is**:
- The arbiter of system boundaries, invariants, and contracts
- The steward of long-term architectural vision
- The tie-breaker when trade-offs are irreconcilable
- The person who says “no” when “yes” would accumulate debt

The Architect **is not**:
- Obligated to accept majority opinion
- Required to justify decisions beyond architectural coherence

The model optimizes for coherence over democracy. It is not "consensus theatre."

## Decision Categories

### 1. Architectural Decisions

Examples:
- Kernel/user boundary changes
- ABI guarantees
- Capability model changes
- Epoch or boot-flow changes
- Trust or privilege model changes

**Process:**
- Must be documented as an ADR
- Require explicit Architect approval
- May invalidate existing ADRs

### 2. Design Decisions

Examples:
- Subsystem decomposition
- Internal interfaces
- Algorithms and data structures

**Process:**
- Prefer discussion and review
- ADR optional but encouraged
- Architect may intervene if coherence is threatened

### 3. Implementation Decisions

Examples:
- Coding style
- Refactoring
- Performance optimizations
- Tooling choices

**Process:**
- Decentralized
- Governed by review and conventions
- No Architect approval required unless architectural impact exists

## Proposal and Review

Anyone may propose an architectural change.

A proposal should:
- Clearly state the problem
- Identify affected boundaries or epochs
- Describe trade-offs
- Acknowledge what invariants it stresses or breaks

Approval criteria are architectural, not political.

## Overruling and Reversal

The Architect may:
- Reverse prior decisions
- Amend ADRs
- Declare past choices obsolete

Reversals should be rare, explicit, and documented.

Stability is preferred. Correctness is mandatory.

## Humor Clause (Non-Binding)

While the title “BDFL” is used with affectionate irony, the underlying reality is serious:

> Distributed consensus does not produce coherent architecture.

Disagreement is welcome. Bikeshedding is not.

## Consequences

This model:
- Prioritizes clarity over consensus
- Makes responsibility explicit
- Prevents architectural drift
- Allows fast, decisive action when needed

The cost is reduced procedural democracy. The benefit is a system that makes sense.

## Final Note

If you are uncomfortable with this model, this may not be the right project for you.

That is not a threat. It is an architectural constraint.
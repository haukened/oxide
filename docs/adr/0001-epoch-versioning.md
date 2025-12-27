# ADR 0001: Epoch-Based Versioning with Codenames

## Status
Accepted

## Context

Oxide is an operating system under active architectural development.
At this stage of the project:

- There is no stable public API or ABI.
- Core system invariants (boot flow, memory ownership, execution model) are still being defined.
- Breaking changes are expected and frequent.
- The primary consumers are developers working on the system itself.

Traditional semantic versioning (SemVer) assumes a stable public interface and
communicates compatibility guarantees to external consumers. Applying SemVer
at this stage would misrepresent the nature of change in the project and
encourage artificial version churn or perpetual pre-1.0 semantics.

Calendar versioning (CalVer) accurately timestamps snapshots but does not
communicate architectural meaning or system invariants.

The project requires a versioning approach that reflects **architectural eras**
rather than compatibility promises.

## Decision

Oxide will use **epoch-based versioning with human-readable codenames** during
early and mid-stage development.

An **epoch** represents a period of architectural consistency defined by a set
of core invariants. Version identifiers communicate *which world the system is
operating in*, not whether changes are backward-compatible.

Each epoch:
- Is introduced intentionally when a fundamental system invariant changes
- May include multiple internal revisions
- Is expected to contain breaking changes
- Is documented explicitly with its defining assumptions

Epochs are identified by:
- A monotonically increasing epoch number
- An associated codename for human reference

Example identifiers:
- `Epoch 0 ("Ignition")`
- `Epoch 1 ("Separation")`

Optional tags (e.g., calendar-based) may be used for snapshots or releases, but
the epoch remains the primary semantic identifier.

## Required Epoch Description

Each epoch must be described in a canonical location (e.g.,
`docs/architecture/epochs.md`) and must include, at minimum:

- **Defining invariants**  
  The architectural assumptions that hold throughout the epoch (e.g.,
  firmware presence, memory ownership model, execution environment).

- **Explicit non-invariants**  
  Areas expected to change or churn within the epoch.

- **Required contracts**  
  Any contracts that must exist for the epoch to be considered valid
  (e.g., presence of a `BootInfo` handoff, defined kernel entry semantics).

- **Exit criteria**  
  The conditions under which the epoch is considered complete and a transition
  to a new epoch is justified.

This description is the authoritative reference for what “stability” means
within the epoch.

## Epoch Transition Rules

Transitions between epochs are **explicit design events**, not incidental
version bumps.

An epoch transition requires:
- A new or updated ADR describing the change in core invariants
- An update to the canonical epoch description document
- A clear statement of which invariants no longer hold

Epoch transitions are expected to be rare and deliberate. Most development
occurs within an epoch.

## Consequences

- Version identifiers reflect architectural meaning rather than API stability.
- Breaking changes within an epoch are normal and expected.
- Epoch transitions are rare and treated as major design events.
- Documentation (especially ADRs and epoch descriptions) becomes the primary
  source of historical context for system evolution.
- Different components may later adopt more specific versioning schemes
  (e.g., SemVer for a stable userspace ABI) without changing the epoch model
  for kernel development.

## Future Considerations

If Oxide eventually exposes stable external interfaces (e.g., kernel ABI,
module API, or userspace ABI), those interfaces may adopt semantic versioning
independently.

Epoch-based versioning will remain appropriate for internal kernel and system
architecture evolution as long as core invariants continue to change.

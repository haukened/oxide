# Monotonic Time Source

This module tracks the kernel’s single monotonic clock, currently implemented via the CPU timestamp counter (TSC).

## Goals

- Provide a cheap, always-increasing time base for diagnostics and scheduling heuristics.
- Allow consumers to obtain raw tick counts even when calibration data is unavailable.
- Lay groundwork for richer wall-clock support without exposing firmware dependencies.

## Initialization

`init_tsc_monotonic(frequency_hz)` installs a `MonotonicClock` the first time it is called. The baseline captures the current TSC value, and the supplied frequency (if non-zero) enables later nanosecond conversion. Re-invocation is harmless; only the first call has effect. See [kernel/src/time/mod.rs#L8-L33](kernel/src/time/mod.rs#L8-L33).

## Reading the Clock

Two query functions expose the clock:

- `monotonic_ticks()` returns elapsed ticks since initialization. This always succeeds once the clock is configured because it only depends on `rdtsc`. [kernel/src/time/mod.rs#L35-L45](kernel/src/time/mod.rs#L35-L45)
- `monotonic_nanos()` converts ticks to nanoseconds when a frequency is known. If the frequency was zero or conversion would overflow, it returns `None`. [kernel/src/time/mod.rs#L47-L63](kernel/src/time/mod.rs#L47-L63)

Consumers should prefer `monotonic_ticks` for relative timing and only request nanoseconds when higher-level code can tolerate the optional result.

## Implementation Notes

- `MonotonicClock` snapshots the baseline tick count and frequency; elapsed ticks use wrapping subtraction to remain valid even if the TSC wraps (practically improbable on modern hardware). [kernel/src/time/mod.rs#L65-L104](kernel/src/time/mod.rs#L65-L104)
- `read_tsc()` issues `rdtsc` with `nomem`/`nostack`/`preserves_flags` to avoid clobbering registers or assuming memory ordering guarantees. [kernel/src/time/mod.rs#L106-L114](kernel/src/time/mod.rs#L106-L114)
- The module is marked `#![allow(dead_code)]` because not all consumers are wired yet; this will change as subsystems adopt the clock.

## Future Work

- Add fallback sources (HPET, ACPI PM timer) for systems where the TSC is unstable.
- Introduce wall-clock initialization (e.g., via RTC or firmware-supplied epoch) layered over the monotonic base.
- Track frequency recalibration or invariance checks to handle TSC drift or power-state changes.

This document will expand alongside the time subsystem; for now it documents the minimal contract relied on by the console timestamping and early diagnostics.

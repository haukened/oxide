#![allow(dead_code)]

use core::{arch::asm, cell::UnsafeCell};

/// Errors that can occur while configuring the monotonic time source.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MonotonicInitError {
    /// A time source has already been installed.
    AlreadyInitialized,
    /// The provided frequency is invalid (for example, zero).
    InvalidFrequency { requested_hz: u64 },
}

struct MonotonicCell(UnsafeCell<Option<MonotonicClock>>);

unsafe impl Sync for MonotonicCell {}

static MONOTONIC_CLOCK: MonotonicCell = MonotonicCell(UnsafeCell::new(None));

/// Configure the global monotonic clock using the processor timestamp counter.
///
/// The optional `frequency_hz` parameter allows the caller to provide a calibrated
/// TSC frequency. When absent, consumers can still retrieve raw tick counts but
/// conversion to absolute time units will not be available.
pub fn init_tsc_monotonic(frequency_hz: Option<u64>) -> Result<(), MonotonicInitError> {
    unsafe {
        let slot = &mut *MONOTONIC_CLOCK.0.get();
        if slot.is_some() {
            return Err(MonotonicInitError::AlreadyInitialized);
        }

        if let Some(freq) = frequency_hz {
            if freq == 0 {
                return Err(MonotonicInitError::InvalidFrequency { requested_hz: freq });
            }
        }

        *slot = Some(MonotonicClock::from_tsc(frequency_hz));
        Ok(())
    }
}

/// Returns the number of ticks elapsed since the monotonic clock was initialised.
/// The units are implementation-defined (TSC ticks for the current implementation).
pub fn monotonic_ticks() -> Option<u64> {
    unsafe {
        let slot = &*MONOTONIC_CLOCK.0.get();
        slot.as_ref().map(|clock| clock.elapsed_ticks())
    }
}

/// Returns the elapsed time in nanoseconds since the monotonic clock was initialised.
/// Only available when the time source frequency was provided during initialisation.
pub fn monotonic_nanos() -> Option<u64> {
    unsafe {
        let slot = &*MONOTONIC_CLOCK.0.get();
        slot.as_ref()?.nanoseconds_since_start()
    }
}

struct MonotonicClock {
    baseline_ticks: u64,
    frequency_hz: Option<u64>,
}

impl MonotonicClock {
    fn from_tsc(frequency_hz: Option<u64>) -> Self {
        let baseline_ticks = unsafe { read_tsc() };
        Self {
            baseline_ticks,
            frequency_hz,
        }
    }

    fn elapsed_ticks(&self) -> u64 {
        let current = unsafe { read_tsc() };
        current.wrapping_sub(self.baseline_ticks)
    }

    fn nanoseconds_since_start(&self) -> Option<u64> {
        let frequency = self.frequency_hz?;
        let ticks = self.elapsed_ticks() as u128;
        let frequency = frequency as u128;
        if frequency == 0 {
            return None;
        }
        let nanos = ticks
            .saturating_mul(1_000_000_000u128)
            .checked_div(frequency)?;
        if nanos > u64::MAX as u128 {
            None
        } else {
            Some(nanos as u64)
        }
    }
}

#[inline(always)]
unsafe fn read_tsc() -> u64 {
    let high: u32;
    let low: u32;
    unsafe {
        asm!("rdtsc", out("edx") high, out("eax") low, options(nomem, nostack, preserves_flags));
    }
    ((high as u64) << 32) | (low as u64)
}

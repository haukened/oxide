use core::{arch::asm, time};

use uefi::boot::stall;

const MEASUREMENT_DELAY_US: u64 = 50_000; // 50 ms for stable measurement

pub fn measure_tsc_frequency() -> Option<u64> {
    let start = unsafe { read_tsc() };

    stall(time::Duration::from_micros(MEASUREMENT_DELAY_US));

    let end = unsafe { read_tsc() };
    let delta = end.wrapping_sub(start);

    if delta == 0 {
        return None;
    }

    let numerator = (delta as u128).saturating_mul(1_000_000u128);
    let frequency = numerator.checked_div(MEASUREMENT_DELAY_US as u128)?;

    if frequency > u64::MAX as u128 {
        None
    } else {
        Some(frequency as u64)
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

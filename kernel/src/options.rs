use core::sync::atomic::{AtomicBool, Ordering};

use oxide_abi::Options;

static DEBUG: AtomicBool = AtomicBool::new(false);
static QUIET: AtomicBool = AtomicBool::new(false);

/// Capture bootloader-supplied debug and quiet flags for later queries.
pub fn init(opts: Options) {
    let debug = opts.debug != 0;
    let quiet = opts.quiet != 0;

    DEBUG.store(debug, Ordering::Relaxed);
    QUIET.store(quiet, Ordering::Relaxed);
}

/// Returns true when debug output should be emitted.
#[inline]
pub fn debug_enabled() -> bool {
    DEBUG.load(Ordering::Relaxed)
}

/// Returns true when quiet mode suppresses diagnostics.
#[inline]
pub fn quiet_enabled() -> bool {
    QUIET.load(Ordering::Relaxed)
}

/// Returns true when diagnostics are enabled (debug on and quiet off).
#[inline]
pub fn diagnostics_enabled() -> bool {
    debug_enabled() && !quiet_enabled()
}

use core::sync::atomic::{AtomicBool, Ordering};

use oxide_abi::Options;

static DEBUG: AtomicBool = AtomicBool::new(false);
static QUIET: AtomicBool = AtomicBool::new(false);

pub fn init(opts: Options) {
    let debug = opts.debug != 0;
    let quiet = opts.quiet != 0;

    DEBUG.store(debug, Ordering::Relaxed);
    QUIET.store(quiet, Ordering::Relaxed);
}

#[inline]
pub fn debug_enabled() -> bool {
    DEBUG.load(Ordering::Relaxed)
}

#[inline]
pub fn quiet_enabled() -> bool {
    QUIET.load(Ordering::Relaxed)
}

#[inline]
pub fn diagnostics_enabled() -> bool {
    debug_enabled() && !quiet_enabled()
}

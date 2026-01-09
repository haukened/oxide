#![no_std]
#![no_main]

use oxide_abi::BootAbi;

use crate::memory::{
    error::{FrameAllocError, MemoryInitError},
    init,
};

mod boot;
mod console;
mod framebuffer;
mod memory;
mod options;
mod time;

/// Kernel entry point called from the UEFI loader.
///
/// # Safety assumptions
/// - `boot_abi_ptr` points to a valid `BootAbi`
/// - Memory is identity-mapped at entry
/// - Interrupts may be enabled by firmware
#[unsafe(no_mangle)]
pub extern "C" fn kernel_main(boot_abi_ptr: *const BootAbi) -> ! {
    // Disable interrupts before doing anything else
    unsafe {
        core::arch::asm!("cli");
    }

    match kernel_run(boot_abi_ptr) {
        Ok(()) => halt(), // This should not actually be possible, as the kernel never exits
        Err(e) => fatal(e), // Fatal error; halt the system
    }
}

fn halt() -> ! {
    crate::println!("System halted.");
    loop {
        core::hint::spin_loop();
    }
}

fn fatal(e: KernelError) -> ! {
    crate::println!("Fatal kernel error: {:?}", e);
    halt();
}

fn kernel_run(boot_abi_ptr: *const BootAbi) -> Result<(), KernelError> {
    // SAFETY: caller (the UEFI loader) must ensure the pointer is valid at entry
    let boot_abi = unsafe { &*boot_abi_ptr };

    boot::validate_boot_abi(boot_abi)?;

    let framebuffer = boot_abi.framebuffer;
    let memory_map = boot_abi.memory_map;

    options::init(boot_abi.options);

    // Clear the framebuffer to assert control
    framebuffer::clear_framebuffer(&framebuffer).expect("framebuffer clear failed");

    if let Ok(storage) = init::bootstrap_console_storage(&memory_map) {
        let _ = console::init(framebuffer, framebuffer::FramebufferColor::WHITE, storage);
    }

    time::init_tsc_monotonic(boot_abi.tsc_frequency_hz);

    crate::println!("Oxide kernel starting...");
    crate::println!("Kernel: Entering epoch 1: Spark.");

    let (freq, unit) = human_readable_hz(boot_abi.tsc_frequency_hz);
    crate::diagln!("Detected CPU frequency: {:.2} {}", freq, unit);

    init::initialize(&memory_map, &framebuffer)?;

    crate::diagln!("Memory subsystem init complete.");

    crate::println!("Kernel: Entering epoch 2: Foundation.");

    Ok(())
}

#[cfg(all(feature = "standalone", not(feature = "dep-loader")))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {
        core::hint::spin_loop();
    }
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum KernelError {
    BootValidation(boot::BootValidationError),
    MemoryInit(MemoryInitError),
    FrameAlloc(FrameAllocError),
}

impl From<boot::BootValidationError> for KernelError {
    fn from(err: boot::BootValidationError) -> Self {
        KernelError::BootValidation(err)
    }
}

impl From<MemoryInitError> for KernelError {
    fn from(err: MemoryInitError) -> Self {
        KernelError::MemoryInit(err)
    }
}

impl From<FrameAllocError> for KernelError {
    fn from(err: FrameAllocError) -> Self {
        KernelError::FrameAlloc(err)
    }
}

fn human_readable_hz(freq_hz: u64) -> (f64, &'static str) {
    const KHZ: f64 = 1_000.0;
    const MHZ: f64 = 1_000_000.0;
    const GHZ: f64 = 1_000_000_000.0;

    let freq = freq_hz as f64;
    if freq >= GHZ {
        (freq / GHZ, "GHz")
    } else if freq >= MHZ {
        (freq / MHZ, "MHz")
    } else if freq >= KHZ {
        (freq / KHZ, "kHz")
    } else {
        (freq, "Hz")
    }
}

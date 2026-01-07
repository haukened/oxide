#![no_std]
#![no_main]

use oxide_abi::BootAbi;

use crate::{errors::KernelError, memory::init};

mod errors;
mod framebuffer;
mod memory;
mod options;

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
    loop {
        core::hint::spin_loop();
    }
}

fn fatal(e: KernelError) -> ! {
    crate::fb_println!("Fatal kernel error: {:?}", e);
    halt();
}

fn kernel_run(boot_abi_ptr: *const BootAbi) -> Result<(), KernelError> {
    // SAFETY: caller (the UEFI loader) must ensure the pointer is valid at entry
    let boot_abi = unsafe { &*boot_abi_ptr };
    let framebuffer = boot_abi.framebuffer;
    let memory_map = boot_abi.memory_map;

    options::init(boot_abi.options);

    // Clear the framebuffer to assert control
    framebuffer::clear_framebuffer(&framebuffer).expect("framebuffer clear failed");

    if unsafe { framebuffer::init_boot_console(framebuffer, framebuffer::FramebufferColor::WHITE) }
        .is_err()
    {
        // No usable console; subsequent fb_* macros become no-ops.
    }

    crate::fb_diagln!("Oxide kernel starting...");

    init::initialize(&memory_map, &framebuffer)?;

    crate::fb_diagln!("Memory subsystem initialized.");

    Ok(())
}

#[cfg(feature = "standalone")]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {
        core::hint::spin_loop();
    }
}

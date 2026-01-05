#![no_std]
#![no_main]

use oxide_abi::BootAbi;

mod framebuffer;

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

    // SAFETY: caller (the UEFI loader) must ensure the pointer is valid at entry
    let boot_abi = unsafe { &*boot_abi_ptr };

    // Clear the framebuffer to assert control
    framebuffer::clear_framebuffer(&boot_abi.framebuffer).expect("framebuffer clear failed");

    // Draw a boot marker to indicate that the kernel has started
    framebuffer::draw_boot_marker(&boot_abi.framebuffer).expect("framebuffer marker failed");

    loop {
        core::hint::spin_loop();
    }
}

#[cfg(feature = "standalone")]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {
        core::hint::spin_loop();
    }
}

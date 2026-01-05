#![no_std]
#![no_main]

use core::panic::PanicInfo;
use oxide_abi::BootAbi;

/// Kernel entry point called from the UEFI loader.
#[unsafe(no_mangle)]
pub extern "C" fn kernel_main(boot_abi_ptr: *const BootAbi) -> ! {
    // SAFETY: caller (the UEFI loader) must ensure the pointer is valid at entry
    let _boot_abi = unsafe { &*boot_abi_ptr };

    loop {
        core::hint::spin_loop();
    }
}

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {
        core::hint::spin_loop();
    }
}

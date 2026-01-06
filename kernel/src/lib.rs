#![no_std]
#![no_main]

use oxide_abi::BootAbi;

use crate::memory::init;

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

    if let Err(err) = init::initialize(&memory_map, &framebuffer) {
        log_memory_failure(err);
        loop {
            core::hint::spin_loop();
        }
    }

    crate::fb_diagln!("Memory subsystem initialized.");

    loop {
        core::hint::spin_loop();
    }
}

fn log_memory_failure(err: init::MemoryInitError) {
    match err {
        init::MemoryInitError::NoUsableMemory => {
            crate::fb_println!("No usable memory frames found.")
        }
        init::MemoryInitError::EmptyMemoryMap => crate::fb_println!("Memory map is empty."),
        init::MemoryInitError::OutOfFrames => {
            crate::fb_println!("Out of frames while copying memory map.")
        }
        init::MemoryInitError::NonContiguous => {
            crate::fb_println!("Failed to allocate contiguous frames for memory map copy.")
        }
        init::MemoryInitError::TooLarge => crate::fb_println!("Memory map too large to copy."),
        init::MemoryInitError::StackDescriptorMissing(addr) => {
            crate::fb_println!("No memory descriptor covers stack address {:#x}", addr)
        }
        init::MemoryInitError::StackRangeOverflow(typ) => {
            crate::fb_println!("Stack descriptor range overflow for type {:#x}", typ)
        }
        init::MemoryInitError::IdentityRangeOverflow { start, end } => {
            crate::fb_println!(
                "Identity range staging overflowed while adding [{:#x}, {:#x}]",
                start,
                end
            )
        }
        init::MemoryInitError::Paging(err) => {
            crate::fb_println!("install_identity_paging failed: {:?}", err)
        }
    }
}

#[cfg(feature = "standalone")]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {
        core::hint::spin_loop();
    }
}

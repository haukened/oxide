#![no_std]
#![no_main]

use oxide_abi::BootAbi;

use crate::{
    framebuffer::BootStage,
    memory::{
        frame::{self, FrameAllocator, UsableFrameIter},
        map::MemoryMapIter,
    },
};

mod framebuffer;
mod memory;

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
    let fb = &boot_abi.framebuffer;

    // Clear the framebuffer to assert control
    framebuffer::clear_framebuffer(fb).expect("framebuffer clear failed");

    // Draw a boot marker to indicate that the kernel has started
    framebuffer::draw_boot_stage(fb, BootStage::EnteredKernel);

    // Ensure we can parse the memory map
    let mem_map = &boot_abi.memory_map;
    for desc in MemoryMapIter::new(mem_map) {
        // For now, just a dummy operation to use the descriptor
        let _ = desc.physical_start;
    }
    framebuffer::draw_boot_stage(fb, BootStage::ParsedMemoryMap);

    // Ensure we can find usable memory frames
    if UsableFrameIter::new(mem_map).next().is_some() {
        framebuffer::draw_boot_stage(fb, BootStage::FoundUsableMemory);
    } else {
        framebuffer::panic_screen(fb);
    }

    // Ensure we can allocate a frame
    let mut alloc = FrameAllocator::new(mem_map);
    if let Some(_frame) = alloc.allocate_frame() {
        framebuffer::draw_boot_stage(fb, BootStage::FrameAllocated);
    } else {
        framebuffer::panic_screen(fb);
    }

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

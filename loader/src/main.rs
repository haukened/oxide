#![no_std]
#![no_main]

use core::time::Duration;
use uefi::prelude::*;

mod framebuffer;

/// UEFI application entry point
#[entry]
fn efi_main() -> Status {
    match run() {
        Ok(()) => Status::SUCCESS,
        Err(e) => e.status(),
    }
}

/// Main application logic, returns Ok on success or Err on failure
/// Get all necessary UEFI services and prepare to launch the kernel
fn run() -> uefi::Result<()> {
    uefi::helpers::init().unwrap();

    // Clear UEFI text console for clean logs
    uefi::system::with_stdout(|stdout| {
        stdout.clear().unwrap();
    });

    // Declare that we are here and alive
    uefi::println!("Oxide UEFI loader starting...");

    // Get the framebuffer info for kernel handoff
    let fb_info = framebuffer::get_framebuffer_info()?;
    print_framebuffer_info(&fb_info);

    timed_reboot_for_testing(10);

    // - memory map / exit boot services
    // - build BootInfo
    // - jump to kernel

    //loop {
    //    core::hint::spin_loop();
    //}
}

/// Reboot the system after a countdown, for testing purposes
/// essential since we don't have power management yet
fn timed_reboot_for_testing(seconds: u64) -> ! {
    let dsec = Duration::from_secs(1);

    uefi::println!("Scheduling test reboot in {} seconds", seconds);
    for i in (1..=seconds).rev() {
        uefi::print!("\rRebooting in {:>2} ", i);
        uefi::boot::stall(dsec);
    }
    uefi::println!("\rRebooting in  0 ");
    uefi::println!("NOW!");

    // ask for reboot
    uefi::runtime::reset(uefi::runtime::ResetType(1), uefi::Status::SUCCESS, None);
}

/// Print framebuffer information to UEFI console
/// Just for debug, and to keep noise out of the main logic
fn print_framebuffer_info(fb_info: &framebuffer::FramebufferInfo) {
    uefi::println!("Framebuffer Info:");
    uefi::println!("  Base Address: {:p}", fb_info.base_address);
    uefi::println!("  Buffer Size: {} bytes", fb_info.buffer_size);
    uefi::println!("  Resolution: {}x{}", fb_info.width, fb_info.height);
    uefi::println!("  Stride: {}", fb_info.stride);
    uefi::println!("  Pixel Format: {:?}", fb_info.pixel_format);
}

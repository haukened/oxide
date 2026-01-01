#![no_std]
#![no_main]

use core::time::Duration;
use uefi::prelude::*;

mod firmware;
mod flags;
mod framebuffer;
mod writer;

const REBOOT_DELAY_SECS: u64 = 10;

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
        if let Err(err) = stdout.clear() {
            uefi::print!("stdout.clear() failed: {:?}\n", err);
        }
    });

    uefi::println!("Oxide UEFI loader starting...");

    let fw_info = firmware::get_info();
    print_firmware_info(&fw_info);

    let fb_info = framebuffer::get_framebuffer_info()?;
    print_framebuffer_info(&fb_info);

    let boot_flags = flags::get_boot_flags();
    print_boot_flags(&boot_flags);

    timed_reboot_for_testing(REBOOT_DELAY_SECS);

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

fn print_firmware_info(info: &firmware::FirmwareInfo) {
    uefi::println!(
        "Firmware Info:\n  Vendor: {}{}\n  Revision: {}",
        info.vendor_str(),
        if info.vendor_was_truncated() {
            " (truncated)"
        } else {
            ""
        },
        info.revision
    );
}

fn print_boot_flags(flags: &flags::BootFlags) {
    uefi::println!("Boot Flags:");
    uefi::println!("  debug: {}", flags.debug);
    uefi::println!("  quiet: {}", flags.quiet);
}

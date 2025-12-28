#![no_std]
#![no_main]

use uefi::prelude::*;
mod framebuffer;
mod logger;

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

    logger::writeln("oxide-loader: starting");

    let fb_info = framebuffer::init()?;

    logger::set_framebuffer_sink(fb_info);

    logger::writeln("oxide-loader: framebuffer ready");

    let _memory_map = unsafe { uefi::boot::exit_boot_services(None) };

    logger::writeln("oxide-loader: memory map captured");
    // - build BootInfo
    // - jump to kernel

    loop {
        core::hint::spin_loop();
    }
}

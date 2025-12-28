#![no_std]
#![no_main]

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

    uefi::println!("oxide-loader: starting");

    let _framebuffer_info = framebuffer::init()?;

    uefi::println!("oxide-loader: framebuffer ready");

    // Next steps will go here:
    // - capture memory map

    // - build BootInfo
    // - ExitBootServices
    // - jump to kernel

    loop {
        core::hint::spin_loop();
    }
}

#![no_std]
#![no_main]

use uefi::prelude::*;

mod abi;
mod firmware;
mod flags;
mod framebuffer;
mod writer;

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
    uefi::helpers::init()?;

    // Clear UEFI text console for clean logs
    uefi::system::with_stdout(|stdout| {
        if let Err(err) = stdout.clear() {
            uefi::print!("stdout.clear() failed: {:?}\n", err);
        }
    });

    uefi::println!("Oxide UEFI loader starting...");

    // pre-allocate memory for the ABI structures we need to build, before exit boot services

    let _fw_info = firmware::get_info();

    let _fb_info = framebuffer::get_framebuffer_info()?;

    let _boot_flags = flags::get_boot_flags();

    // Here we exit boot services, so we lose all UEFI services after this point
    let _mem_map = unsafe { uefi::boot::exit_boot_services(None) };

    // - build BootAbi
    // - jump to kernel

    loop {
        core::hint::spin_loop();
    }
}

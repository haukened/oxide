#![no_std]
#![no_main]

use oxide_kernel::kernel_main;
use uefi::prelude::*;

mod abi;
mod firmware;
mod framebuffer;
mod options;
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
    let boot_abi = abi::alloc_abi_struct()?;
    uefi::println!("Allocated BootAbi at {:p}", boot_abi);

    let fw_info = firmware::get_info();

    let fb_info = framebuffer::get_framebuffer_info()?;
    uefi::println!(
        "Framebuffer: \n  addr={:#?}\n  size={} bytes\n  {}x{}, {} bpp",
        fb_info.base_address,
        fb_info.buffer_size,
        fb_info.width,
        fb_info.height,
        fb_info.pixels_per_scanline * 8 / fb_info.width
    );

    let boot_options = options::get_boot_options();

    // Here we exit boot services, so we lose all UEFI services after this point
    let mem_map = unsafe { uefi::boot::exit_boot_services(None) };

    // - build BootAbi
    abi::build_boot_abi_from_ptr(boot_abi, fw_info, fb_info, boot_options, mem_map);

    // - jump to kernel
    kernel_main(boot_abi as *const _);
}

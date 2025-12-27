#![no_std]
#![no_main]

use uefi::prelude::*;
use uefi::boot;
use uefi::proto::console::gop::{GraphicsOutput, PixelFormat};

#[entry]
fn efi_main() -> Status {
    uefi::helpers::init().unwrap();

    // Clear UEFI text console for clean logs
    uefi::system::with_stdout(|stdout| {
        stdout.clear().unwrap();
    });

    uefi::println!("oxide-loader: starting");

    if let Err(e) = init_framebuffer() {
        uefi::println!("oxide-loader: framebuffer init failed: {:?}", e.status());
        return e.status();
    }

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

fn init_framebuffer() -> uefi::Result {
    // Locate GOP
    let gop_handle = boot::get_handle_for_protocol::<GraphicsOutput>()?;
    let mut gop = boot::open_protocol_exclusive::<GraphicsOutput>(gop_handle)?;

    let mode = gop.current_mode_info();
    let (width, height) = mode.resolution();
    let stride = mode.stride();
    let pixel_format = mode.pixel_format();

    let mut fb = gop.frame_buffer();
    let fb_ptr = fb.as_mut_ptr();
    let fb_size = fb.size();

    let bytes_per_pixel = match pixel_format {
        PixelFormat::Bgr | PixelFormat::Rgb => 4,
        _ => return Err(uefi::Status::UNSUPPORTED.into()),
    };

    // Log what we discovered (this matters more than drawing)
    uefi::println!("oxide-loader: framebuffer:");
    uefi::println!("  resolution: {}x{}", width, height);
    uefi::println!("  stride: {}", stride);
    uefi::println!("  format: {:?}", pixel_format);
    uefi::println!("  size: {} bytes", fb_size);

    // Clear framebuffer to black to establish ownership
    for y in 0..height {
        for x in 0..width {
            let offset = (y * stride + x) * bytes_per_pixel;
            unsafe {
                let p = fb_ptr.add(offset);
                // black
                *p.add(0) = 0x00;
                *p.add(1) = 0x00;
                *p.add(2) = 0x00;
                *p.add(3) = 0x00;
            }
        }
    }

    Ok(())
}
#![no_std]
#![no_main]

use uefi::prelude::*;
use uefi::boot;
use uefi::proto::console::gop::{GraphicsOutput,PixelFormat};

#[entry]
fn efi_main() -> Status {
    // init allocator, logger, panic handler, and console
    uefi::helpers::init().unwrap();

    // clear the screen
    uefi::system::with_stdout(|stdout| {
        stdout.clear().unwrap();
    });

    // draw something persistent. fill screen with GOP
    if let Err(e) = draw_direct_framebuffer() {
        // if graphics fails, last-resort try to say something on console
        uefi::println!("Failed to fill screen with GOP: {:?}", e.status());
        return e.status();
    }

    loop {
        core::hint::spin_loop();
    }
}

fn draw_direct_framebuffer() -> uefi::Result {
    // open the Graphics Output Protocol
    let gop_handle = boot::get_handle_for_protocol::<GraphicsOutput>()?;
    let mut gop = boot::open_protocol_exclusive::<GraphicsOutput>(gop_handle)?;

    let mode = gop.current_mode_info();
    let (width, height) = mode.resolution();
    let stride = mode.stride();
    let pixel_format = mode.pixel_format();

    let mut fb = gop.frame_buffer();
    let fb_ptr = fb.as_mut_ptr();

    // we assume 32bpp (true for most UEFI implementations)
    let bytes_per_pixel = match pixel_format {
        PixelFormat::Bgr => 4,
        PixelFormat::Rgb => 4,
        _ => {
            return Err(uefi::Status::UNSUPPORTED.into());
        }
    };

    // draw a simple rectangle so we know orientation and stride are correct
    let rect_w = width / 4;
    let rect_h = height / 4;
    let start_x = (width - rect_w) / 2;
    let start_y = (height - rect_h) / 2;

    for y in 0..rect_h {
        for x in 0..rect_w {
            let pixel_x = start_x + x;
            let pixel_y = start_y + y;
            let offset = (pixel_y * stride + pixel_x) * bytes_per_pixel;
            unsafe {
                let pixel_ptr = fb_ptr.add(offset);
                // Fill with a solid color (e.g., blue)
                match pixel_format {
                    PixelFormat::Bgr => {
                        *pixel_ptr.add(0) = 0xFF; // Blue
                        *pixel_ptr.add(1) = 0x00; // Green
                        *pixel_ptr.add(2) = 0x00; // Red
                        *pixel_ptr.add(3) = 0x00; 
                    }
                    PixelFormat::Rgb => {
                        *pixel_ptr.add(0) = 0x00; // Red
                        *pixel_ptr.add(1) = 0x00; // Green
                        *pixel_ptr.add(2) = 0xFF; // Blue
                        *pixel_ptr.add(3) = 0x00; 
                    }
                    _ => {}
                }
            }
        }
    }
    Ok(())
}
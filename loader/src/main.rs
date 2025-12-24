#![no_std]
#![no_main]

use uefi::prelude::*;
use uefi::boot;
use uefi::proto::console::gop::{GraphicsOutput, PixelFormat};

#[entry]
fn efi_main() -> Status {
    uefi::helpers::init().unwrap();

    // Last use of UEFI text console
    uefi::system::with_stdout(|stdout| {
        stdout.clear().unwrap();
    });

    if let Err(e) = draw_splash() {
        uefi::println!("Framebuffer draw failed: {:?}", e.status());
        return e.status();
    }

    loop {
        core::hint::spin_loop();
    }
}

fn draw_splash() -> uefi::Result {
    let gop_handle = boot::get_handle_for_protocol::<GraphicsOutput>()?;
    let mut gop = boot::open_protocol_exclusive::<GraphicsOutput>(gop_handle)?;

    let mode = gop.current_mode_info();
    let (width, height) = mode.resolution();
    let stride = mode.stride();
    let pixel_format = mode.pixel_format();

    let mut fb = gop.frame_buffer();
    let fb_ptr = fb.as_mut_ptr();

    let bpp = match pixel_format {
        PixelFormat::Bgr | PixelFormat::Rgb => 4,
        _ => return Err(uefi::Status::UNSUPPORTED.into()),
    };

    // --- helpers ------------------------------------------------------------

    let fill_rect = |x: usize, y: usize, w: usize, h: usize, r: u8, g: u8, b: u8| {
        for dy in 0..h {
            for dx in 0..w {
                let px = x + dx;
                let py = y + dy;
                if px >= width || py >= height {
                    continue;
                }

                let offset = (py * stride + px) * bpp;
                unsafe {
                    let p = fb_ptr.add(offset);
                    match pixel_format {
                        PixelFormat::Bgr => {
                            *p.add(0) = b;
                            *p.add(1) = g;
                            *p.add(2) = r;
                            *p.add(3) = 0;
                        }
                        PixelFormat::Rgb => {
                            *p.add(0) = r;
                            *p.add(1) = g;
                            *p.add(2) = b;
                            *p.add(3) = 0;
                        }
                        _ => {}
                    }
                }
            }
        }
    };

    // --- background ---------------------------------------------------------

    // Dark gray background
    fill_rect(0, 0, width, height, 0x20, 0x20, 0x20);

    // --- OXIDE logo ---------------------------------------------------------

    let letter_w = width / 14;
    let letter_h = height / 6;
    let thickness = letter_w / 5;
    let spacing = letter_w / 3;

    let total_w = letter_w * 5 + spacing * 4;
    let start_x = (width - total_w) / 2;
    let start_y = (height - letter_h) / 2;

    let rust_color = (0xCE, 0x42, 0x2B);

    let mut x = start_x;

    // O
    fill_rect(x, start_y, letter_w, thickness, rust_color.0, rust_color.1, rust_color.2);
    fill_rect(x, start_y + letter_h - thickness, letter_w, thickness, rust_color.0, rust_color.1, rust_color.2);
    fill_rect(x, start_y, thickness, letter_h, rust_color.0, rust_color.1, rust_color.2);
    fill_rect(x + letter_w - thickness, start_y, thickness, letter_h, rust_color.0, rust_color.1, rust_color.2);
    x += letter_w + spacing;

    // X (blocky diagonals, full height)
    let steps = letter_h / thickness + 1;

    for i in 0..steps {
        let y = start_y + (i * letter_h) / steps;

        // left-to-right diagonal
        let x1 = x + (i * letter_w) / steps;
        fill_rect(x1, y, thickness, thickness, rust_color.0, rust_color.1, rust_color.2);

        // right-to-left diagonal
        let x2 = x + letter_w - thickness - (i * letter_w) / steps;
        fill_rect(x2, y, thickness, thickness, rust_color.0, rust_color.1, rust_color.2);
    }

    x += letter_w + spacing;

    // I
    fill_rect(x + letter_w / 2 - thickness / 2, start_y, thickness, letter_h, rust_color.0, rust_color.1, rust_color.2);
    x += letter_w + spacing;

    // D
    fill_rect(x, start_y, thickness, letter_h, rust_color.0, rust_color.1, rust_color.2);
    fill_rect(x, start_y, letter_w - thickness, thickness, rust_color.0, rust_color.1, rust_color.2);
    fill_rect(x, start_y + letter_h - thickness, letter_w - thickness, thickness, rust_color.0, rust_color.1, rust_color.2);
    fill_rect(x + letter_w - thickness, start_y + thickness, thickness, letter_h - 2 * thickness, rust_color.0, rust_color.1, rust_color.2);
    x += letter_w + spacing;

    // E
    fill_rect(x, start_y, thickness, letter_h, rust_color.0, rust_color.1, rust_color.2);
    fill_rect(x, start_y, letter_w, thickness, rust_color.0, rust_color.1, rust_color.2);
    fill_rect(x, start_y + letter_h / 2 - thickness / 2, letter_w * 3 / 4, thickness, rust_color.0, rust_color.1, rust_color.2);
    fill_rect(x, start_y + letter_h - thickness, letter_w, thickness, rust_color.0, rust_color.1, rust_color.2);

    Ok(())
}
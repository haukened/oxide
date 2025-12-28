/// Initializes the framebuffer by acquiring the UEFI Graphics Output Protocol, logging the
/// discovered mode configuration, clearing the backing memory to black, and returning the
/// resolved framebuffer metadata for later use.
use spleen_font::{FONT_8X16, PSF2Font};
use uefi::boot;
use uefi::proto::console::gop::{GraphicsOutput, PixelFormat};

/// Statically compiled font dimensions, because that's what we installed with spleen-font's feature flags.
const FONT_W: usize = 8;
const FONT_H: usize = 16;

/// Metadata describing a bootloader-provided framebuffer, including its memory
/// location, dimensions, pixel stride, and raw pixel representation.
pub struct FramebufferInfo {
    /// Physical base address of the framebuffer (identity-mapped during boot)
    pub base: *mut u8,
    /// Size of the framebuffer in bytes
    pub size: usize,
    /// Width in pixels
    pub width: usize,
    /// Height in pixels
    pub height: usize,
    /// Number of pixels per row
    pub stride: usize,
    /// Pixel layout (facts only; no semantic meaning implied)
    pub pixel_format: FramebufferPixelFormat,
}

/// Project-owned framebuffer pixel layouts.
/// The trailing 8 bits are padding/unused (X), not alpha.
/// Prevents leaking UEFI-specific types outside this module.
pub enum FramebufferPixelFormat {
    XRGB8888, // X | R | G | B (8 bits each)
    BGRX8888, // B | G | R | X (8 bits each)
}

static mut CURSOR_X: usize = 0;
static mut CURSOR_Y: usize = 0;
static mut FONT: Option<PSF2Font<'static>> = None;

/// Initialize the framebuffer by locating the UEFI Graphics Output Protocol (GOP),
/// configuring it if necessary, and returning metadata about the framebuffer.
/// Clears the framebuffer to black to establish ownership.
pub fn init() -> uefi::Result<FramebufferInfo> {
    // Locate GOP
    let gop_handle = boot::get_handle_for_protocol::<GraphicsOutput>()?;
    let mut gop = boot::open_protocol_exclusive::<GraphicsOutput>(gop_handle)?;

    let mode = gop.current_mode_info();
    let (width, height) = mode.resolution();
    let stride = mode.stride();
    let uefi_pixel_format = mode.pixel_format();

    let mut fb = gop.frame_buffer();
    let fb_ptr = fb.as_mut_ptr();
    let fb_size = fb.size();

    let pixel_format = map_pixel_format(uefi_pixel_format).ok_or_else(|| {
        uefi::println!(
            "oxide-loader: unsupported pixel format: {:?}",
            uefi_pixel_format
        );
        uefi::Error::from(uefi::Status::UNSUPPORTED)
    })?;
    let bytes_per_pixel = 4; // UEFI GOP uses 32 bits per pixel in all supported formats

    // Log what we discovered (this matters more than drawing)
    uefi::println!("oxide-loader: framebuffer:");
    uefi::println!("  resolution: {}x{}", width, height);
    uefi::println!("  stride: {}", stride);
    uefi::println!("  format: {:?}", uefi_pixel_format);
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

    unsafe {
        FONT = Some(PSF2Font::new(FONT_8X16).unwrap());
    }

    Ok(FramebufferInfo {
        base: fb_ptr,
        size: fb_size,
        width,
        height,
        stride,
        pixel_format,
    })
}

pub fn write_line(fb: &FramebufferInfo, message: &str) {
    let max_cols = fb.width / FONT_W;
    let max_rows = fb.height / FONT_H;

    unsafe {
        for byte in message.bytes() {
            match byte {
                b'\n' => {
                    CURSOR_X = 0;
                    CURSOR_Y += 1;
                }
                ch => {
                    if CURSOR_X < max_cols && CURSOR_Y < max_rows {
                        draw_char(fb, ch, CURSOR_X, CURSOR_Y);
                    }
                    CURSOR_X += 1;
                }
            }

            if CURSOR_X >= max_cols {
                CURSOR_X = 0;
                CURSOR_Y += 1;
            }

            if CURSOR_Y >= max_rows {
                // Clamp at bottom for now (no scrolling in loader)
                CURSOR_Y = max_rows - 1;
            }
        }

        // Move to next line after message
        CURSOR_X = 0;
        CURSOR_Y += 1;
    }
}

fn map_pixel_format(fmt: PixelFormat) -> Option<FramebufferPixelFormat> {
    match fmt {
        PixelFormat::Rgb => Some(FramebufferPixelFormat::XRGB8888),
        PixelFormat::Bgr => Some(FramebufferPixelFormat::BGRX8888),
        _ => None,
    }
}

fn put_pixel(fb: &FramebufferInfo, x: usize, y: usize, color: u32) {
    let bytes_per_pixel = 4;
    let offset = (y * fb.stride + x) * bytes_per_pixel;

    unsafe {
        let p = fb.base.add(offset);
        match fb.pixel_format {
            FramebufferPixelFormat::XRGB8888 => {
                // X R G B
                *p.add(0) = 0x00;
                *p.add(1) = ((color >> 16) & 0xFF) as u8;
                *p.add(2) = ((color >> 8) & 0xFF) as u8;
                *p.add(3) = (color & 0xFF) as u8;
            }
            FramebufferPixelFormat::BGRX8888 => {
                // B G R X
                *p.add(0) = (color & 0xFF) as u8;
                *p.add(1) = ((color >> 8) & 0xFF) as u8;
                *p.add(2) = ((color >> 16) & 0xFF) as u8;
                *p.add(3) = 0x00;
            }
        }
    }
}

fn draw_char(fb: &FramebufferInfo, ch: u8, cx: usize, cy: usize) {
    let px = cx * FONT_W;
    let py = cy * FONT_H;

    let fg = 0x00FFFFFF; // white
    let bg = 0x00000000; // black

    let buf = [ch];

    let glyph = unsafe { FONT.as_mut().and_then(|font| font.glyph_for_utf8(&buf)) };

    if let Some(glyph) = glyph {
        for (row_y, row) in glyph.enumerate() {
            for (col_x, on) in row.enumerate() {
                let color = if on { fg } else { bg };
                put_pixel(fb, px + col_x, py + row_y, color);
            }
        }
    }
}

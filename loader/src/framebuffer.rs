/// Initializes the framebuffer by acquiring the UEFI Graphics Output Protocol, logging the
/// discovered mode configuration, clearing the backing memory to black, and returning the
/// resolved framebuffer metadata for later use.
use uefi::boot;
use uefi::proto::console::gop::{GraphicsOutput, PixelFormat};

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

/// Map UEFI pixel format to project-owned pixel format.
/// Returns None if the format is unsupported.
fn map_pixel_format(fmt: PixelFormat) -> Option<FramebufferPixelFormat> {
    match fmt {
        PixelFormat::Rgb => Some(FramebufferPixelFormat::XRGB8888),
        PixelFormat::Bgr => Some(FramebufferPixelFormat::BGRX8888),
        _ => None,
    }
}

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

    Ok(FramebufferInfo {
        base: fb_ptr,
        size: fb_size,
        width,
        height,
        stride,
        pixel_format,
    })
}

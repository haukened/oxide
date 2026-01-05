use oxide_abi::Framebuffer;
use uefi::{
    Status,
    boot::{self, OpenProtocolAttributes, OpenProtocolParams},
    proto::console::gop::{GraphicsOutput, PixelFormat},
};

/// Framebuffer information required for kernel handoff.
#[derive(Clone, Copy, Debug)]
pub struct FramebufferInfo {
    /// raw pointer to the framebuffer base address
    /// Must be identity-mapped at Virtual Address == Physical Address
    /// the loader must not change GOP mode after getting this from the UEFI.
    pub base_address: *mut u8,
    pub buffer_size: usize,
    pub width: usize,
    pub height: usize,
    pub pixels_per_scanline: usize,
    pub pixel_format: FramebufferPixelFormat,
}

/// Convert to ABI Framebuffer representation.
impl From<FramebufferInfo> for Framebuffer {
    fn from(fb: FramebufferInfo) -> Self {
        debug_assert!(fb.width <= u32::MAX as usize);
        debug_assert!(fb.height <= u32::MAX as usize);
        debug_assert!(fb.pixels_per_scanline <= u32::MAX as usize);
        Framebuffer {
            base_address: fb.base_address as u64,
            buffer_size: fb.buffer_size as u64,
            width: fb.width as u32,
            height: fb.height as u32,
            // Pixels per scanline
            pixels_per_scanline: fb.pixels_per_scanline as u32,
            pixel_format: match fb.pixel_format {
                FramebufferPixelFormat::Rgb => oxide_abi::PixelFormat::Rgb,
                FramebufferPixelFormat::Bgr => oxide_abi::PixelFormat::Bgr,
            },
        }
    }
}

/// Project-owned pixel format wrapper so UEFI types stay contained.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FramebufferPixelFormat {
    Rgb,
    Bgr,
}

/// Acquire framebuffer metadata without taking exclusive GOP ownership.
pub fn get_framebuffer_info() -> uefi::Result<FramebufferInfo> {
    // first we need to get a non-exclusive access to the Graphics Output Protocol
    // if we had exclusive access, we wouldn't be able to use UEFI text console later
    let gop_handle = uefi::boot::get_handle_for_protocol::<GraphicsOutput>()?;
    let mut gop = unsafe {
        boot::open_protocol::<GraphicsOutput>(
            OpenProtocolParams {
                handle: gop_handle,
                agent: uefi::boot::image_handle(),
                controller: None,
            },
            OpenProtocolAttributes::GetProtocol,
        )?
    };
    let mut fb = gop.frame_buffer();

    let base_address = fb.as_mut_ptr();
    let buffer_size = fb.size();
    let info = gop.current_mode_info();
    let (width, height) = info.resolution();
    let pixels_per_scanline = info.stride();
    let pixel_format = map_pixel_format(info.pixel_format())?;
    Ok(FramebufferInfo {
        base_address,
        buffer_size,
        width,
        height,
        pixels_per_scanline,
        pixel_format,
    })
}

fn map_pixel_format(format: PixelFormat) -> uefi::Result<FramebufferPixelFormat> {
    match format {
        PixelFormat::Rgb => Ok(FramebufferPixelFormat::Rgb),
        PixelFormat::Bgr => Ok(FramebufferPixelFormat::Bgr),
        _ => Err(Status::UNSUPPORTED.into()),
    }
}

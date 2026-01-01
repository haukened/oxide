use uefi::{
    Status,
    boot::{self, OpenProtocolAttributes, OpenProtocolParams},
    proto::console::gop::{GraphicsOutput, PixelFormat},
};

/// Framebuffer information required for kernel handoff.
#[derive(Clone, Copy, Debug)]
pub struct FramebufferInfo {
    /// Raw pointer to the start of the linear framebuffer; valid while the current GOP mode stays active.
    pub base_address: *mut u8,
    pub buffer_size: usize,
    pub width: usize,
    pub height: usize,
    pub stride: usize,
    pub pixel_format: FramebufferPixelFormat,
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
    let stride = info.stride();
    let pixel_format = map_pixel_format(info.pixel_format())?;
    Ok(FramebufferInfo {
        base_address,
        buffer_size,
        width,
        height,
        stride,
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

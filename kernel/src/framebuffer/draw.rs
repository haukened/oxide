use core::{cmp::min, ptr};
use oxide_abi::{Framebuffer, PixelFormat};

use super::{FONT_HEIGHT, FONT_WIDTH, glyph_for};

/// Simple RGB color helper for framebuffer drawing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FramebufferColor {
    r: u8,
    g: u8,
    b: u8,
}

impl FramebufferColor {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    pub const BLACK: Self = Self::new(0x00, 0x00, 0x00);
    pub const WHITE: Self = Self::new(0xFF, 0xFF, 0xFF);

    pub const fn components(self) -> (u8, u8, u8) {
        (self.r, self.g, self.b)
    }
}

/// Minimal viewport over the firmware-provided framebuffer.
#[derive(Clone, Copy, Debug)]
pub struct FramebufferSurface {
    pub base_ptr: *mut u32,
    pub pitch: usize,
    pub width: usize,
    pub height: usize,
    pub pixel_format: PixelFormat,
}

impl FramebufferSurface {
    pub fn new(fb: Framebuffer) -> Result<Self, ()> {
        Self {
            base_ptr: fb.base_address as *mut u32,
            pitch: fb.pixels_per_scanline as usize,
            width: fb.width as usize,
            height: fb.height as usize,
            pixel_format: fb.pixel_format,
        }
        .validate()
    }

    pub fn empty() -> Self {
        Self {
            base_ptr: ptr::null_mut(),
            pitch: 0,
            width: 0,
            height: 0,
            pixel_format: PixelFormat::Rgb,
        }
    }

    pub fn validate(self) -> Result<Self, ()> {
        if self.base_ptr.is_null() || self.pitch == 0 || self.width == 0 || self.height == 0 {
            return Err(());
        }
        Ok(self)
    }
}

/// Clear the framebuffer to black.
///
/// This function is defensive against malformed firmware data.
/// If the framebuffer geometry does not fit within the reported buffer,
/// it returns `Err(())` and performs no writes.
pub fn clear_black(fb: &Framebuffer) -> Result<(), ()> {
    let surface = FramebufferSurface::new(*fb)?;

    let bytes_per_pixel = core::mem::size_of::<u32>();
    let max_pixels = (fb.buffer_size as usize) / bytes_per_pixel;
    if max_pixels == 0 {
        return Err(());
    }

    // Limit clearing to what actually fits in the buffer
    let max_rows = max_pixels / surface.pitch;
    let clear_height = min(surface.height, max_rows);
    if clear_height == 0 {
        return Err(());
    }

    let row_width = min(surface.width, surface.pitch);
    if row_width == 0 {
        return Err(());
    }

    let color = encode_pixel(surface.pixel_format, FramebufferColor::BLACK);

    unsafe {
        for y in 0..clear_height {
            let row_ptr = surface.base_ptr.add(y * surface.pitch);
            for x in 0..row_width {
                row_ptr.add(x).write_volatile(color);
            }
        }
    }

    Ok(())
}

/// Fill a rectangular region with the provided color.
pub fn fill_rect(
    surface: FramebufferSurface,
    origin_x: usize,
    origin_y: usize,
    width: usize,
    height: usize,
    color: FramebufferColor,
) -> Result<(), ()> {
    let surface = surface.validate()?;

    if width == 0 || height == 0 {
        return Ok(());
    }

    if origin_x >= surface.width || origin_y >= surface.height {
        return Err(());
    }

    if origin_x >= surface.pitch {
        return Err(());
    }

    let max_width = min(
        surface.width.saturating_sub(origin_x),
        surface.pitch.saturating_sub(origin_x),
    );
    let draw_width = min(width, max_width);

    let max_height = surface.height.saturating_sub(origin_y);
    let draw_height = min(height, max_height);

    if draw_width == 0 || draw_height == 0 {
        return Err(());
    }

    let pixel = encode_pixel(surface.pixel_format, color);

    unsafe {
        for row in 0..draw_height {
            let row_ptr = surface
                .base_ptr
                .add((origin_y + row) * surface.pitch + origin_x);
            for col in 0..draw_width {
                row_ptr.add(col).write_volatile(pixel);
            }
        }
    }

    Ok(())
}

/// Draw a single glyph bitmap at the given framebuffer coordinates.
pub fn draw_glyph(
    surface: FramebufferSurface,
    start_x: usize,
    start_y: usize,
    byte: u8,
    color: FramebufferColor,
) -> Result<(), ()> {
    let surface = surface.validate()?;

    let pitch = surface.pitch;
    let width = surface.width;
    let height = surface.height;

    if start_x >= width || start_y >= height {
        return Err(());
    }

    if start_x >= pitch {
        return Err(());
    }

    let glyph = glyph_for(byte);
    let draw_width = FONT_WIDTH
        .min(width.saturating_sub(start_x))
        .min(pitch.saturating_sub(start_x));
    let draw_height = FONT_HEIGHT.min(height.saturating_sub(start_y));

    if draw_width == 0 || draw_height == 0 {
        return Err(());
    }

    let pixel = encode_pixel(surface.pixel_format, color);

    unsafe {
        for (row, bitmap_row) in glyph.iter().copied().enumerate().take(draw_height) {
            let row_ptr = surface.base_ptr.add((start_y + row) * pitch + start_x);
            for col in 0..draw_width {
                let bit = FONT_WIDTH - 1 - col;
                if (bitmap_row >> bit) & 1 == 1 {
                    row_ptr.add(col).write_volatile(pixel);
                }
            }
        }
    }

    Ok(())
}

fn encode_pixel(format: PixelFormat, color: FramebufferColor) -> u32 {
    let (r, g, b) = color.components();
    match format {
        PixelFormat::Rgb => u32::from_le_bytes([r, g, b, 0xFF]),
        PixelFormat::Bgr => u32::from_le_bytes([b, g, r, 0xFF]),
    }
}

#[cfg(test)]
mod tests {
    extern crate alloc;

    use super::*;
    use alloc::vec;

    #[test]
    fn framebuffer_color_components_round_trip() {
        let color = FramebufferColor::new(0x12, 0x34, 0x56);
        assert_eq!(color.components(), (0x12, 0x34, 0x56));
    }

    #[test]
    fn encode_pixel_respects_rgb_format() {
        let color = FramebufferColor::new(0xAA, 0xBB, 0xCC);
        let encoded = super::encode_pixel(PixelFormat::Rgb, color);
        assert_eq!(encoded, 0xFF_CC_BB_AA);
    }

    #[test]
    fn encode_pixel_respects_bgr_format() {
        let color = FramebufferColor::new(0x11, 0x22, 0x33);
        let encoded = super::encode_pixel(PixelFormat::Bgr, color);
        assert_eq!(encoded, 0xFF_11_22_33);
    }

    #[test]
    fn framebuffer_surface_validate_rejects_invalid_geometry() {
        let surface = FramebufferSurface {
            base_ptr: core::ptr::null_mut(),
            pitch: 1,
            width: 1,
            height: 1,
            pixel_format: PixelFormat::Rgb,
        };
        assert!(surface.validate().is_err());
    }

    #[test]
    fn fill_rect_blits_only_within_bounds() {
        let pitch = 5;
        let width = 5;
        let height = 4;
        let mut backing = vec![0u32; pitch * height];
        let surface = FramebufferSurface {
            base_ptr: backing.as_mut_ptr(),
            pitch,
            width,
            height,
            pixel_format: PixelFormat::Rgb,
        };

        let color = FramebufferColor::new(0x10, 0x20, 0x30);
        super::fill_rect(surface, 1, 1, 3, 2, color).unwrap();

        let encoded = super::encode_pixel(PixelFormat::Rgb, color);
        for row in 0..height {
            for col in 0..width {
                let idx = row * pitch + col;
                if (1..4).contains(&col) && (1..3).contains(&row) {
                    assert_eq!(backing[idx], encoded);
                } else {
                    assert_eq!(backing[idx], 0);
                }
            }
        }
    }

    #[test]
    fn fill_rect_rejects_out_of_bounds_origin() {
        let pitch = 4;
        let width = 4;
        let height = 4;
        let mut backing = vec![0u32; pitch * height];
        let surface = FramebufferSurface {
            base_ptr: backing.as_mut_ptr(),
            pitch,
            width,
            height,
            pixel_format: PixelFormat::Rgb,
        };

        let result = super::fill_rect(surface, width + 1, 0, 1, 1, FramebufferColor::WHITE);
        assert!(result.is_err());
    }

    #[test]
    fn fill_rect_rejects_out_of_bounds_row() {
        let pitch = 4;
        let width = 4;
        let height = 4;
        let mut backing = vec![0u32; pitch * height];
        let surface = FramebufferSurface {
            base_ptr: backing.as_mut_ptr(),
            pitch,
            width,
            height,
            pixel_format: PixelFormat::Rgb,
        };

        let result = super::fill_rect(surface, 0, height, 1, 1, FramebufferColor::WHITE);
        assert!(result.is_err());
    }

    #[test]
    fn fill_rect_rejects_origin_past_pitch() {
        let pitch = 4;
        let width = 6;
        let height = 4;
        let mut backing = vec![0u32; pitch * height];
        let surface = FramebufferSurface {
            base_ptr: backing.as_mut_ptr(),
            pitch,
            width,
            height,
            pixel_format: PixelFormat::Rgb,
        };

        let result = super::fill_rect(surface, pitch, 0, 1, 1, FramebufferColor::WHITE);
        assert!(result.is_err());
    }

    #[test]
    fn draw_glyph_sets_pixels_for_known_character() {
        let pitch = 8;
        let width = 8;
        let height = FONT_HEIGHT * 2;
        let mut backing = vec![0u32; pitch * height];
        let surface = FramebufferSurface {
            base_ptr: backing.as_mut_ptr(),
            pitch,
            width,
            height,
            pixel_format: PixelFormat::Rgb,
        };

        let color = FramebufferColor::WHITE;
        super::draw_glyph(surface, 0, 0, b'A', color).unwrap();
        let encoded = super::encode_pixel(PixelFormat::Rgb, color);
        assert!(backing.iter().any(|&pixel| pixel == encoded));
    }
}

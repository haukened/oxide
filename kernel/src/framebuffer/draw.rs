use core::cmp::min;
use oxide_abi::{Framebuffer, PixelFormat};

use super::{FONT_HEIGHT, FONT_WIDTH, glyph_for};

/// Simple RGB color helper for framebuffer drawing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FramebufferColor {
    r: u8,
    g: u8,
    b: u8,
}

/// Fill the entire framebuffer with a panic indicator color.
pub fn panic_screen(fb: &Framebuffer) -> ! {
    let width = fb.width as usize;
    let height = fb.height as usize;

    // if the firmware didn't lie, try to fill the screen with red
    if width > 0 && height > 0 {
        let _ = draw_rect(fb, 0, 0, width, height, FramebufferColor::RED);
    }

    // halt
    loop {
        core::hint::spin_loop();
    }
}

#[allow(dead_code)]
impl FramebufferColor {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    pub const BLACK: Self = Self::new(0x00, 0x00, 0x00);
    pub const WHITE: Self = Self::new(0xFF, 0xFF, 0xFF);
    pub const RED: Self = Self::new(0xFF, 0x00, 0x00);
    pub const ORANGE: Self = Self::new(0xFF, 0xA5, 0x00);
    pub const YELLOW: Self = Self::new(0xFF, 0xFF, 0x00);
    pub const GREEN: Self = Self::new(0x00, 0x80, 0x00);
    pub const BLUE: Self = Self::new(0x00, 0x00, 0xFF);
    pub const INDIGO: Self = Self::new(0x4B, 0x00, 0x82);
    pub const VIOLET: Self = Self::new(0x8A, 0x2B, 0xE2);

    /// Convenience constructors for additive color combos.
    pub const CYAN: Self = Self::new(0x00, 0xFF, 0xFF);
    pub const MAGENTA: Self = Self::new(0xFF, 0x00, 0xFF);

    pub const fn components(self) -> (u8, u8, u8) {
        (self.r, self.g, self.b)
    }
}

/// Clear the framebuffer to black.
///
/// This function is defensive against malformed firmware data.
/// If the framebuffer geometry does not fit within the reported buffer,
/// it returns `Err(())` and performs no writes.
pub fn clear_black(fb: &Framebuffer) -> Result<(), ()> {
    if fb.base_address == 0 {
        return Err(());
    }

    let pitch = fb.pixels_per_scanline as usize;
    let width = fb.width as usize;
    let height = fb.height as usize;

    if pitch == 0 || width == 0 || height == 0 {
        return Err(());
    }

    let bytes_per_pixel = core::mem::size_of::<u32>();
    let max_pixels = (fb.buffer_size as usize) / bytes_per_pixel;
    if max_pixels == 0 {
        return Err(());
    }

    // Limit clearing to what actually fits in the buffer
    let max_rows = max_pixels / pitch;
    let clear_height = min(height, max_rows);
    if clear_height == 0 {
        return Err(());
    }

    let (r, g, b) = FramebufferColor::BLACK.components();
    let color = match fb.pixel_format {
        PixelFormat::Rgb => u32::from_le_bytes([r, g, b, 0xFF]),
        PixelFormat::Bgr => u32::from_le_bytes([b, g, r, 0xFF]),
    };

    let base_ptr = fb.base_address as *mut u32;

    unsafe {
        for y in 0..clear_height {
            let row_ptr = base_ptr.add(y * pitch);
            for x in 0..width {
                row_ptr.add(x).write_volatile(color);
            }
        }
    }

    Ok(())
}

/// Draw a solid rectangle at the given framebuffer coordinates.
pub fn draw_rect(
    fb: &Framebuffer,
    start_x: usize,
    start_y: usize,
    size_x: usize,
    size_y: usize,
    color: FramebufferColor,
) -> Result<(), ()> {
    if fb.base_address == 0 {
        return Err(());
    }

    let pitch = fb.pixels_per_scanline as usize;
    let width = fb.width as usize;
    let height = fb.height as usize;

    if pitch == 0 || width == 0 || height == 0 {
        return Err(());
    }

    if size_x == 0 || size_y == 0 {
        return Err(());
    }

    if start_x >= width || start_y >= height {
        return Err(());
    }

    let draw_width = min(size_x, width - start_x);
    let draw_height = min(size_y, height - start_y);
    if draw_width == 0 || draw_height == 0 {
        return Err(());
    }

    let row_span = start_x.checked_add(draw_width).ok_or(())?;
    if row_span > pitch {
        return Err(());
    }

    let bytes_per_pixel = core::mem::size_of::<u32>();
    let max_pixels = (fb.buffer_size as usize) / bytes_per_pixel;
    if max_pixels == 0 {
        return Err(());
    }

    let last_row = start_y
        .checked_add(draw_height)
        .and_then(|v| v.checked_sub(1))
        .ok_or(())?;
    let last_col = start_x
        .checked_add(draw_width)
        .and_then(|v| v.checked_sub(1))
        .ok_or(())?;
    let last_index = last_row
        .checked_mul(pitch)
        .and_then(|row| row.checked_add(last_col))
        .ok_or(())?;
    if last_index >= max_pixels {
        return Err(());
    }

    let (r, g, b) = color.components();
    let pixel = match fb.pixel_format {
        PixelFormat::Rgb => u32::from_le_bytes([r, g, b, 0xFF]),
        PixelFormat::Bgr => u32::from_le_bytes([b, g, r, 0xFF]),
    };

    let base_ptr = fb.base_address as *mut u32;

    unsafe {
        for y in 0..draw_height {
            let row_ptr = base_ptr.add((start_y + y) * pitch + start_x);
            for x in 0..draw_width {
                row_ptr.add(x).write_volatile(pixel);
            }
        }
    }

    Ok(())
}

/// Draw a single glyph bitmap at the given framebuffer coordinates.
pub fn draw_glyph(
    fb: &Framebuffer,
    start_x: usize,
    start_y: usize,
    byte: u8,
    color: FramebufferColor,
) -> Result<(), ()> {
    if fb.base_address == 0 {
        return Err(());
    }

    let pitch = fb.pixels_per_scanline as usize;
    let width = fb.width as usize;
    let height = fb.height as usize;

    if pitch == 0 || width == 0 || height == 0 {
        return Err(());
    }

    if start_x >= width || start_y >= height {
        return Err(());
    }

    let glyph = glyph_for(byte);
    let draw_width = FONT_WIDTH.min(width.saturating_sub(start_x));
    let draw_height = FONT_HEIGHT.min(height.saturating_sub(start_y));

    if draw_width == 0 || draw_height == 0 {
        return Err(());
    }

    let (r, g, b) = color.components();
    let pixel = match fb.pixel_format {
        PixelFormat::Rgb => u32::from_le_bytes([r, g, b, 0xFF]),
        PixelFormat::Bgr => u32::from_le_bytes([b, g, r, 0xFF]),
    };

    let base_ptr = fb.base_address as *mut u32;

    unsafe {
        for row in 0..draw_height {
            let bitmap_row = glyph[row];
            let row_ptr = base_ptr.add((start_y + row) * pitch + start_x);
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

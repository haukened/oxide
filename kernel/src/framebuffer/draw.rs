use core::cmp::min;
use oxide_abi::{Framebuffer, PixelFormat};

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

    let color = match fb.pixel_format {
        PixelFormat::Rgb | PixelFormat::Bgr => u32::from_le_bytes([0x00, 0x00, 0x00, 0xFF]),
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

/// Draw a boot marker (a small rectangle) in the top-left corner of the framebuffer.
pub fn draw_boot_marker(fb: &Framebuffer) -> Result<(), ()> {
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

    const MARKER_SIZE: usize = 64;
    let draw_width = min(width, min(pitch, MARKER_SIZE));
    let draw_height = min(height, MARKER_SIZE);
    if draw_width == 0 || draw_height == 0 {
        return Err(());
    }

    // Verify the marker fits within the buffer
    let last_row = draw_height.checked_sub(1).ok_or(())?;
    let row_offset = last_row.checked_mul(pitch).ok_or(())?;
    let last_col = draw_width.checked_sub(1).ok_or(())?;
    let last_index = row_offset.checked_add(last_col).ok_or(())?;
    if last_index >= max_pixels {
        return Err(());
    }

    let color = match fb.pixel_format {
        PixelFormat::Rgb => u32::from_le_bytes([0xFF, 0x40, 0x20, 0xFF]),
        PixelFormat::Bgr => u32::from_le_bytes([0x20, 0x40, 0xFF, 0xFF]),
    };

    let base_ptr = fb.base_address as *mut u32;

    unsafe {
        for y in 0..draw_height {
            let row_ptr = base_ptr.add(y * pitch);
            for x in 0..draw_width {
                row_ptr.add(x).write_volatile(color);
            }
        }
    }

    Ok(())
}

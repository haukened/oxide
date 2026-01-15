use core::{fmt, ptr};

use oxide_abi::Framebuffer;

use super::{
    FONT_HEIGHT, FONT_WIDTH, FramebufferColor,
    draw::{self, FramebufferSurface},
};

const LINE_SPACING: usize = 4;

pub(crate) fn sanitize_byte(byte: u8) -> u8 {
    match byte {
        b'a'..=b'z' => byte.to_ascii_uppercase(),
        0x20..=0x7E => byte,
        b'\n' | b'\r' => byte,
        b'\t' => b' ',
        _ => b'?',
    }
}

/// Text console that renders glyphs into a UEFI-provided linear framebuffer.
pub struct FramebufferConsole {
    surface: FramebufferSurface,
    viewport: Viewport,
    cursor: Cursor,
    color: FramebufferColor,
}

impl FramebufferConsole {
    pub fn new(fb: Framebuffer, origin_x: usize, origin_y: usize, color: FramebufferColor) -> Self {
        let surface = FramebufferSurface::new(fb).unwrap_or_else(|_| FramebufferSurface::empty());
        let viewport = Viewport::new(surface, origin_x, origin_y);

        Self {
            surface,
            viewport,
            cursor: Cursor::default(),
            color,
        }
    }

    pub fn is_usable(&self) -> bool {
        self.viewport.is_usable()
    }

    pub fn cols(&self) -> usize {
        self.viewport.cols
    }

    pub fn clear(&mut self) -> Result<(), ()> {
        if !self.viewport.is_usable() {
            return Err(());
        }

        let width = self.viewport.cols.saturating_mul(FONT_WIDTH);
        let height = self.viewport.rows.saturating_mul(self.viewport.line_stride);
        draw::fill_rect(
            self.surface,
            self.viewport.origin_x,
            self.viewport.origin_y,
            width,
            height,
            FramebufferColor::BLACK,
        )?;

        self.cursor = Cursor::default();
        Ok(())
    }

    pub fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), ()> {
        if !self.viewport.is_usable() {
            return Err(());
        }

        for &byte in bytes {
            self.put_byte(byte);
        }

        Ok(())
    }

    fn newline(&mut self) {
        self.cursor.col = 0;
        if self.cursor.row + 1 < self.viewport.rows {
            self.cursor.row += 1;
        } else if self.viewport.rows > 0 {
            self.scroll_up();
            self.cursor.row = self.viewport.rows.saturating_sub(1);
        }
    }

    fn put_byte(&mut self, byte: u8) {
        let b = sanitize_byte(byte);

        match b {
            b'\n' => {
                self.newline();
            }
            b'\r' => {
                self.cursor.col = 0;
            }
            _ => {
                if !self.viewport.is_usable() {
                    return;
                }

                if self.cursor.col >= self.viewport.cols {
                    self.newline();
                    if self.cursor.row >= self.viewport.rows {
                        return;
                    }
                }

                if let Some((x, y)) = self.viewport.pixel_position(self.cursor) {
                    let _ = draw::draw_glyph(self.surface, x, y, b, self.color);
                    self.cursor.col += 1;
                }
            }
        }
    }

    fn scroll_up(&mut self) {
        if !self.viewport.is_usable() {
            return;
        }

        let origin_x = self.viewport.origin_x;
        let origin_y = self.viewport.origin_y;
        let line_stride = self.viewport.line_stride;
        let rows = self.viewport.rows;
        let cols = self.viewport.cols;

        if rows == 0 || cols == 0 {
            return;
        }

        let width_pixels = cols.saturating_mul(FONT_WIDTH);
        let surface = self.surface;
        let pitch = surface.pitch;

        if origin_x >= pitch || origin_y >= surface.height {
            return;
        }

        let max_width = pitch
            .saturating_sub(origin_x)
            .min(surface.width.saturating_sub(origin_x));
        let draw_width = width_pixels.min(max_width);
        if draw_width == 0 {
            return;
        }

        if rows == 1 {
            let _ = draw::fill_rect(
                surface,
                origin_x,
                origin_y,
                draw_width,
                line_stride,
                FramebufferColor::BLACK,
            );
            return;
        }

        let available_rows = surface.height.saturating_sub(origin_y);
        if available_rows == 0 {
            return;
        }

        let scroll_rows = line_stride
            .saturating_mul(rows.saturating_sub(1))
            .min(available_rows);
        if scroll_rows == 0 {
            let _ = draw::fill_rect(
                surface,
                origin_x,
                origin_y,
                draw_width,
                line_stride,
                FramebufferColor::BLACK,
            );
            return;
        }

        unsafe {
            for row in 0..scroll_rows {
                let src_row = origin_y + row + line_stride;
                if src_row >= surface.height {
                    break;
                }
                let dst_row = origin_y + row;
                let dst_ptr = surface.base_ptr.add(dst_row * pitch + origin_x);
                let src_ptr = surface.base_ptr.add(src_row * pitch + origin_x);
                ptr::copy(src_ptr, dst_ptr, draw_width);
            }
        }

        let clear_height = line_stride.min(surface.height.saturating_sub(origin_y + scroll_rows));
        let _ = draw::fill_rect(
            surface,
            origin_x,
            origin_y + scroll_rows,
            draw_width,
            clear_height,
            FramebufferColor::BLACK,
        );
    }
}

impl fmt::Write for FramebufferConsole {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        if !self.viewport.is_usable() {
            return Err(fmt::Error);
        }

        for byte in s.bytes() {
            self.put_byte(byte);
        }

        Ok(())
    }
}

#[derive(Clone, Copy, Default)]
struct Cursor {
    col: usize,
    row: usize,
}

struct Viewport {
    origin_x: usize,
    origin_y: usize,
    cols: usize,
    rows: usize,
    line_stride: usize,
}

impl Viewport {
    fn new(surface: FramebufferSurface, origin_x: usize, origin_y: usize) -> Self {
        let width = surface.width.saturating_sub(origin_x);
        let height = surface.height.saturating_sub(origin_y);
        let line_stride = FONT_HEIGHT + LINE_SPACING;
        let cols = width / FONT_WIDTH;
        let rows = if height < FONT_HEIGHT {
            0
        } else {
            ((height - FONT_HEIGHT) / line_stride) + 1
        };

        Self {
            origin_x,
            origin_y,
            cols,
            rows,
            line_stride,
        }
    }

    fn is_usable(&self) -> bool {
        self.cols > 0 && self.rows > 0
    }

    fn pixel_position(&self, cursor: Cursor) -> Option<(usize, usize)> {
        if cursor.col >= self.cols || cursor.row >= self.rows {
            return None;
        }

        let x = self.origin_x + cursor.col * FONT_WIDTH;
        let y = self.origin_y + cursor.row * self.line_stride;
        Some((x, y))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oxide_abi::PixelFormat;

    #[test]
    fn sanitize_byte_filters_control_characters() {
        assert_eq!(sanitize_byte(b'a'), b'A');
        assert_eq!(sanitize_byte(b'Z'), b'Z');
        assert_eq!(sanitize_byte(b'\n'), b'\n');
        assert_eq!(sanitize_byte(b'\t'), b' ');
        assert_eq!(sanitize_byte(0x1B), b'?');
    }

    #[test]
    fn viewport_new_calculates_geometry() {
        let surface = FramebufferSurface {
            base_ptr: core::ptr::null_mut(),
            pitch: 200,
            width: 160,
            height: 60,
            pixel_format: PixelFormat::Rgb,
        };
        let viewport = Viewport::new(surface, 0, 0);
        assert_eq!(viewport.cols, 160 / FONT_WIDTH);
        assert_eq!(viewport.line_stride, FONT_HEIGHT + LINE_SPACING);
        assert!(viewport.rows >= 1);
    }

    #[test]
    fn viewport_pixel_position_within_bounds() {
        let surface = FramebufferSurface {
            base_ptr: core::ptr::null_mut(),
            pitch: 200,
            width: 160,
            height: 80,
            pixel_format: PixelFormat::Rgb,
        };
        let viewport = Viewport::new(surface, 10, 20);
        let cursor = Cursor { col: 2, row: 1 };
        let expected_x = 10 + 2 * FONT_WIDTH;
        let expected_y = 20 + 1 * (FONT_HEIGHT + LINE_SPACING);
        assert_eq!(
            viewport.pixel_position(cursor),
            Some((expected_x, expected_y))
        );
    }

    #[test]
    fn viewport_pixel_position_out_of_bounds_returns_none() {
        let surface = FramebufferSurface {
            base_ptr: core::ptr::null_mut(),
            pitch: 40,
            width: 80,
            height: 40,
            pixel_format: PixelFormat::Rgb,
        };
        let viewport = Viewport::new(surface, 0, 0);
        let cursor = Cursor {
            col: viewport.cols,
            row: 0,
        };
        assert_eq!(viewport.pixel_position(cursor), None);
    }
}

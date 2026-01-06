use core::fmt;

use oxide_abi::Framebuffer;

use super::{
    FONT_HEIGHT, FONT_WIDTH, FramebufferColor,
    draw::{self, FramebufferSurface},
};

const LINE_SPACING: usize = 2;

fn sanitize_byte(byte: u8) -> u8 {
    match byte {
        b'a'..=b'z' => byte.to_ascii_uppercase(),
        0x20..=0x7E => byte,
        b'\n' | b'\r' => byte,
        b'\t' => b' ',
        _ => b'?',
    }
}

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

    fn newline(&mut self) {
        self.cursor.col = 0;
        if self.cursor.row + 1 < self.viewport.rows {
            self.cursor.row += 1;
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

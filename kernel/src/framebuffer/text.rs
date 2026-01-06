use core::fmt;

use oxide_abi::Framebuffer;

use super::{FONT_HEIGHT, FONT_WIDTH, FramebufferColor, draw};

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

/// Draw a single, already-sanitized ASCII byte at the provided coordinates.
pub fn draw_char(
    fb: &Framebuffer,
    start_x: usize,
    start_y: usize,
    byte: u8,
    color: FramebufferColor,
) -> Result<(), ()> {
    draw::draw_glyph(fb, start_x, start_y, byte, color)
}

pub struct FramebufferConsole {
    fb: Framebuffer,
    origin_x: usize,
    origin_y: usize,
    cursor_col: usize,
    cursor_row: usize,
    max_cols: usize,
    max_rows: usize,
    color: FramebufferColor,
}

impl FramebufferConsole {
    pub fn new(fb: Framebuffer, origin_x: usize, origin_y: usize, color: FramebufferColor) -> Self {
        let width = fb.width as usize;
        let height = fb.height as usize;
        let max_cols = width.saturating_sub(origin_x) / FONT_WIDTH;
        let available_height = height.saturating_sub(origin_y);
        let row_stride = FONT_HEIGHT + LINE_SPACING;
        let max_rows = if available_height < FONT_HEIGHT {
            0
        } else {
            ((available_height - FONT_HEIGHT) / row_stride) + 1
        };

        Self {
            fb,
            origin_x,
            origin_y,
            cursor_col: 0,
            cursor_row: 0,
            max_cols,
            max_rows,
            color,
        }
    }

    pub fn is_usable(&self) -> bool {
        self.max_cols > 0 && self.max_rows > 0
    }

    fn newline(&mut self) {
        self.cursor_col = 0;
        if self.cursor_row + 1 < self.max_rows {
            self.cursor_row += 1;
        }
    }

    fn put_byte(&mut self, byte: u8) {
        let b = sanitize_byte(byte);

        match b {
            b'\n' => {
                self.newline();
            }
            b'\r' => {
                self.cursor_col = 0;
            }
            _ => {
                if self.max_cols == 0 || self.max_rows == 0 {
                    return;
                }

                if self.cursor_col >= self.max_cols {
                    self.newline();
                    if self.cursor_row >= self.max_rows {
                        return;
                    }
                }

                let x = self.origin_x + self.cursor_col * FONT_WIDTH;
                let y = self.origin_y + self.cursor_row * (FONT_HEIGHT + LINE_SPACING);
                let _ = draw_char(&self.fb, x, y, b, self.color);
                self.cursor_col += 1;
            }
        }
    }
}

impl fmt::Write for FramebufferConsole {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        if self.max_cols == 0 || self.max_rows == 0 {
            return Err(fmt::Error);
        }

        for byte in s.bytes() {
            self.put_byte(byte);
        }

        Ok(())
    }
}

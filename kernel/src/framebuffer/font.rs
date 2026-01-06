#![allow(dead_code)]

/// Minimal built-in bitmap font for early kernel diagnostics.
///
/// This deliberately covers only the small ASCII subset needed for bring-up
/// (hex digits, a few letters, punctuation). It is not intended to be a full
/// terminal or shell font; once richer text output is required, replace or
/// extend it with a more complete solution.
pub const FONT_WIDTH: usize = 8;
pub const FONT_HEIGHT: usize = 16;

const fn double_rows(rows: [u8; 8]) -> [u8; FONT_HEIGHT] {
    let mut out = [0u8; FONT_HEIGHT];
    let mut i = 0;
    while i < 8 {
        out[i * 2] = rows[i];
        out[i * 2 + 1] = rows[i];
        i += 1;
    }
    out
}

const GLYPH_BLANK: [u8; FONT_HEIGHT] = [0; FONT_HEIGHT];
const GLYPH_DOT: [u8; FONT_HEIGHT] = double_rows([
    0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00011000, 0b00011000,
]);
const GLYPH_COLON: [u8; FONT_HEIGHT] = double_rows([
    0b00000000, 0b00011000, 0b00011000, 0b00000000, 0b00000000, 0b00011000, 0b00011000, 0b00000000,
]);
const GLYPH_QUESTION: [u8; FONT_HEIGHT] = double_rows([
    0b00111100, 0b01100110, 0b00000110, 0b00001100, 0b00011000, 0b00000000, 0b00011000, 0b00011000,
]);

const GLYPH_0: [u8; FONT_HEIGHT] = double_rows([
    0b00111100, 0b01100110, 0b01100110, 0b01100110, 0b01100110, 0b01100110, 0b01100110, 0b00111100,
]);
const GLYPH_G: [u8; FONT_HEIGHT] = double_rows([
    0b00111100, 0b01100110, 0b01100000, 0b01101110, 0b01100110, 0b01100110, 0b01100110, 0b00111100,
]);
const GLYPH_H: [u8; FONT_HEIGHT] = double_rows([
    0b01100110, 0b01100110, 0b01100110, 0b01111110, 0b01100110, 0b01100110, 0b01100110, 0b01100110,
]);
const GLYPH_I: [u8; FONT_HEIGHT] = double_rows([
    0b00111100, 0b00011000, 0b00011000, 0b00011000, 0b00011000, 0b00011000, 0b00011000, 0b00111100,
]);
const GLYPH_J: [u8; FONT_HEIGHT] = double_rows([
    0b00011110, 0b00000110, 0b00000110, 0b00000110, 0b00000110, 0b01100110, 0b01100110, 0b00111100,
]);
const GLYPH_K: [u8; FONT_HEIGHT] = double_rows([
    0b01100110, 0b01101100, 0b01111000, 0b01110000, 0b01111000, 0b01101100, 0b01100110, 0b01100110,
]);
const GLYPH_L: [u8; FONT_HEIGHT] = double_rows([
    0b01100000, 0b01100000, 0b01100000, 0b01100000, 0b01100000, 0b01100000, 0b01100000, 0b01111110,
]);
const GLYPH_M: [u8; FONT_HEIGHT] = double_rows([
    0b01100110, 0b01111110, 0b01111110, 0b01100110, 0b01100110, 0b01100110, 0b01100110, 0b01100110,
]);
const GLYPH_N: [u8; FONT_HEIGHT] = double_rows([
    0b01100110, 0b01110110, 0b01111110, 0b01101110, 0b01100110, 0b01100110, 0b01100110, 0b01100110,
]);
const GLYPH_O: [u8; FONT_HEIGHT] = double_rows([
    0b00111100, 0b01100110, 0b01100110, 0b01100110, 0b01100110, 0b01100110, 0b01100110, 0b00111100,
]);
const GLYPH_Q: [u8; FONT_HEIGHT] = double_rows([
    0b00111100, 0b01100110, 0b01100110, 0b01100110, 0b01100110, 0b01101110, 0b00111100, 0b00001110,
]);
const GLYPH_T: [u8; FONT_HEIGHT] = double_rows([
    0b01111110, 0b00011000, 0b00011000, 0b00011000, 0b00011000, 0b00011000, 0b00011000, 0b00011000,
]);
const GLYPH_U: [u8; FONT_HEIGHT] = double_rows([
    0b01100110, 0b01100110, 0b01100110, 0b01100110, 0b01100110, 0b01100110, 0b01100110, 0b00111100,
]);
const GLYPH_V: [u8; FONT_HEIGHT] = double_rows([
    0b01100110, 0b01100110, 0b01100110, 0b00111100, 0b00111100, 0b00111100, 0b00011000, 0b00011000,
]);
const GLYPH_W: [u8; FONT_HEIGHT] = double_rows([
    0b01100110, 0b01100110, 0b01100110, 0b01100110, 0b01111110, 0b01111110, 0b01100110, 0b01100110,
]);
const GLYPH_Y: [u8; FONT_HEIGHT] = double_rows([
    0b01100110, 0b01100110, 0b00111100, 0b00011000, 0b00011000, 0b00011000, 0b00011000, 0b00011000,
]);
const GLYPH_Z: [u8; FONT_HEIGHT] = double_rows([
    0b01111110, 0b00000110, 0b00001100, 0b00011000, 0b00110000, 0b01100000, 0b01111110, 0b01111110,
]);
const GLYPH_1: [u8; FONT_HEIGHT] = double_rows([
    0b00011000, 0b00111000, 0b00011000, 0b00011000, 0b00011000, 0b00011000, 0b00011000, 0b00111100,
]);
const GLYPH_2: [u8; FONT_HEIGHT] = double_rows([
    0b00111100, 0b01100110, 0b00000110, 0b00001100, 0b00011000, 0b00110000, 0b01100000, 0b01111110,
]);
const GLYPH_3: [u8; FONT_HEIGHT] = double_rows([
    0b00111100, 0b01100110, 0b00000110, 0b00011100, 0b00000110, 0b00000110, 0b01100110, 0b00111100,
]);
const GLYPH_4: [u8; FONT_HEIGHT] = double_rows([
    0b00001100, 0b00011100, 0b00101100, 0b01001100, 0b01111110, 0b00001100, 0b00001100, 0b00001100,
]);
const GLYPH_5: [u8; FONT_HEIGHT] = double_rows([
    0b01111110, 0b01100000, 0b01100000, 0b01111100, 0b00000110, 0b00000110, 0b01100110, 0b00111100,
]);
const GLYPH_6: [u8; FONT_HEIGHT] = double_rows([
    0b00111100, 0b01100000, 0b01100000, 0b01111100, 0b01100110, 0b01100110, 0b01100110, 0b00111100,
]);
const GLYPH_7: [u8; FONT_HEIGHT] = double_rows([
    0b01111110, 0b00000110, 0b00001100, 0b00011000, 0b00110000, 0b00110000, 0b00110000, 0b00110000,
]);
const GLYPH_8: [u8; FONT_HEIGHT] = double_rows([
    0b00111100, 0b01100110, 0b01100110, 0b00111100, 0b01100110, 0b01100110, 0b01100110, 0b00111100,
]);
const GLYPH_9: [u8; FONT_HEIGHT] = double_rows([
    0b00111100, 0b01100110, 0b01100110, 0b00111110, 0b00000110, 0b00000110, 0b01100110, 0b00111100,
]);

const GLYPH_A: [u8; FONT_HEIGHT] = double_rows([
    0b00011000, 0b00111100, 0b01100110, 0b01100110, 0b01111110, 0b01100110, 0b01100110, 0b01100110,
]);
const GLYPH_B: [u8; FONT_HEIGHT] = double_rows([
    0b01111100, 0b01100110, 0b01100110, 0b01111100, 0b01100110, 0b01100110, 0b01100110, 0b01111100,
]);
const GLYPH_C: [u8; FONT_HEIGHT] = double_rows([
    0b00111100, 0b01100110, 0b01100000, 0b01100000, 0b01100000, 0b01100000, 0b01100110, 0b00111100,
]);
const GLYPH_D: [u8; FONT_HEIGHT] = double_rows([
    0b01111000, 0b01101100, 0b01100110, 0b01100110, 0b01100110, 0b01100110, 0b01101100, 0b01111000,
]);
const GLYPH_E: [u8; FONT_HEIGHT] = double_rows([
    0b01111110, 0b01100000, 0b01100000, 0b01111100, 0b01100000, 0b01100000, 0b01100000, 0b01111110,
]);
const GLYPH_F: [u8; FONT_HEIGHT] = double_rows([
    0b01111110, 0b01100000, 0b01100000, 0b01111100, 0b01100000, 0b01100000, 0b01100000, 0b01100000,
]);
const GLYPH_P: [u8; FONT_HEIGHT] = double_rows([
    0b01111100, 0b01100110, 0b01100110, 0b01111100, 0b01100000, 0b01100000, 0b01100000, 0b01100000,
]);
const GLYPH_R: [u8; FONT_HEIGHT] = double_rows([
    0b01111100, 0b01100110, 0b01100110, 0b01111100, 0b01101100, 0b01100110, 0b01100110, 0b01100110,
]);
const GLYPH_S: [u8; FONT_HEIGHT] = double_rows([
    0b00111100, 0b01100110, 0b01100000, 0b00111100, 0b00000110, 0b00000110, 0b01100110, 0b00111100,
]);
const GLYPH_X: [u8; FONT_HEIGHT] = double_rows([
    0b00000000, 0b01100110, 0b00111100, 0b00011000, 0b00111100, 0b01100110, 0b01100110, 0b00000000,
]);

pub fn glyph_for(byte: u8) -> &'static [u8; FONT_HEIGHT] {
    match byte {
        b'0' => &GLYPH_0,
        b'1' => &GLYPH_1,
        b'2' => &GLYPH_2,
        b'3' => &GLYPH_3,
        b'4' => &GLYPH_4,
        b'5' => &GLYPH_5,
        b'6' => &GLYPH_6,
        b'7' => &GLYPH_7,
        b'8' => &GLYPH_8,
        b'9' => &GLYPH_9,
        b'A' | b'a' => &GLYPH_A,
        b'B' | b'b' => &GLYPH_B,
        b'C' | b'c' => &GLYPH_C,
        b'D' | b'd' => &GLYPH_D,
        b'E' | b'e' => &GLYPH_E,
        b'F' | b'f' => &GLYPH_F,
        b'G' | b'g' => &GLYPH_G,
        b'H' | b'h' => &GLYPH_H,
        b'I' | b'i' => &GLYPH_I,
        b'J' | b'j' => &GLYPH_J,
        b'K' | b'k' => &GLYPH_K,
        b'L' | b'l' => &GLYPH_L,
        b'M' | b'm' => &GLYPH_M,
        b'N' | b'n' => &GLYPH_N,
        b'O' | b'o' => &GLYPH_O,
        b'Q' | b'q' => &GLYPH_Q,
        b'P' | b'p' => &GLYPH_P,
        b'R' | b'r' => &GLYPH_R,
        b'S' | b's' => &GLYPH_S,
        b'T' | b't' => &GLYPH_T,
        b'U' | b'u' => &GLYPH_U,
        b'V' | b'v' => &GLYPH_V,
        b'W' | b'w' => &GLYPH_W,
        b'X' | b'x' => &GLYPH_X,
        b'Y' | b'y' => &GLYPH_Y,
        b'Z' | b'z' => &GLYPH_Z,
        b':' => &GLYPH_COLON,
        b'.' => &GLYPH_DOT,
        b'?' => &GLYPH_QUESTION,
        b' ' => &GLYPH_BLANK,
        _ => &GLYPH_QUESTION,
    }
}

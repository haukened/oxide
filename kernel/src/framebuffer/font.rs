#![allow(dead_code)]

/// Minimal built-in bitmap font for early kernel diagnostics.
///
/// This deliberately covers only the small ASCII subset needed for bring-up
/// (hex digits, a few letters, punctuation). It is not intended to be a full
/// terminal or shell font; once richer text output is required, replace or
/// extend it with a more complete solution.
pub const FONT_WIDTH: usize = 8;
pub const FONT_HEIGHT: usize = 16;

const GLYPH_LOOKUP: [&'static [u8; FONT_HEIGHT]; 128] = build_glyph_lookup();

pub fn glyph_for(byte: u8) -> &'static [u8; FONT_HEIGHT] {
    GLYPH_LOOKUP
        .get(byte as usize)
        .copied()
        .unwrap_or(&GLYPH_SYM_QUES)
}

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

/*
    Explanation: Each byte represents a row of 8 pixels in the glyph bitmap.
    A '1' bit indicates a filled pixel, and a '0' bit indicates a blank pixel.
    The glyphs are defined in an 8-row format and then doubled to fit the
    FONT_HEIGHT of 16 for better vertical resolution.

    const GLYPH_A: [u8; FONT_HEIGHT] = double_rows([
        0b00000000, // Row 0  =  □□□□□□□□
        0b00011000, // Row 1  =  □□□■■□□□
        0b00111100, // Row 2  =  □□■■■■□□
        0b01100110, // Row 3  =  □■■□□■■□
        0b01100110, // Row 4  =  □■■□□■■□
        0b01111110, // Row 5  =  □■■■■■■□
        0b01100110, // Row 6  =  □■■□□■■□
        0b01100110, // Row 7  =  □■■□□■■□
    ]);

    (If you squint a little, you can see the letter 'A' in the pattern above)
*/

/* Punctuation and symbols */

const GLYPH_SYM_SPCE: [u8; FONT_HEIGHT] = [0; FONT_HEIGHT];
const GLYPH_SYM_EXCL: [u8; FONT_HEIGHT] = double_rows([
    0b00011000, 0b00011000, 0b00011000, 0b00011000, 0b00011000, 0b00000000, 0b00011000, 0b00000000,
]);
const GLYPH_SYM_AT: [u8; FONT_HEIGHT] = double_rows([
    0b00111100, 0b01000010, 0b10111001, 0b10101001, 0b10111101, 0b10011110, 0b01000000, 0b00111100,
]);
const GLYPH_SYM_HASH: [u8; FONT_HEIGHT] = double_rows([
    0b00100100, 0b00100100, 0b01111110, 0b00100100, 0b00100100, 0b01111110, 0b00100100, 0b00100100,
]);
const GLYPH_SYM_DOLL: [u8; FONT_HEIGHT] = double_rows([
    0b00001000, 0b00111110, 0b01001000, 0b00111100, 0b00001010, 0b01111100, 0b00001000, 0b00000000,
]);
const GLYPH_SYM_PERC: [u8; FONT_HEIGHT] = double_rows([
    0b01100010, 0b01100100, 0b00001000, 0b00010000, 0b00100000, 0b01000110, 0b10000110, 0b00000000,
]);
const GLYPH_SYM_CIRC: [u8; FONT_HEIGHT] = double_rows([
    0b00010000, 0b00101000, 0b01000100, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000,
]);
const GLYPH_SYM_AMPR: [u8; FONT_HEIGHT] = double_rows([
    0b00111000, 0b01000100, 0b01000100, 0b00111000, 0b01001010, 0b01000100, 0b00111010, 0b00000000,
]);
const GLYPH_SYM_ASTR: [u8; FONT_HEIGHT] = double_rows([
    0b00000000, 0b00101000, 0b00010000, 0b01111110, 0b00010000, 0b00101000, 0b00000000, 0b00000000,
]);
const GLYPH_SYM_LPAR: [u8; FONT_HEIGHT] = double_rows([
    0b00001110, 0b00011000, 0b00110000, 0b00110000, 0b00110000, 0b00011000, 0b00001110, 0b00000000,
]);
const GLYPH_SYM_RPAR: [u8; FONT_HEIGHT] = double_rows([
    0b01110000, 0b00110000, 0b00011000, 0b00011000, 0b00011000, 0b00110000, 0b01110000, 0b00000000,
]);
const GLYPH_SYM_DASH: [u8; FONT_HEIGHT] = double_rows([
    0b00000000, 0b00000000, 0b00000000, 0b01111110, 0b00000000, 0b00000000, 0b00000000, 0b00000000,
]);
const GLYPH_SYM_PLUS: [u8; FONT_HEIGHT] = double_rows([
    0b00000000, 0b00010000, 0b00010000, 0b01111110, 0b00010000, 0b00010000, 0b00000000, 0b00000000,
]);
const GLYPH_SYM_UNDS: [u8; FONT_HEIGHT] = double_rows([
    0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b01111110, 0b00000000,
]);
const GLYPH_SYM_EQLS: [u8; FONT_HEIGHT] = double_rows([
    0b00000000, 0b00000000, 0b01111110, 0b00000000, 0b01111110, 0b00000000, 0b00000000, 0b00000000,
]);
const GLYPH_SYM_LBRC: [u8; FONT_HEIGHT] = double_rows([
    0b00011110, 0b00011000, 0b00011000, 0b00011000, 0b00011000, 0b00011000, 0b00011000, 0b00011110,
]);
const GLYPH_SYM_RBRC: [u8; FONT_HEIGHT] = double_rows([
    0b00011110, 0b00000110, 0b00000110, 0b00000110, 0b00000110, 0b00000110, 0b00000110, 0b00011110,
]);
const GLYPH_SYM_PIPE: [u8; FONT_HEIGHT] = double_rows([
    0b00011000, 0b00011000, 0b00011000, 0b00011000, 0b00011000, 0b00011000, 0b00011000, 0b00011000,
]);
const GLYPH_SYM_BSLS: [u8; FONT_HEIGHT] = double_rows([
    0b10000000, 0b01000000, 0b00100000, 0b00010000, 0b00001000, 0b00000100, 0b00000010, 0b00000000,
]);
const GLYPH_SYM_COLN: [u8; FONT_HEIGHT] = double_rows([
    0b00000000, 0b00011000, 0b00011000, 0b00000000, 0b00000000, 0b00011000, 0b00011000, 0b00000000,
]);
const GLYPH_SYM_SEMI: [u8; FONT_HEIGHT] = double_rows([
    0b00000000, 0b00011000, 0b00011000, 0b00000000, 0b00000000, 0b00011000, 0b00011000, 0b00110000,
]);
const GLYPH_SYM_APOS: [u8; FONT_HEIGHT] = double_rows([
    0b00011000, 0b00011000, 0b00011000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000,
]);
const GLYPH_SYM_DQUO: [u8; FONT_HEIGHT] = double_rows([
    0b00110110, 0b00110110, 0b00110110, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000,
]);
const GLYPH_SYM_COMM: [u8; FONT_HEIGHT] = double_rows([
    0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00011000, 0b00011000, 0b00110000,
]);
const GLYPH_SYM_PERD: [u8; FONT_HEIGHT] = double_rows([
    0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00011000, 0b00011000,
]);
const GLYPH_SYM_QUES: [u8; FONT_HEIGHT] = double_rows([
    0b00111100, 0b01100110, 0b00000110, 0b00001100, 0b00011000, 0b00000000, 0b00011000, 0b00000000,
]);
const GLYPH_SYM_LESS: [u8; FONT_HEIGHT] = double_rows([
    0b00000110, 0b00001100, 0b00011000, 0b00110000, 0b00011000, 0b00001100, 0b00000110, 0b00000000,
]);
const GLYPH_SYM_GRTR: [u8; FONT_HEIGHT] = double_rows([
    0b01100000, 0b00110000, 0b00011000, 0b00001100, 0b00011000, 0b00110000, 0b01100000, 0b00000000,
]);
const GLYPH_SYM_FSLS: [u8; FONT_HEIGHT] = double_rows([
    0b00000010, 0b00000100, 0b00001000, 0b00010000, 0b00100000, 0b01000000, 0b10000000, 0b00000000,
]);

/* Letters A-Z */

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
const GLYPH_P: [u8; FONT_HEIGHT] = double_rows([
    0b01111100, 0b01100110, 0b01100110, 0b01111100, 0b01100000, 0b01100000, 0b01100000, 0b01100000,
]);
const GLYPH_Q: [u8; FONT_HEIGHT] = double_rows([
    0b00111100, 0b01100110, 0b01100110, 0b01100110, 0b01100110, 0b01101110, 0b00111100, 0b00001110,
]);
const GLYPH_R: [u8; FONT_HEIGHT] = double_rows([
    0b01111100, 0b01100110, 0b01100110, 0b01111100, 0b01101100, 0b01100110, 0b01100110, 0b01100110,
]);
const GLYPH_S: [u8; FONT_HEIGHT] = double_rows([
    0b00111100, 0b01100110, 0b01100000, 0b00111100, 0b00000110, 0b00000110, 0b01100110, 0b00111100,
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
const GLYPH_X: [u8; FONT_HEIGHT] = double_rows([
    0b01100110, 0b01100110, 0b00111100, 0b00011000, 0b00111100, 0b01100110, 0b01100110, 0b01100110,
]);
const GLYPH_Y: [u8; FONT_HEIGHT] = double_rows([
    0b01100110, 0b01100110, 0b00111100, 0b00011000, 0b00011000, 0b00011000, 0b00011000, 0b00011000,
]);
const GLYPH_Z: [u8; FONT_HEIGHT] = double_rows([
    0b01111110, 0b00000110, 0b00001100, 0b00011000, 0b00110000, 0b01100000, 0b01111110, 0b01111110,
]);

/* Numbers 0-9 */

const GLYPH_0: [u8; FONT_HEIGHT] = double_rows([
    0b00111100, 0b01100110, 0b01100110, 0b01100110, 0b01100110, 0b01100110, 0b01100110, 0b00111100,
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

const fn build_glyph_lookup() -> [&'static [u8; FONT_HEIGHT]; 128] {
    let mut table = [&GLYPH_SYM_QUES; 128];

    table[b'0' as usize] = &GLYPH_0;
    table[b'1' as usize] = &GLYPH_1;
    table[b'2' as usize] = &GLYPH_2;
    table[b'3' as usize] = &GLYPH_3;
    table[b'4' as usize] = &GLYPH_4;
    table[b'5' as usize] = &GLYPH_5;
    table[b'6' as usize] = &GLYPH_6;
    table[b'7' as usize] = &GLYPH_7;
    table[b'8' as usize] = &GLYPH_8;
    table[b'9' as usize] = &GLYPH_9;

    table[b'A' as usize] = &GLYPH_A;
    table[b'B' as usize] = &GLYPH_B;
    table[b'C' as usize] = &GLYPH_C;
    table[b'D' as usize] = &GLYPH_D;
    table[b'E' as usize] = &GLYPH_E;
    table[b'F' as usize] = &GLYPH_F;
    table[b'G' as usize] = &GLYPH_G;
    table[b'H' as usize] = &GLYPH_H;
    table[b'I' as usize] = &GLYPH_I;
    table[b'J' as usize] = &GLYPH_J;
    table[b'K' as usize] = &GLYPH_K;
    table[b'L' as usize] = &GLYPH_L;
    table[b'M' as usize] = &GLYPH_M;
    table[b'N' as usize] = &GLYPH_N;
    table[b'O' as usize] = &GLYPH_O;
    table[b'P' as usize] = &GLYPH_P;
    table[b'Q' as usize] = &GLYPH_Q;
    table[b'R' as usize] = &GLYPH_R;
    table[b'S' as usize] = &GLYPH_S;
    table[b'T' as usize] = &GLYPH_T;
    table[b'U' as usize] = &GLYPH_U;
    table[b'V' as usize] = &GLYPH_V;
    table[b'W' as usize] = &GLYPH_W;
    table[b'X' as usize] = &GLYPH_X;
    table[b'Y' as usize] = &GLYPH_Y;
    table[b'Z' as usize] = &GLYPH_Z;

    table[b'a' as usize] = &GLYPH_A;
    table[b'b' as usize] = &GLYPH_B;
    table[b'c' as usize] = &GLYPH_C;
    table[b'd' as usize] = &GLYPH_D;
    table[b'e' as usize] = &GLYPH_E;
    table[b'f' as usize] = &GLYPH_F;
    table[b'g' as usize] = &GLYPH_G;
    table[b'h' as usize] = &GLYPH_H;
    table[b'i' as usize] = &GLYPH_I;
    table[b'j' as usize] = &GLYPH_J;
    table[b'k' as usize] = &GLYPH_K;
    table[b'l' as usize] = &GLYPH_L;
    table[b'm' as usize] = &GLYPH_M;
    table[b'n' as usize] = &GLYPH_N;
    table[b'o' as usize] = &GLYPH_O;
    table[b'p' as usize] = &GLYPH_P;
    table[b'q' as usize] = &GLYPH_Q;
    table[b'r' as usize] = &GLYPH_R;
    table[b's' as usize] = &GLYPH_S;
    table[b't' as usize] = &GLYPH_T;
    table[b'u' as usize] = &GLYPH_U;
    table[b'v' as usize] = &GLYPH_V;
    table[b'w' as usize] = &GLYPH_W;
    table[b'x' as usize] = &GLYPH_X;
    table[b'y' as usize] = &GLYPH_Y;
    table[b'z' as usize] = &GLYPH_Z;

    table[b'!' as usize] = &GLYPH_SYM_EXCL;
    table[b'"' as usize] = &GLYPH_SYM_DQUO;
    table[b'#' as usize] = &GLYPH_SYM_HASH;
    table[b'$' as usize] = &GLYPH_SYM_DOLL;
    table[b'%' as usize] = &GLYPH_SYM_PERC;
    table[b'&' as usize] = &GLYPH_SYM_AMPR;
    table[b'\'' as usize] = &GLYPH_SYM_APOS;
    table[b'(' as usize] = &GLYPH_SYM_LPAR;
    table[b')' as usize] = &GLYPH_SYM_RPAR;
    table[b'*' as usize] = &GLYPH_SYM_ASTR;
    table[b'+' as usize] = &GLYPH_SYM_PLUS;
    table[b',' as usize] = &GLYPH_SYM_COMM;
    table[b'-' as usize] = &GLYPH_SYM_DASH;
    table[b'.' as usize] = &GLYPH_SYM_PERD;
    table[b'/' as usize] = &GLYPH_SYM_FSLS;
    table[b':' as usize] = &GLYPH_SYM_COLN;
    table[b';' as usize] = &GLYPH_SYM_SEMI;
    table[b'<' as usize] = &GLYPH_SYM_LESS;
    table[b'=' as usize] = &GLYPH_SYM_EQLS;
    table[b'>' as usize] = &GLYPH_SYM_GRTR;
    table[b'?' as usize] = &GLYPH_SYM_QUES;
    table[b'@' as usize] = &GLYPH_SYM_AT;
    table[b'[' as usize] = &GLYPH_SYM_LBRC;
    table[b'\\' as usize] = &GLYPH_SYM_BSLS;
    table[b']' as usize] = &GLYPH_SYM_RBRC;
    table[b'^' as usize] = &GLYPH_SYM_CIRC;
    table[b'_' as usize] = &GLYPH_SYM_UNDS;
    table[b'|' as usize] = &GLYPH_SYM_PIPE;
    table[b' ' as usize] = &GLYPH_SYM_SPCE;

    table
}

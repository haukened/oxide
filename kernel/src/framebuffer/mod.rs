use oxide_abi::Framebuffer;

mod draw;
mod font;
pub mod text;

pub use draw::FramebufferColor;
pub use font::{FONT_HEIGHT, FONT_WIDTH, glyph_for};

pub fn clear_framebuffer(fb: &Framebuffer) -> Result<(), ()> {
    draw::clear_black(fb)
}

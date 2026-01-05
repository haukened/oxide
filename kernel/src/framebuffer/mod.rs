use oxide_abi::Framebuffer;

mod draw;

pub fn clear_framebuffer(fb: &Framebuffer) -> Result<(), ()> {
    draw::clear_black(fb)
}

pub fn draw_boot_marker(fb: &Framebuffer) -> Result<(), ()> {
    draw::draw_boot_marker(fb)
}

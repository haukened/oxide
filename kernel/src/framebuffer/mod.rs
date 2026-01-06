use oxide_abi::Framebuffer;

mod boot;
mod draw;

pub use boot::BootStage;
pub use draw::FramebufferColor;

pub fn clear_framebuffer(fb: &Framebuffer) -> Result<(), ()> {
    draw::clear_black(fb)
}

pub fn draw_rect(
    fb: &Framebuffer,
    start_x: usize,
    start_y: usize,
    size_x: usize,
    size_y: usize,
    color: FramebufferColor,
) -> Result<(), ()> {
    draw::draw_rect(fb, start_x, start_y, size_x, size_y, color)
}

pub fn panic_screen(fb: &Framebuffer) -> ! {
    draw::panic_screen(fb);
}

pub fn draw_boot_stage(fb: &Framebuffer, stage: boot::BootStage) {
    boot::draw_boot_stage(fb, stage);
}

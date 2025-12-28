use crate::framebuffer::{FramebufferInfo, write_line};

static mut FRAMEBUFFER: Option<FramebufferInfo> = None;

pub fn set_framebuffer_sink(fb: FramebufferInfo) {
    unsafe {
        FRAMEBUFFER = Some(fb);
    }
}

pub fn writeln(message: &str) {
    unsafe {
        match &FRAMEBUFFER {
            Some(fb) => write_line(fb, message),
            None => uefi::println!("{}", message),
        }
    }
}

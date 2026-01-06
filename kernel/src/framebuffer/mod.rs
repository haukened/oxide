#![allow(unused_imports)]
#![allow(dead_code)]
use oxide_abi::Framebuffer;

mod boot;
mod draw;
mod font;
mod text;

use core::fmt;

pub use boot::BootStage;
pub use draw::{FramebufferColor, draw_glyph};
pub use font::{FONT_HEIGHT, FONT_WIDTH, glyph_for};
pub use text::{FramebufferConsole, draw_char, draw_text};

use core::cell::UnsafeCell;

struct ConsoleCell(UnsafeCell<Option<text::FramebufferConsole>>);

unsafe impl Sync for ConsoleCell {}

static BOOT_CONSOLE_STORAGE: ConsoleCell = ConsoleCell(UnsafeCell::new(None));

pub unsafe fn init_boot_console(fb: Framebuffer, color: FramebufferColor) {
    unsafe {
        *BOOT_CONSOLE_STORAGE.0.get() =
            Some(text::FramebufferConsole::new(fb, 0, FONT_HEIGHT, color));
    }
}

pub(crate) fn console_write(args: fmt::Arguments<'_>) {
    unsafe {
        if let Some(console) = (*BOOT_CONSOLE_STORAGE.0.get()).as_mut() {
            let _ = fmt::Write::write_fmt(console, args);
        }
    }
}

#[macro_export]
macro_rules! fb_print {
    ($($arg:tt)*) => {
        $crate::framebuffer::console_write(core::format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! fb_println {
    () => {
        $crate::framebuffer::console_write(core::format_args!("\n"))
    };
    ($fmt:expr $(, $arg:tt)*) => {
        $crate::framebuffer::console_write(core::format_args!(concat!($fmt, "\n") $(, $arg)*))
    };
}

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

use oxide_abi::Framebuffer;

mod draw;
mod font;
mod text;

use core::fmt;

pub use draw::FramebufferColor;
pub use font::{FONT_HEIGHT, FONT_WIDTH, glyph_for};

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

#[macro_export]
macro_rules! fb_diag {
    ($($arg:tt)*) => {
        if $crate::options::diagnostics_enabled() {
            $crate::framebuffer::console_write(core::format_args!($($arg)*));
        }
    };
}

#[macro_export]
macro_rules! fb_diagln {
    () => {
        if $crate::options::diagnostics_enabled() {
            $crate::framebuffer::console_write(core::format_args!("\n"));
        }
    };
    ($fmt:expr $(, $arg:tt)*) => {
        if $crate::options::diagnostics_enabled() {
            $crate::framebuffer::console_write(core::format_args!(concat!($fmt, "\n") $(, $arg)*));
        }
    };
}

pub fn clear_framebuffer(fb: &Framebuffer) -> Result<(), ()> {
    draw::clear_black(fb)
}

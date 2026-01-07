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

pub unsafe fn init_boot_console(fb: Framebuffer, color: FramebufferColor) -> Result<(), ()> {
    let console = text::FramebufferConsole::new(fb, 0, FONT_HEIGHT, color);

    if !console.is_usable() {
        return Err(());
    }

    unsafe {
        *BOOT_CONSOLE_STORAGE.0.get() = Some(console);
    }

    Ok(())
}

pub(crate) fn console_write(args: fmt::Arguments<'_>) -> fmt::Result {
    unsafe {
        let storage = &mut *BOOT_CONSOLE_STORAGE.0.get();
        let result = match storage.as_mut() {
            Some(console) => fmt::Write::write_fmt(console, args),
            None => Err(fmt::Error),
        };

        if result.is_err() {
            *storage = None;
        }

        result
    }
}

#[allow(dead_code)]
pub(crate) fn console_available() -> bool {
    unsafe {
        let storage = &*BOOT_CONSOLE_STORAGE.0.get();
        storage.is_some()
    }
}

#[macro_export]
macro_rules! fb_print {
    ($($arg:tt)*) => {
        {
            let _ = $crate::framebuffer::console_write(core::format_args!($($arg)*));
        }
    };
}

#[macro_export]
macro_rules! fb_println {
    () => {
        {
            let _ = $crate::framebuffer::console_write(core::format_args!("\n"));
        }
    };
    ($fmt:expr $(, $arg:tt)*) => {
        {
            let _ = $crate::framebuffer::console_write(core::format_args!(concat!($fmt, "\n") $(, $arg)*));
        }
    };
}

#[macro_export]
macro_rules! fb_diag {
    ($($arg:tt)*) => {
        {
            if $crate::options::diagnostics_enabled() {
                let _ = $crate::framebuffer::console_write(core::format_args!($($arg)*));
            }
        }
    };
}

#[macro_export]
macro_rules! fb_diagln {
    () => {
        {
            if $crate::options::diagnostics_enabled() {
                let _ = $crate::framebuffer::console_write(core::format_args!("\n"));
            }
        }
    };
    ($fmt:expr $(, $arg:tt)*) => {
        {
            if $crate::options::diagnostics_enabled() {
                let _ = $crate::framebuffer::console_write(core::format_args!(concat!($fmt, "\n") $(, $arg)*));
            }
        }
    };
}

pub fn clear_framebuffer(fb: &Framebuffer) -> Result<(), ()> {
    draw::clear_black(fb)
}

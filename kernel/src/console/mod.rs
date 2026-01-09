//! Framebuffer-backed kernel console with timestamped history.

use core::{cell::UnsafeCell, cmp::min, fmt, mem};

use oxide_abi::Framebuffer;

use crate::{
    framebuffer::{self, FramebufferColor},
    time,
};

const MAX_LINE_CHARS: usize = 160;
const HISTORY_CAPACITY: usize = 128;
const TIMESTAMP_PREFIX_MAX: usize = 32;
#[derive(Clone, Copy)]
struct LineSlot {
    len: u16,
    timestamp: Timestamp,
    data: [u8; MAX_LINE_CHARS],
}

impl LineSlot {
    const EMPTY: Self = Self {
        len: 0,
        timestamp: Timestamp::ZERO,
        data: [0; MAX_LINE_CHARS],
    };

    fn write(&mut self, timestamp: Timestamp, line: &[u8]) {
        self.timestamp = timestamp;
        let copy_len = min(line.len(), MAX_LINE_CHARS);
        self.len = copy_len as u16;
        self.data[..copy_len].copy_from_slice(&line[..copy_len]);
        if copy_len < MAX_LINE_CHARS {
            self.data[copy_len..].fill(0);
        }
    }
}

/// Backing storage for the console's persistent line history.
pub struct ConsoleStorage {
    slots: &'static mut [LineSlot],
}

impl ConsoleStorage {
    /// Returns the number of bytes required to store the history buffer.
    pub const fn required_bytes() -> usize {
        HISTORY_CAPACITY * mem::size_of::<LineSlot>()
    }

    /// Interpret the physical memory at `start` as console storage.
    ///
    /// # Safety
    /// The caller must guarantee the region is appropriately sized and mapped
    /// for exclusive console use.
    pub unsafe fn from_physical(start: u64) -> Self {
        let ptr = start as *mut LineSlot;
        let slots = unsafe { core::slice::from_raw_parts_mut(ptr, HISTORY_CAPACITY) };
        for slot in slots.iter_mut() {
            *slot = LineSlot::EMPTY;
        }
        Self { slots }
    }

    fn into_slots(self) -> &'static mut [LineSlot] {
        self.slots
    }
}

/// Errors produced when initialising the global console.
#[derive(Debug)]
pub enum ConsoleInitError {
    AlreadyInitialized,
    FramebufferUnavailable,
}

struct ConsoleCell(UnsafeCell<Option<ConsoleState>>);

unsafe impl Sync for ConsoleCell {}

static CONSOLE_STATE: ConsoleCell = ConsoleCell(UnsafeCell::new(None));

/// Install the framebuffer console using the provided storage and colour.
pub fn init(
    framebuffer: Framebuffer,
    color: FramebufferColor,
    storage: ConsoleStorage,
) -> Result<(), ConsoleInitError> {
    unsafe {
        let slot = &mut *CONSOLE_STATE.0.get();
        if slot.is_some() {
            return Err(ConsoleInitError::AlreadyInitialized);
        }

        let mut console = framebuffer::text::FramebufferConsole::new(
            framebuffer,
            0,
            framebuffer::FONT_HEIGHT,
            color,
        );

        if !console.is_usable() {
            return Err(ConsoleInitError::FramebufferUnavailable);
        }

        console
            .clear()
            .map_err(|_| ConsoleInitError::FramebufferUnavailable)?;

        let state = ConsoleState::new(console, storage.into_slots());
        *slot = Some(state);

        Ok(())
    }
}

/// Forward formatted output into the global console, if initialised.
pub fn write(args: fmt::Arguments<'_>) -> fmt::Result {
    unsafe {
        let state_slot = &mut *CONSOLE_STATE.0.get();
        let state = state_slot.as_mut().ok_or(fmt::Error)?;
        state.write_fmt(args)
    }
}

struct ConsoleState {
    fb: framebuffer::text::FramebufferConsole,
    history: History,
    line: LineBuffer,
    current_column: usize,
    columns: usize,
    current_timestamp: Option<Timestamp>,
}

impl ConsoleState {
    fn new(fb: framebuffer::text::FramebufferConsole, slots: &'static mut [LineSlot]) -> Self {
        let columns = fb.cols();
        Self {
            fb,
            history: History::new(slots),
            line: LineBuffer::new(),
            current_column: 0,
            columns,
            current_timestamp: None,
        }
    }

    fn write_fmt(&mut self, args: fmt::Arguments<'_>) -> fmt::Result {
        let mut writer = ConsoleWriter { state: self };
        fmt::write(&mut writer, args)
    }

    fn handle_str(&mut self, s: &str) -> Result<(), ()> {
        for byte in s.bytes() {
            self.handle_byte(byte)?;
        }
        Ok(())
    }

    fn handle_byte(&mut self, byte: u8) -> Result<(), ()> {
        let sanitized = framebuffer::text::sanitize_byte(byte);
        match sanitized {
            b'\n' => {
                self.ensure_line_prefix()?;
                self.fb.write_bytes(&[sanitized])?;
                self.finish_line();
            }
            b'\r' => {
                self.fb.write_bytes(&[sanitized])?;
                self.line.clear();
                self.current_column = 0;
                self.current_timestamp = None;
            }
            _ => {
                self.ensure_line_prefix()?;

                if self.columns > 0 && self.current_column >= self.columns {
                    self.finish_line();
                    self.ensure_line_prefix()?;
                }

                if self.line.len() < MAX_LINE_CHARS {
                    self.line.push(sanitized);
                }

                self.fb.write_bytes(&[sanitized])?;

                if self.columns > 0 {
                    self.current_column = self.current_column.saturating_add(1);
                }
            }
        }

        Ok(())
    }

    fn finish_line(&mut self) {
        let timestamp = self
            .current_timestamp
            .unwrap_or_else(|| self.capture_timestamp());

        let line = self.line.as_slice();
        self.history.push(timestamp, line);
        self.line.clear();
        self.current_column = 0;
        self.current_timestamp = None;
    }

    fn ensure_line_prefix(&mut self) -> Result<(), ()> {
        if self.line.len() == 0 {
            let timestamp = self
                .current_timestamp
                .unwrap_or_else(|| self.capture_timestamp());
            self.current_timestamp = Some(timestamp);

            let mut prefix_buf = [0u8; TIMESTAMP_PREFIX_MAX];
            let prefix_len = format_timestamp_prefix(&mut prefix_buf, timestamp);

            if self.line.len() < MAX_LINE_CHARS {
                let available = MAX_LINE_CHARS - self.line.len();
                let copy_len = prefix_len.min(available);
                self.line.extend_from_slice(&prefix_buf[..copy_len]);
            }

            self.fb.write_bytes(&prefix_buf[..prefix_len])?;

            if self.columns > 0 {
                self.current_column = self
                    .current_column
                    .saturating_add(prefix_len)
                    .min(self.columns);

                if self.current_column >= self.columns {
                    self.finish_line();
                    return Ok(());
                }
            }
        }

        Ok(())
    }

    fn capture_timestamp(&self) -> Timestamp {
        if let Some(nanos) = time::monotonic_nanos() {
            Timestamp {
                value: nanos,
                is_nanos: true,
            }
        } else {
            let ticks = time::monotonic_ticks().unwrap_or(0);
            Timestamp {
                value: ticks,
                is_nanos: false,
            }
        }
    }
}

struct ConsoleWriter<'a> {
    state: &'a mut ConsoleState,
}

impl fmt::Write for ConsoleWriter<'_> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.state.handle_str(s).map_err(|_| fmt::Error)
    }
}

struct History {
    slots: &'static mut [LineSlot],
    start: usize,
    len: usize,
}

impl History {
    fn new(slots: &'static mut [LineSlot]) -> Self {
        Self {
            slots,
            start: 0,
            len: 0,
        }
    }

    fn push(&mut self, timestamp: Timestamp, line: &[u8]) {
        if self.slots.is_empty() {
            return;
        }

        let capacity = self.slots.len();
        let index = if self.len < capacity {
            (self.start + self.len) % capacity
        } else {
            self.start
        };

        self.slots[index].write(timestamp, line);

        if self.len < capacity {
            self.len += 1;
        } else {
            self.start = (self.start + 1) % capacity;
        }
    }
}

struct LineBuffer {
    data: [u8; MAX_LINE_CHARS],
    len: usize,
}

impl LineBuffer {
    const fn new() -> Self {
        Self {
            data: [0; MAX_LINE_CHARS],
            len: 0,
        }
    }

    fn push(&mut self, byte: u8) {
        if self.len < MAX_LINE_CHARS {
            self.data[self.len] = byte;
            self.len += 1;
        }
    }

    fn clear(&mut self) {
        self.len = 0;
    }

    fn len(&self) -> usize {
        self.len
    }

    fn as_slice(&self) -> &[u8] {
        &self.data[..self.len]
    }

    fn extend_from_slice(&mut self, bytes: &[u8]) {
        let available = MAX_LINE_CHARS.saturating_sub(self.len);
        let copy_len = bytes.len().min(available);
        if copy_len == 0 {
            return;
        }

        let start = self.len;
        let end = start + copy_len;
        self.data[start..end].copy_from_slice(&bytes[..copy_len]);
        self.len += copy_len;
    }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {{
        let _ = $crate::console::write(core::format_args!($($arg)*));
    }};
}

#[macro_export]
macro_rules! println {
    () => {{
        let _ = $crate::console::write(core::format_args!("\n"));
    }};
    ($fmt:expr $(, $arg:expr)* $(,)?) => {{
        let _ = $crate::console::write(core::format_args!(concat!($fmt, "\n") $(, $arg)*));
    }};
}

#[macro_export]
macro_rules! diag {
    ($($arg:tt)*) => {{
        if $crate::options::diagnostics_enabled() {
            let _ = $crate::console::write(core::format_args!($($arg)*));
        }
    }};
}

#[macro_export]
macro_rules! diagln {
    () => {{
        if $crate::options::diagnostics_enabled() {
            let _ = $crate::console::write(core::format_args!("\n"));
        }
    }};
    ($fmt:expr $(, $arg:expr)* $(,)?) => {{
        if $crate::options::diagnostics_enabled() {
            let _ = $crate::console::write(core::format_args!(concat!($fmt, "\n") $(, $arg)*));
        }
    }};
}

#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {{
        if $crate::options::debug_enabled() {
            let _ = $crate::console::write(core::format_args!($($arg)*));
        }
    }};
}

#[macro_export]
macro_rules! debugln {
    () => {{
        if $crate::options::debug_enabled() {
            let _ = $crate::console::write(core::format_args!("\n"));
        }
    }};
    ($fmt:expr $(, $arg:expr)* $(,)?) => {{
        if $crate::options::debug_enabled() {
            let _ = $crate::console::write(core::format_args!(concat!($fmt, "\n") $(, $arg)*));
        }
    }};
}

#[macro_export]
macro_rules! debug_structured {
    ($fmt:expr, [$(( $key:expr, $value:expr )),* $(,)?] $(, $arg:expr)*) => {{
        if $crate::options::debug_enabled() {
            $crate::println!($fmt $(, $arg)*);
            $(
                $crate::println!("  {}={}", $key, $value);
            )*
        }
    }};
}

fn format_timestamp_prefix(buf: &mut [u8; TIMESTAMP_PREFIX_MAX], timestamp: Timestamp) -> usize {
    let mut index = 0;
    buf[index] = b'[';
    index += 1;

    if timestamp.is_nanos {
        let seconds = timestamp.value / 1_000_000_000;
        let micros = ((timestamp.value % 1_000_000_000) / 1_000) as u32;

        index += write_decimal(&mut buf[index..], seconds);
        buf[index] = b'.';
        index += 1;
        index += write_fixed_width_decimal(&mut buf[index..], micros, 6);
    } else {
        index += write_decimal(&mut buf[index..], timestamp.value);
    }

    buf[index] = b']';
    index += 1;
    buf[index] = b' ';
    index += 1;
    index
}

fn write_decimal(out: &mut [u8], mut value: u64) -> usize {
    let mut tmp = [0u8; 20];
    let mut digits = 0;

    if value == 0 {
        tmp[0] = b'0';
        digits = 1;
    } else {
        while value > 0 {
            let digit = (value % 10) as u8;
            tmp[digits] = b'0' + digit;
            digits += 1;
            value /= 10;
        }
    }

    for i in 0..digits {
        out[i] = tmp[digits - 1 - i];
    }

    digits
}

fn write_fixed_width_decimal(out: &mut [u8], mut value: u32, width: usize) -> usize {
    for i in 0..width {
        let digit = (value % 10) as u8;
        out[width - 1 - i] = b'0' + digit;
        value /= 10;
    }
    width
}

#[derive(Clone, Copy)]
struct Timestamp {
    value: u64,
    is_nanos: bool,
}

impl Timestamp {
    const ZERO: Self = Self {
        value: 0,
        is_nanos: false,
    };
}

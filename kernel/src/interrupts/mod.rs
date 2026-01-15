//! Interrupt Descriptor Table setup and gate management primitives.
//!
use core::cell::UnsafeCell;
use core::sync::atomic::{AtomicBool, Ordering};
use core::{arch::asm, mem::size_of};

/// Total number of entries supported by the Interrupt Descriptor Table.
const IDT_ENTRIES: usize = 256;

#[repr(C, align(16))]
/// In-memory representation of the Interrupt Descriptor Table (IDT).
pub struct Idt {
    entries: [IdtEntry; IDT_ENTRIES],
}

static IDT_CONFIGURED: AtomicBool = AtomicBool::new(false);
static IDT_STORAGE: IdtSlot = IdtSlot::new();

struct IdtSlot(UnsafeCell<Idt>);

unsafe impl Sync for IdtSlot {}

impl IdtSlot {
    const fn new() -> Self {
        Self(UnsafeCell::new(Idt::new()))
    }

    unsafe fn with_mut<R>(&self, f: impl FnOnce(&mut Idt) -> R) -> R {
        let ptr = self.0.get();
        unsafe { f(&mut *ptr) }
    }

    unsafe fn load(&self) {
        let ptr = self.0.get();
        unsafe { (&*ptr).load() }
    }
}

/// Errors that can occur while installing the IDT.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InterruptInitError {
    /// Attempted to initialise the IDT more than once.
    AlreadyInitialized,
}

/// Prepare and load the Interrupt Descriptor Table for the calling CPU.
///
/// The IDT entries are configured exactly once (on the first caller) and the
/// finalised table is then loaded for every core that invokes this routine.
/// The optional `core_index` allows the caller to log which CPU performed the
/// load; pass `None` when initialising from the bootstrap processor.
pub fn init(core_index: Option<usize>) -> Result<(), InterruptInitError> {
    let code_selector = read_cs();

    let first_config = IDT_CONFIGURED
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_ok();

    unsafe {
        if first_config {
            IDT_STORAGE.with_mut(|idt| {
                configure_exceptions(idt, code_selector);
                configure_irqs(idt, code_selector);
            });
        }
        IDT_STORAGE.load();
    }

    log_installation(first_config, core_index);

    Ok(())
}

impl Idt {
    /// Constructs a new IDT with all gates marked as missing.
    pub const fn new() -> Self {
        Self {
            entries: [IdtEntry::missing(); IDT_ENTRIES],
        }
    }

    /// Installs a handler for the supplied interrupt vector.
    pub fn set_gate(
        &mut self,
        vector: u8,
        handler: InterruptHandler,
        selector: u16,
        options: GateOptions,
    ) {
        self.entries[vector as usize] = IdtEntry::new(handler.address(), selector, options);
    }

    /// Removes any handler assigned to the supplied interrupt vector.
    pub fn clear_gate(&mut self, vector: u8) {
        self.entries[vector as usize] = IdtEntry::missing();
    }

    /// # Safety
    /// Caller must ensure the table remains valid for the lifetime of the active CPU.
    ///
    /// Loads the IDT register for the current CPU with the table represented by `self`.
    pub unsafe fn load(&self) {
        let pointer = IdtPointer::new(&self.entries);
        // Load the IDT register with the supplied table pointer.
        // SAFETY: caller ensures the IDT lives for the lifetime of the CPU table.
        unsafe {
            asm!("lidt [{0}]", in(reg) &pointer, options(nostack, preserves_flags));
        }
    }
}

impl Default for Idt {
    fn default() -> Self {
        Self::new()
    }
}

#[repr(C, packed)]
struct IdtPointer {
    limit: u16,
    base: u64,
}

impl IdtPointer {
    /// Builds a pointer suitable for the `lidt` instruction from the supplied entries.
    fn new(entries: &[IdtEntry; IDT_ENTRIES]) -> Self {
        let size = size_of::<IdtEntry>() * entries.len();
        debug_assert!(size > 0 && size <= u16::MAX as usize + 1);
        Self {
            limit: (size - 1) as u16,
            base: entries.as_ptr() as u64,
        }
    }
}

#[derive(Clone, Copy)]
/// Programmable attributes associated with an IDT gate.
///
/// The `type_attr` field mirrors the layout defined by the x86_64 architecture:
///
/// ```text
///  7   6   5   4   3   2   1   0
/// +---+---+---+---+-----------+
/// | P | DPL | 0 |  Gate Type |
/// +---+---+---+---+-----------+
/// ```
///
/// - `P` (bit 7) flags whether the descriptor is present.
/// - `DPL` (bits 5-6) encodes the descriptor privilege level.
/// - `Gate Type` (bits 0-3) selects the gate variant (0xE interrupt, 0xF trap).
///
/// Bits 4 and 7 default to `1` in constructors so the gate is active in ring 0 unless
/// overridden with the builder-style helpers.
pub struct GateOptions {
    type_attr: u8,
    ist: u8,
}

impl GateOptions {
    /// Creates options for a present interrupt gate with DPL0 and an implicit clear of the interrupt flag.
    /// Sets `type_attr` to `0b1000_1110` (`present = 1`, `dpl = 0`, gate type = `0xE`).
    pub const fn interrupt() -> Self {
        Self {
            type_attr: 0b1000_1110,
            ist: 0,
        }
    }

    /// Creates options for a present trap gate with DPL0 that leaves the interrupt flag untouched.
    /// Sets `type_attr` to `0b1000_1111` (`present = 1`, `dpl = 0`, gate type = `0xF`).
    pub const fn trap() -> Self {
        Self {
            type_attr: 0b1000_1111,
            ist: 0,
        }
    }

    /// Overrides the descriptor privilege level (DPL) field (bits 5-6 of `type_attr`).
    pub const fn with_privilege(self, dpl: u8) -> Self {
        let cleared = self.type_attr & !0b0110_0000;
        let updated = cleared | ((dpl & 0b11) << 5);
        Self {
            type_attr: updated,
            ..self
        }
    }

    /// Marks the gate as present or not present (bit 7 of `type_attr`).
    pub const fn with_present(self, present: bool) -> Self {
        let attr = if present {
            self.type_attr | 0b1000_0000
        } else {
            self.type_attr & !0b1000_0000
        };
        Self {
            type_attr: attr,
            ..self
        }
    }

    /// Selects an Interrupt Stack Table entry to use when the gate triggers.
    pub const fn with_ist(self, ist_index: u8) -> Self {
        Self {
            ist: ist_index & 0b111,
            ..self
        }
    }
}

/// Wrapper that stores the address of an interrupt handler entry point.
pub struct InterruptHandler {
    addr: usize,
}

impl InterruptHandler {
    /// Creates a handler from a raw function pointer.
    pub const fn new(addr: usize) -> Self {
        Self { addr }
    }

    /// Converts an `extern "C"` function into a handler wrapper.
    pub fn from_fn(handler: extern "C" fn()) -> Self {
        Self {
            addr: handler as usize,
        }
    }

    const fn address(self) -> usize {
        self.addr
    }
}

impl From<extern "C" fn()> for InterruptHandler {
    fn from(handler: extern "C" fn()) -> Self {
        Self::from_fn(handler)
    }
}

#[repr(C, packed)]
#[derive(Clone, Copy)]
struct IdtEntry {
    offset_low: u16,
    selector: u16,
    ist: u8,
    type_attr: u8,
    offset_mid: u16,
    offset_high: u32,
    zero: u32,
}

impl IdtEntry {
    const fn missing() -> Self {
        Self {
            offset_low: 0,
            selector: 0,
            ist: 0,
            type_attr: 0,
            offset_mid: 0,
            offset_high: 0,
            zero: 0,
        }
    }

    fn new(handler: usize, selector: u16, options: GateOptions) -> Self {
        let handler = handler as u64;
        let offset_low = handler as u16;
        let offset_mid = (handler >> 16) as u16;
        let offset_high = (handler >> 32) as u32;
        Self {
            offset_low,
            selector,
            ist: options.ist,
            type_attr: options.type_attr,
            offset_mid,
            offset_high,
            zero: 0,
        }
    }
}

/// Reads the current code segment selector.
fn read_cs() -> u16 {
    let selector: u16;
    unsafe {
        asm!("mov {0:x}, cs", out(reg) selector, options(nomem, preserves_flags));
    }
    selector
}

/// Configure architectural exception vectors with simple fatal handlers.
fn configure_exceptions(idt: &mut Idt, selector: u16) {
    install_gate(
        idt,
        0x00,
        divide_error_handler,
        selector,
        GateOptions::interrupt(),
    );
    install_gate(idt, 0x03, breakpoint_handler, selector, GateOptions::trap());
    install_gate(
        idt,
        0x06,
        invalid_opcode_handler,
        selector,
        GateOptions::interrupt(),
    );
    install_gate(
        idt,
        0x08,
        double_fault_handler,
        selector,
        GateOptions::interrupt(),
    );
    install_gate(
        idt,
        0x0D,
        general_protection_handler,
        selector,
        GateOptions::interrupt(),
    );
    install_gate(
        idt,
        0x0E,
        page_fault_handler,
        selector,
        GateOptions::interrupt(),
    );
}

/// Configure a minimal set of legacy IRQ vectors with diagnostic stubs.
fn configure_irqs(idt: &mut Idt, selector: u16) {
    install_gate(idt, 0x20, timer_handler, selector, GateOptions::interrupt());
    install_gate(
        idt,
        0x21,
        keyboard_handler,
        selector,
        GateOptions::interrupt(),
    );
}

fn install_gate(
    idt: &mut Idt,
    vector: u8,
    handler: extern "C" fn(),
    selector: u16,
    options: GateOptions,
) {
    idt.set_gate(
        vector,
        InterruptHandler::from_fn(handler),
        selector,
        options.with_present(true),
    );
}

fn log_installation(first_config: bool, core_index: Option<usize>) {
    match (first_config, core_index) {
        (true, Some(core)) => {
            crate::diagln!("IDT configured for core {}.", core);
        }
        (true, None) => {
            crate::diagln!("IDT configured for bootstrap core.");
        }
        (false, Some(core)) => {
            crate::debug!("IDT loaded for core {}.\n", core);
        }
        (false, None) => {
            crate::debug!("IDT loaded for bootstrap core.\n");
        }
    }
}

fn report_fatal_trap(name: &str, vector: u8) {
    crate::println!("EXCEPTION: {}", name);
    crate::diagln!("Trap vector: {:#04x}", vector);

    if vector == 0x0E {
        let fault_addr = read_cr2();
        crate::diagln!("Fault address (CR2): {:#018x}", fault_addr);
        crate::diagln!("Page-fault error code capture not yet implemented.");
    }

    crate::diagln!("Register dump unavailable (handler stubs pending full context capture).");
}

#[cold]
extern "C" fn divide_error_handler() {
    report_fatal_trap("Divide Error", 0x00);
    halt_cpu();
}

#[cold]
extern "C" fn invalid_opcode_handler() {
    report_fatal_trap("Invalid Opcode", 0x06);
    halt_cpu();
}

#[cold]
extern "C" fn double_fault_handler() {
    report_fatal_trap("Double Fault", 0x08);
    halt_cpu();
}

#[cold]
extern "C" fn general_protection_handler() {
    report_fatal_trap("General Protection Fault", 0x0D);
    halt_cpu();
}

#[cold]
extern "C" fn page_fault_handler() {
    report_fatal_trap("Page Fault", 0x0E);
    halt_cpu();
}

extern "C" fn breakpoint_handler() {
    crate::debug!("Breakpoint interrupt\n");
}

extern "C" fn timer_handler() {
    crate::debug!("Timer IRQ\n");
}

extern "C" fn keyboard_handler() {
    crate::debug!("Keyboard IRQ\n");
}

fn halt_cpu() -> ! {
    crate::println!("Halting CPU.");
    loop {
        unsafe {
            asm!("hlt", options(nomem, nostack));
        }
    }
}

fn read_cr2() -> u64 {
    let value: u64;
    unsafe {
        asm!("mov {0}, cr2", out(reg) value, options(nomem, preserves_flags));
    }
    value
}

#[cfg(test)]
extern crate std;

mod tests {
    #[allow(dead_code)]
    extern "C" fn dummy_handler() {}

    #[test]
    fn gate_options_interrupt_defaults() {
        let options = super::GateOptions::interrupt();
        assert_eq!(options.type_attr, 0b1000_1110);
        assert_eq!(options.ist, 0);
    }

    #[test]
    fn gate_options_trap_defaults() {
        let options = super::GateOptions::trap();
        assert_eq!(options.type_attr, 0b1000_1111);
        assert_eq!(options.ist, 0);
    }

    #[test]
    fn gate_options_with_privilege_updates_dpl_bits() {
        let options = super::GateOptions::interrupt().with_privilege(3);
        assert_eq!(options.type_attr & 0b0110_0000, 0b0110_0000);
    }

    #[test]
    fn gate_options_with_present_clears_bit_when_false() {
        let options = super::GateOptions::interrupt().with_present(false);
        assert_eq!(options.type_attr & 0b1000_0000, 0);
    }

    #[test]
    fn gate_options_with_ist_masks_to_three_bits() {
        let options = super::GateOptions::interrupt().with_ist(0b1010);
        assert_eq!(options.ist, 0b010);
    }

    #[test]
    fn interrupt_handler_from_fn_tracks_address() {
        let handler = super::InterruptHandler::from_fn(dummy_handler);
        assert_eq!(handler.addr, dummy_handler as usize);
    }

    #[test]
    fn interrupt_handler_new_tracks_address() {
        let handler = super::InterruptHandler::new(dummy_handler as usize);
        assert_eq!(handler.addr, dummy_handler as usize);
    }

    #[test]
    fn idt_new_initialises_all_entries_missing() {
        let idt = super::Idt::new();
        for entry in idt.entries.iter().copied() {
            let super::IdtEntry {
                offset_low,
                selector,
                ist,
                type_attr,
                offset_mid,
                offset_high,
                ..
            } = entry;
            assert_eq!(offset_low, 0);
            assert_eq!(selector, 0);
            assert_eq!(ist, 0);
            assert_eq!(type_attr, 0);
            assert_eq!(offset_mid, 0);
            assert_eq!(offset_high, 0);
        }
    }

    #[test]
    fn idt_set_gate_populates_vector_entry() {
        let mut idt = super::Idt::new();
        let selector = 0x0028u16;
        let options = super::GateOptions::interrupt().with_privilege(2);
        let type_attr = options.type_attr;
        let ist = options.ist;

        idt.set_gate(
            0x21,
            super::InterruptHandler::from_fn(dummy_handler),
            selector,
            options,
        );

        let entry = idt.entries[0x21];
        let handler_addr = dummy_handler as usize as u64;
        let super::IdtEntry {
            selector: actual_selector,
            type_attr: actual_attr,
            ist: actual_ist,
            offset_low,
            offset_mid,
            offset_high,
            ..
        } = entry;
        assert_eq!(actual_selector, selector);
        assert_eq!(actual_attr, type_attr);
        assert_eq!(actual_ist, ist);
        assert_eq!(offset_low as u64, handler_addr & 0xFFFF);
        assert_eq!(offset_mid as u64, (handler_addr >> 16) & 0xFFFF);
        assert_eq!(offset_high as u64, (handler_addr >> 32) & 0xFFFF_FFFF);
    }

    #[test]
    fn idt_clear_gate_resets_vector_entry() {
        let mut idt = super::Idt::new();
        let selector = 0x0028u16;
        idt.set_gate(
            0x10,
            super::InterruptHandler::from_fn(dummy_handler),
            selector,
            super::GateOptions::interrupt(),
        );
        idt.clear_gate(0x10);

        let entry = idt.entries[0x10];
        let super::IdtEntry {
            selector,
            type_attr,
            offset_low,
            offset_mid,
            offset_high,
            ..
        } = entry;
        assert_eq!(selector, 0);
        assert_eq!(type_attr, 0);
        assert_eq!(offset_low, 0);
        assert_eq!(offset_mid, 0);
        assert_eq!(offset_high, 0);
    }

    #[test]
    fn idt_pointer_new_matches_entry_slice_layout() {
        let entries = [super::IdtEntry::missing(); super::IDT_ENTRIES];
        let pointer = super::IdtPointer::new(&entries);
        let expected_limit =
            (core::mem::size_of::<super::IdtEntry>() * super::IDT_ENTRIES - 1) as u16;
        let expected_base = entries.as_ptr() as u64;
        let super::IdtPointer { limit, base } = pointer;
        assert_eq!(limit, expected_limit);
        assert_eq!(base, expected_base);
    }

    #[test]
    fn install_gate_sets_present_bit() {
        let mut idt = super::Idt::new();
        let selector = 0x0030u16;
        let options = super::GateOptions::interrupt().with_present(false);

        super::install_gate(&mut idt, 0x40, dummy_handler, selector, options);

        let entry = idt.entries[0x40];
        let super::IdtEntry {
            selector: actual_selector,
            type_attr,
            ..
        } = entry;
        assert_eq!(actual_selector, selector);
        assert_ne!(type_attr & 0b1000_0000, 0);
    }

    #[test]
    fn configure_exceptions_installs_expected_vectors() {
        let mut idt = super::Idt::new();
        let selector = 0x0040u16;
        super::configure_exceptions(&mut idt, selector);

        let expected = [
            (0x00u8, super::GateOptions::interrupt()),
            (0x03u8, super::GateOptions::trap()),
            (0x06u8, super::GateOptions::interrupt()),
            (0x08u8, super::GateOptions::interrupt()),
            (0x0Du8, super::GateOptions::interrupt()),
            (0x0Eu8, super::GateOptions::interrupt()),
        ];

        for (vector, opts) in expected {
            let entry = idt.entries[vector as usize];
            let super::IdtEntry {
                selector: actual_selector,
                type_attr,
                offset_low,
                offset_mid,
                offset_high,
                ..
            } = entry;
            assert_eq!(actual_selector, selector);
            assert_eq!(type_attr, opts.type_attr);
            assert!(offset_low != 0 || offset_mid != 0 || offset_high != 0);
        }
    }

    #[test]
    fn configure_irqs_installs_expected_vectors() {
        let mut idt = super::Idt::new();
        let selector = 0x0050u16;
        super::configure_irqs(&mut idt, selector);

        let expected = [
            (0x20u8, super::GateOptions::interrupt()),
            (0x21u8, super::GateOptions::interrupt()),
        ];

        for (vector, opts) in expected {
            let entry = idt.entries[vector as usize];
            let super::IdtEntry {
                selector: actual_selector,
                type_attr,
                offset_low,
                offset_mid,
                offset_high,
                ..
            } = entry;
            assert_eq!(actual_selector, selector);
            assert_eq!(type_attr, opts.type_attr);
            assert!(offset_low != 0 || offset_mid != 0 || offset_high != 0);
        }
    }

    #[test]
    fn sanity_test() {
        // this should unconditionally pass
        assert_eq!(1, 1);
    }
}

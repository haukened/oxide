#![allow(dead_code)]

use crate::memory::{error::PagingError, frame::FrameAllocator};
use oxide_abi::Framebuffer;

/// 4 KiB page size.
pub const PAGE_SIZE: u64 = 4096;
/// 2 MiB huge page size.
pub const HUGE_PAGE_SIZE: u64 = 2 * 1024 * 1024;

const ENTRIES: usize = 512;

// Page table flags
const PTE_PRESENT: u64 = 1 << 0;
const PTE_WRITABLE: u64 = 1 << 1;
// const PTE_USER: u64 = 1 << 2;
// const PTE_WRITE_THROUGH: u64 = 1 << 3;
// const PTE_CACHE_DISABLE: u64 = 1 << 4;
// const PTE_ACCESSED: u64 = 1 << 5;
// const PTE_DIRTY: u64 = 1 << 6;
const PTE_PS: u64 = 1 << 7; // Page Size (1 = 2MiB at PD level)

// masks and helpers
const ADDR_MASK_4K: u64 = 0x000f_ffff_ffff_f000;
const ADDR_MASK_2M: u64 = 0x000f_ffff_ffe0_0000;

/// A single 4 KiB page table with 512 entries (PML4, PDPT, PD, or PT).
#[repr(C, align(4096))]
struct PageTable {
    entries: [u64; ENTRIES],
}

impl PageTable {
    #[allow(unsafe_op_in_unsafe_fn)]
    unsafe fn zero(&mut self) {
        core::ptr::write_bytes(self.entries.as_mut_ptr(), 0, ENTRIES);
    }
}

/// Mininam interface allocator needs to provide for paging bring-up
pub trait PhysFrameAlloc {
    /// Allocate a single physical frame (4 KiB aligned).
    fn allocate_frame(&mut self) -> Option<u64>;
}

/// Implement PhysFrameAlloc for our FrameAllocator
impl PhysFrameAlloc for FrameAllocator<'_> {
    fn allocate_frame(&mut self) -> Option<u64> {
        self.alloc()
    }
}

/// Build identity-mapped page tables and switch CR3 to them.
///
/// This is designed for UEFI bring-up where long mode + paging already exist.
/// We replace the firmware’s page tables with ours.
///
/// What it maps:
/// - Low identity region `[0, low_bytes)` using 2 MiB pages
/// - The framebuffer physical range using 2 MiB pages
/// - Any additional ranges supplied in `extra_ranges`
///
/// Safety assumptions:
/// - Physical memory is identity-mapped at entry (VA == PA) for the regions we touch
/// - Interrupts are disabled (recommended)
/// - UEFI boot: CR4.PAE=1, EFER.LME=1, paging already enabled
pub unsafe fn install_identity_paging<A: PhysFrameAlloc>(
    alloc: &mut A,
    fb: &Framebuffer,
    low_bytes: u64,
    extra_ranges: &[(u64, u64)],
) -> Result<u64, PagingError> {
    // allocate root tables
    let pml4_phys = alloc.allocate_frame().ok_or(PagingError::OutOfFrames)?;
    let pdpt_phys = alloc.allocate_frame().ok_or(PagingError::OutOfFrames)?;

    let pml4 = phys_as_table_mut(pml4_phys);
    let pdpt = phys_as_table_mut(pdpt_phys);

    unsafe {
        pml4.zero();
        pdpt.zero();
    }

    // Write PML4[0] to point to our PDPT
    pml4.entries[0] = (pdpt_phys & ADDR_MASK_4K) | PTE_PRESENT | PTE_WRITABLE;

    // map low memory region
    map_identity_range_2mib(alloc, pdpt, 0, low_bytes)?;

    // map framebuffer region (may be above low_bytes)
    let fb_start = fb.base_address;
    let fb_end =
        fb.base_address
            .checked_add(fb.buffer_size)
            .ok_or(PagingError::AddressOverflow(
                fb.base_address,
                fb.buffer_size,
            ))?;

    map_identity_range_2mib(alloc, pdpt, fb_start, fb_end)?;

    // map any additional required identity ranges
    for &(start, end) in extra_ranges {
        map_identity_range_2mib(alloc, pdpt, start, end)?;
    }

    // switch to our page tables (flushes TLB)
    load_cr3(pml4_phys);

    // force a full memory barrier after changing page tables
    core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);

    Ok(pml4_phys)
}

fn map_identity_range_2mib<A: PhysFrameAlloc>(
    alloc: &mut A,
    pdpt: &mut PageTable,
    start: u64,
    end: u64,
) -> Result<(), PagingError> {
    if start >= end {
        return Ok(());
    }

    let start_aligned = align_down(start, HUGE_PAGE_SIZE);
    let end_aligned = align_up(end, HUGE_PAGE_SIZE);

    let mut addr = start_aligned;
    while addr < end_aligned {
        // We only wired PML4[0]; that covers the lower canonical half (0..512GiB)
        let pml4_index = ((addr >> 39) & 0x1ff) as usize;

        if pml4_index != 0 {
            return Err(PagingError::UnsupportedAddress(addr));
        }

        let pdpt_index = ((addr >> 30) & 0x1ff) as usize;
        let pd_index = ((addr >> 21) & 0x1ff) as usize;

        let pd_phys = ensure_pd(alloc, pdpt, pdpt_index)?;
        let pd = phys_as_table_mut(pd_phys);

        // Map the 2 MiB page at PD level
        pd.entries[pd_index] = (addr & ADDR_MASK_2M) | PTE_PRESENT | PTE_WRITABLE | PTE_PS;

        addr = addr
            .checked_add(HUGE_PAGE_SIZE)
            .ok_or(PagingError::AddressOverflow(addr, HUGE_PAGE_SIZE))?;
    }

    Ok(())
}

// Ensure PDPT[pdpt_index] exists, allocating if necessary
fn ensure_pd<A: PhysFrameAlloc>(
    alloc: &mut A,
    pdpt: &mut PageTable,
    index: usize,
) -> Result<u64, PagingError> {
    if pdpt.entries[index] & PTE_PRESENT == 0 {
        let pd_phys = alloc.allocate_frame().ok_or(PagingError::OutOfFrames)?;
        let pd = phys_as_table_mut(pd_phys);
        unsafe {
            pd.zero();
        }
        pdpt.entries[index] = (pd_phys & ADDR_MASK_4K) | PTE_PRESENT | PTE_WRITABLE;
    }
    let pd_phys = pdpt.entries[index] & ADDR_MASK_4K;
    Ok(pd_phys)
}

fn phys_as_table_mut(phys: u64) -> &'static mut PageTable {
    let ptr = phys as *mut PageTable;
    unsafe { &mut *ptr }
}

#[inline(always)]
fn align_down(addr: u64, align: u64) -> u64 {
    debug_assert!(align.is_power_of_two());
    addr & !(align - 1)
}

#[inline(always)]
fn align_up(addr: u64, align: u64) -> u64 {
    debug_assert!(align.is_power_of_two());
    (addr + align - 1) & !(align - 1)
}

/// Load CR3 with the physical address of the PML4 table.
/// # Safety: `pml4_phys` must point to a valid PML4 table (4 KiB aligned).
fn load_cr3(pml4_phys: u64) {
    let val = pml4_phys & ADDR_MASK_4K;
    unsafe {
        core::arch::asm!(
            "mov cr3, {0}",
            in(reg) val,
            options(nostack, preserves_flags),
        );
    }
}

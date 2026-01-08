use core::{mem, ptr, slice};

use crate::memory::allocator::{self, ReservedRegion};
use crate::memory::error::{FrameAllocError, MemoryInitError, PagingError};
use crate::memory::frame::{FRAME_SIZE, FrameAllocator, UsableFrameIter};
use crate::memory::map::{descriptor_range, find_descriptor_containing};
use crate::memory::paging::{HUGE_PAGE_SIZE, install_identity_paging};
use oxide_abi::{Framebuffer, MemoryMap};

const LOW_IDENTITY_LIMIT: u64 = 1 * 1024 * 1024 * 1024; // 1 GiB
/// Identity ranges are limited because the install path only needs a few
/// critical regions (map copy, stack, kernel image, occasional extras).
/// This keeps the staging structure stack-allocated with predictable size.
const MAX_IDENTITY_RANGES: usize = 4;

struct IdentityRanges {
    entries: [(u64, u64); MAX_IDENTITY_RANGES],
    len: usize,
}

impl IdentityRanges {
    fn new() -> Self {
        Self {
            entries: [(0, 0); MAX_IDENTITY_RANGES],
            len: 0,
        }
    }

    fn push(&mut self, range: (u64, u64)) -> Result<(), MemoryInitError> {
        if range.0 >= range.1 {
            return Ok(());
        }

        if self.entries[..self.len]
            .iter()
            .any(|&existing| existing == range)
        {
            return Ok(());
        }

        if self.len >= MAX_IDENTITY_RANGES {
            crate::fb_diagln!(
                "IDENTITY RANGE CAP HIT WHILE STAGING [{:#x}, {:#x}]",
                (range.0),
                (range.1)
            );
            return Err(MemoryInitError::IdentityRangeOverflow {
                start: range.0,
                end: range.1,
            });
        }

        self.entries[self.len] = range;
        self.len += 1;
        Ok(())
    }

    fn as_slice(&self) -> &[(u64, u64)] {
        &self.entries[..self.len]
    }
}

const MAX_RESERVATIONS: usize = 8;

struct ReservationList {
    entries: [ReservedRegion; MAX_RESERVATIONS],
    len: usize,
}

impl ReservationList {
    fn new() -> Self {
        Self {
            entries: [ReservedRegion { start: 0, end: 0 }; MAX_RESERVATIONS],
            len: 0,
        }
    }

    fn push(&mut self, range: (u64, u64)) -> Result<(), MemoryInitError> {
        let (start, end) = range;
        if start >= end {
            return Ok(());
        }

        let region = ReservedRegion { start, end };

        if self.entries[..self.len]
            .iter()
            .any(|&existing| existing == region)
        {
            return Ok(());
        }

        if self.len >= MAX_RESERVATIONS {
            crate::fb_diagln!(
                "RESERVATION CAP HIT WHILE STAGING [{:#x}, {:#x}]",
                start,
                end
            );
            return Err(MemoryInitError::IdentityRangeOverflow { start, end });
        }

        self.entries[self.len] = region;
        self.len += 1;
        Ok(())
    }

    fn extend(&mut self, ranges: &[(u64, u64)]) -> Result<(), MemoryInitError> {
        for &range in ranges {
            self.push(range)?;
        }
        Ok(())
    }

    fn as_slice(&self) -> &[ReservedRegion] {
        &self.entries[..self.len]
    }

    fn len(&self) -> usize {
        self.len
    }
}

struct StorageSlice<T: 'static> {
    slice: &'static mut [Option<T>],
    region: ReservedRegion,
}

/// Allocate a slice of `Option<T>` from physical memory frames and expose it as a
/// leaked `'static` reference for the runtime allocator metadata.
///
/// # Safety
/// The caller must ensure that the returned physical range remains identity-mapped
/// and is never reclaimed for other purposes.
unsafe fn carve_option_storage<T: Copy + 'static>(
    allocator: &mut FrameAllocator,
    slots: usize,
) -> Result<StorageSlice<T>, MemoryInitError> {
    debug_assert!(slots > 0);

    let element_size = mem::size_of::<Option<T>>();
    if element_size == 0 {
        return Err(MemoryInitError::TooLarge);
    }

    let bytes = slots
        .checked_mul(element_size)
        .ok_or(MemoryInitError::TooLarge)?;

    let frame_bytes = FRAME_SIZE as usize;
    let frames = ((bytes + frame_bytes - 1) / frame_bytes).max(1);

    let phys_start = allocator
        .alloc_contiguous(frames)
        .map_err(|err| match err {
            FrameAllocError::OutOfFrames => MemoryInitError::OutOfFrames,
            FrameAllocError::NonContiguous { expected, found } => {
                MemoryInitError::NonContiguous { expected, found }
            }
            FrameAllocError::InvalidRequest => MemoryInitError::EmptyMemoryMap,
        })?;

    let phys_end = phys_start + (frames as u64 * FRAME_SIZE);
    let slice_ptr = phys_start as *mut Option<T>;
    let storage = unsafe { slice::from_raw_parts_mut(slice_ptr, slots) };
    storage.fill(None);

    Ok(StorageSlice {
        slice: storage,
        region: ReservedRegion {
            start: phys_start,
            end: phys_end,
        },
    })
}

struct StackInfo {
    descriptor_type: u32,
    range: (u64, u64),
}

pub fn initialize(
    memory_map: &MemoryMap,
    framebuffer: &Framebuffer,
) -> Result<(), MemoryInitError> {
    crate::fb_diagln!("Initializing memory subsystem...");

    ensure_usable_memory(memory_map)?;

    let mut frame_allocator = FrameAllocator::new(memory_map);

    let CopiedMemoryMap {
        map: kernel_memory_map,
        phys_range: map_copy_range,
    } = copy_memory_map(memory_map, &mut frame_allocator)?;

    crate::fb_diagln!("Memory map copied successfully.");

    let rsp = current_stack_pointer();
    crate::fb_diagln!("Current RSP: {:#x}", rsp);

    let stack_info = loader_stack_info(memory_map, rsp)?;

    crate::fb_diagln!(
        "Preserving memory map copy range [{:#x}, {:#x}]",
        (map_copy_range.0),
        (map_copy_range.1)
    );

    let mut identity_ranges = IdentityRanges::new();
    identity_ranges.push(map_copy_range)?;

    let stack_type = stack_info.descriptor_type;
    let (stack_start, stack_end) = stack_info.range;

    crate::fb_diagln!(
        "Preserving loader stack type {:#x} range [{:#x}, {:#x}]",
        stack_type,
        stack_start,
        stack_end
    );

    identity_ranges.push((stack_start, stack_end))?;

    let code_addr = initialize as usize as u64;
    if let Some(((code_start, code_end), code_type)) =
        kernel_code_identity_range(memory_map, code_addr)
    {
        crate::fb_diagln!(
            "Preserving kernel code type {:#x} range [{:#x}, {:#x}]",
            code_type,
            code_start,
            code_end
        );
        identity_ranges.push((code_start, code_end))?;
    } else {
        crate::fb_println!(
            "WARNING: KERNEL CODE ADDRESS {:#x} MISSING FROM MEMORY MAP.",
            code_addr
        );
    }

    let ranges = identity_ranges.as_slice();

    log_identity_alignment(ranges);

    let mut reservations = ReservationList::new();
    reservations.extend(ranges)?;

    let framebuffer_end = framebuffer
        .base_address
        .checked_add(framebuffer.buffer_size)
        .ok_or_else(|| {
            MemoryInitError::Paging(PagingError::AddressOverflow(
                framebuffer.base_address,
                framebuffer.buffer_size,
            ))
        })?;

    reservations.push((framebuffer.base_address, framebuffer_end))?;

    let storage_plan = allocator::runtime_storage_plan(&kernel_memory_map, reservations.len() + 2)
        .map_err(MemoryInitError::Allocator)?;

    let StorageSlice {
        slice: free_storage,
        region: free_region,
    } = unsafe {
        carve_option_storage::<allocator::PhysFrame>(&mut frame_allocator, storage_plan.free_slots)?
    };
    reservations.push((free_region.start, free_region.end))?;

    let StorageSlice {
        slice: reserved_storage,
        region: reserved_region,
    } = unsafe {
        carve_option_storage::<ReservedRegion>(&mut frame_allocator, storage_plan.reserved_slots)?
    };
    reservations.push((reserved_region.start, reserved_region.end))?;

    allocator::initialize_runtime_allocator(
        kernel_memory_map,
        reservations.as_slice(),
        free_storage,
        reserved_storage,
    )?;

    let paging_result = allocator::with_runtime_allocator(|alloc| unsafe {
        install_identity_paging(alloc, framebuffer, LOW_IDENTITY_LIMIT, ranges)
    });

    match paging_result {
        Some(result) => {
            let _cr3 = result.map_err(MemoryInitError::Paging)?;
        }
        None => {
            debug_assert!(false, "runtime allocator unavailable during paging setup");
            return Err(MemoryInitError::AllocatorUnavailable);
        }
    }

    crate::fb_diagln!("Identity paging installed.");

    crate::fb_println!("Memory subsystem initialization complete.");
    Ok(())
}

struct CopiedMemoryMap {
    map: MemoryMap,
    phys_range: (u64, u64),
}

fn copy_memory_map(
    original: &MemoryMap,
    alloc: &mut FrameAllocator,
) -> Result<CopiedMemoryMap, MemoryInitError> {
    if original.map_size == 0 {
        return Err(MemoryInitError::EmptyMemoryMap);
    }

    let map_size = original.map_size;

    if map_size > usize::MAX as u64 {
        return Err(MemoryInitError::TooLarge);
    }
    let frame_count = ((map_size + FRAME_SIZE - 1) / FRAME_SIZE) as usize;

    if frame_count == 0 {
        return Err(MemoryInitError::EmptyMemoryMap);
    }

    let first = alloc
        .alloc_contiguous(frame_count)
        .map_err(|err| match err {
            FrameAllocError::OutOfFrames => MemoryInitError::OutOfFrames,
            FrameAllocError::NonContiguous { expected, found } => {
                MemoryInitError::NonContiguous { expected, found }
            }
            FrameAllocError::InvalidRequest => MemoryInitError::EmptyMemoryMap,
        })?;

    let copy_bytes = map_size as usize;
    let dest_ptr = first as *mut u8;
    let src_ptr = original.descriptors_phys as *const u8;

    unsafe {
        ptr::copy_nonoverlapping(src_ptr, dest_ptr, copy_bytes);
    }

    let mut map = *original;
    map.descriptors_phys = first;

    let phys_end = first + (frame_count as u64 * FRAME_SIZE);

    Ok(CopiedMemoryMap {
        map,
        phys_range: (first, phys_end),
    })
}

fn ensure_usable_memory(memory_map: &MemoryMap) -> Result<(), MemoryInitError> {
    if UsableFrameIter::new(memory_map).next().is_some() {
        Ok(())
    } else {
        crate::fb_println!("No usable memory frames found.");
        Err(MemoryInitError::NoUsableMemory)
    }
}

fn current_stack_pointer() -> u64 {
    let mut rsp: u64;
    unsafe {
        core::arch::asm!("mov {}, rsp", out(reg) rsp);
    }
    rsp
}

fn loader_stack_info(memory_map: &MemoryMap, rsp: u64) -> Result<StackInfo, MemoryInitError> {
    let descriptor = find_descriptor_containing(memory_map, rsp)
        .ok_or(MemoryInitError::StackDescriptorMissing(rsp))?;

    let descriptor_type = descriptor.typ;

    let range =
        descriptor_range(descriptor).ok_or(MemoryInitError::StackRangeOverflow(descriptor_type))?;

    Ok(StackInfo {
        descriptor_type,
        range,
    })
}

fn kernel_code_identity_range(memory_map: &MemoryMap, code_addr: u64) -> Option<((u64, u64), u32)> {
    let descriptor = find_descriptor_containing(memory_map, code_addr)?;
    let range = descriptor_range(descriptor)?;
    Some((range, descriptor.typ))
}

fn log_identity_alignment(ranges: &[(u64, u64)]) {
    for &(start, end) in ranges {
        let aligned_start = start & !(HUGE_PAGE_SIZE - 1);
        let aligned_end = (end + HUGE_PAGE_SIZE - 1) & !(HUGE_PAGE_SIZE - 1);
        crate::fb_diagln!(
            "Mapping identity range [{:#x}, {:#x}] aligned to [{:#x}, {:#x}]",
            start,
            end,
            aligned_start,
            aligned_end
        );
    }
}

use core::ptr;

use crate::memory::frame::{FRAME_SIZE, FrameAllocator, UsableFrameIter};
use crate::memory::map::{descriptor_range, find_descriptor_containing};
use crate::memory::paging::{HUGE_PAGE_SIZE, PagingError, install_identity_paging};
use oxide_abi::{Framebuffer, MemoryMap};

const LOW_IDENTITY_LIMIT: u64 = 1 * 1024 * 1024 * 1024; // 1 GiB
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

struct StackInfo {
    descriptor_type: u32,
    range: (u64, u64),
}

pub fn initialize(
    memory_map: &MemoryMap,
    framebuffer: &Framebuffer,
) -> Result<(), MemoryInitError> {
    crate::fb_println!("Initializing memory subsystem...");

    ensure_usable_memory(memory_map)?;

    let mut allocator = FrameAllocator::new(memory_map);

    let CopiedMemoryMap {
        map: kernel_memory_map,
        phys_range: map_copy_range,
    } = copy_memory_map(memory_map, &mut allocator)?;

    crate::fb_diagln!("Memory map copied successfully.");

    // supress warning until we have a consumer
    let _ = kernel_memory_map;

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

    unsafe {
        install_identity_paging(&mut allocator, framebuffer, LOW_IDENTITY_LIMIT, ranges)
            .map_err(MemoryInitError::Paging)?;
    }

    crate::fb_diagln!("Identity paging installed.");

    crate::fb_println!("Memory subsystem initialization complete.");
    Ok(())
}

#[derive(Clone, Copy, Debug)]
pub enum MemoryInitError {
    NoUsableMemory,
    EmptyMemoryMap,
    OutOfFrames,
    NonContiguous,
    TooLarge,
    StackDescriptorMissing(u64),
    StackRangeOverflow(u32),
    IdentityRangeOverflow { start: u64, end: u64 },
    Paging(PagingError),
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

    let first = alloc.alloc().ok_or(MemoryInitError::OutOfFrames)?;
    let mut prev = first;

    for _ in 1..frame_count {
        let next = alloc.alloc().ok_or(MemoryInitError::OutOfFrames)?;
        if next != prev + FRAME_SIZE {
            return Err(MemoryInitError::NonContiguous);
        }
        prev = next;
    }

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

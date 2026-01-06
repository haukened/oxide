use core::ptr;

use crate::fb_println;
use crate::memory::frame::{FRAME_SIZE, FrameAllocator, UsableFrameIter};
use crate::memory::map::{MemoryMapIter, descriptor_range, find_descriptor_containing};
use crate::memory::paging::{PagingError, install_identity_paging};
use oxide_abi::{Framebuffer, MemoryMap};

const LOW_IDENTITY_LIMIT: u64 = 1 * 1024 * 1024 * 1024; // 1 GiB

pub fn initialize(
    memory_map: &MemoryMap,
    framebuffer: &Framebuffer,
) -> Result<(), MemoryInitError> {
    for desc in MemoryMapIter::new(memory_map) {
        let _ = desc.physical_start;
    }
    fb_println!("Memory map parsed successfully.");

    if UsableFrameIter::new(memory_map).next().is_none() {
        fb_println!("No usable memory frames found.");
        return Err(MemoryInitError::NoUsableMemory);
    }
    fb_println!("Found usable memory frames.");

    let mut allocator = FrameAllocator::new(memory_map);

    let CopiedMemoryMap {
        map: kernel_memory_map,
        phys_range: map_copy_range,
    } = copy_memory_map(memory_map, &mut allocator)?;

    fb_println!("Memory map copied successfully.");

    let _ = kernel_memory_map;

    let mut rsp: u64;
    unsafe {
        core::arch::asm!("mov {}, rsp", out(reg) rsp);
    }
    fb_println!("Current RSP: {:#x}", rsp);

    let loader_descriptor = find_descriptor_containing(memory_map, rsp)
        .ok_or(MemoryInitError::StackDescriptorMissing(rsp))?;

    let descriptor_type = loader_descriptor.typ;

    let (stack_start, stack_end) = descriptor_range(loader_descriptor)
        .ok_or(MemoryInitError::StackRangeOverflow(descriptor_type))?;

    fb_println!(
        "Preserving memory map copy range [{:#x}, {:#x}]",
        (map_copy_range.0),
        (map_copy_range.1)
    );

    const MAX_IDENTITY_RANGES: usize = 4;
    let mut identity_ranges = [(0u64, 0u64); MAX_IDENTITY_RANGES];
    let mut identity_len = 0usize;

    let mut push_range = |range: (u64, u64)| {
        if range.0 >= range.1 {
            return;
        }
        if identity_ranges[..identity_len]
            .iter()
            .any(|&(start, end)| start == range.0 && end == range.1)
        {
            return;
        }
        if identity_len < MAX_IDENTITY_RANGES {
            identity_ranges[identity_len] = range;
            identity_len += 1;
        } else {
            fb_println!("IDENTITY RANGE OVERFLOW WHILE STAGING PAGING.");
        }
    };

    push_range(map_copy_range);

    fb_println!(
        "Preserving loader stack type {:#x} range [{:#x}, {:#x}]",
        descriptor_type,
        stack_start,
        stack_end
    );
    push_range((stack_start, stack_end));

    let code_addr = initialize as usize as u64;
    if let Some(code_desc) = find_descriptor_containing(memory_map, code_addr) {
        if let Some((code_start, code_end)) = descriptor_range(code_desc) {
            fb_println!(
                "Preserving kernel code type {:#x} range [{:#x}, {:#x}]",
                (code_desc.typ),
                code_start,
                code_end
            );
            push_range((code_start, code_end));
        }
    } else {
        fb_println!(
            "WARNING: KERNEL CODE ADDRESS {:#x} MISSING FROM MEMORY MAP.",
            code_addr
        );
    }

    let ranges = &identity_ranges[..identity_len];

    for &(start, end) in ranges {
        let aligned_start = start & !(crate::memory::paging::HUGE_PAGE_SIZE - 1);
        let aligned_end = (end + crate::memory::paging::HUGE_PAGE_SIZE - 1)
            & !(crate::memory::paging::HUGE_PAGE_SIZE - 1);
        fb_println!(
            "Mapping identity range [{:#x}, {:#x}] aligned to [{:#x}, {:#x}]",
            start,
            end,
            aligned_start,
            aligned_end
        );
    }

    unsafe {
        install_identity_paging(&mut allocator, framebuffer, LOW_IDENTITY_LIMIT, ranges)
            .map_err(MemoryInitError::Paging)?;
    }

    fb_println!("Identity paging installed.");

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

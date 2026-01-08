#![allow(dead_code)]

/*
- Tighten error plumbing: replace the internal Result<(), ()> placeholders with proper
  AllocInitError/AllocationError variants, and ensure overflow cases bubble up cleanly.

- Integration work: hook PhysicalAllocator::from_memory_map into memory::init, replace the ad‑hoc
  FrameAllocator once the runtime allocator is ready, and expose a stable API for downstream modules.

- Validation: add unit/struct tests or assertions to cover allocation/free cycles, coalescing, and
  reservation carving.
*/

use crate::memory::{frame::FRAME_SIZE, map::MemoryMapIter};
use core::cmp::{max, min};
use oxide_abi::{EfiMemoryType, MemoryMap};

/// Physical frame identifier capturing a contiguous run of pages.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PhysFrame {
    /// Physical start address of the run (must be page aligned).
    pub start: u64,
    /// Number of 4 KiB frames within the run.
    pub count: u64,
}

impl PhysFrame {
    pub const fn new(start: u64, count: u64) -> Self {
        Self { start, count }
    }
}

/// Represents a region that must remain reserved and unavailable for allocation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReservedRegion {
    pub start: u64,
    pub end: u64,
}

/// Allocation failure reasons surfaced to callers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AllocationError {
    OutOfMemory,
    Alignment,
    UnsupportedSize,
}

/// Errors that can arise while building the runtime allocator from firmware state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AllocInitError {
    Empty,
    InvalidDescriptor,
}

/// Describes the operations supported by the kernel's physical frame allocator.
pub struct PhysicalAllocator<'a> {
    /// Copy of the firmware memory map retained for provenance/debugging.
    map: MemoryMap,
    /// Current list of free physical frame runs managed by the allocator.
    free: FrameRunList<'a>,
    /// Regions that must remain reserved and cannot be handed out.
    reserved: ReservedList<'a>,
}

/// Backing storage wrapper for free frame runs.
struct FrameRunList<'a> {
    entries: &'a mut [Option<PhysFrame>],
    len: usize,
}

impl<'a> FrameRunList<'a> {
    fn new(storage: &'a mut [Option<PhysFrame>]) -> Self {
        storage.fill(None);
        Self {
            entries: storage,
            len: 0,
        }
    }

    fn capacity(&self) -> usize {
        self.entries.len()
    }

    fn len(&self) -> usize {
        self.len
    }

    fn as_slice(&self) -> &[Option<PhysFrame>] {
        &self.entries
    }

    fn push(&mut self, frame: PhysFrame) -> Result<(), ()> {
        if frame.count == 0 {
            return Ok(());
        }

        for slot in self.entries.iter_mut() {
            if slot.is_none() {
                *slot = Some(frame);
                self.len += 1;
                return Ok(());
            }
        }

        Err(())
    }

    fn remove_slot(&mut self, index: usize) {
        if index < self.entries.len() {
            if self.entries[index].take().is_some() {
                self.len = self.len.saturating_sub(1);
            }
        }
    }

    fn insert(&mut self, frame: PhysFrame) -> Result<(), ()> {
        if frame.count == 0 {
            return Ok(());
        }

        let mut new_start = frame.start;
        let mut new_end = span_end(frame.start, frame.count).ok_or(())?;

        for slot in self.entries.iter_mut() {
            if let Some(existing) = slot {
                let existing_start = existing.start;
                let existing_end = match span_end(existing.start, existing.count) {
                    Some(end) => end,
                    None => continue,
                };

                if new_end < existing_start || new_start > existing_end {
                    continue;
                }

                new_start = new_start.min(existing_start);
                new_end = new_end.max(existing_end);
                *slot = None;
                self.len = self.len.saturating_sub(1);
            }
        }

        let span = new_end.checked_sub(new_start).ok_or(())?;
        if span % FRAME_SIZE != 0 {
            return Err(());
        }

        let count = span / FRAME_SIZE;
        self.push(PhysFrame::new(new_start, count))
    }

    fn allocate_count(&mut self, frames: u64) -> Option<PhysFrame> {
        if frames == 0 {
            return None;
        }

        for idx in 0..self.entries.len() {
            let run = match self.entries[idx] {
                Some(run) => run,
                None => continue,
            };

            if run.count < frames {
                continue;
            }

            let alloc = PhysFrame::new(run.start, frames);

            if run.count == frames {
                self.remove_slot(idx);
                return Some(alloc);
            }

            let advance_bytes = match frames.checked_mul(FRAME_SIZE) {
                Some(bytes) => bytes,
                None => continue,
            };

            if let Some(new_start) = run.start.checked_add(advance_bytes) {
                let remaining = run.count - frames;
                self.entries[idx] = Some(PhysFrame::new(new_start, remaining));
                return Some(alloc);
            }
        }

        None
    }

    fn subtract_range(&mut self, start: u64, end: u64) -> Result<(), ()> {
        if start >= end {
            return Ok(());
        }

        let range_start = align_down(start);
        let range_end = align_up(end);

        if range_start >= range_end {
            return Ok(());
        }

        let mut idx = 0;
        while idx < self.entries.len() {
            let run = match self.entries[idx] {
                Some(run) => run,
                None => {
                    idx += 1;
                    continue;
                }
            };

            let run_end = match span_end(run.start, run.count) {
                Some(end) => end,
                None => {
                    self.remove_slot(idx);
                    continue;
                }
            };

            if range_end <= run.start || range_start >= run_end {
                idx += 1;
                continue;
            }

            self.remove_slot(idx);

            let cut_start = max(run.start, range_start);
            let cut_end = min(run_end, range_end);

            if cut_start > run.start {
                let left_bytes = cut_start - run.start;
                if left_bytes % FRAME_SIZE != 0 {
                    return Err(());
                }
                let left_count = left_bytes / FRAME_SIZE;
                if left_count > 0 && self.push(PhysFrame::new(run.start, left_count)).is_err() {
                    return Err(());
                }
            }

            if cut_end < run_end {
                let right_bytes = run_end - cut_end;
                if right_bytes % FRAME_SIZE != 0 {
                    return Err(());
                }
                let right_count = right_bytes / FRAME_SIZE;
                if right_count > 0 && self.push(PhysFrame::new(cut_end, right_count)).is_err() {
                    return Err(());
                }
            }

            // Restart the scan because new segments may have been appended earlier.
            idx = 0;
        }

        Ok(())
    }

    fn iter(&self) -> FreeRegionIter<'_> {
        FreeRegionIter {
            entries: self.as_slice(),
            index: 0,
        }
    }
}

/// Backing storage wrapper for reserved regions.
struct ReservedList<'a> {
    entries: &'a mut [Option<ReservedRegion>],
    len: usize,
}

impl<'a> ReservedList<'a> {
    fn new(storage: &'a mut [Option<ReservedRegion>]) -> Self {
        storage.fill(None);
        Self {
            entries: storage,
            len: 0,
        }
    }

    fn capacity(&self) -> usize {
        self.entries.len()
    }

    fn len(&self) -> usize {
        self.len
    }

    fn as_slice(&self) -> &[Option<ReservedRegion>] {
        &self.entries
    }

    fn push(&mut self, region: ReservedRegion) -> Result<(), ()> {
        if region.start >= region.end {
            return Err(());
        }

        for slot in self.entries.iter_mut() {
            if slot.is_none() {
                *slot = Some(region);
                self.len += 1;
                return Ok(());
            }
        }

        Err(())
    }

    fn iter(&self) -> ReservedRegionIter<'_> {
        ReservedRegionIter {
            entries: self.as_slice(),
            index: 0,
        }
    }
}

impl<'a> PhysicalAllocator<'a> {
    /// Build a runtime allocator using the copied memory map and any regions that must remain
    /// reserved (loader allocations, ACPI, framebuffer, etc.).
    pub fn from_memory_map(
        map: MemoryMap,
        reservations: &[ReservedRegion],
        free_storage: &'a mut [Option<PhysFrame>],
        reserved_storage: &'a mut [Option<ReservedRegion>],
    ) -> Result<Self, AllocInitError> {
        if map.map_size == 0 || map.entry_count == 0 {
            return Err(AllocInitError::Empty);
        }

        let mut free = FrameRunList::new(free_storage);
        for descriptor in MemoryMapIter::new(&map) {
            if descriptor.typ != EfiMemoryType::ConventionalMemory as u32 {
                continue;
            }

            if descriptor.number_of_pages == 0 {
                continue;
            }

            let frame = PhysFrame::new(descriptor.physical_start, descriptor.number_of_pages);
            free.push(frame)
                .map_err(|_| AllocInitError::InvalidDescriptor)?;
        }

        if free.len() == 0 {
            return Err(AllocInitError::Empty);
        }

        let mut reserved = ReservedList::new(reserved_storage);
        for &region in reservations {
            reserved
                .push(region)
                .map_err(|_| AllocInitError::InvalidDescriptor)?;
            free.subtract_range(region.start, region.end)
                .map_err(|_| AllocInitError::InvalidDescriptor)?;
        }

        Ok(Self {
            map,
            free,
            reserved,
        })
    }

    /// Allocate a single 4 KiB frame.
    pub fn allocate(&mut self) -> Result<PhysFrame, AllocationError> {
        self.allocate_order(0)
    }

    /// Allocate `2^order` contiguous frames (order 0 = 1 frame, order 9 = 512 frames / 2 MiB).
    pub fn allocate_order(&mut self, order: u8) -> Result<PhysFrame, AllocationError> {
        let frames = match 1u64.checked_shl(order as u32) {
            Some(count) if count > 0 => count,
            _ => return Err(AllocationError::UnsupportedSize),
        };

        self.free
            .allocate_count(frames)
            .ok_or(AllocationError::OutOfMemory)
    }

    /// Free a previously allocated run of frames.
    pub fn free(&mut self, frame: PhysFrame) {
        if frame.count == 0 {
            return;
        }

        if self.free.insert(frame).is_err() {
            debug_assert!(false, "free list capacity exhausted");
        }
    }

    /// Mark an arbitrary region as reserved after initialization.
    pub fn reserve(&mut self, region: ReservedRegion) {
        if self.reserved.push(region).is_err() {
            debug_assert!(false, "reserved list capacity exhausted");
            return;
        }

        if self.free.subtract_range(region.start, region.end).is_err() {
            debug_assert!(false, "failed to carve reserved region from free list");
        }
    }

    /// Iterate over all free ranges currently tracked by the allocator.
    pub fn free_regions(&self) -> FreeRegionIter<'_> {
        self.free.iter()
    }

    /// Iterate over all reserved ranges currently tracked by the allocator.
    pub fn reserved_regions(&self) -> ReservedRegionIter<'_> {
        self.reserved.iter()
    }
}

/// Iterator over free regions. Placeholder until the backing store is decided.
pub struct FreeRegionIter<'a> {
    entries: &'a [Option<PhysFrame>],
    index: usize,
}

impl<'a> Iterator for FreeRegionIter<'a> {
    type Item = PhysFrame;

    fn next(&mut self) -> Option<Self::Item> {
        while self.index < self.entries.len() {
            let item = self.entries[self.index];
            self.index += 1;
            if let Some(frame) = item {
                return Some(frame);
            }
        }
        None
    }
}

/// Iterator over reserved regions.
pub struct ReservedRegionIter<'a> {
    entries: &'a [Option<ReservedRegion>],
    index: usize,
}

impl<'a> Iterator for ReservedRegionIter<'a> {
    type Item = ReservedRegion;

    fn next(&mut self) -> Option<Self::Item> {
        while self.index < self.entries.len() {
            let item = self.entries[self.index];
            self.index += 1;
            if let Some(region) = item {
                return Some(region);
            }
        }
        None
    }
}

fn span_end(start: u64, count: u64) -> Option<u64> {
    count
        .checked_mul(FRAME_SIZE)
        .and_then(|bytes| start.checked_add(bytes))
}

fn align_down(value: u64) -> u64 {
    let mask = FRAME_SIZE - 1;
    value & !mask
}

fn align_up(value: u64) -> u64 {
    let mask = FRAME_SIZE - 1;
    match value.checked_add(mask) {
        Some(sum) => sum & !mask,
        None => u64::MAX & !mask,
    }
}

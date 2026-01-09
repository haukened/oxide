#![allow(dead_code)]

/*
- Integration work: hook PhysicalAllocator::from_memory_map into memory::init, replace the ad‑hoc
    FrameAllocator once the runtime allocator is ready, and expose a stable API for downstream modules.

- Validation: add unit/struct tests or assertions to cover allocation/free cycles, coalescing, and
    reservation carving.
*/

use crate::memory::{
    error::{PhysAllocError, PhysAllocInitError},
    frame::FRAME_SIZE,
    map::MemoryMapIter,
};
use core::{
    cell::UnsafeCell,
    cmp::{max, min},
};
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

/// Capacities required to host allocator bookkeeping structures.
pub struct StoragePlan {
    pub free_slots: usize,
    pub reserved_slots: usize,
}

impl StoragePlan {
    /// Total number of optional slots required across free and reserved lists.
    pub fn total_slots(&self) -> usize {
        self.free_slots.saturating_add(self.reserved_slots)
    }
}

/// Derive storage requirements for the runtime allocator based on the firmware map
/// and the reservations that must be honored. The number of potential free runs is
/// bounded by the set of descriptor and reservation boundaries, so capacity is sized
/// proportionally without relying on fixed constants.
pub fn runtime_storage_plan(
    map: &MemoryMap,
    reservation_count: usize,
) -> Result<StoragePlan, PhysAllocInitError> {
    if map.map_size == 0 || map.entry_count == 0 {
        crate::diagln!("unable to create storage plan: empty memory map");
        return Err(PhysAllocInitError::Empty);
    }

    // Count the number of conventional memory regions in the map
    let mut conventional_regions = 0usize;
    let count_iter = MemoryMapIter::new(map);
    for descriptor in count_iter {
        if descriptor.typ == EfiMemoryType::ConventionalMemory as u32
            && descriptor.number_of_pages > 0
        {
            conventional_regions += 1;
        }
    }

    if conventional_regions == 0 {
        return Err(PhysAllocInitError::Empty);
    }

    let boundary_count = conventional_regions
        .saturating_add(reservation_count)
        .saturating_mul(2);
    let free_slots = boundary_count.max(conventional_regions);

    let reserved_slots = reservation_count.saturating_add(conventional_regions.max(4));

    crate::debug_structured!(
        "runtime storage plan:",
        [
            ("entries", map.entry_count),
            ("conventional", conventional_regions),
            ("reservations", reservation_count),
        ]
    );

    Ok(StoragePlan {
        free_slots,
        reserved_slots,
    })
}

struct AllocatorCell {
    inner: UnsafeCell<Option<PhysicalAllocator<'static>>>,
}

impl AllocatorCell {
    const fn new() -> Self {
        Self {
            inner: UnsafeCell::new(None),
        }
    }

    fn initialize(
        &self,
        map: MemoryMap,
        reservations: &[ReservedRegion],
        free_storage: &'static mut [Option<PhysFrame>],
        reserved_storage: &'static mut [Option<ReservedRegion>],
    ) -> Result<(), PhysAllocInitError> {
        let slot = unsafe { &mut *self.inner.get() };
        if slot.is_some() {
            return Err(PhysAllocInitError::AlreadyInitialized);
        }

        let allocator =
            PhysicalAllocator::from_memory_map(map, reservations, free_storage, reserved_storage)?;

        *slot = Some(allocator);
        Ok(())
    }

    fn with<R>(&self, f: impl FnOnce(&mut PhysicalAllocator<'static>) -> R) -> Option<R> {
        unsafe {
            let slot = &mut *self.inner.get();
            slot.as_mut().map(f)
        }
    }
}

unsafe impl Sync for AllocatorCell {}

static GLOBAL_ALLOCATOR: AllocatorCell = AllocatorCell::new();

/// Install the global physical allocator using the provided storage slices.
pub fn initialize_runtime_allocator(
    map: MemoryMap,
    reservations: &[ReservedRegion],
    free_storage: &'static mut [Option<PhysFrame>],
    reserved_storage: &'static mut [Option<ReservedRegion>],
) -> Result<(), PhysAllocInitError> {
    GLOBAL_ALLOCATOR.initialize(map, reservations, free_storage, reserved_storage)
}

/// Execute a closure with mutable access to the global physical allocator.
pub fn with_runtime_allocator<R>(
    f: impl FnOnce(&mut PhysicalAllocator<'static>) -> R,
) -> Option<R> {
    GLOBAL_ALLOCATOR.with(f)
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

#[derive(Clone, Copy, Debug)]
struct FrameSpan {
    start: u64,
    end: u64,
}

impl FrameSpan {
    fn new(start: u64, end: u64) -> Result<Self, PhysAllocError> {
        if start >= end {
            return Err(PhysAllocError::RangeMisaligned { start, end });
        }

        if !start.is_multiple_of(FRAME_SIZE) || !end.is_multiple_of(FRAME_SIZE) {
            return Err(PhysAllocError::RangeMisaligned { start, end });
        }

        Ok(Self { start, end })
    }

    fn from_frame(frame: PhysFrame) -> Result<Self, PhysAllocError> {
        if frame.count == 0 {
            return Err(PhysAllocError::RangeMisaligned {
                start: frame.start,
                end: frame.start,
            });
        }

        let end =
            span_end(frame.start, frame.count).ok_or_else(|| PhysAllocError::RangeOverflow {
                start: frame.start,
                end: frame
                    .start
                    .saturating_add(frame.count.saturating_mul(FRAME_SIZE)),
            })?;

        Self::new(frame.start, end)
    }

    fn merge(self, other: Self) -> Result<Self, PhysAllocError> {
        let start = min(self.start, other.start);
        let end = max(self.end, other.end);
        Self::new(start, end)
    }

    fn overlaps(&self, other: &Self) -> bool {
        self.start < other.end && other.start < self.end
    }

    fn frame_count(&self) -> Result<u64, PhysAllocError> {
        let span = self
            .end
            .checked_sub(self.start)
            .ok_or(PhysAllocError::RangeOverflow {
                start: self.start,
                end: self.end,
            })?;

        if span % FRAME_SIZE != 0 {
            return Err(PhysAllocError::RangeMisaligned {
                start: self.start,
                end: self.end,
            });
        }

        Ok(span / FRAME_SIZE)
    }

    fn into_frame(self) -> Result<PhysFrame, PhysAllocError> {
        let count = self.frame_count()?;
        Ok(PhysFrame::new(self.start, count))
    }

    fn subtract(self, removal: &Self) -> Result<(Option<Self>, Option<Self>), PhysAllocError> {
        if !self.overlaps(removal) {
            return Ok((Some(self), None));
        }

        let removal_start = max(self.start, removal.start);
        let removal_end = min(self.end, removal.end);

        let left = if self.start < removal_start {
            Some(Self::new(self.start, removal_start)?)
        } else {
            None
        };

        let right = if removal_end < self.end {
            Some(Self::new(removal_end, self.end)?)
        } else {
            None
        };

        Ok((left, right))
    }
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
        self.entries
    }

    fn push(&mut self, frame: PhysFrame) -> Result<(), PhysAllocError> {
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

        Err(PhysAllocError::StorageExhausted {
            capacity: self.capacity(),
        })
    }

    fn remove_slot(&mut self, index: usize) {
        if index < self.entries.len() && self.entries[index].take().is_some() {
            self.len = self.len.saturating_sub(1);
        }
    }

    fn insert(&mut self, frame: PhysFrame) -> Result<(), PhysAllocError> {
        if frame.count == 0 {
            return Ok(());
        }

        let initial_span = FrameSpan::from_frame(frame)?;
        let merged_span = self.coalesce_span(initial_span)?;

        self.push_span(merged_span)
    }

    fn allocate_count(&mut self, frames: u64) -> Result<Option<PhysFrame>, PhysAllocError> {
        if frames == 0 {
            return Err(PhysAllocError::UnsupportedFrameCount { frames });
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
                return Ok(Some(alloc));
            }

            let advance_bytes =
                frames
                    .checked_mul(FRAME_SIZE)
                    .ok_or_else(|| PhysAllocError::RangeOverflow {
                        start: run.start,
                        end: run.start.saturating_add(frames.saturating_mul(FRAME_SIZE)),
                    })?;

            let new_start = run.start.checked_add(advance_bytes).ok_or_else(|| {
                PhysAllocError::RangeOverflow {
                    start: run.start,
                    end: run.start.saturating_add(advance_bytes),
                }
            })?;

            let remaining = run.count - frames;
            self.entries[idx] = Some(PhysFrame::new(new_start, remaining));
            return Ok(Some(alloc));
        }

        Ok(None)
    }

    fn coalesce_span(&mut self, mut span: FrameSpan) -> Result<FrameSpan, PhysAllocError> {
        while let Some(index) = self.first_overlapping_index(&span)? {
            let existing = self.take_span(index)?;
            span = span.merge(existing)?;
        }

        Ok(span)
    }

    fn first_overlapping_index(&self, span: &FrameSpan) -> Result<Option<usize>, PhysAllocError> {
        for (idx, slot) in self.entries.iter().enumerate() {
            let Some(run) = slot else { continue };
            let existing_span = FrameSpan::from_frame(*run)?;
            if span.overlaps(&existing_span) {
                return Ok(Some(idx));
            }
        }

        Ok(None)
    }

    fn take_span(&mut self, index: usize) -> Result<FrameSpan, PhysAllocError> {
        let slot = self
            .entries
            .get_mut(index)
            .ok_or(PhysAllocError::OutOfMemory)?; // invalid index indicates corrupted state

        let run = slot.take().ok_or(PhysAllocError::OutOfMemory)?;
        self.len = self.len.saturating_sub(1);

        FrameSpan::from_frame(run)
    }

    fn push_span(&mut self, span: FrameSpan) -> Result<(), PhysAllocError> {
        let frame = span.into_frame()?;
        self.push(frame)
    }

    fn subtract_range(&mut self, start: u64, end: u64) -> Result<(), PhysAllocError> {
        if start >= end {
            return Ok(());
        }

        let range_start = align_down(start);
        let range_end = align_up(end);

        if range_start >= range_end {
            return Ok(());
        }

        let removal_span = FrameSpan::new(range_start, range_end)?;

        for idx in 0..self.entries.len() {
            let run = match self.entries[idx] {
                Some(run) => run,
                None => continue,
            };

            let existing_span = FrameSpan::from_frame(run)?;

            if !existing_span.overlaps(&removal_span) {
                continue;
            }

            self.remove_slot(idx);

            let (left, right) = existing_span.subtract(&removal_span)?;

            if let Some(span) = left {
                self.push_span(span)?;
            }

            if let Some(span) = right {
                self.push_span(span)?;
            }
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
        self.entries
    }

    fn push(&mut self, region: ReservedRegion) -> Result<(), PhysAllocError> {
        if region.start >= region.end {
            return Err(PhysAllocError::InvalidRegion {
                start: region.start,
                end: region.end,
            });
        }

        for slot in self.entries.iter_mut() {
            if slot.is_none() {
                *slot = Some(region);
                self.len += 1;
                return Ok(());
            }
        }

        Err(PhysAllocError::StorageExhausted {
            capacity: self.capacity(),
        })
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
    ) -> Result<Self, PhysAllocInitError> {
        if map.map_size == 0 || map.entry_count == 0 {
            return Err(PhysAllocInitError::Empty);
        }

        let mut free = FrameRunList::new(free_storage);
        for (index, descriptor) in MemoryMapIter::new(&map).enumerate() {
            if descriptor.typ != EfiMemoryType::ConventionalMemory as u32 {
                continue;
            }

            if descriptor.number_of_pages == 0 {
                continue;
            }

            let frame = PhysFrame::new(descriptor.physical_start, descriptor.number_of_pages);
            free.push(frame)
                .map_err(|err| descriptor_error(index, err))?;
        }

        let free_run_len = free.len();
        let free_run_capacity = free.capacity();
        crate::debug_structured!(
            "runtime allocator free runs populated:",
            [("used", free_run_len), ("capacity", free_run_capacity),]
        );

        if free.len() == 0 {
            return Err(PhysAllocInitError::Empty);
        }

        let mut reserved = ReservedList::new(reserved_storage);
        for &region in reservations {
            let region_start = region.start;
            let region_end = region.end;
            reserved
                .push(region)
                .map_err(|err| reservation_error(region, err))?;
            free.subtract_range(region_start, region_end)
                .map_err(|err| reservation_error(region, err))?;
        }

        let reserved_count = reserved.len();
        let free_remaining = free.len();
        crate::debug_structured!(
            "runtime allocator reservations applied:",
            [("used", reserved_count), ("free", free_remaining)]
        );

        Ok(Self {
            map,
            free,
            reserved,
        })
    }

    /// Allocate a single 4 KiB frame.
    pub fn allocate(&mut self) -> Result<PhysFrame, PhysAllocError> {
        self.allocate_order(0)
    }

    /// Allocate `2^order` contiguous frames (order 0 = 1 frame, order 9 = 512 frames / 2 MiB).
    pub fn allocate_order(&mut self, order: u8) -> Result<PhysFrame, PhysAllocError> {
        let frames = match 1u64.checked_shl(order as u32) {
            Some(count) if count > 0 => count,
            _ => return Err(PhysAllocError::UnsupportedFrameCount { frames: 0 }),
        };

        match self.free.allocate_count(frames)? {
            Some(frame) => Ok(frame),
            None => Err(PhysAllocError::OutOfMemory),
        }
    }

    /// Free a previously allocated run of frames.
    pub fn free(&mut self, frame: PhysFrame) -> Result<(), PhysAllocError> {
        if frame.count == 0 {
            return Ok(());
        }

        self.free.insert(frame)
    }

    /// Mark an arbitrary region as reserved after initialization.
    pub fn reserve(&mut self, region: ReservedRegion) -> Result<(), PhysAllocError> {
        self.reserved.push(region)?;
        self.free.subtract_range(region.start, region.end)
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

fn descriptor_error(index: usize, error: PhysAllocError) -> PhysAllocInitError {
    PhysAllocInitError::InvalidDescriptor { index, error }
}

fn reservation_error(region: ReservedRegion, error: PhysAllocError) -> PhysAllocInitError {
    PhysAllocInitError::ReservationConflict {
        start: region.start,
        end: region.end,
        error,
    }
}

fn align_down(value: u64) -> u64 {
    let mask = FRAME_SIZE - 1;
    value & !mask
}

fn align_up(value: u64) -> u64 {
    let mask = FRAME_SIZE - 1;
    match value.checked_add(mask) {
        Some(sum) => sum & !mask,
        None => !mask,
    }
}

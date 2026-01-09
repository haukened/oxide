use core::cell::UnsafeCell;

use oxide_abi::{EfiMemoryType, MemoryMap};

use crate::memory::{
    allocator::ReservedRegion, error::MemoryInitError, frame::FRAME_SIZE, map::MemoryMapIter,
};

const MAX_EARLY_RESERVATIONS: usize = 16;

struct ReservationList {
    entries: [ReservedRegion; MAX_EARLY_RESERVATIONS],
    len: usize,
}

impl ReservationList {
    const fn new() -> Self {
        Self {
            entries: [ReservedRegion { start: 0, end: 0 }; MAX_EARLY_RESERVATIONS],
            len: 0,
        }
    }

    fn push(&mut self, region: ReservedRegion) -> Result<(), MemoryInitError> {
        if region.start >= region.end {
            return Err(MemoryInitError::TooLarge);
        }

        if self.len >= MAX_EARLY_RESERVATIONS {
            return Err(MemoryInitError::TooLarge);
        }

        // Keep reservations sorted by start for predictable iteration.
        let mut index = self.len;
        for i in 0..self.len {
            if self.entries[i].start > region.start {
                index = i;
                break;
            }
        }

        // Shift elements to make room when inserting in the middle.
        if index < self.len {
            let mut j = self.len;
            while j > index {
                self.entries[j] = self.entries[j - 1];
                j -= 1;
            }
        }

        self.entries[index] = region;
        self.len += 1;
        Ok(())
    }

    fn overlaps(&self, region: ReservedRegion) -> Option<ReservedRegion> {
        self.entries[..self.len]
            .iter()
            .find(|&&existing| {
                ranges_overlap(existing.start, existing.end, region.start, region.end)
            })
            .copied()
    }

    fn contains(&self, addr: u64) -> Option<ReservedRegion> {
        self.entries[..self.len]
            .iter()
            .find(|&&existing| addr >= existing.start && addr < existing.end)
            .copied()
    }

    fn iter(&self) -> impl Iterator<Item = ReservedRegion> + '_ {
        self.entries[..self.len].iter().copied()
    }
}

struct ReservationCell(UnsafeCell<ReservationList>);

unsafe impl Sync for ReservationCell {}

static EARLY_RESERVATIONS: ReservationCell =
    ReservationCell(UnsafeCell::new(ReservationList::new()));

/// Allocate a physical region during early boot and record it as reserved.
pub fn allocate_region(map: &MemoryMap, bytes: usize) -> Result<ReservedRegion, MemoryInitError> {
    if bytes == 0 {
        return Err(MemoryInitError::TooLarge);
    }

    let alloc_bytes = align_up(bytes as u64, FRAME_SIZE).ok_or(MemoryInitError::TooLarge)?;

    let iter = MemoryMapIter::new(map);
    for descriptor in iter {
        if descriptor.typ != EfiMemoryType::ConventionalMemory as u32 {
            continue;
        }
        if descriptor.number_of_pages == 0 {
            continue;
        }

        let region_start = align_up(descriptor.physical_start.max(FRAME_SIZE), FRAME_SIZE)
            .ok_or(MemoryInitError::TooLarge)?;
        let region_size = descriptor
            .number_of_pages
            .checked_mul(FRAME_SIZE)
            .ok_or(MemoryInitError::TooLarge)?;
        let region_end = descriptor
            .physical_start
            .checked_add(region_size)
            .ok_or(MemoryInitError::TooLarge)?;

        if region_start >= region_end {
            continue;
        }

        let mut candidate = region_start;
        while let Some(end) = candidate.checked_add(alloc_bytes) {
            if end > region_end {
                break;
            }

            let candidate_region = ReservedRegion {
                start: candidate,
                end,
            };

            if let Some(existing) = find_overlap(candidate_region) {
                candidate = align_up(existing.end, FRAME_SIZE).ok_or(MemoryInitError::TooLarge)?;
                continue;
            }

            unsafe {
                reserve(candidate_region)?;
            }
            return Ok(candidate_region);
        }
    }

    Err(MemoryInitError::OutOfFrames)
}

pub(crate) fn contains_address(addr: u64) -> Option<ReservedRegion> {
    unsafe { (*EARLY_RESERVATIONS.0.get()).contains(addr) }
}

/// Iterate over all early reservations in insertion order.
pub fn for_each<F>(mut f: F)
where
    F: FnMut(ReservedRegion),
{
    let list = unsafe { &*EARLY_RESERVATIONS.0.get() };
    for region in list.iter() {
        f(region);
    }
}

unsafe fn reserve(region: ReservedRegion) -> Result<(), MemoryInitError> {
    let list = unsafe { &mut *EARLY_RESERVATIONS.0.get() };

    if list.overlaps(region).is_some() {
        return Err(MemoryInitError::TooLarge);
    }

    list.push(region)
}

fn find_overlap(region: ReservedRegion) -> Option<ReservedRegion> {
    unsafe { (*EARLY_RESERVATIONS.0.get()).overlaps(region) }
}

fn align_up(value: u64, align: u64) -> Option<u64> {
    if align == 0 {
        return None;
    }
    let mask = align - 1;
    let sum = value.checked_add(mask)?;
    Some(sum & !mask)
}

fn ranges_overlap(a_start: u64, a_end: u64, b_start: u64, b_end: u64) -> bool {
    a_start < b_end && b_start < a_end
}

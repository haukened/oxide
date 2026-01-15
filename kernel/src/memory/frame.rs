use crate::memory::{early, error::FrameAllocError, map::MemoryMapIter};
use oxide_abi::{EfiMemoryType, MemoryMap};

/// Size of a physical memory frame in bytes (4 KiB).
pub const FRAME_SIZE: u64 = 4096;

/// Iterator-backed helper for walking usable frames prior to the runtime allocator.
pub struct FrameAllocator<'a> {
    iter: UsableFrameIter<'a>,
}

struct RunTracker {
    required: usize,
    start: Option<u64>,
    previous: u64,
    length: usize,
}

impl RunTracker {
    fn new(required: usize) -> Self {
        Self {
            required,
            start: None,
            previous: 0,
            length: 0,
        }
    }

    fn update(&mut self, frame: u64, gaps: &mut GapTracker) {
        match self.start {
            None => self.restart(frame),
            Some(_) => {
                let expected = self.previous.checked_add(FRAME_SIZE);
                match expected {
                    Some(next) if frame == next => {
                        self.previous = frame;
                        self.length += 1;
                    }
                    Some(next) => {
                        gaps.record((next, frame));
                        self.restart(frame);
                    }
                    None => {
                        let hinted = self.previous.saturating_add(FRAME_SIZE);
                        gaps.record((hinted, frame));
                        self.restart(frame);
                    }
                }
            }
        }
    }

    fn restart(&mut self, frame: u64) {
        self.start = Some(frame);
        self.previous = frame;
        self.length = 1;
    }

    fn is_complete(&self) -> bool {
        self.length >= self.required && self.start.is_some()
    }

    fn start_address(&self) -> Option<u64> {
        self.start
    }
}

struct GapTracker {
    first_gap: Option<(u64, u64)>,
}

impl GapTracker {
    fn new() -> Self {
        Self { first_gap: None }
    }

    fn record(&mut self, gap: (u64, u64)) {
        if self.first_gap.is_none() {
            self.first_gap = Some(gap);
        }
    }

    fn first(&self) -> Option<(u64, u64)> {
        self.first_gap
    }
}

impl<'a> FrameAllocator<'a> {
    /// Create a frame allocator over the provided firmware memory map.
    pub fn new(map: &'a MemoryMap) -> Self {
        Self {
            iter: UsableFrameIter::new(map),
        }
    }

    /// Allocate a single physical memory frame.
    pub fn alloc(&mut self) -> Option<u64> {
        self.iter.next()
    }

    /// Allocate `frame_count` contiguous frames, returning the physical start address.
    pub fn alloc_contiguous(&mut self, frame_count: usize) -> Result<u64, FrameAllocError> {
        debug_assert!(frame_count > 0);
        if frame_count == 0 {
            return Err(FrameAllocError::InvalidRequest);
        }

        let mut run = RunTracker::new(frame_count);
        let mut gaps = GapTracker::new();

        for frame in self.iter.by_ref() {
            run.update(frame, &mut gaps);

            if run.is_complete()
                && let Some(start) = run.start_address()
            {
                return Ok(start);
            }
        }

        if let Some((expected, found)) = gaps.first() {
            return Err(FrameAllocError::NonContiguous { expected, found });
        }

        Err(FrameAllocError::OutOfFrames)
    }
}

/// Iterator over frame-aligned physical addresses from the firmware memory map.
pub struct UsableFrameIter<'a> {
    desc_iter: MemoryMapIter<'a>,
    current_range: Option<(u64, u64)>, // [start, end)
}

impl<'a> UsableFrameIter<'a> {
    pub fn new(map: &'a MemoryMap) -> Self {
        Self {
            desc_iter: MemoryMapIter::new(map),
            current_range: None,
        }
    }
}

impl<'a> Iterator for UsableFrameIter<'a> {
    type Item = u64; // physical frame address

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(frame) = self.next_frame_from_range() {
                return Some(frame);
            }

            self.load_next_range()?;
        }
    }
}

fn align_up(value: u64) -> u64 {
    (value + FRAME_SIZE - 1) & !(FRAME_SIZE - 1)
}

impl<'a> UsableFrameIter<'a> {
    fn next_frame_from_range(&mut self) -> Option<u64> {
        while let Some((start, end)) = &mut self.current_range {
            if *start >= *end {
                self.current_range = None;
                return None;
            }

            let frame = *start;
            *start += FRAME_SIZE;

            if let Some(reservation) = early::contains_address(frame) {
                if let Some(next_start) = Self::aligned_reservation_end(reservation.end, *end) {
                    self.current_range = Some((next_start, *end));
                    continue;
                }

                self.current_range = None;
                continue;
            }

            return Some(frame);
        }

        None
    }

    fn load_next_range(&mut self) -> Option<()> {
        for desc in self.desc_iter.by_ref() {
            if desc.typ != EfiMemoryType::ConventionalMemory as u32 {
                continue;
            }

            if desc.number_of_pages == 0 {
                continue;
            }

            let region_size = match desc.number_of_pages.checked_mul(FRAME_SIZE) {
                Some(size) => size,
                None => continue,
            };

            let mut start = desc.physical_start;
            let end = match start.checked_add(region_size) {
                Some(end) => end,
                None => continue,
            };

            start = align_up(start);

            if start < FRAME_SIZE {
                start = FRAME_SIZE;
            }

            if start >= end {
                continue;
            }

            self.current_range = Some((start, end));
            return Some(());
        }

        None
    }

    fn aligned_reservation_end(reservation_end: u64, range_end: u64) -> Option<u64> {
        let next_start = align_up(reservation_end).max(FRAME_SIZE);

        (next_start < range_end).then_some(next_start)
    }
}

#[cfg(test)]
mod tests {
    extern crate alloc;

    use super::*;
    use crate::memory::error::FrameAllocError;
    use alloc::{boxed::Box, vec, vec::Vec};
    use oxide_abi::{EfiMemoryType, MemoryDescriptor, MemoryMap};

    fn descriptor(typ: EfiMemoryType, physical_start: u64, pages: u64) -> MemoryDescriptor {
        MemoryDescriptor {
            typ: typ as u32,
            _pad: 0,
            physical_start,
            virtual_start: 0,
            number_of_pages: pages,
            attribute: 0,
        }
    }

    fn build_map(descriptors: Vec<MemoryDescriptor>) -> (MemoryMap, Box<[MemoryDescriptor]>) {
        let entry_size = core::mem::size_of::<MemoryDescriptor>() as u32;
        let entry_count = descriptors.len() as u32;
        let backing: Box<[MemoryDescriptor]> = descriptors.into_boxed_slice();
        let map = MemoryMap {
            descriptors_phys: backing.as_ptr() as u64,
            map_size: (entry_size as u64) * (entry_count as u64),
            entry_size,
            entry_version: 1,
            entry_count,
        };

        (map, backing)
    }

    #[test]
    fn alloc_returns_frames_in_sequence() {
        let descriptors = vec![descriptor(EfiMemoryType::ConventionalMemory, FRAME_SIZE, 3)];
        let (map, _backing) = build_map(descriptors);
        let mut allocator = FrameAllocator::new(&map);

        assert_eq!(allocator.alloc(), Some(FRAME_SIZE));
        assert_eq!(allocator.alloc(), Some(FRAME_SIZE * 2));
        assert_eq!(allocator.alloc(), Some(FRAME_SIZE * 3));
        assert_eq!(allocator.alloc(), None);
    }

    #[test]
    fn alloc_skips_non_conventional_regions() {
        let descriptors = vec![
            descriptor(EfiMemoryType::LoaderCode, FRAME_SIZE, 2),
            descriptor(EfiMemoryType::ConventionalMemory, FRAME_SIZE * 5, 1),
        ];
        let (map, _backing) = build_map(descriptors);
        let mut allocator = FrameAllocator::new(&map);

        assert_eq!(allocator.alloc(), Some(FRAME_SIZE * 5));
        assert_eq!(allocator.alloc(), None);
    }

    #[test]
    fn alloc_contiguous_returns_run_start() {
        let descriptors = vec![descriptor(EfiMemoryType::ConventionalMemory, FRAME_SIZE, 4)];
        let (map, _backing) = build_map(descriptors);
        let mut allocator = FrameAllocator::new(&map);

        assert_eq!(allocator.alloc_contiguous(2), Ok(FRAME_SIZE));
        assert_eq!(allocator.alloc(), Some(FRAME_SIZE * 3));
    }

    #[cfg(debug_assertions)]
    #[test]
    #[should_panic(expected = "frame_count > 0")]
    fn alloc_contiguous_rejects_zero_request() {
        let descriptors = vec![descriptor(EfiMemoryType::ConventionalMemory, FRAME_SIZE, 1)];
        let (map, _backing) = build_map(descriptors);
        let mut allocator = FrameAllocator::new(&map);

        let _ = allocator.alloc_contiguous(0);
    }

    #[cfg(not(debug_assertions))]
    #[test]
    fn alloc_contiguous_rejects_zero_request() {
        let descriptors = vec![descriptor(EfiMemoryType::ConventionalMemory, FRAME_SIZE, 1)];
        let (map, _backing) = build_map(descriptors);
        let mut allocator = FrameAllocator::new(&map);

        assert_eq!(
            allocator.alloc_contiguous(0),
            Err(FrameAllocError::InvalidRequest)
        );
    }

    #[test]
    fn alloc_contiguous_detects_gaps() {
        let descriptors = vec![
            descriptor(EfiMemoryType::ConventionalMemory, FRAME_SIZE, 2),
            descriptor(EfiMemoryType::ConventionalMemory, FRAME_SIZE * 5, 2),
        ];
        let (map, _backing) = build_map(descriptors);
        let mut allocator = FrameAllocator::new(&map);

        assert_eq!(
            allocator.alloc_contiguous(3),
            Err(FrameAllocError::NonContiguous {
                expected: FRAME_SIZE * 3,
                found: FRAME_SIZE * 5,
            })
        );
    }

    #[test]
    fn alloc_contiguous_reports_out_of_frames() {
        let descriptors = vec![descriptor(EfiMemoryType::ConventionalMemory, FRAME_SIZE, 2)];
        let (map, _backing) = build_map(descriptors);
        let mut allocator = FrameAllocator::new(&map);

        assert_eq!(
            allocator.alloc_contiguous(4),
            Err(FrameAllocError::OutOfFrames)
        );
    }
}

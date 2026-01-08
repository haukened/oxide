use crate::memory::{error::FrameAllocError, map::MemoryMapIter};
use oxide_abi::{EfiMemoryType, MemoryMap};

/// Size of a physical memory frame in bytes (4 KiB).
pub const FRAME_SIZE: u64 = 4096;

pub struct FrameAllocator<'a> {
    iter: UsableFrameIter<'a>,
}

impl<'a> FrameAllocator<'a> {
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

        let first = self.alloc().ok_or(FrameAllocError::OutOfFrames)?;
        let mut prev = first;

        for _ in 1..frame_count {
            let next = self.alloc().ok_or(FrameAllocError::OutOfFrames)?;
            let expected = prev + FRAME_SIZE;
            if next != expected {
                return Err(FrameAllocError::NonContiguous {
                    expected,
                    found: next,
                });
            }
            prev = next;
        }

        Ok(first)
    }
}

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
            // If we are currently iterating a range, continue
            if let Some((start, end)) = &mut self.current_range {
                if *start < *end {
                    let frame = *start;
                    *start += FRAME_SIZE;
                    return Some(frame);
                } else {
                    self.current_range = None;
                }
            }

            // Advance to the next memory descriptor
            let desc = self.desc_iter.next()?;

            // Only conventional memory is usable
            if desc.typ != EfiMemoryType::ConventionalMemory as u32 {
                continue;
            }

            // Skip regions with zero pages
            if desc.number_of_pages == 0 {
                continue;
            }

            // Compute region bounds safely
            let region_size = match desc.number_of_pages.checked_mul(FRAME_SIZE) {
                Some(size) => size,
                None => continue,
            };

            // Compute end address safely
            let mut start = desc.physical_start;
            let end = match start.checked_add(region_size) {
                Some(end) => end,
                None => continue,
            };

            // Align start up to FRAME_SIZE
            start = (start + FRAME_SIZE - 1) & !(FRAME_SIZE - 1);

            // Never yield the zero frame; treat anything below FRAME_SIZE as reserved
            if start < FRAME_SIZE {
                start = FRAME_SIZE;
            }

            // If the aligned range is invalid, skip it
            if start >= end {
                continue;
            }

            // Set the current range for iteration
            self.current_range = Some((start, end));
        }
    }
}

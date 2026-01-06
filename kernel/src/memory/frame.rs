use crate::memory::map::MemoryMapIter;
use oxide_abi::{EfiMemoryType, MemoryMap};

/// Size of a physical memory frame in bytes (4 KiB).
pub const FRAME_SIZE: u64 = 4096;

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

            // If the aligned range is invalid, skip it
            if start >= end {
                continue;
            }

            // Set the current range for iteration
            self.current_range = Some((start, end));
        }
    }
}

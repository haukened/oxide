use crate::memory::frame::FRAME_SIZE;
use oxide_abi::{MemoryDescriptor, MemoryMap};

/// Iterator over firmware memory descriptors backed by a raw buffer.
pub struct MemoryMapIter<'a> {
    base: usize,
    entry_size: usize,
    remaining: u32,
    _marker: core::marker::PhantomData<&'a ()>,
}

impl<'a> MemoryMapIter<'a> {
    /// Construct an iterator for the given memory-map snapshot.
    pub fn new(map: &'a MemoryMap) -> Self {
        Self {
            base: map.descriptors_phys as usize,
            entry_size: map.entry_size as usize,
            remaining: map.entry_count,
            _marker: core::marker::PhantomData,
        }
    }
}

/// Compute the inclusive-exclusive physical byte range described by the entry.
pub fn descriptor_range(desc: &MemoryDescriptor) -> Option<(u64, u64)> {
    let len = desc.number_of_pages.checked_mul(FRAME_SIZE)?;
    let end = desc.physical_start.checked_add(len)?;
    Some((desc.physical_start, end))
}

/// Locate the descriptor that covers the supplied physical address.
pub fn find_descriptor_containing(
    map: &MemoryMap,
    addr: u64,
) -> Option<&MemoryDescriptor> {
    for desc in MemoryMapIter::new(map) {
        if let Some((start, end)) = descriptor_range(desc)
            && addr >= start && addr < end {
                return Some(desc);
            }
    }
    None
}

impl<'a> Iterator for MemoryMapIter<'a> {
    type Item = &'a MemoryDescriptor;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            return None;
        }

        let desc = unsafe { &*(self.base as *const MemoryDescriptor) };
        self.base = self.base.wrapping_add(self.entry_size);
        self.remaining -= 1;

        Some(desc)
    }
}

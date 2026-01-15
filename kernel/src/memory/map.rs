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
pub fn find_descriptor_containing(map: &MemoryMap, addr: u64) -> Option<&MemoryDescriptor> {
    for desc in MemoryMapIter::new(map) {
        if let Some((start, end)) = descriptor_range(desc)
            && addr >= start
            && addr < end
        {
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

#[cfg(test)]
mod tests {
    extern crate alloc;

    use super::*;
    use alloc::{boxed::Box, vec, vec::Vec};
    use oxide_abi::{EfiMemoryType, MemoryDescriptor};

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

    #[test]
    fn descriptor_range_returns_span() {
        let desc = descriptor(EfiMemoryType::ConventionalMemory, 0x2000, 3);
        assert_eq!(
            descriptor_range(&desc),
            Some((0x2000, 0x2000 + 3 * FRAME_SIZE))
        );
    }

    #[test]
    fn descriptor_range_detects_overflow() {
        let desc = descriptor(
            EfiMemoryType::ConventionalMemory,
            u64::MAX - FRAME_SIZE + 1,
            u64::MAX,
        );
        assert!(descriptor_range(&desc).is_none());
    }

    #[test]
    fn find_descriptor_containing_returns_match() {
        let descriptors = vec![
            descriptor(EfiMemoryType::LoaderCode, 0x1000, 1),
            descriptor(EfiMemoryType::ConventionalMemory, 0x4000, 2),
        ];
        let (map, _backing) = build_map(descriptors);

        let found = find_descriptor_containing(&map, 0x4000 + FRAME_SIZE / 2).unwrap();
        assert_eq!(found.physical_start, 0x4000);
    }

    #[test]
    fn find_descriptor_containing_returns_none_when_absent() {
        let descriptors = vec![descriptor(EfiMemoryType::ConventionalMemory, 0x8000, 1)];
        let (map, _backing) = build_map(descriptors);

        assert!(find_descriptor_containing(&map, 0x1000).is_none());
    }

    #[test]
    fn memory_map_iter_yields_descriptors_in_order() {
        let descriptors = vec![
            descriptor(EfiMemoryType::ConventionalMemory, 0x1000, 1),
            descriptor(EfiMemoryType::ConventionalMemory, 0x2000, 1),
            descriptor(EfiMemoryType::ConventionalMemory, 0x3000, 1),
        ];
        let (map, _backing) = build_map(descriptors);

        let collected: Vec<u64> = MemoryMapIter::new(&map)
            .map(|desc| desc.physical_start)
            .collect();

        assert_eq!(collected, vec![0x1000, 0x2000, 0x3000]);
    }
}

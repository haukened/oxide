use oxide_abi::{MemoryDescriptor, MemoryMap};

pub struct MemoryMapIter<'a> {
    base: *const u8,
    entry_size: usize,
    remaining: u32,
    _marker: core::marker::PhantomData<&'a ()>,
}

impl<'a> MemoryMapIter<'a> {
    pub fn new(map: &'a MemoryMap) -> Self {
        Self {
            base: map.descriptors_phys as *const u8,
            entry_size: map.entry_size as usize,
            remaining: map.entry_count,
            _marker: core::marker::PhantomData,
        }
    }
}

impl<'a> Iterator for MemoryMapIter<'a> {
    type Item = &'a MemoryDescriptor;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            return None;
        }

        let desc = unsafe { &*(self.base as *const MemoryDescriptor) };

        self.base = unsafe { self.base.add(self.entry_size) };
        self.remaining -= 1;

        Some(desc)
    }
}

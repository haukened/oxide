#![allow(dead_code)]

use oxide_abi::MemoryMap;

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
pub struct PhysicalAllocator {
    map: MemoryMap,
}

impl PhysicalAllocator {
    /// Build a runtime allocator using the copied memory map and any regions that must remain
    /// reserved (loader allocations, ACPI, framebuffer, etc.).
    pub fn from_memory_map(
        map: MemoryMap,
        reservations: &[ReservedRegion],
    ) -> Result<Self, AllocInitError> {
        let _ = (map, reservations);
        todo!("allocator initialization")
    }

    /// Allocate a single 4 KiB frame.
    pub fn allocate(&mut self) -> Result<PhysFrame, AllocationError> {
        todo!("allocate single frame")
    }

    /// Allocate `2^order` contiguous frames (order 0 = 1 frame, order 9 = 512 frames / 2 MiB).
    pub fn allocate_order(&mut self, order: u8) -> Result<PhysFrame, AllocationError> {
        let _ = order;
        todo!("allocate order")
    }

    /// Free a previously allocated run of frames.
    pub fn free(&mut self, frame: PhysFrame) {
        let _ = frame;
        todo!("free frame")
    }

    /// Mark an arbitrary region as reserved after initialization.
    pub fn reserve(&mut self, region: ReservedRegion) {
        let _ = region;
        todo!("reserve region")
    }

    /// Iterate over all free ranges currently tracked by the allocator.
    pub fn free_regions(&self) -> FreeRegionIter<'_> {
        todo!("iterate free regions")
    }

    /// Iterate over all reserved ranges currently tracked by the allocator.
    pub fn reserved_regions(&self) -> ReservedRegionIter<'_> {
        todo!("iterate reserved regions")
    }
}

/// Iterator over free regions. Placeholder until the backing store is decided.
pub struct FreeRegionIter<'a> {
    _marker: core::marker::PhantomData<&'a ()>,
}

impl<'a> Iterator for FreeRegionIter<'a> {
    type Item = PhysFrame;

    fn next(&mut self) -> Option<Self::Item> {
        todo!("next free region")
    }
}

/// Iterator over reserved regions.
pub struct ReservedRegionIter<'a> {
    _marker: core::marker::PhantomData<&'a ()>,
}

impl<'a> Iterator for ReservedRegionIter<'a> {
    type Item = ReservedRegion;

    fn next(&mut self) -> Option<Self::Item> {
        todo!("next reserved region")
    }
}

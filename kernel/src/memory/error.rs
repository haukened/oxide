#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PagingError {
    OutOfFrames,
    AddressOverflow(u64, u64),
    UnsupportedAddress(u64),
}

impl core::fmt::Debug for PagingError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            PagingError::OutOfFrames => write!(f, "PagingError::OutOfFrames"),
            PagingError::AddressOverflow(start, size) => {
                write!(
                    f,
                    "PagingError::AddressOverflow(start: {:#x}, size: {:#x})",
                    start, size
                )
            }
            PagingError::UnsupportedAddress(addr) => {
                write!(f, "PagingError::UnsupportedAddress({:#x})", addr)
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum MemoryInitError {
    NoUsableMemory,
    EmptyMemoryMap,
    OutOfFrames,
    NonContiguous { expected: u64, found: u64 },
    TooLarge,
    StackDescriptorMissing(u64),
    StackRangeOverflow(u32),
    IdentityRangeOverflow { start: u64, end: u64 },
    Paging(PagingError),
}

impl core::fmt::Debug for MemoryInitError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            MemoryInitError::NoUsableMemory => write!(f, "MemoryInitError::NoUsableMemory"),
            MemoryInitError::EmptyMemoryMap => write!(f, "MemoryInitError::EmptyMemoryMap"),
            MemoryInitError::OutOfFrames => write!(f, "MemoryInitError::OutOfFrames"),
            MemoryInitError::NonContiguous { expected, found } => write!(
                f,
                "MemoryInitError::NonContiguous {{ expected: {:#x}, found: {:#x} }}",
                expected, found
            ),
            MemoryInitError::TooLarge => write!(f, "MemoryInitError::TooLarge"),
            MemoryInitError::StackDescriptorMissing(id) => {
                write!(f, "MemoryInitError::StackDescriptorMissing({})", id)
            }
            MemoryInitError::StackRangeOverflow(id) => {
                write!(f, "MemoryInitError::StackRangeOverflow({})", id)
            }
            MemoryInitError::IdentityRangeOverflow { start, end } => write!(
                f,
                "MemoryInitError::IdentityRangeOverflow {{ start: {:#x}, end: {:#x} }}",
                start, end
            ),
            MemoryInitError::Paging(err) => write!(f, "MemoryInitError::Paging({:?})", err),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum FrameAllocError {
    OutOfFrames,
    NonContiguous { expected: u64, found: u64 },
    InvalidRequest,
}

impl core::fmt::Debug for FrameAllocError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            FrameAllocError::OutOfFrames => write!(f, "FrameAllocError::OutOfFrames"),
            FrameAllocError::NonContiguous { expected, found } => write!(
                f,
                "FrameAllocError::NonContiguous {{ expected: {:#x}, found: {:#x} }}",
                expected, found
            ),
            FrameAllocError::InvalidRequest => write!(f, "FrameAllocError::InvalidRequest"),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PhysAllocError {
    OutOfMemory,
    UnsupportedFrameCount { frames: u64 },
    RangeOverflow { start: u64, end: u64 },
    RangeMisaligned { start: u64, end: u64 },
    StorageExhausted { capacity: usize },
    InvalidRegion { start: u64, end: u64 },
}

impl core::fmt::Debug for PhysAllocError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            PhysAllocError::OutOfMemory => write!(f, "PhysAllocError::OutOfMemory"),
            PhysAllocError::UnsupportedFrameCount { frames } => write!(
                f,
                "PhysAllocError::UnsupportedFrameCount {{ frames: {} }}",
                frames
            ),
            PhysAllocError::RangeOverflow { start, end } => write!(
                f,
                "PhysAllocError::RangeOverflow {{ start: {:#x}, end: {:#x} }}",
                start, end
            ),
            PhysAllocError::RangeMisaligned { start, end } => write!(
                f,
                "PhysAllocError::RangeMisaligned {{ start: {:#x}, end: {:#x} }}",
                start, end
            ),
            PhysAllocError::StorageExhausted { capacity } => write!(
                f,
                "PhysAllocError::StorageExhausted {{ capacity: {} }}",
                capacity
            ),
            PhysAllocError::InvalidRegion { start, end } => write!(
                f,
                "PhysAllocError::InvalidRegion {{ start: {:#x}, end: {:#x} }}",
                start, end
            ),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PhysAllocInitError {
    Empty,
    InvalidDescriptor {
        index: usize,
        error: PhysAllocError,
    },
    ReservationConflict {
        start: u64,
        end: u64,
        error: PhysAllocError,
    },
}

impl core::fmt::Debug for PhysAllocInitError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            PhysAllocInitError::Empty => write!(f, "PhysAllocInitError::Empty"),
            PhysAllocInitError::InvalidDescriptor { index, error } => write!(
                f,
                "PhysAllocInitError::InvalidDescriptor {{ index: {}, error: {:?} }}",
                index, error
            ),
            PhysAllocInitError::ReservationConflict { start, end, error } => write!(
                f,
                "PhysAllocInitError::ReservationConflict {{ start: {:#x}, end: {:#x}, error: {:?} }}",
                start, end, error
            ),
        }
    }
}

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
                    "PagingError::AddressOverflow(start={:#x}, size={:#x})",
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

use crate::memory::error::{FrameAllocError, MemoryInitError};

#[derive(Debug)]
#[allow(dead_code)]
pub enum KernelError {
    MemoryInit(MemoryInitError),
    FrameAlloc(FrameAllocError),
}

impl From<MemoryInitError> for KernelError {
    fn from(err: MemoryInitError) -> Self {
        KernelError::MemoryInit(err)
    }
}

impl From<FrameAllocError> for KernelError {
    fn from(err: FrameAllocError) -> Self {
        KernelError::FrameAlloc(err)
    }
}

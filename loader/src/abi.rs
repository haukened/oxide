use core::mem::{MaybeUninit, size_of};
use oxide_abi::BootAbi;
use uefi::boot::{AllocateType, MemoryType, allocate_pages};

/// Allocates the BootAbi in LOADER_DATA memory.
///
/// The returned reference is effectively `'static` because the allocation
/// is intentionally leaked and survives ExitBootServices. The kernel assumes
/// ownership after handoff.
pub fn alloc_abi_struct() -> uefi::Result<*mut BootAbi> {
    // Round up to whole pages
    let abi_size = size_of::<BootAbi>();
    let page_size = 4096;
    let pages = (abi_size + page_size - 1) / page_size;

    // Allocate physically contiguous pages for the ABI structure
    // use LOADER_DATA so the kernel can access it after EBS
    let phys_addr = allocate_pages(AllocateType::AnyPages, MemoryType::LOADER_DATA, pages)?;

    // Cast the physical address to a pointer to BootAbi
    let abi_ptr = phys_addr.as_ptr().cast::<MaybeUninit<BootAbi>>();

    unsafe {
        // Initialize the memory to zero to avoid uninitialized data
        // and enfire determinism for debugging
        abi_ptr.write(MaybeUninit::zeroed());

        // Get a mutable reference to the initialized BootAbi
        let abi = &mut *abi_ptr.cast::<BootAbi>();

        // Defensively initialize version field from the crate constant
        abi.version = oxide_abi::ABI_VERSION;

        Ok(abi)
    }
}

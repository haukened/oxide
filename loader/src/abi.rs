use core::mem::{MaybeUninit, size_of};
use oxide_abi::BootAbi;
use uefi::{
    boot::{AllocateType, MemoryType, allocate_pages},
    mem::memory_map::{MemoryMap, MemoryMapOwned},
};

use crate::{firmware::FirmwareInfo, framebuffer::FramebufferInfo, options::BootOptions};

/// Allocates the BootAbi in LOADER_DATA memory.
///
/// The returned reference is effectively `'static` because the allocation
/// is intentionally leaked and survives ExitBootServices. The kernel assumes
/// ownership after handoff.
pub fn alloc_abi_struct() -> uefi::Result<*mut BootAbi> {
    // Round up to whole pages
    let abi_size = size_of::<BootAbi>();
    let page_size = 4096;
    let pages = abi_size.div_ceil(page_size);

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

/// Convert UEFI MemoryMapOwned to ABI MemoryMap representation.
fn convert_memory_map(mem: MemoryMapOwned) -> oxide_abi::MemoryMap {
    let meta = mem.meta();
    let buf = mem.buffer();

    let abi = oxide_abi::MemoryMap {
        // Physical address of the memory descriptors.
        descriptors_phys: buf.as_ptr() as u64,
        // use buf.len() instead of meta.map_size to reflect actual buffer size
        map_size: buf.len() as u64,
        // The reported memory descriptor size.
        entry_size: meta.desc_size as u32,
        // the version of the descriptor structure
        entry_version: meta.desc_version,
        // number of keys in the map
        entry_count: mem.len() as u32,
    };

    core::mem::forget(mem);

    abi
}

/// Safe code to build the BootAbi structure.
fn build_boot_abi(
    abi: &mut BootAbi,
    fw: FirmwareInfo,
    fb: FramebufferInfo,
    options: BootOptions,
    tsc_frequency_hz: Option<u64>,
    mem: MemoryMapOwned,
) {
    abi.firmware = fw.into();
    abi.framebuffer = fb.into();
    abi.options = options.into();
    abi.tsc_frequency_hz = tsc_frequency_hz.unwrap_or(0);
    abi.memory_map = convert_memory_map(mem);
}

/// Unsafe wrapper to build BootAbi from raw pointer.
/// Since we're lying to the borrow checker, caller must ensure pointer validity.
/// But lie in one place and don't infect the safe wrapper.
pub fn build_boot_abi_from_ptr(
    abi_ptr: *mut BootAbi,
    fw: FirmwareInfo,
    fb: FramebufferInfo,
    options: BootOptions,
    tsc_frequency_hz: Option<u64>,
    mem: MemoryMapOwned,
) {
    unsafe {
        let abi = &mut *abi_ptr;
        build_boot_abi(abi, fw, fb, options, tsc_frequency_hz, mem);
    }
}

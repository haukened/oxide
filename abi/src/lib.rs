#![no_std]

/// the static version of the ABI
pub const ABI_VERSION: u32 = 1;

/// Shared ABI between the UEFI loader and Oxide kernel.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct BootAbi {
    pub version: u32,
    pub options: Options,
    pub firmware: Firmware,
    pub framebuffer: Framebuffer,
    pub memory_map: MemoryMap,
}

/// Boot options from the loader to kernel.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct Options {
    /// Debug flag (1 = enabled, 0 = disabled).
    pub debug: u8,
    /// Quiet flag (1 = enabled, 0 = disabled).
    pub quiet: u8,
}

/// Numeric identifiers for UEFI memory types.
/// These correspond to `EFI_MEMORY_TYPE` values returned in UEFI memory maps.
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EfiMemoryType {
    /// Not used / undefined region.
    ReservedMemoryType = 0,
    /// Memory occupied by a loaded UEFI application’s code.
    LoaderCode = 1,
    /// Memory occupied by a loaded UEFI application’s data.
    LoaderData = 2,
    /// Memory containing boot services driver code.
    BootServicesCode = 3,
    /// Memory containing boot services driver data.
    BootServicesData = 4,
    /// Memory containing runtime services driver code.
    RuntimeServicesCode = 5,
    /// Memory containing runtime services driver data.
    RuntimeServicesData = 6,
    /// Free (unallocated) memory available for OS use.
    ConventionalMemory = 7,
    /// Memory that firmware has marked unusable due to errors.
    UnusableMemory = 8,
    /// Memory containing ACPI tables that can be reclaimed by the OS.
    ACPIReclaimMemory = 9,
    /// Memory reserved by firmware for ACPI non-volatile storage.
    ACPIMemoryNVS = 10,
    /// Region of memory mapped I/O for firmware/UEFI use.
    MemoryMappedIO = 11,
    /// Region of memory mapped I/O Port Space.
    MemoryMappedIOPortSpace = 12,
    /// Reserved by firmware for processor-specific code.
    PalCode = 13,
    /// Persistent memory (byte-addressable non-volatile).
    PersistentMemory = 14,
    /// Max defined memory type + sentinel (not a usable type).
    MaxMemoryType = 15,
    // EFI spec allows for future memory types beyond this value.
    // As such, this enum is not exhaustive, and should be used as a semantic wrapper only
    // not as a storage type for raw UEFI memory type values.
}

/// Firmware info for the kernel.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Firmware {
    /// Firmware revision number.
    pub revision: u32,
    /// Firmware vendor string as UTF-8.
    pub vendor: [u8; 32],
    /// Length of the vendor string in bytes.
    pub vendor_len: u8,
    /// True if the vendor string was truncated.
    pub vendor_truncated: u8,
}

/// Framebuffer info for early output.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Framebuffer {
    /// Physical address of the linear framebuffer.
    pub base_address: u64,
    /// Size of the framebuffer in bytes.
    pub buffer_size: u64,
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
    /// Number of pixels per scanline.
    pub stride: u32,
    /// Pixel format.
    pub pixel_format: PixelFormat,
}

/// A minimal UEFI memory range descriptor.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct MemoryDescriptor {
    /// Type of memory region as raw EFI_MEMORY_TYPE value.
    pub typ: u32,
    /// Padding for 8-byte alignment.
    pub _pad: u32,
    /// Physical start address of the region, identity-mapped at entry
    /// Virtual Address == Physical Address when passed to the kernel.
    pub physical_start: u64,
    /// The number of 4 KiB pages in the region.
    pub number_of_pages: u64,
    /// Bitmask Attribute flags for the region. (See UEFI spec.)
    pub attribute: u64,
}

/// A snapshot of the memory map.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct MemoryMap {
    /// Physical address of descriptor buffer (loader must allocate).
    pub descriptors_phys: u64,
    /// Valid number of bytes in the buffer.
    pub map_size: u64,
    /// Descriptor stride (UEFI uses this).
    pub entry_size: u32,
    /// UEFI descriptor version.
    pub entry_version: u32,
    /// Convenience: number of entries (map_size / entry_size).
    pub entry_count: u32,
}

/// Pixel format of framebuffer.
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PixelFormat {
    /// PixelRedGreenBlueReserved8BitPerColor.
    Rgb = 0,
    /// PixelBlueGreenRedReserved8BitPerColor.
    Bgr = 1,
}

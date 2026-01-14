//! Validation helpers for the loader-to-kernel handoff.

use core::mem::{align_of, size_of};

use oxide_abi::{ABI_VERSION, BootAbi, Framebuffer, MemoryDescriptor, MemoryMap, PixelFormat};

/// Errors that can occur while validating loader-provided boot data.
#[derive(Debug)]
pub enum BootValidationError {
    VersionMismatch { expected: u32, found: u32 },
    FramebufferInvalid(&'static str),
    MemoryMapInvalid(&'static str),
}

/// Validate the loader handoff structure before the kernel touches its fields.
///
/// Ensures the ABI version matches, framebuffer geometry is sane, and the
/// memory-map metadata falls within expected bounds.
pub fn validate_boot_abi(abi: &BootAbi) -> Result<(), BootValidationError> {
    if abi.version != ABI_VERSION {
        return Err(BootValidationError::VersionMismatch {
            expected: ABI_VERSION,
            found: abi.version,
        });
    }

    validate_framebuffer(&abi.framebuffer)?;
    validate_memory_map(&abi.memory_map)?;

    Ok(())
}

fn validate_framebuffer(fb: &Framebuffer) -> Result<(), BootValidationError> {
    if fb.base_address == 0 {
        return Err(BootValidationError::FramebufferInvalid(
            "framebuffer base address is null",
        ));
    }

    if fb.buffer_size == 0 {
        return Err(BootValidationError::FramebufferInvalid(
            "framebuffer buffer size is zero",
        ));
    }

    if fb.width == 0 || fb.height == 0 {
        return Err(BootValidationError::FramebufferInvalid(
            "framebuffer dimensions are zero",
        ));
    }

    if fb.pixels_per_scanline == 0 {
        return Err(BootValidationError::FramebufferInvalid(
            "pixels per scanline is zero",
        ));
    }

    if fb.pixels_per_scanline < fb.width {
        return Err(BootValidationError::FramebufferInvalid(
            "pixels per scanline smaller than width",
        ));
    }

    match fb.pixel_format {
        PixelFormat::Rgb | PixelFormat::Bgr => {}
    }

    let bytes_per_pixel = size_of::<u32>() as u128;
    let stride = fb.pixels_per_scanline as u128;
    let height = fb.height as u128;
    let required_bytes = bytes_per_pixel
        .saturating_mul(stride)
        .saturating_mul(height);

    if fb.buffer_size < required_bytes as u64 {
        return Err(BootValidationError::FramebufferInvalid(
            "framebuffer buffer smaller than required size",
        ));
    }

    Ok(())
}

fn validate_memory_map(map: &MemoryMap) -> Result<(), BootValidationError> {
    if map.descriptors_phys == 0 {
        return Err(BootValidationError::MemoryMapInvalid(
            "descriptor buffer address is null",
        ));
    }

    let required_alignment = align_of::<MemoryDescriptor>() as u64;
    if required_alignment > 0 && !map.descriptors_phys.is_multiple_of(required_alignment) {
        return Err(BootValidationError::MemoryMapInvalid(
            "descriptor buffer address not aligned",
        ));
    }

    if map.entry_size == 0 {
        return Err(BootValidationError::MemoryMapInvalid("entry size is zero"));
    }

    if map.map_size == 0 {
        return Err(BootValidationError::MemoryMapInvalid("map size is zero"));
    }

    let descriptor_size = size_of::<MemoryDescriptor>() as u32;
    if map.entry_size < descriptor_size {
        return Err(BootValidationError::MemoryMapInvalid(
            "entry size smaller than memory descriptor",
        ));
    }

    if map.entry_count == 0 {
        return Err(BootValidationError::MemoryMapInvalid(
            "no memory descriptors",
        ));
    }

    let entry_size = map.entry_size as u64;
    if !map.map_size.is_multiple_of(entry_size) {
        return Err(BootValidationError::MemoryMapInvalid(
            "map size not divisible by entry size",
        ));
    }

    let max_entries = map.map_size / entry_size;
    if map.entry_count as u64 > max_entries {
        return Err(BootValidationError::MemoryMapInvalid(
            "entry count exceeds buffer capacity",
        ));
    }

    Ok(())
}

#[cfg(test)]
extern crate std;

#[cfg(test)]
mod tests {
    use super::*;
    use oxide_abi::{BootAbi, Firmware, Options, PixelFormat};

    fn valid_framebuffer() -> Framebuffer {
        Framebuffer {
            base_address: 0x1000,
            buffer_size: 2_000_000,
            width: 800,
            height: 600,
            pixels_per_scanline: 800,
            pixel_format: PixelFormat::Rgb,
        }
    }

    fn valid_memory_map() -> MemoryMap {
        let entry_size = core::mem::size_of::<MemoryDescriptor>() as u32;
        MemoryMap {
            descriptors_phys: 0x2000,
            map_size: entry_size as u64 * 4,
            entry_size,
            entry_version: 1,
            entry_count: 4,
        }
    }

    fn empty_firmware() -> Firmware {
        Firmware {
            revision: 0,
            vendor: [0; 32],
            vendor_len: 0,
            vendor_truncated: 0,
        }
    }

    fn valid_boot_abi() -> BootAbi {
        BootAbi {
            version: ABI_VERSION,
            options: Options::default(),
            firmware: empty_firmware(),
            framebuffer: valid_framebuffer(),
            tsc_frequency_hz: 0,
            memory_map: valid_memory_map(),
        }
    }

    #[test]
    fn validate_boot_abi_accepts_valid_data() {
        let abi = valid_boot_abi();
        assert!(validate_boot_abi(&abi).is_ok());
    }

    #[test]
    fn validate_boot_abi_rejects_version_mismatch() {
        let mut abi = valid_boot_abi();
        abi.version = ABI_VERSION + 1;
        assert!(matches!(
            validate_boot_abi(&abi),
            Err(BootValidationError::VersionMismatch { expected, found })
                if expected == ABI_VERSION && found == ABI_VERSION + 1
        ));
    }

    #[test]
    fn validate_framebuffer_rejects_null_base() {
        let mut fb = valid_framebuffer();
        fb.base_address = 0;
        assert!(matches!(
            validate_framebuffer(&fb),
            Err(BootValidationError::FramebufferInvalid(reason))
                if reason.contains("base address")
        ));
    }

    #[test]
    fn validate_framebuffer_rejects_small_buffer() {
        let mut fb = valid_framebuffer();
        fb.buffer_size = 1;
        assert!(matches!(
            validate_framebuffer(&fb),
            Err(BootValidationError::FramebufferInvalid(reason))
                if reason.contains("smaller")
        ));
    }

    #[test]
    fn validate_framebuffer_requires_nonzero_buffer_size() {
        let mut fb = valid_framebuffer();
        fb.buffer_size = 0;
        assert!(matches!(
            validate_framebuffer(&fb),
            Err(BootValidationError::FramebufferInvalid(reason))
                if reason.contains("buffer size is zero")
        ));
    }

    #[test]
    fn validate_framebuffer_rejects_zero_dimensions() {
        let mut fb = valid_framebuffer();
        fb.width = 0;
        assert!(matches!(
            validate_framebuffer(&fb),
            Err(BootValidationError::FramebufferInvalid(reason))
                if reason.contains("dimensions")
        ));
    }

    #[test]
    fn validate_framebuffer_requires_pixels_per_scanline() {
        let mut fb = valid_framebuffer();
        fb.pixels_per_scanline = 0;
        assert!(matches!(
            validate_framebuffer(&fb),
            Err(BootValidationError::FramebufferInvalid(reason))
                if reason.contains("scanline is zero")
        ));
    }

    #[test]
    fn validate_framebuffer_rejects_stride_smaller_than_width() {
        let mut fb = valid_framebuffer();
        fb.pixels_per_scanline = fb.width - 1;
        assert!(matches!(
            validate_framebuffer(&fb),
            Err(BootValidationError::FramebufferInvalid(reason))
                if reason.contains("smaller than width")
        ));
    }

    #[test]
    fn validate_framebuffer_allows_bgr_pixel_format() {
        let mut fb = valid_framebuffer();
        fb.pixel_format = PixelFormat::Bgr;
        assert!(validate_framebuffer(&fb).is_ok());
    }

    #[test]
    fn validate_memory_map_rejects_unaligned_buffer() {
        let mut map = valid_memory_map();
        map.descriptors_phys = 0x1234;
        assert!(matches!(
            validate_memory_map(&map),
            Err(BootValidationError::MemoryMapInvalid(reason))
                if reason.contains("aligned")
        ));
    }

    #[test]
    fn validate_memory_map_rejects_excess_entries() {
        let mut map = valid_memory_map();
        map.entry_count = 10;
        assert!(matches!(
            validate_memory_map(&map),
            Err(BootValidationError::MemoryMapInvalid(reason))
                if reason.contains("count exceeds")
        ));
    }

    #[test]
    fn validate_memory_map_requires_nonzero_entry_size() {
        let mut map = valid_memory_map();
        map.entry_size = 0;
        assert!(matches!(
            validate_memory_map(&map),
            Err(BootValidationError::MemoryMapInvalid(reason))
                if reason.contains("entry size is zero")
        ));
    }

    #[test]
    fn validate_memory_map_requires_nonzero_map_size() {
        let mut map = valid_memory_map();
        map.map_size = 0;
        assert!(matches!(
            validate_memory_map(&map),
            Err(BootValidationError::MemoryMapInvalid(reason))
                if reason.contains("map size is zero")
        ));
    }

    #[test]
    fn validate_memory_map_rejects_descriptor_smaller_than_expected() {
        let mut map = valid_memory_map();
        map.entry_size = (core::mem::size_of::<MemoryDescriptor>() as u32) - 1;
        assert!(matches!(
            validate_memory_map(&map),
            Err(BootValidationError::MemoryMapInvalid(reason))
                if reason.contains("smaller than memory descriptor")
        ));
    }

    #[test]
    fn validate_memory_map_requires_entries_present() {
        let mut map = valid_memory_map();
        map.entry_count = 0;
        assert!(matches!(
            validate_memory_map(&map),
            Err(BootValidationError::MemoryMapInvalid(reason))
                if reason.contains("no memory descriptors")
        ));
    }

    #[test]
    fn validate_memory_map_requires_map_size_multiple_of_entry_size() {
        let mut map = valid_memory_map();
        map.map_size = map.entry_size as u64 * map.entry_count as u64 + 1;
        assert!(matches!(
            validate_memory_map(&map),
            Err(BootValidationError::MemoryMapInvalid(reason))
                if reason.contains("not divisible")
        ));
    }

    #[test]
    fn validate_memory_map_requires_nonzero_descriptor_buffer() {
        let mut map = valid_memory_map();
        map.descriptors_phys = 0;
        assert!(matches!(
            validate_memory_map(&map),
            Err(BootValidationError::MemoryMapInvalid(reason))
                if reason.contains("address is null")
        ));
    }
}

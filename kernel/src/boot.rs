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

    if required_bytes == 0 {
        return Err(BootValidationError::FramebufferInvalid(
            "required framebuffer bytes underflow",
        ));
    }

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
    if required_alignment > 0 && map.descriptors_phys % required_alignment != 0 {
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
    if map.map_size % entry_size != 0 {
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

#![no_std]
#![no_main]
use core::cmp::min;
use oxide_abi::{BootAbi, Framebuffer, PixelFormat};

/// Kernel entry point called from the UEFI loader.
#[unsafe(no_mangle)]
pub extern "C" fn kernel_main(boot_abi_ptr: *const BootAbi) -> ! {
    // SAFETY: caller (the UEFI loader) must ensure the pointer is valid at entry
    let boot_abi = unsafe { &*boot_abi_ptr };

    let _ = draw_boot_marker(&boot_abi.framebuffer);

    loop {
        core::hint::spin_loop();
    }
}

fn draw_boot_marker(fb: &Framebuffer) -> Result<(), ()> {
    if fb.base_address == 0 || fb.buffer_size < core::mem::size_of::<u32>() as u64 {
        return Err(());
    }

    let pitch = fb.pixels_per_scanline as usize;
    if pitch == 0 {
        return Err(());
    }

    let width = fb.width as usize;
    let height = fb.height as usize;
    if width == 0 || height == 0 {
        return Err(());
    }

    const MARKER_SIZE: usize = 64;
    let draw_width = min(width, min(pitch, MARKER_SIZE));
    let draw_height = min(height, MARKER_SIZE);
    if draw_width == 0 || draw_height == 0 {
        return Err(());
    }

    let max_pixels = (fb.buffer_size as usize) / core::mem::size_of::<u32>();
    if max_pixels == 0 {
        return Err(());
    }

    let last_row = draw_height.checked_sub(1).ok_or(())?;
    let row_offset = last_row.checked_mul(pitch).ok_or(())?;
    let last_col = draw_width.checked_sub(1).ok_or(())?;
    let last_index = row_offset.checked_add(last_col).ok_or(())?;
    if last_index >= max_pixels {
        return Err(());
    }

    let red: u8 = 0xFF;
    let green: u8 = 0x40;
    let blue: u8 = 0x20;
    let color = match fb.pixel_format {
        PixelFormat::Rgb => u32::from_le_bytes([red, green, blue, 0]),
        PixelFormat::Bgr => u32::from_le_bytes([blue, green, red, 0]),
    };

    let base_ptr = fb.base_address as *mut u32;
    unsafe {
        for y in 0..draw_height {
            let row_ptr = base_ptr.add(y * pitch);
            for x in 0..draw_width {
                row_ptr.add(x).write_volatile(color);
            }
        }
    }

    Ok(())
}

#[cfg(feature = "standalone")]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {
        core::hint::spin_loop();
    }
}

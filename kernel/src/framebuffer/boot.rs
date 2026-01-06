use crate::framebuffer::{FramebufferColor, draw::panic_screen, draw_rect};

const BOOT_STAGE_SIZE_X: usize = 64;
const BOOT_STAGE_SIZE_Y: usize = 32;

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(dead_code)]
pub enum BootStage {
    EnteredKernel = 0,
    ParsedMemoryMap = 1,
    FoundUsableMemory = 2,
    FrameAllocated = 3,
    PagingEnabled = 4,
    // add more as needed
}

fn boot_stage_color(stage: BootStage) -> FramebufferColor {
    match stage {
        BootStage::EnteredKernel => FramebufferColor::RED,
        BootStage::ParsedMemoryMap => FramebufferColor::ORANGE,
        BootStage::FoundUsableMemory => FramebufferColor::YELLOW,
        BootStage::FrameAllocated => FramebufferColor::GREEN,
        BootStage::PagingEnabled => FramebufferColor::BLUE,
    }
}

fn boot_stage_position_x(stage: BootStage) -> usize {
    (stage as usize) * BOOT_STAGE_SIZE_X
}

/// Draw a boot stage indicator rectangle at the top of the screen.
pub fn draw_boot_stage(fb: &oxide_abi::Framebuffer, stage: BootStage) {
    let color = boot_stage_color(stage);
    let pos_x = boot_stage_position_x(stage);
    draw_rect(fb, pos_x, 0, BOOT_STAGE_SIZE_X, BOOT_STAGE_SIZE_Y, color).unwrap_or_else(|_| {
        panic_screen(fb);
    });
}

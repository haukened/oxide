use uefi::system;

use crate::writer::FixedBufWriter;

#[allow(dead_code)]
pub struct FirmwareInfo {
    pub revision: u32,
    pub vendor: [u8; 63],
    pub vsize: u8,
}

impl FirmwareInfo {
    pub fn vendor_str(&self) -> &str {
        let len = self.vsize as usize;
        core::str::from_utf8(&self.vendor[..len]).unwrap_or("INVALID UTF-8")
    }
}

pub fn get_info() -> FirmwareInfo {
    let vendor16 = system::firmware_vendor();
    let revision = system::firmware_revision();
    let (vendor, vsize) = copy_vendor_string(&vendor16);

    FirmwareInfo {
        revision,
        vendor,
        vsize,
    }
}

fn copy_vendor_string(vendor: &uefi::CStr16) -> ([u8; 63], u8) {
    let mut buf = [0u8; 63];
    let mut writer = FixedBufWriter::new(&mut buf);

    let _ = vendor.as_str_in_buf(&mut writer);
    let len = writer.len() as u8;

    (buf, len)
}

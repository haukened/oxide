use crate::writer::FixedBufWriter;
use uefi::system;

/// Maximum number of UTF-8 bytes we capture from the firmware vendor string.
const VENDOR_CAP: usize = 32;

/// Firmware revision and vendor metadata captured at startup.
#[allow(dead_code)]
pub struct FirmwareInfo {
    pub revision: u32,
    vendor: [u8; VENDOR_CAP],
    vendor_len: usize,
    vendor_truncated: bool,
}

#[allow(dead_code)]
impl FirmwareInfo {
    /// Return the firmware vendor as UTF-8, falling back to a placeholder if invalid.
    pub fn vendor_str(&self) -> &str {
        core::str::from_utf8(&self.vendor[..self.vendor_len]).unwrap_or("INVALID UTF-8")
    }

    /// True when the vendor string exceeded our fixed buffer and had to be truncated.
    pub fn vendor_was_truncated(&self) -> bool {
        self.vendor_truncated
    }
}

/// Query UEFI runtime services for basic firmware metadata.
pub fn get_info() -> FirmwareInfo {
    let vendor16 = system::firmware_vendor();
    let revision = system::firmware_revision();
    let (vendor, vendor_len, vendor_truncated) = copy_vendor_string(&vendor16);

    FirmwareInfo {
        revision,
        vendor,
        vendor_len,
        vendor_truncated,
    }
}

/// Convert the firmware vendor from UCS-2 to UTF-8 within a bounded buffer.
fn copy_vendor_string(vendor: &uefi::CStr16) -> ([u8; VENDOR_CAP], usize, bool) {
    let mut buf = [0u8; VENDOR_CAP];
    let mut writer = FixedBufWriter::new(&mut buf);

    let truncated = vendor.as_str_in_buf(&mut writer).is_err();
    let len = writer.len();

    (buf, len, truncated)
}

use uefi::system;

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

struct FixedBufWriter<'a> {
    buf: &'a mut [u8],
    len: usize,
}

impl<'a> FixedBufWriter<'a> {
    fn new(buf: &'a mut [u8]) -> Self {
        FixedBufWriter { buf, len: 0 }
    }

    fn len(&self) -> usize {
        self.len
    }
}

impl<'a> core::fmt::Write for FixedBufWriter<'a> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let bytes = s.as_bytes();
        let remaining = self.buf.len() - self.len;
        let to_copy = bytes.len().min(remaining);

        self.buf[self.len..self.len + to_copy].copy_from_slice(&bytes[..to_copy]);
        self.len += to_copy;

        Ok(())
    }
}

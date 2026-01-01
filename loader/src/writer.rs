/// Minimal fixed-size buffer that implements `core::fmt::Write` without allocation.
pub struct FixedBufWriter<'a> {
    buf: &'a mut [u8],
    len: usize,
}

impl<'a> FixedBufWriter<'a> {
    /// Create a new writer wrapping the provided byte slice.
    pub fn new(buf: &'a mut [u8]) -> Self {
        FixedBufWriter { buf, len: 0 }
    }

    /// Number of bytes successfully written into the buffer so far.
    pub fn len(&self) -> usize {
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

        if to_copy == bytes.len() {
            Ok(())
        } else {
            Err(core::fmt::Error)
        }
    }
}

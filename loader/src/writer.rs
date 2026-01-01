pub struct FixedBufWriter<'a> {
    buf: &'a mut [u8],
    len: usize,
}

impl<'a> FixedBufWriter<'a> {
    pub fn new(buf: &'a mut [u8]) -> Self {
        FixedBufWriter { buf, len: 0 }
    }

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

        Ok(())
    }
}

pub struct AsciiCharIterator<'s> {
    utf8_string: &'s [u8],
    byte_index: usize,
}

impl<'s> AsciiCharIterator<'s> {
    #[inline]
    pub const fn new(string: &'s str) -> Self {
        Self {
            utf8_string: string.as_bytes(),
            byte_index: 0,
        }
    }
}

impl Iterator for AsciiCharIterator<'_> {
    type Item = u8;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let byte =  self.utf8_string.get(self.byte_index).copied()?;
            self.byte_index += 1;
            if byte.is_ascii() {
                break Some(byte);
            }
            if byte >= 0b1100_0000 {  // UTF-8 start byte
                break Some(u8::MAX);
            }
        }
    }
}

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
        let byte = *self.utf8_string.get(self.byte_index)?;

        let leading_ones = byte.leading_ones();
        if leading_ones == 0 {
            self.byte_index += 1;
            Some(byte)
        } else {
            self.byte_index += leading_ones as usize;
            debug_assert_ne!(leading_ones, 1);
            Some(u8::MAX)
        }
    }
}

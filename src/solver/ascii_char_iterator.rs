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
        let byte = self.utf8_string.get(self.byte_index).copied()?;
        if byte.is_ascii() {
            self.byte_index += 1;
            Some(byte)
        } else {
            self.byte_index += if byte >= 0b1111_0000 {
                4
            } else if byte >= 0b1110_0000 {
                3
            } else {
                debug_assert!(byte >= 0b1100_0000);
                2
            };
            Some(u8::MAX)
        }
    }
}

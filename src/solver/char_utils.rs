pub trait CharUtils {
    #[expect(clippy::wrong_self_convention)]
    fn is_whitespace(self) -> bool;
    fn to_char(self) -> char;
    #[expect(clippy::wrong_self_convention)]
    fn is_ascii(self) -> bool;
    fn to_ascii_char(self) -> Option<u8>;
}

impl CharUtils for char {
    #[inline]
    fn is_whitespace(self) -> bool {
        Self::is_whitespace(self)
    }

    #[inline]
    fn to_char(self) -> char {
        self
    }

    #[inline]
    fn is_ascii(self) -> bool {
        Self::is_ascii(&self)
    }

    #[inline]
    fn to_ascii_char(self) -> Option<u8> {
        if self.is_ascii() {
            Some(self as u8)
        } else {
            None
        }
    }
}

impl CharUtils for u8 {
    #[inline]
    fn is_whitespace(self) -> bool {
        debug_assert!(Self::is_ascii(&self));

        self.is_ascii_whitespace()
    }

    #[inline]
    fn to_char(self) -> char {
        debug_assert!(Self::is_ascii(&self));

        char::from(self)
    }

    #[inline]
    fn is_ascii(self) -> bool {
        debug_assert!(Self::is_ascii(&self));

        true
    }

    #[inline]
    fn to_ascii_char(self) -> Option<u8> {
        debug_assert!(Self::is_ascii(&self));

        Some(self)
    }
}

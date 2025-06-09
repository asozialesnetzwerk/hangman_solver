pub trait CharUtils {
    #[expect(clippy::wrong_self_convention)]
    fn is_whitespace(self) -> bool;
    fn to_char(self) -> char;
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
}

impl CharUtils for u8 {
    #[inline]
    fn is_whitespace(self) -> bool {
        debug_assert!(self.is_ascii());

        self.is_ascii_whitespace()
    }

    #[inline]
    fn to_char(self) -> char {
        debug_assert!(self.is_ascii());

        char::from(self)
    }
}

#[cfg(feature = "wasm-bindgen")]
use js_sys::JsString;

use crate::solver::ascii_char_iterator::AsciiCharIterator;

pub trait CharCollection {
    #[must_use]
    #[inline]
    #[allow(dead_code)]
    fn all_chars_are_ascii(&self) -> bool {
        self.iter_chars().all(|ch| ch.is_ascii())
    }

    #[must_use]
    #[inline]
    fn char_count(&self) -> usize {
        self.iter_chars().count()
    }

    #[must_use]
    #[inline]
    fn first_char(&self) -> Option<char> {
        self.iter_chars().next()
    }

    #[must_use]
    fn iter_chars(&self) -> impl Iterator<Item = char> + '_;

    /// Gets the first byte.
    /// If the byte is a valid ascii char it has the value of the ascii char.
    /// If it isn't the exact value is nonsensical and should not be relied upon.
    #[must_use]
    #[inline]
    fn first_ascii_char(&self) -> Option<u8> {
        self.iter_ascii_chars().next()
    }

    /// Iterates over `char_count()` bytes.
    /// If a byte is a valid ascii char it has the value of the ascii char.
    /// If it isn't the exact value is nonsensical and should not be relied upon.
    #[must_use]
    fn iter_ascii_chars(&self) -> impl Iterator<Item = u8> + '_;
}

impl CharCollection for String {
    #[inline]
    fn all_chars_are_ascii(&self) -> bool {
        self.is_ascii()
    }

    #[inline]
    fn char_count(&self) -> usize {
        self.chars().count()
    }

    #[inline]
    fn iter_chars(&self) -> impl Iterator<Item = char> + '_ {
        self.chars()
    }

    #[inline]
    fn first_ascii_char(&self) -> Option<u8> {
        self.as_bytes().first().copied()
    }

    #[inline]
    fn iter_ascii_chars(&self) -> impl Iterator<Item = u8> + '_ {
        AsciiCharIterator::new(self.as_str())
    }
}

impl CharCollection for str {
    #[inline]
    fn all_chars_are_ascii(&self) -> bool {
        self.is_ascii()
    }

    #[inline]
    fn char_count(&self) -> usize {
        self.chars().count()
    }

    #[inline]
    fn iter_chars(&self) -> impl Iterator<Item = char> + '_ {
        self.chars()
    }

    #[inline]
    fn first_ascii_char(&self) -> Option<u8> {
        self.as_bytes().first().copied()
    }

    #[inline]
    fn iter_ascii_chars(&self) -> impl Iterator<Item = u8> + '_ {
        AsciiCharIterator::new(self)
    }
}

impl CharCollection for &str {
    #[inline]
    fn all_chars_are_ascii(&self) -> bool {
        self.is_ascii()
    }

    #[inline]
    fn char_count(&self) -> usize {
        self.chars().count()
    }

    #[inline]
    fn iter_chars(&self) -> impl Iterator<Item = char> + '_ {
        self.chars()
    }

    #[inline]
    fn first_ascii_char(&self) -> Option<u8> {
        self.as_bytes().first().copied()
    }

    #[inline]
    fn iter_ascii_chars(&self) -> impl Iterator<Item = u8> + '_ {
        AsciiCharIterator::new(self)
    }
}

impl CharCollection for [char] {
    #[inline]
    fn char_count(&self) -> usize {
        self.len()
    }

    #[inline]
    fn iter_chars(&self) -> impl Iterator<Item = char> + '_ {
        self.iter().copied()
    }

    #[inline]
    fn iter_ascii_chars(&self) -> impl Iterator<Item = u8> + '_ {
        self.iter()
            .map(|ch| if ch.is_ascii() { *ch as u8 } else { u8::MAX })
    }
}

impl CharCollection for Vec<char> {
    #[inline]
    fn char_count(&self) -> usize {
        self.len()
    }

    #[inline]
    fn iter_chars(&self) -> impl Iterator<Item = char> + '_ {
        self.iter().copied()
    }

    #[inline]
    fn iter_ascii_chars(&self) -> impl Iterator<Item = u8> + '_ {
        self.iter()
            .map(|ch| if ch.is_ascii() { *ch as u8 } else { u8::MAX })
    }
}

#[cfg(feature = "wasm-bindgen")]
impl CharCollection for JsString {
    #[inline]
    fn iter_chars(&self) -> impl Iterator<Item = char> + '_ {
        self.iter().map(|u: u16| {
            char::from_u32(u32::from(u)).expect("failed to parse char")
        })
    }

    #[inline]
    fn iter_ascii_chars(&self) -> impl Iterator<Item = u8> + '_ {
        self.iter_chars()
            .map(|ch| if ch.is_ascii() { ch as u8 } else { u8::MAX })
    }
}

#[cfg(test)]
mod test {
    use itertools::Itertools;

    use crate::solver::char_collection::CharCollection;

    #[test]
    fn test_iter_ascii_chars_ascii() {
        let ascii_strings = ["Hello, world!", "abcde", "test"];

        for string in ascii_strings {
            assert!(string.is_ascii());
            assert!(string.all_chars_are_ascii());
            assert_eq!(string.char_count(), string.len());
            assert_eq!(string.iter_ascii_chars().count(), string.len());
            assert_eq!(
                string.iter_ascii_chars().next().map(|b| b.is_ascii()),
                Some(true)
            );
            assert_eq!(
                string.first_ascii_char().map(|b| b.is_ascii()),
                Some(true)
            );
            assert_eq!(
                string.iter_ascii_chars().next(),
                string.first_ascii_char()
            );
            assert_eq!(
                string.iter_ascii_chars().next(),
                string.as_bytes().first().copied()
            );
            assert_eq!(
                string.iter_ascii_chars().collect_vec().as_slice(),
                string.as_bytes()
            );
        }
    }

    #[test]
    fn test_iter_ascii_chars() {
        let strings = ["µ ASCII TEXT", "äöüßÄÖÜẞ", "🤓🦈"];

        for string in strings {
            assert!(!string.is_ascii());
            assert!(!string.all_chars_are_ascii());

            assert_eq!(
                string.iter_ascii_chars().count(),
                string.chars().count()
            );
            assert_eq!(
                string.iter_ascii_chars().next().map(|b| b.is_ascii()),
                Some(false)
            );
            assert_eq!(
                string.first_ascii_char().map(|b| b.is_ascii()),
                Some(false)
            );

            assert_eq!(
                string.iter_chars().map(|ch| ch.is_ascii()).collect_vec(),
                string
                    .iter_ascii_chars()
                    .map(|ch| ch.is_ascii())
                    .collect_vec()
            );
            assert_eq!(
                string
                    .iter_chars()
                    .map(|ch| if ch.is_ascii() { Some(ch as u8) } else { None })
                    .collect_vec(),
                string
                    .iter_ascii_chars()
                    .map(|ch| if ch.is_ascii() { Some(ch) } else { None })
                    .collect_vec()
            );
        }
    }
}

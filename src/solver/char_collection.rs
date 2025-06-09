#[cfg(feature = "wasm-bindgen")]
use js_sys::JsString;

use crate::solver::ascii_char_iterator::AsciiCharIterator;

#[expect(clippy::indexing_slicing)]
#[inline]
pub const fn count_codepoints(string: &str) -> usize {
    let string = string.as_bytes();
    let mut count = 0;

    let mut i = 0;
    loop {
        if i >= string.len() {
            break count;
        }
        let byte = string[i];

        if byte < 128 || byte >= 0b1100_0000 {
            count += 1;
        }

        i += 1;
    }
}

const _: () = assert!(count_codepoints("abcd1234") == 8);
const _: () = assert!(count_codepoints("äöüßÄÖÜẞ") == 8);

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
        count_codepoints(self)
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
        count_codepoints(self)
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
        count_codepoints(self)
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

    use crate::solver::char_collection::{CharCollection, count_codepoints};

    #[test]
    fn test_count_codepoints() {
        let strings = ["abcd1234", "äöüßÄÖÜẞ", "🤓🦈"];
        for string in strings {
            let count = string.chars().count();
            assert_eq!(count_codepoints(string), count);
            assert_eq!(string.char_count(), count);
            assert_eq!(string.iter_chars().count(), count);
            assert_eq!(string.iter_ascii_chars().count(), count);
            assert_eq!(string.all_chars_are_ascii(), string.is_ascii());
        }
    }

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

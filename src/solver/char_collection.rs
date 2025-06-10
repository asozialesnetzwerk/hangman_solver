// SPDX-License-Identifier: EUPL-1.2

pub trait CharCollection {
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
}

impl<CC: CharCollection> CharCollection for &CC {
    #[inline]
    fn iter_chars(&self) -> impl Iterator<Item = char> + '_ {
        CC::iter_chars(self)
    }

    #[inline]
    fn char_count(&self) -> usize {
        CC::char_count(self)
    }

    #[inline]
    fn first_char(&self) -> Option<char> {
        CC::first_char(self)
    }
}

impl CharCollection for String {
    #[inline]
    fn char_count(&self) -> usize {
        self.chars().count()
    }

    #[inline]
    fn iter_chars(&self) -> impl Iterator<Item = char> + '_ {
        self.chars()
    }
}

impl CharCollection for str {
    #[inline]
    fn char_count(&self) -> usize {
        self.chars().count()
    }

    #[inline]
    fn iter_chars(&self) -> impl Iterator<Item = char> + '_ {
        self.chars()
    }
}

impl CharCollection for &str {
    #[inline]
    fn char_count(&self) -> usize {
        self.chars().count()
    }

    #[inline]
    fn iter_chars(&self) -> impl Iterator<Item = char> + '_ {
        self.chars()
    }
}

impl<const L: usize> CharCollection for [char; L] {
    #[inline]
    fn char_count(&self) -> usize {
        L
    }

    #[inline]
    fn iter_chars(&self) -> impl Iterator<Item = char> + '_ {
        self.iter().copied()
    }

    #[inline]
    fn first_char(&self) -> Option<char> {
        if L > 0 { self.first().copied() } else { None }
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
}

#[cfg(feature = "wasm-bindgen")]
use js_sys::JsString;

#[cfg(feature = "wasm-bindgen")]
impl CharCollection for JsString {
    #[inline]
    fn iter_chars(&self) -> impl Iterator<Item = char> + '_ {
        self.iter().map(|u: u16| {
            char::from_u32(u32::from(u)).expect("failed to parse char")
        })
    }
}

#[cfg(test)]
mod test {
    use itertools::Itertools;

    use crate::solver::char_collection::CharCollection;

    #[test]
    fn test_iter_ascii() {
        let ascii_strings = ["Hello, world!", "abcde", "test"];

        for string in ascii_strings {
            assert!(string.is_ascii());
            assert_eq!(string.char_count(), string.len());
        }
    }

    #[test]
    fn test_iter_ascii_chars() {
        let strings = ["µ ASCII TEXT", "äöüßÄÖÜẞ", "🤓🦈"];

        for string in strings.map(String::from) {
            assert!(!string.is_ascii());
            assert!(string.first_char().is_some());
            assert_eq!(string.chars().count(), string.char_count());
            assert_eq!(string.chars().next(), string.first_char());
            assert_eq!(
                string.chars().collect_vec(),
                string.iter_chars().collect_vec()
            );
        }

        for string in strings {
            assert!(!string.is_ascii());
            assert!(string.first_char().is_some());
            assert_eq!(string.chars().count(), string.char_count());
            assert_eq!(string.chars().next(), string.first_char());
            assert_eq!(
                string.chars().collect_vec(),
                string.iter_chars().collect_vec()
            );
        }
    }
}

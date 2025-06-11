// SPDX-License-Identifier: EUPL-1.2
use std::convert::Infallible;

pub trait InfallibleCharCollection {
    #[must_use]
    fn char_count(&self) -> usize;

    #[must_use]
    fn first_char(&self) -> Option<char>;

    #[must_use]
    fn iter_chars(&self) -> impl Iterator<Item = char> + '_;
}

impl<CC: CharCollection<Error = Infallible> + ?Sized> InfallibleCharCollection
    for CC
{
    #[inline]
    fn char_count(&self) -> usize {
        CC::char_count(self).unwrap_infallible()
    }

    #[inline]
    fn first_char(&self) -> Option<char> {
        CC::first_char(self).unwrap_infallible()
    }

    #[inline]
    fn iter_chars(&self) -> impl Iterator<Item = char> + '_ {
        CC::iter_chars(self)
            .unwrap_infallible()
            .map(UnwrapInfallible::unwrap_infallible)
    }
}

pub trait CharCollection {
    type Error;

    #[inline]
    fn char_count(&self) -> Result<usize, Self::Error> {
        Ok(self.iter_chars()?.count())
    }

    #[inline]
    fn first_char(&self) -> Result<Option<char>, Self::Error> {
        if let Some(x) = self.iter_chars()?.next() {
            Ok(Some(x?))
        } else {
            Ok(None)
        }
    }

    fn iter_chars(
        &self,
    ) -> Result<impl Iterator<Item = Result<char, Self::Error>> + '_, Self::Error>;
}

impl<Err, CC: CharCollection<Error = Err>> CharCollection for &CC {
    type Error = Err;

    #[inline]
    fn iter_chars(
        &self,
    ) -> Result<impl Iterator<Item = Result<char, Self::Error>> + '_, Self::Error>
    {
        CC::iter_chars(self)
    }

    #[inline]
    fn char_count(&self) -> Result<usize, Self::Error> {
        CC::char_count(self)
    }

    #[inline]
    fn first_char(&self) -> Result<Option<char>, Self::Error> {
        CC::first_char(self)
    }
}

impl CharCollection for String {
    type Error = Infallible;

    #[inline]
    fn char_count(&self) -> Result<usize, Self::Error> {
        Ok(self.chars().count())
    }

    #[inline]
    fn first_char(&self) -> Result<Option<char>, Self::Error> {
        Ok(self.chars().next())
    }

    #[inline]
    fn iter_chars(
        &self,
    ) -> Result<impl Iterator<Item = Result<char, Self::Error>> + '_, Self::Error>
    {
        Ok(self.chars().map(Result::Ok))
    }
}

impl CharCollection for str {
    type Error = Infallible;

    #[inline]
    fn char_count(&self) -> Result<usize, Self::Error> {
        Ok(self.chars().count())
    }

    #[inline]
    fn first_char(&self) -> Result<Option<char>, Self::Error> {
        Ok(self.chars().next())
    }

    #[inline]
    fn iter_chars(
        &self,
    ) -> Result<impl Iterator<Item = Result<char, Self::Error>> + '_, Self::Error>
    {
        Ok(self.chars().map(Result::Ok))
    }
}

impl CharCollection for &str {
    type Error = Infallible;

    #[inline]
    fn char_count(&self) -> Result<usize, Self::Error> {
        Ok(self.chars().count())
    }

    #[inline]
    fn first_char(&self) -> Result<Option<char>, Self::Error> {
        Ok(self.chars().next())
    }

    #[inline]
    fn iter_chars(
        &self,
    ) -> Result<impl Iterator<Item = Result<char, Self::Error>> + '_, Self::Error>
    {
        Ok(self.chars().map(Result::Ok))
    }
}

impl<const L: usize> CharCollection for [char; L] {
    type Error = Infallible;

    #[inline]
    fn char_count(&self) -> Result<usize, Self::Error> {
        Ok(L)
    }

    #[inline]
    fn iter_chars(
        &self,
    ) -> Result<impl Iterator<Item = Result<char, Self::Error>> + '_, Self::Error>
    {
        Ok(self.iter().copied().map(Result::Ok))
    }

    #[inline]
    fn first_char(&self) -> Result<Option<char>, Self::Error> {
        if L > 0 {
            Ok(self.first().copied())
        } else {
            Ok(None)
        }
    }
}

impl CharCollection for [char] {
    type Error = Infallible;

    #[inline]
    fn char_count(&self) -> Result<usize, Self::Error> {
        Ok(self.len())
    }

    #[inline]
    fn iter_chars(
        &self,
    ) -> Result<impl Iterator<Item = Result<char, Self::Error>> + '_, Self::Error>
    {
        Ok(self.iter().copied().map(Result::Ok))
    }

    #[inline]
    fn first_char(&self) -> Result<Option<char>, Self::Error> {
        Ok(self.first().copied())
    }
}

impl CharCollection for Vec<char> {
    type Error = Infallible;

    #[inline]
    fn char_count(&self) -> Result<usize, Self::Error> {
        Ok(self.len())
    }

    #[inline]
    fn iter_chars(
        &self,
    ) -> Result<impl Iterator<Item = Result<char, Self::Error>> + '_, Self::Error>
    {
        Ok(self.iter().copied().map(Result::Ok))
    }

    #[inline]
    fn first_char(&self) -> Result<Option<char>, Self::Error> {
        Ok(self.first().copied())
    }
}

#[cfg(feature = "wasm-bindgen")]
use js_sys::JsString;

#[cfg(feature = "wasm-bindgen")]
struct CodepointIterator<'js> {
    js_string: &'js JsString,
    utf16_idx: u32,
}

#[cfg(feature = "wasm-bindgen")]
impl Iterator for CodepointIterator<'_> {
    type Item = char;

    #[inline]
    #[expect(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
    fn next(&mut self) -> Option<Self::Item> {
        if self.utf16_idx >= self.js_string.length() {
            None
        } else {
            use wasm_bindgen::{JsValue, UnwrapThrowExt};

            let x: JsValue = self.js_string.code_point_at(self.utf16_idx);

            let ch = x.as_f64().unwrap_throw() as u32;

            if ch > u32::from(u16::MAX) {
                self.utf16_idx += 2;
            } else {
                self.utf16_idx += 1;
            }

            Some(char::try_from(ch).unwrap_throw())
        }
    }
}

#[cfg(feature = "wasm-bindgen")]
impl CharCollection for JsString {
    type Error = Infallible;

    #[inline]
    fn iter_chars(
        &self,
    ) -> Result<impl Iterator<Item = Result<char, Self::Error>> + '_, Self::Error>
    {
        let it = CodepointIterator {
            js_string: self,
            utf16_idx: 0,
        };
        Ok(it.map(Result::Ok))
    }
}

#[cfg(feature = "pyo3")]
use pyo3::prelude::*;
use unwrap_infallible::UnwrapInfallible;

#[cfg(feature = "pyo3")]
impl CharCollection for pyo3::Bound<'_, pyo3::types::PyString> {
    type Error = PyErr;

    #[inline]
    fn char_count(&self) -> PyResult<usize> {
        self.len()
    }

    #[inline]
    fn first_char(&self) -> PyResult<Option<char>> {
        if self.char_count()? == 0 {
            Ok(None)
        } else {
            Ok(Some(self.get_item(0)?.extract()?))
        }
    }

    #[inline]
    fn iter_chars(
        &self,
    ) -> PyResult<impl Iterator<Item = Result<char, Self::Error>> + '_> {
        Ok(self.try_iter()?.map(|ch| ch?.extract::<char>()))
    }
}

#[cfg(test)]
mod test {
    use itertools::Itertools;
    use unwrap_infallible::UnwrapInfallible;

    use crate::solver::char_collection::CharCollection;

    #[test]
    fn test_iter_ascii() {
        let ascii_strings = ["Hello, world!", "abcde", "test"];

        for string in ascii_strings {
            assert!(string.is_ascii());
            assert_eq!(string.char_count().unwrap_infallible(), string.len());
        }
    }

    #[test]
    fn test_iter_ascii_chars() {
        let strings = ["µ ASCII TEXT", "äöüßÄÖÜẞ", "🤓🦈"];

        for string in strings.map(String::from) {
            assert!(!string.is_ascii());
            assert!(string.first_char().unwrap_infallible().is_some());
            assert_eq!(
                string.chars().count(),
                string.char_count().unwrap_infallible()
            );
            assert_eq!(
                string.chars().next(),
                string.first_char().unwrap_infallible()
            );
            assert_eq!(
                string.chars().collect_vec(),
                string
                    .iter_chars()
                    .unwrap_infallible()
                    .collect::<Result<Vec<_>, _>>()
                    .unwrap_infallible()
            );
        }

        for string in strings {
            assert!(!string.is_ascii());
            assert!(string.first_char().unwrap_infallible().is_some());
            assert_eq!(
                string.chars().count(),
                string.char_count().unwrap_infallible()
            );
            assert_eq!(
                string.chars().next(),
                string.first_char().unwrap_infallible()
            );
            assert_eq!(
                string.chars().collect_vec(),
                string
                    .iter_chars()
                    .unwrap_infallible()
                    .collect::<Result<Vec<_>, _>>()
                    .unwrap_infallible()
            );
        }
    }
}

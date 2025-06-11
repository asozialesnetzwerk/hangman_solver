// SPDX-License-Identifier: EUPL-1.2
use std::convert::Infallible;
use crate::solver::infallible_char_collection::InfallibleCharCollection;

#[cfg(feature = "pyo3")]
use pyo3::prelude::*;


pub trait CharCollection {
    type Error;

    fn char_count(&self) -> Result<usize, Self::Error>;

    fn first_char(&self) -> Result<Option<char>, Self::Error>;

    fn iter_chars(
        &self,
    ) -> Result<impl Iterator<Item = Result<char, Self::Error>> + '_, Self::Error>;
}

impl<CC: InfallibleCharCollection> CharCollection for CC {
    type Error = Infallible;

    #[inline]
    fn iter_chars(
        &self,
    ) -> Result<impl Iterator<Item = Result<char, Self::Error>> + '_, Self::Error>
    {
        Ok(self.iter_chars().map(Result::Ok))
    }

    #[inline]
    fn char_count(&self) -> Result<usize, Self::Error> {
        Ok(CC::char_count(self))
    }

    #[inline]
    fn first_char(&self) -> Result<Option<char>, Self::Error> {
        Ok(CC::first_char(self))
    }
}

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

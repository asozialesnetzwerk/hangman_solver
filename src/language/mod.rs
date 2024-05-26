// SPDX-License-Identifier: EUPL-1.2
use std::num::NonZeroUsize;

#[cfg(feature = "pyo3")]
use std::ops::Div;

#[cfg(feature = "pyo3")]
use pyo3::create_exception;
#[cfg(feature = "pyo3")]
use pyo3::exceptions::PyValueError;
#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

use crate::solver::char_collection::CharCollection;

#[cfg_attr(feature = "pyo3", pyclass)]
pub struct StringChunkIter {
    pub word_length: usize,
    padded_word_byte_count: NonZeroUsize,
    is_ascii: bool,
    index: usize,
    string: &'static str,
}

impl StringChunkIter {
    #[must_use]
    pub const fn new(
        word_length: usize,
        string: &'static str,
        padded_word_byte_count: usize,
    ) -> Self {
        Self {
            word_length,
            padded_word_byte_count: if let Some(non_zero) =
                NonZeroUsize::new(padded_word_byte_count)
            {
                non_zero
            } else {
                NonZeroUsize::MIN
            },
            index: if string.is_empty() { usize::MAX } else { 0 },
            is_ascii: word_length == padded_word_byte_count,
            string,
        }
    }
}

impl Iterator for StringChunkIter {
    type Item = &'static str;

    #[must_use]
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let end_index =
            self.index.checked_add(self.padded_word_byte_count.get())?;
        debug_assert_ne!(self.index, end_index);

        let result = self.string.get(self.index..end_index)?;

        let result = if self.is_ascii {
            result
        } else {
            result.trim_start_matches('\0')
        };

        debug_assert!(end_index <= self.string.len());
        self.index = end_index;
        debug_assert!(!result.contains('\0'));
        debug_assert_eq!(result.char_count(), self.word_length);
        Some(result)
    }
}

#[cfg(feature = "pyo3")]
#[pymethods]
impl StringChunkIter {
    #[must_use]
    const fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    #[must_use]
    fn __next__(&mut self) -> Option<&'static str> {
        self.next()
    }

    #[must_use]
    pub fn __len__(&self) -> usize {
        self.string.len().div(self.padded_word_byte_count)
    }

    #[must_use]
    pub fn __length_hint__(&self) -> usize {
        self.string
            .len()
            .checked_sub(self.index)
            .map_or(0, |rest| rest.div(self.padded_word_byte_count))
    }
}

include!(concat!(env!("OUT_DIR"), "/language.rs"));

#[cfg(feature = "pyo3")]
create_exception!(hangman_solver, UnknownLanguageError, PyValueError);

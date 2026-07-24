// SPDX-License-Identifier: EUPL-1.2

pub mod word_sequence;

use std::num::NonZeroUsize;

#[cfg(feature = "pyo3")]
use pyo3::create_exception;
#[cfg(feature = "pyo3")]
use pyo3::exceptions::PyValueError;
#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

pub use word_sequence::WordSequence;

#[allow(unsafe_code)]
#[cfg_attr(feature = "pyo3", pyclass)]
pub struct StringChunkIter {
    padded_word_byte_count: NonZeroUsize,
    is_ascii: bool,
    index: usize,
    string: &'static str,
}

impl StringChunkIter {
    #[inline]
    fn remaining_words(&self) -> usize {
        self.string
            .len()
            .checked_sub(self.index)
            .map_or(0, |rest| rest / self.padded_word_byte_count.get())
    }
}

impl ExactSizeIterator for StringChunkIter {}

impl Iterator for StringChunkIter {
    type Item = &'static str;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.nth(0)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let length = self.remaining_words();

        (length, Some(length))
    }

    #[inline]
    fn count(self) -> usize {
        self.remaining_words()
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        if self.index == self.string.len() {
            return None;
        }

        let end_index = self.index.checked_add(
            self.padded_word_byte_count.get().checked_mul(1 + n)?,
        )?;
        if end_index > self.string.len() {
            debug_assert!(n > 0);
            self.index = self.string.len();
            return None;
        }
        debug_assert_ne!(self.index, end_index);

        let result = self
            .string
            .get((end_index - self.padded_word_byte_count.get())..end_index)?;

        let result = if self.is_ascii {
            result
        } else {
            result.trim_start_matches('\0')
        };

        debug_assert!(end_index <= self.string.len());
        self.index = end_index;
        debug_assert!(!result.contains('\0'));
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
        self.remaining_words()
    }
}

include!(concat!(env!("OUT_DIR"), "/language.rs"));

#[cfg(feature = "pyo3")]
create_exception!(hangman_solver, UnknownLanguageError, PyValueError);

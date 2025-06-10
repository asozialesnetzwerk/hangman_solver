// SPDX-License-Identifier: EUPL-1.2

pub mod word_sequence;

use std::num::NonZeroUsize;

#[cfg(feature = "pyo3")]
use std::ops::Div;

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

impl Iterator for StringChunkIter {
    type Item = &'static str;

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
        self.string
            .len()
            .checked_sub(self.index)
            .map_or(0, |rest| rest.div(self.padded_word_byte_count))
    }
}

include!(concat!(env!("OUT_DIR"), "/language.rs"));

#[cfg(feature = "pyo3")]
create_exception!(hangman_solver, UnknownLanguageError, PyValueError);

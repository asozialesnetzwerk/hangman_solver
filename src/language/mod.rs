// SPDX-License-Identifier: EUPL-1.2

#[cfg(feature = "pyo3")]
use pyo3::create_exception;
#[cfg(feature = "pyo3")]
use pyo3::exceptions::PyValueError;
#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

#[cfg_attr(feature = "pyo3", pyclass)]
pub struct StringChunkIter {
    pub word_length: usize,
    padded_word_byte_count: usize,
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
            index: 0,
            string,
            padded_word_byte_count,
        }
    }
}

impl Iterator for StringChunkIter {
    type Item = &'static str;

    #[must_use]
    fn next(&mut self) -> Option<Self::Item> {
        let index = self.index;
        if index >= self.string.len() {
            return None;
        }
        self.index += self.padded_word_byte_count;

        let result = &self.string.get(index..self.index)?;
        if result.len() == self.word_length {
            debug_assert!(!result.starts_with('\0'));
            return Some(result);
        }
        let result = result.trim_start_matches('\0');
        debug_assert!(result.len() >= self.word_length);
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
    const fn __len__(&self) -> usize {
        self.string.len() / self.padded_word_byte_count
    }
}

include!(concat!(env!("OUT_DIR"), "/language.rs"));

#[cfg(feature = "pyo3")]
create_exception!(hangman_solver, UnknownLanguageError, PyValueError);

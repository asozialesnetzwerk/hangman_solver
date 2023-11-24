// SPDX-License-Identifier: EUPL-1.2

use cfg_if::cfg_if;

#[cfg(feature = "pyo3")]
use pyo3::create_exception;
#[cfg(feature = "pyo3")]
use pyo3::exceptions::PyValueError;
#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

cfg_if! {
    if #[cfg(feature = "pyo3")] {
        #[pyclass]
        pub struct StringChunkIter {
            word_length: usize,
            padded_word_byte_count: usize,
            index: usize,
            string: &'static str,
        }
    } else {
        pub struct StringChunkIter {
            word_length: usize,
            padded_word_byte_count: usize,
            index: usize,
            string: &'static str,
        }
    }
}

impl StringChunkIter {
    pub fn new(
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

    fn next(&mut self) -> Option<Self::Item> {
        let index = self.index;
        if index >= self.string.len() {
            return None;
        }
        self.index += self.padded_word_byte_count;

        let result = &self.string[index..self.index];
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
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__(&mut self) -> Option<&'static str> {
        self.next()
    }

    fn __len__(&self) -> usize {
        self.string.len() / self.padded_word_byte_count
    }
}

include!(concat!(env!("OUT_DIR"), "/language.rs"));

#[cfg(feature = "pyo3")]
create_exception!(hangman_solver, UnknownLanguageError, PyValueError);

#[cfg(feature = "pyo3")]
#[pymethods]
impl Language {
    #[getter]
    fn value(&self) -> &'static str {
        self.name()
    }

    #[staticmethod]
    fn values() -> Vec<Language> {
        Language::all()
    }

    #[staticmethod]
    #[pyo3(signature = (name, default = None))]
    pub fn parse_string(
        name: &str,
        default: Option<Language>,
    ) -> PyResult<Language> {
        Language::from_string(name)
            .or(default)
            .ok_or(UnknownLanguageError::new_err(name.to_owned()))
    }
}

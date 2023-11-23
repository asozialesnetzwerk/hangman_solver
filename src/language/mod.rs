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
            index: usize,
            string: &'static str,
        }
    } else {
        pub struct StringChunkIter {
            word_length: usize,
            index: usize,
            string: &'static str,
        }
    }
}

impl StringChunkIter {
    pub fn new(word_length: usize, string: &'static str) -> StringChunkIter {
        StringChunkIter {
            word_length,
            index: 0,
            string,
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
        self.index += self.word_length;

        Some(&self.string[index..self.index])
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
        self.string.len() / self.word_length
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

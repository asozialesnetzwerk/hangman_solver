// SPDX-License-Identifier: EUPL-1.2

#[cfg(feature = "pyo3")]
mod reversed_string_chunk_iter;
mod string_chunk_iter;
mod word_sequence;

pub use string_chunk_iter::StringChunkIter;
pub use word_sequence::WordSequence;

include!(concat!(env!("OUT_DIR"), "/language.rs"));

#[cfg(feature = "pyo3")]
pyo3::create_exception!(
    hangman_solver,
    UnknownLanguageError,
    pyo3::exceptions::PyValueError
);

#[cfg(feature = "pyo3")]
#[pyo3::pymethods]
impl Language {
    /// Return all languages.
    #[staticmethod]
    #[must_use]
    #[allow(clippy::use_self)]
    const fn values() -> [Self; Language::all().len()] {
        Self::all()
    }

    /// The string value of the language.
    #[allow(clippy::trivially_copy_pass_by_ref)]
    #[getter]
    #[must_use]
    const fn value(&self) -> &'static str {
        self.name()
    }

    /// Parse a string into a language.
    #[staticmethod]
    #[pyo3(signature = (name, default = None))]
    pub fn parse_string(
        name: &str,
        default: Option<Self>,
    ) -> pyo3::prelude::PyResult<Self> {
        Self::from_string(name)
            .or(default)
            .ok_or_else(|| UnknownLanguageError::new_err(name.to_owned()))
    }
}

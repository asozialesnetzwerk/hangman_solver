// SPDX-License-Identifier: EUPL-1.2
#![warn(
    clippy::missing_const_for_fn,
    clippy::nursery,
    clippy::pedantic,
    clippy::todo
)]
#![deny(clippy::indexing_slicing, clippy::panic, clippy::unwrap_used)]
#![allow(clippy::missing_errors_doc, clippy::option_if_let_else)]
#![deny(unsafe_code)]
mod language;
mod solver;

pub use crate::solver::{
    CharCollection, HangmanResult, InfallibleCharCollection, Pattern,
};

pub use crate::language::{Language, StringChunkIter, WordSequence};

#[cfg(feature = "wasm-bindgen")]
pub use crate::solver::WasmHangmanResult;
#[cfg(feature = "wasm-bindgen")]
use js_sys::JsString;
#[cfg(feature = "pyo3")]
use pyo3::types::PyString;
#[cfg(feature = "wasm-bindgen")]
use wasm_bindgen::prelude::*;

#[cfg(feature = "pyo3")]
pub use crate::language::UnknownLanguageError;
#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

#[cfg(feature = "pyo3")]
#[derive(FromPyObject)]
pub enum InvalidLetters<'py> {
    String(Bound<'py, PyString>),
    Chars(Vec<char>),
}

#[cfg(feature = "pyo3")]
#[pyfunction]
#[pyo3(signature = (pattern_string, invalid_letters, language, max_words_to_collect))]
#[allow(clippy::needless_pass_by_value)]
pub fn solve(
    pattern_string: Bound<'_, PyString>,
    invalid_letters: InvalidLetters<'_>,
    language: Language,
    max_words_to_collect: usize,
) -> PyResult<HangmanResult> {
    match invalid_letters {
        InvalidLetters::String(invalid_letters) => crate::solver::solve(
            &pattern_string,
            &invalid_letters,
            true,
            language,
            Some(max_words_to_collect),
        ),
        InvalidLetters::Chars(invalid_letters) => crate::solver::solve(
            &pattern_string,
            &invalid_letters,
            true,
            language,
            Some(max_words_to_collect),
        ),
    }
}

#[cfg(feature = "pyo3")]
#[pyfunction]
#[pyo3(signature = (pattern_string, invalid_letters, language, max_words_to_collect))]
#[allow(clippy::needless_pass_by_value)]
pub fn solve_crossword(
    pattern_string: Bound<'_, PyString>,
    invalid_letters: InvalidLetters<'_>,
    language: Language,
    max_words_to_collect: usize,
) -> PyResult<HangmanResult> {
    match invalid_letters {
        InvalidLetters::String(invalid_letters) => crate::solver::solve(
            &pattern_string,
            &invalid_letters,
            false,
            language,
            Some(max_words_to_collect),
        ),
        InvalidLetters::Chars(invalid_letters) => crate::solver::solve(
            &pattern_string,
            &invalid_letters,
            false,
            language,
            Some(max_words_to_collect),
        ),
    }
}

#[must_use]
#[cfg(feature = "pyo3")]
#[pyfunction]
#[pyo3(signature = (language, word_length))]
pub const fn read_words_with_length(
    language: Language,
    word_length: usize,
) -> WordSequence {
    language.read_words(word_length)
}

#[cfg(feature = "pyo3")]
#[pymodule]
#[pyo3(name = "_solver")]
pub fn hangman_solver(py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(solve, m)?)?;
    m.add_function(wrap_pyfunction!(solve_crossword, m)?)?;
    m.add_function(wrap_pyfunction!(read_words_with_length, m)?)?;
    m.add(
        "UnknownLanguageError",
        py.get_type::<UnknownLanguageError>(),
    )?;
    m.add_class::<HangmanResult>()?;
    m.add_class::<Language>()?;
    Ok(())
}

#[cfg(feature = "wasm-bindgen")]
#[wasm_bindgen]
#[allow(clippy::needless_pass_by_value)]
pub fn solve_hangman(
    all_words: Vec<JsString>,
    pattern_string: JsString,
    invalid_letters: JsString,
    max_words_to_collect: usize,
    crossword_mode: bool,
) -> Result<WasmHangmanResult, JsValue> {
    use crate::solver::solve_js;

    Ok(solve_js(
        &mut all_words.iter(),
        &pattern_string,
        &invalid_letters,
        Some(max_words_to_collect),
        crossword_mode,
    ))
}

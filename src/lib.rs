// SPDX-License-Identifier: EUPL-1.2
#![warn(
    clippy::missing_const_for_fn,
    clippy::nursery,
    clippy::option_if_let_else,
    clippy::pedantic,
    clippy::todo
)]
#![deny(clippy::indexing_slicing, clippy::panic, clippy::unwrap_used)]
#![allow(clippy::missing_errors_doc)]
mod language;
mod solver;

pub use crate::language::{Language, StringChunkIter};

pub use crate::solver::{solve_hangman_puzzle, HangmanResult, Pattern};

#[cfg(feature = "pyo3")]
pub use crate::language::UnknownLanguageError;
#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

#[cfg(feature = "pyo3")]
#[pyfunction]
#[pyo3(signature = (pattern_string, invalid_letters, language, max_words_to_collect))]
#[allow(clippy::needless_pass_by_value)]
pub fn solve(
    pattern_string: String,
    invalid_letters: Vec<char>,
    language: Language,
    max_words_to_collect: usize,
) -> PyResult<HangmanResult> {
    let pattern = Pattern::new(&pattern_string, &invalid_letters, true);
    Ok(solve_hangman_puzzle(
        &pattern,
        language,
        Some(max_words_to_collect),
    ))
}

#[cfg(feature = "pyo3")]
#[pyfunction]
#[pyo3(signature = (pattern_string, invalid_letters, language, max_words_to_collect))]
#[allow(clippy::needless_pass_by_value)]
pub fn solve_crossword(
    pattern_string: String,
    invalid_letters: Vec<char>,
    language: Language,
    max_words_to_collect: usize,
) -> PyResult<HangmanResult> {
    let pattern = Pattern::new(&pattern_string, &invalid_letters, false);
    Ok(solve_hangman_puzzle(
        &pattern,
        language,
        Some(max_words_to_collect),
    ))
}

#[cfg(feature = "pyo3")]
#[pyfunction]
#[pyo3(signature = (language, word_length))]
pub const fn read_words_with_length(
    language: Language,
    word_length: usize,
) -> PyResult<StringChunkIter> {
    Ok(language.read_words(word_length))
}

#[cfg(feature = "pyo3")]
#[pymodule]
#[pyo3(name = "_solver")]
pub fn hangman_solver(py: Python<'_>, m: &PyModule) -> PyResult<()> {
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

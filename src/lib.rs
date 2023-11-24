// SPDX-License-Identifier: EUPL-1.2
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
#[pyo3(signature = (pattern_string, invalid_letters, language))]
pub fn solve(
    pattern_string: String,
    invalid_letters: Vec<char>,
    language: Language,
) -> PyResult<HangmanResult> {
    let pattern = Pattern::new(&pattern_string, &invalid_letters, true);
    Ok(solve_hangman_puzzle(&pattern, language))
}

#[cfg(feature = "pyo3")]
#[pyfunction]
#[pyo3(signature = (pattern_string, invalid_letters, language))]
pub fn solve_crossword(
    pattern_string: String,
    invalid_letters: Vec<char>,
    language: Language,
) -> PyResult<HangmanResult> {
    let pattern = Pattern::new(&pattern_string, &invalid_letters, false);
    Ok(solve_hangman_puzzle(&pattern, language))
}

#[cfg(feature = "pyo3")]
#[pyfunction]
#[pyo3(signature = (language, word_length))]
pub fn read_words_with_length(
    language: Language,
    word_length: usize,
) -> PyResult<StringChunkIter> {
    Ok(language.read_words(word_length))
}

#[cfg(feature = "pyo3")]
#[pymodule]
pub(crate) fn hangman_solver(py: Python<'_>, m: &PyModule) -> PyResult<()> {
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

mod language;
mod solver;

use crate::language::Language;

use crate::solver::{solve_hangman_puzzle, HangmanResult};
use pyo3::create_exception;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

#[pyfunction]
#[pyo3(signature = (pattern_string, invalid_letters, language))]
pub fn solve(
    pattern_string: String,
    invalid_letters: Vec<char>,
    language: Language,
) -> PyResult<HangmanResult> {
    Ok(solve_hangman_puzzle(
        pattern_string.as_str(),
        &invalid_letters,
        language,
    ))
}

create_exception!(hangman_solver, UnknownLanguageError, PyValueError);

#[pyfunction]
#[pyo3(signature = (name, default = None))]
pub fn parse_language(
    name: &str,
    default: Option<Language>,
) -> PyResult<Language> {
    Language::from_string(name)
        .or(default)
        .ok_or(UnknownLanguageError::new_err(name.to_owned()))
}

// #[pyfunction]
// #[pyo3(signature = (language, word_length))]
// pub fn read_words_with_length(
//     language: Language,
//     word_length: usize,
// ) -> PyResult<StringChunkIter<'static>> {
//     Ok(language.read_words(word_length))
// }

#[pymodule]
pub(crate) fn hangman_solver(py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(solve, m)?)?;
    m.add_function(wrap_pyfunction!(parse_language, m)?)?;
    m.add(
        "UnknownLanguageError",
        py.get_type::<UnknownLanguageError>(),
    )?;
    m.add_class::<HangmanResult>()?;
    m.add_class::<Language>()?;
    Ok(())
}

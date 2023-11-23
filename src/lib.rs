mod language;
mod solver;

use crate::language::Language;

use crate::solver::{solve_hangman_puzzle, HangmanResult};
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

#[pymodule]
pub(crate) fn hangman_solver(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(solve, m)?)?;
    m.add_class::<HangmanResult>()?;
    m.add_class::<Language>()?;
    Ok(())
}

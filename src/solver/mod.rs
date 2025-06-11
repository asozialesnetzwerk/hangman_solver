// SPDX-License-Identifier: EUPL-1.2

#[cfg(feature = "wasm-bindgen")]
use js_sys::JsString;

use crate::solver::hangman_result::HangmanResult;
#[cfg(feature = "wasm-bindgen")]
use crate::solver::hangman_result::WasmHangmanResult;
use crate::solver::pattern::Pattern;
use crate::{Language, solver::char_collection::CharCollection};

pub mod char_collection;
pub mod char_trait;
pub mod char_utils;
pub mod hangman_result;
pub mod pattern;

#[inline]
#[allow(dead_code)]
pub fn solve<'a, 'b, E1, E2, Err: From<E1> + From<E2>>(
    pattern: impl CharCollection<Error = E1> + 'a,
    invalid_letters: impl CharCollection<Error = E2> + 'b,
    letters_in_pattern_have_no_other_occurrences: bool,
    language: Language,
    max_words_to_collect: Option<usize>,
) -> Result<HangmanResult, Err> {
    let pattern = Pattern::new::<'a, 'b, E1, E2, Err>(
        pattern,
        invalid_letters,
        letters_in_pattern_have_no_other_occurrences,
    )?;

    Ok(pattern.solve(language, max_words_to_collect))
}

#[cfg(feature = "wasm-bindgen")]
#[inline]
#[allow(dead_code)]
pub fn solve_js<'a>(
    all_words: &mut impl Iterator<Item = &'a JsString>,
    pattern_string: &JsString,
    invalid_letters: &JsString,
    max_words_to_collect: Option<usize>,
    crossword_mode: bool,
) -> WasmHangmanResult {
    use unwrap_infallible::UnwrapInfallible as _;

    let pattern =
        Pattern::new(pattern_string, invalid_letters, !crossword_mode)
            .unwrap_infallible();

    pattern.solve_with_words(all_words, max_words_to_collect)
}

#[cfg(test)]
mod test {
    use unwrap_infallible::UnwrapInfallible;

    #[test]
    pub fn test_solve_no_max_words() {
        let hr = super::solve(
            "__r_el_ier",
            ['i', 'r', 'x', 'ä'],
            true,
            crate::Language::DeUmlauts,
            None,
        )
        .unwrap_infallible();

        assert_eq!(hr.input, "__r_el_ier");
        assert_eq!(hr.invalid, vec!['x', 'ä']);
        assert_eq!(
            hr.letter_frequency,
            vec![
                ('t', 2),
                ('u', 2),
                ('b', 1),
                ('g', 1),
                ('m', 1),
                ('w', 1),
                ('z', 1),
                ('ü', 1)
            ]
        );
        assert_eq!(hr.matching_words_count, 3);
        assert_eq!(
            hr.possible_words,
            vec!["gürteltier", "murmeltier", "wurzelbier"]
        );
        assert_eq!(hr.language, crate::Language::DeUmlauts);
    }

    #[test]
    pub fn test_solve_max_1() {
        let hr = super::solve(
            "__r_el_ier",
            ['i', 'r', 'x', 'ä'],
            true,
            crate::Language::DeUmlauts,
            Some(1),
        )
        .unwrap_infallible();

        assert_eq!(hr.input, "__r_el_ier");
        assert_eq!(hr.invalid, vec!['x', 'ä']);
        assert_eq!(
            hr.letter_frequency,
            vec![
                ('t', 2),
                ('u', 2),
                ('b', 1),
                ('g', 1),
                ('m', 1),
                ('w', 1),
                ('z', 1),
                ('ü', 1)
            ]
        );
        assert_eq!(hr.matching_words_count, 3);
        assert_eq!(hr.possible_words, vec!["gürteltier"]);
        assert_eq!(hr.language, crate::Language::DeUmlauts);
    }
}

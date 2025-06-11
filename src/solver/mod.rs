// SPDX-License-Identifier: EUPL-1.2

use std::convert::Infallible;

#[cfg(feature = "wasm-bindgen")]
use js_sys::JsString;
use unwrap_infallible::UnwrapInfallible;

use crate::Language;
#[cfg(feature = "wasm-bindgen")]
pub use crate::solver::hangman_result::WasmHangmanResult;

pub use crate::solver::char_collection::CharCollection;
pub use crate::solver::hangman_result::HangmanResult;
pub use crate::solver::infallible_char_collection::InfallibleCharCollection;
pub use crate::solver::pattern::Pattern;

mod char_collection;
mod char_trait;
mod char_utils;
mod hangman_result;
mod infallible_char_collection;
mod pattern;

#[inline]
#[allow(dead_code)]
pub fn solve<E1, E2, Err: From<E1> + From<E2>>(
    pattern: &(impl CharCollection<Error = E1> + ?Sized),
    invalid_letters: &(impl CharCollection<Error = E2> + ?Sized),
    letters_in_pattern_have_no_other_occurrences: bool,
    language: Language,
    max_words_to_collect: Option<usize>,
) -> Result<HangmanResult, Err> {
    let pattern = Pattern::new::<E1, E2, Err>(
        pattern,
        invalid_letters,
        letters_in_pattern_have_no_other_occurrences,
    )?;

    Ok(pattern.solve(language, max_words_to_collect))
}

#[inline]
#[allow(dead_code)]
pub fn solve_infallible(
    pattern: &(impl CharCollection<Error = Infallible> + ?Sized),
    invalid_letters: &(impl CharCollection<Error = Infallible> + ?Sized),
    letters_in_pattern_have_no_other_occurrences: bool,
    language: Language,
    max_words_to_collect: Option<usize>,
) -> HangmanResult {
    solve(
        pattern,
        invalid_letters,
        letters_in_pattern_have_no_other_occurrences,
        language,
        max_words_to_collect,
    )
    .unwrap_infallible()
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
            &['i', 'r', 'x', 'ä'],
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
            &['i', 'r', 'x', 'ä'],
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

#[cfg(feature = "wasm-bindgen")]
use js_sys::JsString;

use crate::solver::hangman_result::HangmanResult;
#[cfg(feature = "wasm-bindgen")]
use crate::solver::hangman_result::WasmHangmanResult;
use crate::{Language, Pattern, solver::char_collection::CharCollection};

// SPDX-License-Identifier: EUPL-1.2
pub mod ascii_char_iterator;
pub mod char_collection;
pub mod char_trait;
pub mod char_utils;
pub mod generic_char_collection;
pub mod hangman_result;
pub mod pattern;

#[inline]
#[allow(dead_code)]
pub fn solve<T, V>(
    pattern: &T,
    invalid_letters: &V,
    letters_in_pattern_have_no_other_occurrences: bool,
    language: Language,
    max_words_to_collect: Option<usize>,
) -> HangmanResult
where
    T: CharCollection,
    V: CharCollection,
{
    if pattern.all_chars_are_ascii() && invalid_letters.all_chars_are_ascii() {
        let pattern = Pattern::<u8>::new(
            pattern,
            invalid_letters,
            letters_in_pattern_have_no_other_occurrences,
        );

        pattern.solve(language, max_words_to_collect)
    } else {
        let pattern = Pattern::<char>::new(
            pattern,
            invalid_letters,
            letters_in_pattern_have_no_other_occurrences,
        );

        pattern.solve(language, max_words_to_collect)
    }
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
    if pattern_string.all_chars_are_ascii()
        && invalid_letters.all_chars_are_ascii()
    {
        let pattern = Pattern::<u8>::new(
            pattern_string,
            invalid_letters,
            !crossword_mode,
        );

        pattern.solve_with_words(all_words, max_words_to_collect)
    } else {
        let pattern = Pattern::<char>::new(
            pattern_string,
            invalid_letters,
            !crossword_mode,
        );

        pattern.solve_with_words(all_words, max_words_to_collect)
    }
}

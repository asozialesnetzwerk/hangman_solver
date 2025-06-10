#[cfg(feature = "wasm-bindgen")]
use js_sys::JsString;

use crate::solver::hangman_result::HangmanResult;
#[cfg(feature = "wasm-bindgen")]
use crate::solver::hangman_result::WasmHangmanResult;
use crate::solver::pattern::Pattern;
use crate::{Language, solver::char_collection::CharCollection};

// SPDX-License-Identifier: EUPL-1.2
pub mod ascii_char_iterator;
pub mod char_collection;
pub mod char_trait;
pub mod char_utils;
pub mod hangman_result;
pub mod pattern;

#[inline]
#[allow(dead_code)]
pub fn solve<'a, 'b>(
    pattern: impl CharCollection + 'a,
    invalid_letters: impl CharCollection + 'b,
    letters_in_pattern_have_no_other_occurrences: bool,
    language: Language,
    max_words_to_collect: Option<usize>,
) -> HangmanResult
{
    let pattern = Pattern::new(
        pattern,
        invalid_letters,
        letters_in_pattern_have_no_other_occurrences,
    );

    pattern.solve(language, max_words_to_collect)
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
    let pattern =
            Pattern::new(pattern_string, invalid_letters, !crossword_mode);

        pattern.solve_with_words(all_words, max_words_to_collect)
}

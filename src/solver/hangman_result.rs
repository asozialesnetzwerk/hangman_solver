// SPDX-License-Identifier: EUPL-1.2

use cfg_if::cfg_if;
use std::fmt::Display;

use crate::Language;
use crate::solver::infallible_char_collection::InfallibleCharCollection as _;

#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

#[cfg(feature = "wasm-bindgen")]
use js_sys::JsString;
#[cfg(feature = "wasm-bindgen")]
use wasm_bindgen::prelude::*;

#[inline]
fn join_with_max_length<T: ExactSizeIterator<Item = String>>(
    strings: T,
    sep: &str,
    max_len: usize,
) -> impl Display {
    let last_index = strings.len() - 1;
    let mut string = String::with_capacity(max_len);
    for (i, item) in strings.enumerate() {
        let current_sep = if i == 0 { "" } else { sep };
        let min_next_len = if i == last_index { 0 } else { sep.len() + 3 };
        if string.char_count()
            + current_sep.len()
            + item.char_count()
            + min_next_len
            > max_len
        {
            string.extend([current_sep, "..."]);
            break;
        }
        string.extend([current_sep, &item]);
    }
    debug_assert!(string.char_count() <= max_len);
    string
}

cfg_if! {
    if #[cfg(feature = "pyo3")] {
        #[pyclass]
        pub struct HangmanResult {
            #[pyo3(get)]
            pub input: String,
            #[pyo3(get)]
            pub matching_words_count: u32,
            #[pyo3(get)]
            pub invalid: Vec<char>,
            #[pyo3(get, name = "words")]
            pub possible_words: Vec<&'static str>,
            #[pyo3(get)]
            pub language: Language,
            #[pyo3(get)]
            pub letter_frequency: Vec<(char, u32)>,
        }

        #[pymethods]
        impl HangmanResult {
            fn __repr__(&self) -> String {
                let id = self as *const Self;
                let count = self.matching_words_count;
                let lang = self.language.name();
                let pattern = &self.input;
                let invalid = &self.invalid;

                if let Some(word) = (count == 1).then_some(()).and_then(|()| self.possible_words.first()) {
                    format!("<HangmanResult lang={lang} pattern={pattern} invalid={invalid:?} count={count} word={word} at {id:?}>")
                } else if count == 1 {
                    let letters: Box<[char]> = self.letter_frequency.iter().map(|(ch, _)| *ch).collect();
                    format!("<HangmanResult lang={lang} pattern={pattern} invalid={invalid:?} count={count} letters={letters:?} at {id:?}>")
                } else if let Some(mcl) = self.letter_frequency.first().map(|(ch, _)| ch) {
                    format!("<HangmanResult lang={lang} pattern={pattern} invalid={invalid:?} count={count} guess={mcl} at {id:?}>")
                } else {
                    format!("<HangmanResult lang={lang} pattern={pattern} invalid={invalid:?} count={count} at {id:?}>")
                }
            }
        }
    } else {
        pub struct HangmanResult {
            pub input: String,
            pub invalid: Vec<char>,
            pub matching_words_count: u32,
            pub possible_words: Vec<&'static str>,
            pub language: Language,
            pub letter_frequency: Vec<(char, u32)>,
        }
    }
}

impl std::fmt::Display for HangmanResult {
    fn fmt(&self, file: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let max_line_length: usize = file.width().unwrap_or(80);
        let invalid: String = self.invalid.iter().collect();
        write!(
            file,
            "Found {} words (input: {}, invalid: {})",
            self.matching_words_count, self.input, invalid,
        )?;
        if self.possible_words.is_empty() {
            return Ok(());
        }
        writeln!(file)?;
        write!(
            file,
            " words:   {}",
            join_with_max_length(
                self.possible_words.iter().map(|word| String::from(*word)),
                ", ",
                max_line_length - " words:   ".len(),
            )
        )?;

        if !self.letter_frequency.is_empty() {
            writeln!(file)?;
            write!(
                file,
                " letters: {}",
                join_with_max_length(
                    self.letter_frequency
                        .iter()
                        .map(|(ch, f)| format!("{ch}: {f}")),
                    ", ",
                    max_line_length - " letters: ".len(),
                )
            )?;
        }
        Ok(())
    }
}

#[cfg(feature = "wasm-bindgen")]
#[wasm_bindgen(getter_with_clone)]
pub struct WasmHangmanResult {
    #[wasm_bindgen(readonly)]
    pub input: JsString,
    #[wasm_bindgen(readonly)]
    pub invalid: JsString,
    #[wasm_bindgen(readonly)]
    pub matching_words_count: u32,
    #[wasm_bindgen(readonly)]
    pub possible_words: Vec<JsString>,
    #[wasm_bindgen(readonly)]
    pub letter_frequency: JsString,
}

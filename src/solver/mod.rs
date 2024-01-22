// SPDX-License-Identifier: EUPL-1.2
mod char_collection;

use cfg_if::cfg_if;
use std::char;
use std::iter::zip;
use std::rc::Rc;

use crate::language::Language;
use crate::solver::char_collection::CharCollection;

use counter::Counter;
use itertools::Itertools;

#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

const WILDCARD_CHAR: char = '_';
const WILDCARD_ALIASES: [char; 2] = ['#', '?'];
const RESERVED_CHARS: [char; 5] = ['#', '?', '_', '\0', '\n'];

#[inline]
fn join_with_max_length<T: ExactSizeIterator<Item = String>>(
    strings: T,
    sep: &str,
    max_len: usize,
) -> String {
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

type CountedLetters = Box<[(char, u32)]>;

cfg_if! {
    if #[cfg(feature = "pyo3")] {
        #[pyclass]
        pub struct HangmanResult {
            #[pyo3(get)]
            pub input: Rc<[char]>,
            #[pyo3(get)]
            pub matching_words_count: u32,
            #[pyo3(get)]
            pub invalid: Rc<[char]>,
            #[pyo3(get, name = "words")]
            pub possible_words: Box<[&'static str]>,
            #[pyo3(get)]
            pub language: Language,
            #[pyo3(get)]
            pub letter_frequency: CountedLetters,
        }
    } else {
        pub struct HangmanResult {
            pub input: Rc<[char]>,
            pub invalid: Rc<[char]>,
            pub matching_words_count: u32,
            pub possible_words: Box<[&'static str]>,
            pub language: Language,
            pub letter_frequency: Box<[(char, u32)]>,
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
            self.matching_words_count,
            self.input.iter().collect::<String>(),
            invalid,
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
        };
        Ok(())
    }
}

#[cfg(feature = "wasm-bindgen")]
use js_sys::{Array, JsString, Number};
#[cfg(feature = "wasm-bindgen")]
use wasm_bindgen::prelude::*;

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

#[allow(clippy::struct_field_names)]
pub struct Pattern {
    pub invalid_letters: Rc<[char]>,
    pub pattern: Rc<[char]>,
    pub first_letter: char,
    /// true for normal hangman mode
    letters_in_pattern_have_no_other_occurrences: bool,
}

impl Pattern {
    #[must_use]
    pub fn new<T: CharCollection, V: CharCollection>(
        pattern: &T,
        invalid_letters: &V,
        letters_in_pattern_have_no_other_occurrences: bool,
    ) -> Self {
        let pattern: Rc<[char]> = pattern
            .iter_chars()
            .flat_map(char::to_lowercase)
            .filter(|ch| !ch.is_whitespace())
            .map(|ch| {
                if WILDCARD_ALIASES.contains(&ch) {
                    WILDCARD_CHAR
                } else {
                    ch
                }
            })
            .collect();

        let additional_invalid: &[char] =
            if letters_in_pattern_have_no_other_occurrences {
                &pattern
            } else {
                &[]
            };

        let invalid_letters: Rc<[char]> = additional_invalid
            .iter()
            .copied()
            .chain(invalid_letters.iter_chars())
            .filter(|ch| *ch != WILDCARD_CHAR && !ch.is_whitespace())
            .unique()
            .collect();

        let first_letter = *pattern.first().unwrap_or(&WILDCARD_CHAR);

        Self {
            invalid_letters,
            pattern,
            first_letter,
            letters_in_pattern_have_no_other_occurrences,
        }
    }

    #[inline]
    #[must_use]
    const fn first_letter_is_wildcard(&self) -> bool {
        self.first_letter == WILDCARD_CHAR
    }

    #[must_use]
    fn first_letter_matches<CC: CharCollection + ?Sized>(
        &self,
        word: &&CC,
    ) -> bool {
        // This only makes sense if first_letter_is_wildcard is false
        debug_assert!(!self.first_letter_is_wildcard());
        word.first_char()
            .map_or(false, |char| self.first_letter == char)
    }

    #[must_use]
    fn matches<CC: CharCollection + ?Sized>(&self, word: &&CC) -> bool {
        // This only makes sense if word has the same length as the pattern
        debug_assert_eq!(word.char_count(), self.pattern.len());
        // none of the reserved chars shall be in the word
        debug_assert_eq!(
            RESERVED_CHARS
                .iter()
                .filter(|ch| word.iter_chars().contains(ch))
                .count(),
            0
        );
        for (p, w) in zip(self.pattern.iter(), word.iter_chars()) {
            if *p == WILDCARD_CHAR {
                if self.invalid_letters.contains(&w) {
                    return false;
                }
            } else if *p != w {
                return false;
            }
        }
        true
    }

    #[inline]
    #[must_use]
    fn known_letters_count(&self) -> usize {
        self.pattern
            .iter()
            .filter(|ch| **ch != WILDCARD_CHAR)
            .count()
    }

    #[must_use]
    fn _collect_count_and_create_letter_frequency<
        'a,
        'b,
        CC: CharCollection + ?Sized + 'a,
        T: Iterator<Item = &'a CC>,
    >(
        &self,
        words: &'b mut T,
        max_words_to_collect: Option<usize>,
    ) -> (u32, Counter<char, u32>, Box<[&'a CC]>) {
        let mut letter_counter: Counter<char, u32> = Counter::new();

        let update_counter: fn(&mut Counter<char, u32>, &CC) =
            if self.letters_in_pattern_have_no_other_occurrences {
                |counter, word| counter.update(word.iter_chars().unique())
            } else {
                |counter, word| counter.update(word.iter_chars())
            };

        let mut words =
            words.inspect(|word| update_counter(&mut letter_counter, word));

        let (words_vec, additional_count): (Box<[&'a CC]>, usize) =
            if let Some(n) = max_words_to_collect {
                (words.by_ref().take(n).collect(), words.count())
            } else {
                (words.collect(), 0)
            };

        let words_count = u32::try_from(additional_count + words_vec.len())
            .unwrap_or(u32::MAX);

        if self.letters_in_pattern_have_no_other_occurrences {
            for letter in self.pattern.iter() {
                letter_counter.remove(letter);
            }
        } else {
            for letter in
                self.pattern.iter().filter(|char| **char != WILDCARD_CHAR)
            {
                if let Some(new_count) = letter_counter
                    .get(letter)
                    .and_then(|c| c.checked_sub(words_count))
                    .and_then(|c| if c == 0 { None } else { Some(c) })
                {
                    letter_counter.insert(*letter, new_count);
                } else {
                    letter_counter.remove(letter);
                }
            }
        }

        (words_count, letter_counter, words_vec)
    }

    #[must_use]
    pub fn solve(
        &self,
        language: Language,
        max_words_to_collect: Option<usize>,
    ) -> HangmanResult {
        let mut all_words = language.read_words(self.pattern.len());
        let (possible_words, letter_frequency, matching_words_count) =
            self._solve_internal(&mut all_words, max_words_to_collect);

        let mut invalid: Vec<char> = self
            .invalid_letters
            .iter()
            .filter(|ch| !self.pattern.contains(*ch))
            .copied()
            .collect();

        invalid.sort_unstable();
        HangmanResult {
            input: self.pattern.clone(),
            invalid: invalid.into(),
            possible_words,
            language,
            letter_frequency,
            matching_words_count,
        }
    }

    #[cfg(feature = "wasm-bindgen")]
    #[must_use]
    #[allow(dead_code)]
    pub fn solve_with_words<'a, 'b, T: Iterator<Item = &'a JsString>>(
        &self,
        all_words: &'b mut T,
        max_words_to_collect: Option<usize>,
    ) -> WasmHangmanResult {
        let (possible_words, letter_frequency, matching_words_count) =
            self._solve_internal(all_words, max_words_to_collect);

        let mut invalid: Vec<char> = self
            .invalid_letters
            .iter()
            .filter(|ch| !self.pattern.contains(*ch))
            .copied()
            .collect();

        let mut letter_frequency_string: Array = Array::new();

        for (char, count) in letter_frequency {
            letter_frequency_string.push(&JsString::from(char));
            letter_frequency_string.push(&JsString::from(": "));
            letter_frequency_string.push(&Number::from(count));
        }

        invalid.sort_unstable();
        WasmHangmanResult {
            input: JsString::from_chars(self.pattern.iter().copied()),
            invalid: JsString::from_chars(invalid.iter().copied()),
            possible_words: possible_words
                .into_iter()
                .map(JsString::to_string)
                .collect(),
            letter_frequency: letter_frequency_string.join(", "),
            matching_words_count,
        }
    }

    #[must_use]
    fn _solve_internal<
        'a,
        'b,
        CC: CharCollection + ?Sized + 'a,
        T: Iterator<Item = &'a CC>,
    >(
        &self,
        all_words: &'b mut T,
        max_words_to_collect: Option<usize>,
    ) -> (Box<[&'a CC]>, CountedLetters, u32) {
        let (word_count, letter_frequency, words) =
            if self.known_letters_count() == 0
                && self.invalid_letters.is_empty()
            {
                self._collect_count_and_create_letter_frequency(
                    all_words,
                    max_words_to_collect,
                )
            } else if self.first_letter_is_wildcard() {
                let mut filtered_words =
                    all_words.filter(|word| self.matches(word));
                self._collect_count_and_create_letter_frequency(
                    &mut filtered_words,
                    max_words_to_collect,
                )
            } else {
                let mut filtered_words = all_words
                    .skip_while(|word| !self.first_letter_matches(word))
                    .take_while(|word| self.first_letter_matches(word))
                    .filter(|word| self.matches(word));
                self._collect_count_and_create_letter_frequency(
                    &mut filtered_words,
                    max_words_to_collect,
                )
            };

        (
            words,
            letter_frequency.most_common_ordered().into(),
            word_count,
        )
    }
}

// SPDX-License-Identifier: EUPL-1.2
use std::char;
use std::iter::zip;

use crate::language::Language;
use crate::solver::char_collection::CharCollection;
use crate::solver::char_trait::ControlChars;
use crate::solver::char_utils::CharUtils;
use crate::solver::hangman_result::HangmanResult;
#[cfg(feature = "wasm-bindgen")]
use crate::solver::hangman_result::WasmHangmanResult;

use counter::Counter;
use itertools::Itertools;

#[cfg(feature = "wasm-bindgen")]
use js_sys::JsString;

#[allow(clippy::struct_field_names)]
pub struct Pattern {
    invalid_letters: Vec<char>,
    pattern: Vec<char>,
    first_letter: char,
    /// true for normal hangman mode
    letters_in_pattern_have_no_other_occurrences: bool,
    known_letters_count: usize,
    invalid_ascii_letters: [bool; 128],
}

#[allow(dead_code)]
impl Pattern {
    #[must_use]
    pub const fn invalid_letters(&self) -> &[char] {
        self.invalid_letters.as_slice()
    }

    #[must_use]
    pub const fn pattern(&self) -> &[char] {
        self.pattern.as_slice()
    }
}

#[expect(clippy::used_underscore_items)]
impl Pattern {
    #[must_use]
    #[inline]
    pub fn new<'a, 'b>(
        pattern: impl CharCollection + 'a,
        invalid_letters: impl CharCollection + 'b,
        letters_in_pattern_have_no_other_occurrences: bool,
    ) -> Self {
        let mut known_letters_count = 0;
        let pattern_as_chars: Vec<char> = pattern
            .iter_chars()
            .flat_map(char::to_lowercase)
            .filter(|ch| !ch.is_whitespace())
            .map(|ch| {
                if ch.is_wildcard() {
                    char::WILDCARD
                } else {
                    known_letters_count += 1;
                    ch
                }
            })
            .collect();

        let additional_invalid: &[char] =
            if letters_in_pattern_have_no_other_occurrences {
                &pattern_as_chars
            } else {
                &[]
            };

        let invalid_letters_vec: Vec<char> = additional_invalid
            .iter()
            .copied()
            .chain(invalid_letters.iter_chars())
            .filter(|ch| !ch.is_normalised_wildcard() && !ch.is_whitespace())
            .unique()
            .collect();

        let first_letter = *pattern_as_chars.first().unwrap_or(&char::WILDCARD);

        let mut invalid_ascii_letters = [false; 128];

        for ch in &invalid_letters_vec {
            if let Some(b) = ch
                .to_ascii_char()
                .map(usize::from)
                .and_then(|ch| invalid_ascii_letters.get_mut(ch))
            {
                *b = true;
            }
        }

        Self {
            invalid_letters: invalid_letters_vec,
            pattern: pattern_as_chars,
            first_letter,
            letters_in_pattern_have_no_other_occurrences,
            known_letters_count,
            invalid_ascii_letters,
        }
    }

    #[inline]
    #[must_use]
    fn first_letter_is_wildcard(&self) -> bool {
        debug_assert_eq!(
            self.first_letter.is_wildcard(),
            self.first_letter.is_normalised_wildcard()
        );
        self.first_letter.is_normalised_wildcard()
    }

    #[must_use]
    #[inline]
    fn first_letter_matches<CC: CharCollection + ?Sized>(
        &self,
        word: &&CC,
    ) -> bool {
        // This only makes sense if first_letter_is_wildcard is false
        debug_assert!(!self.first_letter_is_wildcard());
        word.first_char() == Some(self.first_letter)
    }

    #[inline]
    pub(super) fn letter_is_valid(&self, letter: char) -> bool {
        self.invalid_letters.is_empty()
            || !letter
                .to_ascii_char()
                .map(usize::from)
                .and_then(|ch| self.invalid_ascii_letters.get(ch))
                .copied()
                .unwrap_or(false)
                && !self.invalid_letters.contains(&letter)
    }

    #[must_use]
    #[inline]
    fn matches<CC: CharCollection + ?Sized>(&self, word: &&CC) -> bool {
        // This only makes sense if word has the same length as the pattern
        debug_assert_eq!(word.char_count(), self.pattern.len());
        // none of the reserved chars shall be in the word
        debug_assert_eq!(
            char::RESERVED
                .iter()
                .filter(|ch| word.iter_chars().contains(ch))
                .count(),
            0
        );
        for (p, w) in zip(self.pattern.iter(), word.iter_chars()) {
            if *p == char::WILDCARD {
                if !self.letter_is_valid(w) {
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
        debug_assert_eq!(
            self.known_letters_count,
            self.pattern
                .iter()
                .filter(|ch| !ch.is_normalised_wildcard())
                .count()
        );

        self.known_letters_count
    }

    #[must_use]
    #[inline]
    fn _collect_count_and_create_letter_frequency<
        'a,
        'b,
        CC: CharCollection + ?Sized + 'a,
        T: Iterator<Item = &'a CC>,
    >(
        &self,
        words: &'b mut T,
        max_words_to_collect: Option<usize>,
    ) -> (u32, Counter<char, u32>, Vec<&'a CC>) {
        let mut letter_counter: Counter<char, u32> = Counter::new();

        let update_counter: fn(&mut Counter<char, u32>, &CC) =
            if self.letters_in_pattern_have_no_other_occurrences {
                |counter, word| counter.update(word.iter_chars().unique())
            } else {
                |counter, word| counter.update(word.iter_chars())
            };

        let mut words =
            words.inspect(|word| update_counter(&mut letter_counter, word));

        let (words_vec, additional_count): (Vec<&'a CC>, usize) =
            if let Some(n) = max_words_to_collect {
                (words.by_ref().take(n).collect(), words.count())
            } else {
                (words.collect(), 0)
            };

        let words_count = u32::try_from(additional_count + words_vec.len())
            .unwrap_or(u32::MAX);

        if self.letters_in_pattern_have_no_other_occurrences {
            for letter in &self.pattern {
                if let Some(count) = letter_counter.remove(letter) {
                    debug_assert_eq!(count, words_count);
                }
            }
        } else {
            for letter in self
                .pattern
                .iter()
                .filter(|char| !char.is_normalised_wildcard())
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
    #[inline]
    pub fn solve(
        &self,
        language: Language,
        max_words_to_collect: Option<usize>,
    ) -> HangmanResult {
        let mut all_words = language.read_words(self.pattern.len()).into_iter();
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
            input: self.pattern.iter().collect(),
            invalid,
            possible_words,
            language,
            letter_frequency,
            matching_words_count,
        }
    }

    #[must_use]
    #[inline]
    fn _solve_internal<
        'a,
        'b,
        CC: CharCollection + ?Sized + 'a,
        T: Iterator<Item = &'a CC>,
    >(
        &self,
        all_words: &'b mut T,
        max_words_to_collect: Option<usize>,
    ) -> (Vec<&'a CC>, Vec<(char, u32)>, u32) {
        let (word_count, letter_frequency, words) =
            if self.invalid_letters.is_empty()
                && self.known_letters_count() == 0
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

        (words, letter_frequency.most_common_ordered(), word_count)
    }
}

#[cfg(feature = "wasm-bindgen")]
#[expect(clippy::used_underscore_items)]
impl Pattern {
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

        let mut letter_frequency_string: String = String::new();

        for (char, count) in letter_frequency {
            if !letter_frequency_string.is_empty() {
                letter_frequency_string.push_str(", ");
            }
            letter_frequency_string.push(char);
            letter_frequency_string.push_str(": ");
            letter_frequency_string.push_str(&count.to_string());
        }

        invalid.sort_unstable();
        WasmHangmanResult {
            input: JsString::from(self.pattern.iter().collect::<String>()),
            invalid: JsString::from(invalid.iter().collect::<String>()),
            possible_words: possible_words
                .into_iter()
                .map(JsString::to_string)
                .collect(),
            letter_frequency: JsString::from(letter_frequency_string),
            matching_words_count,
        }
    }
}

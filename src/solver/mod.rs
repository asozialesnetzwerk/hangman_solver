// SPDX-License-Identifier: EUPL-1.2
use cfg_if::cfg_if;
use std::char;
use std::iter::zip;

use crate::language::{Language, StringChunkIter};

use counter::Counter;
use itertools::Itertools;

#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

const WILDCARD_CHAR: char = '_';
const WILDCARD_CHAR_AS_STR: &str = "_";
const WILDCARD_ALIASES: [char; 2] = ['#', '?'];
const RESERVED_CHARS: [char; 5] = ['#', '?', '_', '\0', '\n'];

trait CharCount {
    fn char_count(&self) -> usize;
}

impl CharCount for String {
    #[inline]
    fn char_count(&self) -> usize {
        self.chars().count()
    }
}

impl CharCount for str {
    #[inline]
    fn char_count(&self) -> usize {
        self.chars().count()
    }
}

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
        };
        Ok(())
    }
}

pub struct Pattern {
    pub invalid_letters: Vec<char>,
    pub pattern: Vec<char>,
    pub first_letter: char,
    /// true for normal hangman mode
    letters_in_pattern_have_no_other_occurrences: bool,
}

impl Pattern {
    pub fn new(
        pattern: &str,
        invalid_letters: &[char],
        letters_in_pattern_have_no_other_occurrences: bool,
    ) -> Self {
        let pattern_as_chars: Vec<char> = pattern
            .to_lowercase()
            .replace(WILDCARD_ALIASES, WILDCARD_CHAR_AS_STR)
            .chars()
            .filter(|ch| !ch.is_whitespace())
            .collect();

        let additional_invalid: Vec<char> =
            if letters_in_pattern_have_no_other_occurrences {
                pattern_as_chars.clone()
            } else {
                vec![]
            };

        let invalid_letters_vec: Vec<char> = additional_invalid
            .iter()
            .chain(invalid_letters)
            .copied()
            .filter(|ch| *ch != WILDCARD_CHAR && !ch.is_whitespace())
            .unique()
            .collect();

        let first_letter = *pattern_as_chars.first().unwrap_or(&WILDCARD_CHAR);

        Self {
            invalid_letters: invalid_letters_vec,
            pattern: pattern_as_chars,
            first_letter,
            letters_in_pattern_have_no_other_occurrences,
        }
    }

    #[inline]
    const fn first_letter_is_wildcard(&self) -> bool {
        self.first_letter == WILDCARD_CHAR
    }

    fn first_letter_matches(&self, word: &str) -> bool {
        // This only makes sense if first_letter_is_wildcard is false
        debug_assert!(!self.first_letter_is_wildcard());
        word.chars()
            .next()
            .map_or(false, |char| self.first_letter == char)
    }

    fn matches(&self, word: &str) -> bool {
        // This only makes sense if word has the same length as the pattern
        debug_assert_eq!(word.char_count(), self.pattern.len());
        // none of the reserved chars shall be in the word
        debug_assert_eq!(
            RESERVED_CHARS
                .iter()
                .filter(|ch| word.chars().contains(ch))
                .count(),
            0
        );
        for (p, w) in zip(self.pattern.iter(), word.chars()) {
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

    fn collect_count_and_create_letter_frequency<
        T: Iterator<Item = &'static str>,
    >(
        &self,
        words: T,
        max_words_to_collect: Option<usize>,
    ) -> (u32, Counter<char, u32>, Vec<&'static str>) {
        let mut letter_counter: Counter<char, u32> = Counter::new();

        let update_counter: fn(&mut Counter<char, u32>, &str) =
            if self.letters_in_pattern_have_no_other_occurrences {
                |counter, word| counter.update(word.chars().unique())
            } else {
                |counter, word| counter.update(word.chars())
            };

        let mut words = words
            .inspect(|word: &&str| update_counter(&mut letter_counter, word));

        let (words_vec, additional_count): (Vec<&'static str>, usize) =
            if let Some(n) = max_words_to_collect {
                (words.by_ref().take(n).collect(), words.count())
            } else {
                (words.collect(), 0)
            };

        let words_count = u32::try_from(additional_count + words_vec.len())
            .unwrap_or(u32::MAX);

        if self.letters_in_pattern_have_no_other_occurrences {
            for letter in &self.pattern {
                letter_counter.remove(letter);
            }
        } else {
            for letter in
                self.pattern.iter().filter(|char| **char != WILDCARD_CHAR)
            {
                letter_counter[letter] -= words_count;
            }
        }

        (words_count, letter_counter, words_vec)
    }
}

#[must_use]
pub fn solve_hangman_puzzle(
    pattern: &Pattern,
    language: Language,
    max_words_to_collect: Option<usize>,
) -> HangmanResult {
    let all_words: StringChunkIter = language.read_words(pattern.pattern.len());

    let (word_count, letter_frequency, words) = if pattern.known_letters_count()
        == 0
        && pattern.invalid_letters.is_empty()
    {
        pattern.collect_count_and_create_letter_frequency(
            all_words,
            max_words_to_collect,
        )
    } else if pattern.first_letter_is_wildcard() {
        pattern.collect_count_and_create_letter_frequency(
            all_words.filter(|word| pattern.matches(word)),
            max_words_to_collect,
        )
    } else {
        pattern.collect_count_and_create_letter_frequency(
            all_words
                .skip_while(|word| !pattern.first_letter_matches(word))
                .take_while(|word| pattern.first_letter_matches(word))
                .filter(|word| pattern.matches(word)),
            max_words_to_collect,
        )
    };

    let input_string: String = pattern.pattern.iter().collect();

    let mut invalid_in_result: Vec<char> = pattern
        .invalid_letters
        .iter()
        .filter(|ch| !pattern.pattern.contains(*ch))
        .copied()
        .collect();

    invalid_in_result.sort_unstable();
    HangmanResult {
        input: input_string,
        invalid: invalid_in_result,
        possible_words: words,
        language,
        letter_frequency: letter_frequency.most_common_ordered(),
        matching_words_count: word_count,
    }
}

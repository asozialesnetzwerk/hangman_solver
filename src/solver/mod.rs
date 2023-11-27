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
            pub invalid: Vec<char>,
            #[pyo3(get, name = "words")]
            pub possible_words: Vec<&'static str>,
            #[pyo3(get)]
            pub language: Language,
            pub letter_frequency: Counter<char, u32>,
        }
    } else {
        pub struct HangmanResult {
            pub input: String,
            pub invalid: Vec<char>,
            pub possible_words: Vec<&'static str>,
            pub language: Language,
            pub letter_frequency: Counter<char, u32>,
        }
    }
}

impl HangmanResult {}

#[cfg(feature = "pyo3")]
#[pymethods]
impl HangmanResult {
    #[pyo3(letter_frequency)]
    pub fn py_get_letter_frequency(
        &self,
    ) -> std::collections::HashMap<char, u32> {
        self.letter_frequency.into_map()
    }
}

impl std::fmt::Display for HangmanResult {
    fn fmt(&self, file: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let max_line_length: usize = file.width().unwrap_or(80);
        let invalid: String = self.invalid.iter().collect();
        write!(
            file,
            "Found {} words (input: {}, invalid: {})",
            self.possible_words.len(),
            self.input,
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

        let letters: Vec<(char, u32)> =
            self.letter_frequency.most_common_ordered();
        if !letters.is_empty() {
            writeln!(file)?;
            write!(
                file,
                " letters: {}",
                join_with_max_length(
                    letters.iter().map(|(ch, f)| format!("{ch}: {f}")),
                    ", ",
                    max_line_length - " letters: ".len(),
                )
            )?;
        };
        Ok(())
    }
}

pub struct Pattern {
    invalid_letters: Vec<char>,
    pattern: Vec<char>,
    first_letter: char,
}

impl Pattern {
    pub fn new(
        pattern: &str,
        invalid_letters: &[char],
        add_pattern_to_invalid: bool,
    ) -> Self {
        let pattern_as_chars: Vec<char> = pattern
            .to_lowercase()
            .replace(WILDCARD_ALIASES, WILDCARD_CHAR_AS_STR)
            .chars()
            .filter(|ch| !ch.is_whitespace())
            .collect();

        let additional_invalid: Vec<char> = if add_pattern_to_invalid {
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
}

fn collect_and_create_letter_frequency<T: Iterator<Item = &'static str>>(
    words: T,
    pattern: &Pattern,
) -> (Vec<&'static str>, Counter<char, u32>) {
    let mut letter_counter: Counter<char, u32> = Counter::new();

    let words: Vec<&'static str> = words
        .inspect(|word| letter_counter.update(word.chars().unique()))
        .collect::<Vec<&'static str>>();

    for letter in &pattern.pattern {
        letter_counter.remove(letter);
    }

    (words, letter_counter)
}

#[must_use]
pub fn solve_hangman_puzzle(
    pattern: &Pattern,
    language: Language,
) -> HangmanResult {
    let all_words: StringChunkIter = language.read_words(pattern.pattern.len());

    let (possible_words, letter_frequency): (
        Vec<&'static str>,
        Counter<char, u32>,
    ) = if pattern.known_letters_count() == 0
        && pattern.invalid_letters.is_empty()
    {
        collect_and_create_letter_frequency(all_words, pattern)
    } else if pattern.first_letter_is_wildcard() {
        collect_and_create_letter_frequency(
            all_words.filter(|word| pattern.matches(word)),
            pattern,
        )
    } else {
        collect_and_create_letter_frequency(
            all_words
                .skip_while(|word| !pattern.first_letter_matches(word))
                .take_while(|word| pattern.first_letter_matches(word))
                .filter(|word| pattern.matches(word)),
            pattern,
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
        possible_words,
        language,
        letter_frequency,
    }
}

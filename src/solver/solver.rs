// SPDX-License-Identifier: EUPL-1.2
pub mod solver;

use std::collections::HashSet;
use std::fmt::Formatter;
use std::hash::Hasher;
use std::io::{self, BufRead, Write as IoWrite};
use std::iter::zip;
use std::path::PathBuf;
use std::{char, fs};

use counter::Counter;
use directories::ProjectDirs;
use memoise::memoise;

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum Language {
    DE,
    EN,
}

impl Language {
    #[must_use]
    pub fn from_string(string: &str) -> Option<Self> {
        match string.to_lowercase().as_str() {
            "de" => Some(Self::DE),
            "en" => Some(Self::EN),
            _ => None,
        }
    }

    #[must_use]
    pub fn as_string(&self) -> &str {
        match self {
            Self::DE => "de",
            Self::EN => "en",
        }
    }

    pub fn read_words(self, length: usize) -> Vec<&'static str> {
        match self {
            Self::DE => include!(concat!(env!("OUT_DIR"), "/de.txt.rs")),
            Self::EN => include!(concat!(env!("OUT_DIR"), "/en.txt.rs")),
        }
    }
}

fn join_with_max_length(
    strings: &[String],
    sep: &str,
    max_len: usize,
) -> String {
    let mut string = String::with_capacity(max_len);
    let last_index = strings.len() - 1;
    for (i, item) in strings.iter().enumerate() {
        let current_sep = if i == 0 { "" } else { sep };
        let min_next_len = if i == last_index { 0 } else { sep.len() + 3 };
        if string.len() + current_sep.len() + item.len() + min_next_len
            > max_len
        {
            string.extend([current_sep, "..."]);
            break;
        }
        string.extend([current_sep, item]);
    }
    debug_assert!(string.len() <= max_len);
    string
}

pub struct HangmanResult {
    input: String,
    invalid: Vec<char>,
    possible_words: Vec<String>,
    language: Language,
}

impl HangmanResult {
    fn get_letter_frequency(&self) -> Counter<char, u32> {
        let input_chars: HashSet<char> = self.input.chars().collect();
        self.possible_words
            .iter()
            .flat_map(|word| word.chars().collect::<HashSet<char>>())
            .filter(|ch| !input_chars.contains(ch))
            .collect::<Counter<char, u32>>()
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
                &self.possible_words,
                ", ",
                max_line_length - " words:   ".len(),
            )
        )?;

        let letters: Vec<(char, u32)> =
            self.get_letter_frequency().most_common_ordered();
        if !letters.is_empty() {
            writeln!(file)?;
            write!(
                file,
                " letters: {}",
                join_with_max_length(
                    &(letters
                        .iter()
                        .map(|(ch, f)| format!("{ch}: {f}"))
                        .collect::<Vec<String>>()),
                    ", ",
                    max_line_length - " letters: ".len(),
                )
            )?;
        };
        Ok(())
    }
}

#[derive(Debug, Copy, Clone)]
enum WordListError {
    NoCacheFolder,
    Io { kind: io::ErrorKind },
}

impl From<io::Error> for WordListError {
    fn from(value: io::Error) -> Self {
        Self::Io { kind: value.kind() }
    }
}

impl std::fmt::Display for WordListError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoCacheFolder => {
                write!(f, "No cache folder.")
            }
            Self::Io { kind } => {
                write!(f, "{kind}")
            }
        }
    }
}

fn read_words(language: Language, length: usize) -> Vec<&'static str> {
    language.read_words(length)
}

pub struct Pattern {
    invalid_letters: HashSet<char>,
    pattern: Vec<char>,
    first_letter: char,
}

impl Pattern {
    fn new(pattern: &str, invalid_letters: &[char]) -> Self {
        let pattern_as_chars: Vec<char> = pattern
            .to_lowercase()
            .replace(['-', '?'], "_")
            .chars()
            .filter(|ch| !ch.is_whitespace())
            .collect();

        let invalid_letters_set: HashSet<char> = pattern_as_chars
            .iter()
            .chain(invalid_letters.iter())
            .copied()
            .filter(|ch| *ch != '_' && !ch.is_whitespace())
            .collect();

        let first_letter = *pattern_as_chars.first().unwrap_or(&'_');

        Self {
            invalid_letters: invalid_letters_set,
            pattern: pattern_as_chars,
            first_letter,
        }
    }

    const fn first_letter_is_wildcard(&self) -> bool {
        self.first_letter == '_'
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
        debug_assert_eq!(word.len(), self.pattern.len());
        for (p, w) in zip(self.pattern.iter(), word.chars()) {
            if *p == '_' {
                if self.invalid_letters.contains(&w) {
                    return false;
                }
            } else if *p != w {
                return false;
            }
        }
        true
    }

    fn known_letters_count(&self) -> usize {
        self.pattern.iter().filter(|ch| **ch != '_').count()
    }
}

#[must_use]
pub fn solve_hangman_puzzle(
    pattern_string: &str,
    invalid_letters: &[char],
    language: Language,
) -> HangmanResult {
    let pattern = Pattern::new(pattern_string, invalid_letters);

    let possible_words: Vec<&'static str> = if pattern.known_letters_count()
        == 0
        && pattern.invalid_letters.is_empty()
    {
        read_words(language, pattern.pattern.len()).collect()
    } else if pattern.first_letter_is_wildcard() {
        read_words(language, pattern.pattern.len())
            .filter(|word| pattern.matches(word))
            .collect()
    } else {
        read_words(language, pattern.pattern.len())
            .skip_while(|word| !pattern.first_letter_matches(word))
            .take_while(|word| pattern.first_letter_matches(word))
            .filter(|word| pattern.matches(word))
            .collect()
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
    }
}
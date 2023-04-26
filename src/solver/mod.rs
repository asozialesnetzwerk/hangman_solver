// SPDX-License-Identifier: EUPL-1.2
use std::collections::hash_map::DefaultHasher;
use std::collections::HashSet;
use std::fmt::Formatter;
use std::fs::File;
use std::hash::Hasher;
use std::io::{self, BufRead, BufReader, Write as IoWrite};
use std::iter::zip;
use std::path::{Path, PathBuf};
use std::{char, fs};

use counter::Counter;
use directories::ProjectDirs;
use itertools::Itertools;
use memoise::memoise;
use pyo3::prelude::*;

#[pyclass]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Language {
    DE,
    DEUmlauts,
    EN,
}

impl Language {
    #[must_use]
    pub fn from_string(string: &str) -> Option<Self> {
        match string.to_lowercase().as_str() {
            "de" => Some(Self::DE),
            "en" => Some(Self::EN),
            "de-umlauts" | "de_umlauts" => Some(Self::DEUmlauts),
            _ => None,
        }
    }

    #[must_use]
    pub fn as_string(&self) -> &str {
        match self {
            Self::DE => "de",
            Self::EN => "en",
            Self::DEUmlauts => "de-umlauts",
        }
    }

    pub fn read_words(self) -> Box<dyn Iterator<Item = String>> {
        Box::new(
            match self {
                Self::DEUmlauts => include_str!(r"../../words/de.txt"),
                Self::EN => include_str!(r"../../words/en.txt"),
                Self::DE => {
                    return Box::new(
                        Self::DEUmlauts
                            .read_words()
                            .map(|word| {
                                word.replace('ß', "ss")
                                    .replace('ä', "ae")
                                    .replace('ö', "oe")
                                    .replace('ü', "ue")
                            })
                            .unique(),
                    );
                }
            }
            .lines()
            .map(std::string::ToString::to_string),
        )
    }

    #[must_use]
    fn get_words_data_hash(self) -> u64 {
        if self == Self::DE {
            return Self::DEUmlauts.get_words_data_hash();
        }
        let mut s = DefaultHasher::new();
        for word in self.read_words() {
            s.write(word.as_bytes());
        }
        s.finish()
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

#[pyclass]
#[derive(FromPyObject)]
pub struct HangmanResult {
    #[pyo3(get)]
    pub input: String,
    #[pyo3(get)]
    pub invalid: Vec<char>,
    #[pyo3(get, name = "words")]
    pub possible_words: Vec<String>,
    #[pyo3(get)]
    pub language: Language,
}

impl HangmanResult {
    pub fn get_letter_frequency(&self) -> Counter<char, u32> {
        let input_chars: HashSet<char> = self.input.chars().collect();
        self.possible_words
            .iter()
            .flat_map(|word| word.chars().collect::<HashSet<char>>())
            .filter(|ch| !input_chars.contains(ch))
            .collect::<Counter<char, u32>>()
    }
}

#[pymethods]
impl HangmanResult {
    pub fn letter_frequency(&self) -> std::collections::HashMap<char, u32> {
        self.get_letter_frequency().into_map()
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

fn get_cache_dir() -> Option<PathBuf> {
    ProjectDirs::from("org", "asozial", "hangman_solver")
        .map(|proj_dirs| proj_dirs.cache_dir().to_path_buf())
}

#[memoise(language)]
fn get_words_cache_folder(language: Language) -> Option<PathBuf> {
    let words_cache_dir: PathBuf = get_cache_dir()?.join("words");
    let hash: String = format!("{:x}", language.get_words_data_hash());

    let lang_words_dir: PathBuf = words_cache_dir.join(language.as_string());
    let words_dir: PathBuf = lang_words_dir.join(&*hash);

    if lang_words_dir.exists() {
        // remove old cache data
        for entry in fs::read_dir(&lang_words_dir)
            .ok()?
            .filter_map(std::result::Result::ok)
        {
            if entry.path() == words_dir && entry.path().is_dir() {
                continue;
            }
            if fs::remove_dir_all(entry.path()).is_err() {
                eprintln!(
                    "Warning: Deleting old data in {} failed.",
                    entry.path().to_str().unwrap_or("")
                );
            }
        }
    }
    if fs::create_dir_all(&words_dir).is_err() {
        eprintln!(
            "Failed to create {}",
            words_dir.to_str().unwrap_or("cache dir")
        );
        return None;
    }

    Some(words_dir)
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

#[memoise(language, length)]
fn get_wordlist_file(
    language: Language,
    length: usize,
) -> Result<PathBuf, WordListError> {
    let words_dir =
        get_words_cache_folder(language).ok_or(WordListError::NoCacheFolder)?;
    let file_name: PathBuf = words_dir.join(format!("{length}.txt"));
    if !file_name.exists() {
        let mut file = File::create(Path::new(&file_name))?;
        for word in language.read_words().filter(|word| word.len() == length) {
            file.write_all(word.as_bytes())?;
            file.write_all(b"\n")?;
        }
    }
    Ok(file_name)
}

fn read_lines_of_file(
    path: &Path,
) -> Result<impl Iterator<Item = String>, io::Error> {
    Ok(BufReader::new(File::open(path)?)
        .lines()
        .filter_map(std::result::Result::ok))
}

fn read_words_without_cache(
    language: Language,
    length: usize,
) -> impl Iterator<Item = String> {
    language
        .read_words()
        .filter(move |word| word.len() == length)
}

fn read_words(
    language: Language,
    length: usize,
) -> Box<dyn Iterator<Item = String>> {
    let it = match get_wordlist_file(language, length) {
        Ok(file_path) => read_lines_of_file(&file_path).map(Box::new).ok(),
        Err(e) => {
            eprintln!("Error: {e}");
            None
        }
    };
    if let Some(x) = it {
        x
    } else {
        Box::new(read_words_without_cache(language, length))
    }
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

    let possible_words = if pattern.known_letters_count() == 0
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

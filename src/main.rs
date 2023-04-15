use std::collections::hash_map::DefaultHasher;
use std::collections::HashSet;
use std::fmt::Formatter;
use std::fs::File;
use std::hash::Hasher;
use std::io::{self, BufRead, BufReader, Write as IoWrite};
use std::iter::zip;
use std::path::{Path, PathBuf};
use std::process::exit;
use std::{char, fs};

use counter::Counter;
use directories::ProjectDirs;
use memoise::memoise;

#[derive(Copy, Clone)]
enum Language {
    DE,
}

impl Language {
    fn as_string(&self) -> &str {
        match self {
            Language::DE => "de",
        }
    }
}

struct HangmanResult {
    input: String,
    invalid: Vec<char>,
    possible_words: Vec<String>,
}

impl HangmanResult {
    fn get_letter_frequency(&self) -> Counter<char, u32> {
        let input_chars: HashSet<char> = self.input.chars().collect();
        self.possible_words
            .iter()
            .flat_map(|word| word.chars().collect::<HashSet<char>>())
            .filter(|ch| !{
                self.invalid.contains(ch) || input_chars.contains(ch)
            })
            .collect::<Counter<char, u32>>()
    }

    fn print(
        &self,
        print_count: usize,
        letters_print_count: usize,
        mut file: impl IoWrite,
    ) -> Result<(), io::Error> {
        let invalid: String = self.invalid.iter().collect();
        writeln!(
            file,
            "Found {} words (input: {}, invalid: {})",
            self.possible_words.len(),
            self.input,
            invalid,
        )?;
        if self.possible_words.is_empty() {
            writeln!(file, "\n")?;
            return Ok(());
        }
        write!(file, " words:   ")?;
        for w in self.possible_words.iter().take(print_count) {
            write!(file, "{}, ", w)?;
        }
        writeln!(
            file,
            "{}",
            if print_count < self.possible_words.len() {
                "..."
            } else {
                ""
            }
        )?;
        let letters: Vec<(char, u32)> =
            self.get_letter_frequency().most_common_ordered();
        if letters.is_empty() {
            writeln!(file)?;
        } else {
            write!(file, " letters: ")?;
            for (ch, freq) in letters.iter().take(letters_print_count) {
                write!(file, "{}: {}, ", ch, freq)?;
            }
            writeln!(
                file,
                "{}\n",
                if letters_print_count < letters.len() {
                    "..."
                } else {
                    ""
                }
            )?;
        };
        Ok(())
    }
}

fn get_full_wordlist_file(language: Language) -> String {
    format!("words/{}.txt", language.as_string())
}

fn get_full_wordlist_file_hash(
    language: Language,
) -> Result<String, io::Error> {
    Ok(format!("{:x}", hash_words(read_all_words(language)?)))
}

fn get_cache_dir() -> Option<PathBuf> {
    ProjectDirs::from("org", "asozial", "hangman_solver")
        .map(|proj_dirs| proj_dirs.cache_dir().to_path_buf())
}

#[memoise(language)]
fn get_words_cache_folder(language: Language) -> Option<PathBuf> {
    let words_cache_dir: PathBuf = get_cache_dir()?.join("words");
    let hash: String = get_full_wordlist_file_hash(language)
        .unwrap_or_else(|_| language.as_string().to_string());

    let lang_words_dir: PathBuf = words_cache_dir.join(language.as_string());
    let words_dir: PathBuf = lang_words_dir.join(&*hash);

    if lang_words_dir.exists() {
        // remove old cache data
        for entry in fs::read_dir(&lang_words_dir).ok()?.filter_map(|e| e.ok())
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
        WordListError::Io { kind: value.kind() }
    }
}

impl std::fmt::Display for WordListError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            WordListError::NoCacheFolder => {
                write!(f, "No cache folder.")
            }
            WordListError::Io { kind } => {
                write!(f, "{}", kind)
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
    let file_name: PathBuf = words_dir.join(format!("{}.txt", length));
    if !file_name.exists() {
        let mut file = File::create(Path::new(&file_name))?;
        for word in
            read_all_words(language)?.filter(|word| word.len() == length)
        {
            file.write_all(word.as_bytes())?;
            file.write_all("\n".as_bytes())?;
        }
    }
    Ok(file_name)
}

fn read_all_words(
    language: Language,
) -> Result<impl Iterator<Item = String>, io::Error> {
    let file = File::open(get_full_wordlist_file(language))?;
    Ok(BufReader::new(file).lines().filter_map(|line| line.ok()))
}

fn hash_words(words: impl Iterator<Item = String>) -> u64 {
    let mut s = DefaultHasher::new();
    for word in words {
        s.write(word.as_bytes());
    }
    s.finish()
}

fn read_words(
    language: Language,
    length: usize,
) -> Result<Box<dyn Iterator<Item = String>>, io::Error> {
    Ok(match get_wordlist_file(language, length) {
        Ok(file_path) => {
            let file = File::open(file_path)?;
            Box::new(BufReader::new(file).lines().filter_map(|line| line.ok()))
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            Box::new(
                read_all_words(language)?
                    .filter(move |word| word.len() == length),
            )
        }
    })
}

struct Pattern {
    invalid_letters: HashSet<char>,
    pattern: Vec<char>,
    first_letter: char,
}

impl Pattern {
    fn create(pattern: String, invalid_letters: Vec<char>) -> Pattern {
        let pattern_as_chars: Vec<char> = pattern
            .to_lowercase()
            .chars()
            .filter(|ch| (*ch != ' '))
            .collect();

        let mut invalid_letters_set: HashSet<char> = HashSet::new();

        for l in pattern_as_chars
            .iter()
            .chain(invalid_letters.iter())
            .filter(|ch| **ch != '_' && !(**ch).is_whitespace())
        {
            invalid_letters_set.insert(*l);
        }

        let first_letter = *pattern_as_chars.first().unwrap_or(&'_');

        Pattern {
            invalid_letters: invalid_letters_set,
            pattern: pattern_as_chars,
            first_letter,
        }
    }

    fn _length_matches(&self, word: &String) -> bool {
        word.len() == self.pattern.len()
    }

    fn first_letter_matches_or_is_wildcard(&self, word: &str) -> bool {
        self.first_letter == '_'
            || self.first_letter == word.chars().next().unwrap_or('_')
    }

    fn matches(&self, word: &str) -> bool {
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

fn solve_hangman_puzzle(
    pattern_string: String,
    invalid_letters: Vec<char>,
    language: Language,
) -> Result<HangmanResult, io::Error> {
    let pattern = Pattern::create(pattern_string, invalid_letters);

    let possible_words = if pattern.known_letters_count() == 0
        && pattern.invalid_letters.is_empty()
    {
        read_words(language, pattern.pattern.len())?.collect()
    } else if pattern.first_letter == '_' {
        read_words(language, pattern.pattern.len())?
            .filter(|word| pattern.matches(word))
            .collect()
    } else {
        read_words(language, pattern.pattern.len())?
            .skip_while(|word| {
                !pattern.first_letter_matches_or_is_wildcard(word)
            })
            .take_while(|word| {
                pattern.first_letter_matches_or_is_wildcard(word)
            })
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

    invalid_in_result.sort();
    Ok(HangmanResult {
        input: input_string,
        invalid: invalid_in_result,
        possible_words,
    })
}

fn main() {
    let mut buffer = String::new();
    let stdin = io::stdin();

    loop {
        let mut handle = stdin.lock();

        match handle.read_line(&mut buffer) {
            Ok(result) => {
                if buffer.is_empty() {
                    exit(result as i32);
                }
                let input: Vec<&str> = buffer.splitn(2, ' ').collect();
                let hr = solve_hangman_puzzle(
                    input[0].to_string(),
                    input.get(1).unwrap_or(&"").chars().collect(),
                    Language::DE,
                )
                .expect("Solving should be possible.");
                hr.print(10, 16, io::stdout()).expect("Printing to stdout");
            }
            Err(error) => {
                eprintln!("{}", error);
                exit(1)
            }
        }

        buffer.clear();
    }
}

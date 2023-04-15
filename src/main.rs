use std::collections::hash_map::DefaultHasher;
use std::collections::{HashMap, HashSet};
use std::fmt::Write;
use std::fs::File;
use std::hash::Hasher;
use std::io::{self, BufRead, BufReader, Write as IoWrite};
use std::iter::zip;
use std::path::{Path, PathBuf};
use std::process::exit;
use std::{char, fs};

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

fn get_unique_chars_in_word(word: &str) -> HashSet<char> {
    let mut chars = HashSet::new();
    for ch in word.chars() {
        chars.insert(ch);
    }
    chars
}

impl HangmanResult {
    fn get_letter_frequency(&self) -> HashMap<char, u32> {
        let mut map = HashMap::new();
        for x in self
            .possible_words
            .iter()
            .flat_map(|word| get_unique_chars_in_word(word))
        {
            if !self.invalid.contains(&x) && !self.input.contains(x) {
                map.insert(x, map.get(&x).unwrap_or(&0) + 1);
            }
        }
        map
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
        let mut letters: Vec<(char, u32)> = self.get_letter_frequency().into_iter().collect();
        if letters.is_empty() {
            writeln!(file)?;
        } else {
            write!(file, " letters: ")?;
            letters.sort_by_key(|tuple| (-(tuple.1 as i64), tuple.0));
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

fn get_full_wordlist_file_hash(language: Language) -> String {
    format!("{:x}", hash_words(read_all_words(language)))
}

fn get_cache_dir() -> Option<PathBuf> {
    ProjectDirs::from("org", "asozial", "hangman_solver")
        .map(|proj_dirs| proj_dirs.cache_dir().to_path_buf())
}

#[memoise(language)]
fn get_words_cache_folder(language: Language) -> Option<PathBuf> {
    let words_cache_dir: PathBuf;
    if let Some(cache_dir) = get_cache_dir() {
        words_cache_dir = cache_dir.join("words");
    } else {
        return None;
    }
    let hash: String = get_full_wordlist_file_hash(language);

    let lang_words_dir: PathBuf = words_cache_dir.join(language.as_string());
    let words_dir: PathBuf = lang_words_dir.join(&*hash);

    if lang_words_dir.exists() && !words_dir.exists() {
        // remove old cache data
        fs::remove_dir_all(lang_words_dir).expect("Deleting cache dir");
    }
    fs::create_dir_all(Path::new(&words_dir)).expect("Create cache dir");

    Some(words_dir)
}

#[memoise(language, length)]
fn get_wordlist_file(language: Language, length: usize) -> Result<PathBuf, String> {
    if let Some(words_dir) = get_words_cache_folder(language) {
        let file_name: PathBuf = words_dir.join(format!("{}.txt", length));
        if !file_name.exists() {
            match File::create(Path::new(&file_name)) {
                Ok(mut file) => {
                    for word in read_all_words(language).filter(|word| word.len() == length) {
                        file.write_all(word.as_bytes()).expect("writing cache");
                        file.write_all("\n".as_bytes()).expect("writing cache");
                    }
                }
                Err(e) => return Err(e.to_string()),
            }
        }
        Ok(file_name)
    } else {
        Err("No cache dir".to_string())
    }
}

fn read_all_words(language: Language) -> impl Iterator<Item = String> {
    let file = File::open(get_full_wordlist_file(language)).unwrap();
    BufReader::new(file).lines().filter_map(|line| line.ok())
}

fn hash_words(words: impl Iterator<Item = String>) -> u64 {
    let mut s = DefaultHasher::new();
    for word in words {
        s.write(word.as_bytes());
    }
    s.finish()
}

fn read_words(language: Language, length: usize) -> Box<dyn Iterator<Item = String>> {
    match get_wordlist_file(language, length) {
        Ok(file_path) => {
            let file = File::open(file_path).unwrap();
            Box::new(BufReader::new(file).lines().filter_map(|line| line.ok()))
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            Box::new(read_all_words(language).filter(move |word| word.len() == length))
        }
    }
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
        self.first_letter == '_' || self.first_letter == word.chars().next().unwrap_or('_')
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
) -> HangmanResult {
    let pattern = Pattern::create(pattern_string, invalid_letters);

    let possible_words = if pattern.known_letters_count() == 0 && pattern.invalid_letters.is_empty()
    {
        read_words(language, pattern.pattern.len()).collect()
    } else if pattern.first_letter == '_' {
        read_words(language, pattern.pattern.len())
            .filter(|word| pattern.matches(word))
            .collect()
    } else {
        read_words(language, pattern.pattern.len())
            .skip_while(|word| !pattern.first_letter_matches_or_is_wildcard(word))
            .take_while(|word| pattern.first_letter_matches_or_is_wildcard(word))
            .filter(|word| pattern.matches(word))
            .collect()
    };

    let mut input_as_string = String::new();
    for ch in pattern.pattern {
        input_as_string.write_char(ch).unwrap();
    }
    let mut invalid_in_result: Vec<char> = pattern
        .invalid_letters
        .iter()
        .filter(|ch| !input_as_string.contains(**ch))
        .copied()
        .collect();
    invalid_in_result.sort();
    HangmanResult {
        input: input_as_string,
        invalid: invalid_in_result,
        possible_words,
    }
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
                    input[1].chars().collect(),
                    Language::DE,
                );
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

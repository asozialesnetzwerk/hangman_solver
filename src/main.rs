use std::char;
use std::collections::{HashMap, HashSet};
use std::fmt::Write;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write as IoWrite};
use std::iter::zip;
use std::process::exit;

struct HangmanResult {
    input: String,
    invalid: Vec<char>,
    possible_words: Vec<String>,
}

fn get_unique_chars_in_word(word: String) -> HashSet<char> {
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
            .flat_map(|word| get_unique_chars_in_word(word.clone()))
        {
            if !self.invalid.contains(&x) && !self.input.contains(x) {
                map.insert(x, map.get(&x).unwrap_or(&0) + 1);
            }
        }
        map
    }

    fn print(&self, print_count: usize, mut file: impl IoWrite) {
        write!(
            file,
            "Found {} words (input: {}, invalid: ",
            self.possible_words.len(),
            self.input
        )
        .unwrap();
        self.invalid
            .iter()
            .for_each(|ch| write!(file, "{}", ch.escape_debug()).unwrap());
        writeln!(file, ")\n").unwrap();
        for w in self.possible_words.iter().take(print_count) {
            write!(file, "{}, ", w).unwrap();
        }
        if print_count < self.possible_words.len() {
            writeln!(file, "...").unwrap();
        } else {
            writeln!(file).unwrap();
        }
        let mut letters: Vec<(char, u32)> = self.get_letter_frequency().into_iter().collect();
        letters.sort_by_key(|tuple| tuple.1);
        letters.reverse();
        for (ch, freq) in letters {
            write!(file, "{}: {}, ", ch, freq).unwrap();
        }
        writeln!(file).unwrap();
    }
}

fn read_words(language: String) -> impl Iterator<Item = String> {
    let file = File::open("words/".to_owned() + &language + ".txt").unwrap();

    BufReader::new(file).lines().filter_map(|line| line.ok())
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

        Pattern {
            invalid_letters: invalid_letters_set,
            pattern: pattern_as_chars.clone(),
            first_letter: *pattern_as_chars.first().unwrap_or(&'_'),
        }
    }

    fn length_matches(&self, word: &String) -> bool {
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
}

fn solve_hangman_puzzle(
    pattern_string: String,
    invalid_letters: Vec<char>,
    language: String,
) -> HangmanResult {
    let pattern = Pattern::create(pattern_string, invalid_letters);

    let possible_words = if pattern.first_letter == '_' {
        read_words(language)
            .filter(|word| pattern.length_matches(word))
            .filter(|word| pattern.matches(word))
            .collect()
    } else {
        read_words(language)
            .skip_while(|word| !pattern.first_letter_matches_or_is_wildcard(word))
            .take_while(|word| pattern.first_letter_matches_or_is_wildcard(word))
            .filter(|word| pattern.length_matches(word))
            .filter(|word| pattern.matches(word))
            .collect()
    };

    let mut input_as_string = String::new();
    for ch in pattern.pattern {
        input_as_string.write_char(ch).unwrap();
    }
    let invalid_in_result = pattern
        .invalid_letters
        .iter()
        .filter(|ch| !input_as_string.contains(**ch))
        .copied()
        .collect();
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
        let result = handle.read_line(&mut buffer);

        if result.is_ok() {
            if buffer.is_empty() {
                exit(result.unwrap() as i32);
            }
            let input: Vec<&str> = buffer.splitn(2, ' ').collect();
            let hr = solve_hangman_puzzle(
                input[0].to_string(),
                input[1].chars().collect(),
                "de".to_string(),
            );
            if hr.possible_words.is_empty() {
                println!("Nothing found");
            } else {
                hr.print(10, io::stdout());
            }
        } else {
            eprintln!("{}", result.unwrap_err());
            exit(1)
        }

        buffer.clear();
    }
}

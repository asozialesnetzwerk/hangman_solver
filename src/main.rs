use std::char;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::iter::zip;
use std::process::exit;

struct HangmanResult {
    input: String,
    invalid: Vec<char>,
    possible_words: Vec<String>,
}

fn read_words(language: String, word_length: usize) -> impl Iterator<Item = String> {
    let file = File::open("words/".to_owned() + &language + ".txt").unwrap();

    BufReader::new(file)
        .lines()
        .filter_map(|line| line.ok())
        .filter(move |line| line.len() == word_length)
}

struct Pattern {
    invalid_letters: Vec<char>,
    pattern: String,
}

impl Pattern {
    fn first_letter_matches(&self, word: &String) -> bool {
        let first_pattern_char = self.pattern.chars().next().unwrap();
        if first_pattern_char == '_' {
            true
        } else {
            first_pattern_char == word.chars().next().unwrap()
        }
    }

    fn matches(&self, word: &String) -> bool {
        if word.len() != self.pattern.len() {
            return false;
        }
        for (p, w) in zip(self.pattern.chars(), word.chars()) {
            if p == '_' {
                if self.invalid_letters.contains(&w) {
                    return false;
                }
            } else if p != w {
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
    let invalid: Vec<char> = pattern_string
        .chars()
        .chain(invalid_letters.clone())
        .collect();
    let pattern = Pattern {
        invalid_letters: invalid,
        pattern: pattern_string.to_string(),
    };

    HangmanResult {
        input: pattern_string.clone(),
        invalid: invalid_letters,
        possible_words: read_words(language, pattern_string.len())
            .take_while(|word| pattern.first_letter_matches(word))
            .filter(|word| pattern.matches(word))
            .collect(),
    }
}

fn main() {
    let mut buffer = String::new();
    let stdin = io::stdin();

    while 1 == 1 {
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
            for w in hr.possible_words {
                print!("{}, ", w);
            }
            println!();
        } else {
            eprintln!("{}", result.unwrap_err());
            exit(1)
        }

        buffer.clear();
    }
}

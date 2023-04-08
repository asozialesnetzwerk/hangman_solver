use std::char;
use std::collections::HashSet;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::iter::zip;
use std::process::exit;

struct HangmanResult {
    input: String,
    invalid: Vec<char>,
    possible_words: Vec<String>,
}

fn read_words(language: String) -> impl Iterator<Item = String> {
    let file = File::open("words/".to_owned() + &language + ".txt").unwrap();

    BufReader::new(file).lines().filter_map(|line| line.ok())
}

struct Pattern {
    invalid_letters: HashSet<char>,
    pattern: String,
}

impl Pattern {
    fn create(pattern: String, invalid_letters: Vec<char>) -> Pattern {
        let mut invalid_letters_set: HashSet<char> = HashSet::new();
        for l in pattern
            .chars()
            .filter(|ch| *ch != '_')
            .chain(invalid_letters)
        {
            invalid_letters_set.insert(l);
        }

        Pattern {
            invalid_letters: invalid_letters_set,
            pattern,
        }
    }

    fn first_letter_matches_and_is_no_wildcard(&self, word: &String) -> bool {
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
    let pattern = Pattern::create(pattern_string.clone(), invalid_letters.clone());

    HangmanResult {
        input: pattern_string,
        invalid: invalid_letters,
        possible_words: read_words(language)
            .take_while(|word| pattern.first_letter_matches_and_is_no_wildcard(word))
            .filter(|word| pattern.matches(word))
            .collect(),
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

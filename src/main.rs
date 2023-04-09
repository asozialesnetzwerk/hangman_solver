use std::char;
use std::collections::{HashMap, HashSet};
use std::fmt::Write;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
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

    fn first_letter_matches_or_is_wildcard(&self, word: &String) -> bool {
        self.first_letter == '_' || self.first_letter == word.chars().next().unwrap_or('_')
    }

    fn matches(&self, word: &String) -> bool {
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

    fn contains_wildcard(&self) -> bool {
        self.pattern.contains(&'_')
    }
}

fn solve_hangman_puzzle(
    pattern_string: String,
    invalid_letters: Vec<char>,
    language: String,
) -> HangmanResult {
    let pattern = Pattern::create(pattern_string.clone(), invalid_letters.clone());

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
        input_as_string.write_char(ch);
    }
    let invalid_in_result = pattern
        .invalid_letters
        .iter()
        .filter(|ch| !input_as_string.contains(**ch))
        .map(|ch| *ch)
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
            if hr.possible_words.len() == 0 {
                println!("Nothing found")
            } else {
                print!(
                    "Found {} words (input: {}, invalid: ",
                    hr.possible_words.len(),
                    hr.input
                );
                hr.invalid
                    .iter()
                    .for_each(|ch| print!("{}", ch.escape_debug()));
                println!(")\n");
                let print_count = 10;
                for w in hr.possible_words.iter().take(print_count) {
                    print!("{}, ", w);
                }
                if print_count < hr.possible_words.len() {
                    println!("...")
                } else {
                    println!();
                }
                let mut letters: Vec<(char, u32)> = hr.get_letter_frequency().into_iter().collect();
                letters.sort_by_key(|tuple| (*tuple).1);
                letters.reverse();
                for (ch, freq) in letters {
                    print!("{}: {}, ", ch, freq);
                }
                println!()
            }
        } else {
            eprintln!("{}", result.unwrap_err());
            exit(1)
        }

        buffer.clear();
    }
}

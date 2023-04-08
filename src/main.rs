use std::char;
use std::io::{self, BufRead};
use std::process::exit;

struct HangmanResult {
    input: String,
    invalid: String,
    possible_words: Vec<String>,
}


fn solve_hangman_puzzle(
    puzzle: String, invalid_letters: Vec<char>, language: String
) -> HangmanResult {
    HangmanResult {
        input: "".to_string(),
        invalid: "".to_string(),
        possible_words: vec![],
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
            let input: Vec<&str> = buffer.splitn(2, " ").collect();
            solve_hangman_puzzle(
                input[0].to_string(),
                input[1].chars().collect(),
                "de".to_string()
            );
            println!("{}", buffer);
        } else {
            eprintln!("{}", result.unwrap_err());
            exit(1)
        }

        buffer.clear();
    }
}

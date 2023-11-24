// SPDX-License-Identifier: EUPL-1.2
mod language;
mod solver;

use std::char;
use std::env;
use std::io::{self, BufRead};
use std::process::exit;

use itertools::Itertools;
use terminal_size::{terminal_size, Width};

use crate::language::Language;
use crate::solver::{solve_hangman_puzzle, Pattern};

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.is_empty() {
        eprintln!("Please set a language as argument.");
        eprintln!(
            "Valid languages: {}",
            Language::all()
                .iter()
                .map(language::Language::name)
                .join(", ")
        );
        exit(2);
    } else if args.len() != 1 {
        eprintln!("Too many arguments");
        exit(1);
    };

    let lang = args
        .get(0)
        .and_then(|lang| Language::from_string(lang))
        .map_or_else(
            || {
                eprintln!("Invalid language");
                exit(1);
            },
            |x| x,
        );

    let mut buffer = String::new();
    let stdin = io::stdin();

    loop {
        let r = stdin.lock().read_line(&mut buffer);
        match r {
            Ok(result) => {
                if buffer.is_empty() {
                    exit(i32::from(result != 0));
                }
                let input: Vec<&str> = buffer.splitn(2, ' ').collect();
                let pattern = Pattern::new(
                    input[0],
                    &(input
                        .get(1)
                        .unwrap_or(&"")
                        .chars()
                        .collect::<Vec<char>>()),
                    true,
                );
                let hr = solve_hangman_puzzle(&pattern, lang);
                assert!(hr.language == lang);
                let width: usize = if let Some((Width(w), _)) = terminal_size()
                {
                    w.into()
                } else {
                    80
                };
                println!("{hr:─^width$}");
            }
            Err(error) => {
                eprintln!("{error}");
            }
        }

        buffer.clear();
    }
}

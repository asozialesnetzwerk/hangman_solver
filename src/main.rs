// SPDX-License-Identifier: EUPL-1.2
#![warn(
    clippy::missing_const_for_fn,
    clippy::nursery,
    clippy::option_if_let_else,
    clippy::pedantic,
    clippy::todo
)]
#![deny(clippy::indexing_slicing, clippy::panic, clippy::unwrap_used)]
#![allow(clippy::missing_errors_doc)]
mod language;
mod solver;

use std::char;
use std::env;
use std::io::{self, BufRead};
use std::process::exit;

use itertools::Itertools;
#[cfg(feature = "terminal_size")]
use terminal_size::{terminal_size, Width};

use crate::language::Language;
use crate::solver::{solve_hangman_puzzle, Pattern};

fn get_terminal_width() -> usize {
    #[cfg(feature = "terminal_size")]
    if let Some((Width(w), _)) = terminal_size() {
        return w.into();
    }

    80
}

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
        .first()
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

                let width = get_terminal_width();

                let input: Vec<&str> = buffer.splitn(2, ' ').collect();
                let pattern = Pattern::new(
                    input.first().unwrap_or(&""),
                    &(input
                        .get(1)
                        .unwrap_or(&"")
                        .chars()
                        .collect::<Vec<char>>()),
                    true,
                );
                let hr = solve_hangman_puzzle(
                    &pattern,
                    lang,
                    Some(width / pattern.pattern.len() + 1),
                );
                assert!(hr.language == lang);

                println!("{hr:─^width$}");
            }
            Err(error) => {
                eprintln!("{error}");
            }
        }

        buffer.clear();
    }
}

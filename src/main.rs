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
use crate::solver::Pattern;

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
                let hr = pattern
                    .solve(lang, Some(width / pattern.pattern.len() + 1));
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

#[test]
fn test_itering_words() -> Result<(), String> {
    use crate::solver::char_collection::CharCollection;
    for lang in Language::all() {
        let mut total_words = 0usize;
        for i in 0..100usize {
            if i != lang.read_words(i).word_length {
                return Err(format!(
                    "{} != {} (lang={lang:?})",
                    i,
                    lang.read_words(i).word_length
                ));
            }
            let mut word_count = 0;
            for word in lang.read_words(i) {
                if word.len() < i {
                    return Err(format!(
                        "{} < {} (word={word}, lang={lang:?})",
                        word.len(),
                        i
                    ));
                }
                if word.char_count() != i {
                    return Err(format!(
                        "{} != {} (word={word}, lang={lang:?})",
                        word.char_count(),
                        i
                    ));
                }
                word_count += 1;
            }
            #[cfg(feature = "pyo3")]
            {
                if word_count != lang.read_words(i).__len__() {
                    return Err(format!("__len__: {word_count} != {} (lang={lang:?})",lang.read_words(i).__len__()));
                }
                if word_count != lang.read_words(i).__length_hint__() {
                    return Err(format!("__length_hint__: {word_count} != {} (lang={lang:?})",lang.read_words(i).__length_hint__()));
                }
                let mut word_iterator = lang.read_words(i);
                while word_iterator.next().is_some() {}
                let len_hint = word_iterator.__length_hint__();
                if len_hint != 0  {
                    return Err(format!(
                        "__length_hint__: {len_hint} != 0 (lang={lang:?})"
                    ));
                }
            }
            total_words += word_count;
        }
        if total_words < 50_000 {
            return Err(format!("only {total_words} for {lang:?}"));
        }
    }
    Ok(())
}

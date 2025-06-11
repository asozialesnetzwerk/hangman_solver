// SPDX-License-Identifier: EUPL-1.2
#![warn(
    clippy::missing_const_for_fn,
    clippy::nursery,
    clippy::pedantic,
    clippy::todo
)]
#![deny(clippy::indexing_slicing, clippy::panic, clippy::unwrap_used)]
#![allow(clippy::missing_errors_doc, clippy::option_if_let_else)]
#![deny(unsafe_code)]
mod language;
mod solver;

use std::env;
use std::io::{self, BufRead};
use std::process::exit;

use itertools::Itertools;
#[cfg(feature = "terminal_size")]
use terminal_size::{Width, terminal_size};
use unwrap_infallible::UnwrapInfallible;

use crate::language::Language;
use crate::solver::InfallibleCharCollection as _;
use crate::solver::solve;

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
    }

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

                let mut input =
                    buffer.splitn(2, ' ').collect::<Box<[&str]>>().into_iter();
                let pattern: &str = input.next().unwrap_or("");
                let invalid: &str = input.next().unwrap_or("");
                let hr = solve(
                    pattern,
                    invalid,
                    true,
                    lang,
                    Some(width / pattern.char_count() + 1),
                )
                .unwrap_infallible();
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
    for lang in Language::all() {
        let mut total_words = 0usize;
        for i in 0..100usize {
            if i != lang.read_words(i).word_char_count() {
                return Err(format!(
                    "{} != {} (lang={lang:?})",
                    i,
                    lang.read_words(i).word_char_count()
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
                    return Err(format!(
                        "__len__: {word_count} != {} (lang={lang:?})",
                        lang.read_words(i).__len__()
                    ));
                }
                let mut word_iterator = lang.read_words(i).into_iter();
                while word_iterator.next().is_some() {}
                let len = word_iterator.__len__();
                if len != 0 {
                    return Err(format!("__len__: {len} != 0 (lang={lang:?})"));
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

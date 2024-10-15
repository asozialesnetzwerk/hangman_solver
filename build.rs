#![allow(clippy::unwrap_used)]
#![forbid(unsafe_code)]

use easy_parallel::Parallel;
use inflector::Inflector;
use itertools::Itertools;
use std::env;
use std::fs;
use std::fs::File;
use std::io;
use std::io::BufRead;
use std::io::BufReader;
use std::iter::Iterator;
use std::path::{Path, PathBuf};
use std::time::Instant;
use unicode_segmentation::UnicodeSegmentation;

type StrConv = fn(String) -> String;

fn read_lines_of_file(
    path: &Path,
) -> Result<impl Iterator<Item = String>, io::Error> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    Ok(reader.lines().map_while(Result::ok))
}

#[derive(Clone, Eq, PartialEq, Debug)]
struct WordsData {
    pub path: String,
    pub lang: String,
    pub conv: StrConv,
}

impl WordsData {
    fn clone_with_lang(&self, lang: String) -> Self {
        Self::new(self.path.clone(), lang, self.conv)
    }

    fn new(path: String, lang: String, conv: StrConv) -> Self {
        Self { path, lang, conv }
    }

    fn from_path(path: &Path) -> Self {
        let lang: &str = path
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .trim_end_matches(".txt");

        let path_str: &str = path.to_str().unwrap();

        Self::new(String::from(path_str), String::from(lang), |string| string)
    }

    fn read_lines(&self) -> impl Iterator<Item = String> {
        read_lines_of_file(Path::new(self.path.as_str()))
            .expect("Reading file should not fail.")
            .filter(|word| !word.is_empty())
            .map(|word| word.to_lowercase())
            .map(self.conv)
    }

    fn enum_name(&self) -> String {
        self.lang.replace('-', "_").to_pascal_case()
    }

    fn out_file_name(&self) -> String {
        format!("{}.txt.rs", self.lang)
    }

    fn dest_path(&self) -> PathBuf {
        get_out_dir_joined(self.out_file_name())
    }
}
fn get_out_dir_joined(path: String) -> PathBuf {
    let out_dir: &String = &env::var("OUT_DIR").unwrap();
    Path::new(out_dir).join(path)
}

fn write_words_data(words_data: &WordsData) {
    let start = Instant::now();

    let lang = words_data.lang.as_str();
    let mut words: Vec<(usize, String)> = words_data
        .read_lines()
        .map(|word| (word.chars().count(), word))
        .collect();

    words.sort_unstable();
    words.dedup();

    let mut max_string_lit_len: usize = 0;

    let mut output = String::from("match length {");
    for chunk in
        words.chunk_by(|(length_a, _), (length_b, _)| *length_a == *length_b)
    {
        let char_count = chunk.first().expect("needs to have first").0;
        let max_word_byte_count: usize = chunk
            .iter()
            .map(|(_, word)| word.as_str().len())
            .max()
            .expect("word group needs to have max length");

        max_string_lit_len =
            max_string_lit_len.max(max_word_byte_count * chunk.len());

        let start_of_case = format!(
            "{char_count} => (NonZeroUsize::MIN.saturating_add({max_word_byte_count} - 1), \""
        );
        const END_OF_CASE: &str = "\"),\n";
        output.reserve(
            max_word_byte_count * chunk.len()
                + start_of_case.len()
                + END_OF_CASE.len(),
        );
        output.push_str(&start_of_case);

        for (_, word) in chunk {
            assert_eq!(
                word.graphemes(true).count(),
                char_count,
                "{lang}: {word} has graphemes",
            );
            assert_eq!(
                word.unicode_words().count(),
                1,
                "{lang}: {word} is multiple words",
            );

            for _ in 0..(max_word_byte_count - word.len()) {
                output.push('\0');
            }
            for ch in word.chars() {
                if ch == '"' {
                    output.push('\\');
                }
                output.push(ch);
            }
        }
        output += END_OF_CASE;
    }
    output.push_str("_ => (NonZeroUsize::MIN, \"\")}");
    fs::write(words_data.dest_path(), output).unwrap();

    println!("cargo:warning={max_string_lit_len}");

    println!(
        "cargo:warning=-- write_words_data {lang} bytes {:?}",
        start.elapsed()
    );
}

const UMLAUTS: [char; 4] = ['ß', 'ä', 'ö', 'ü'];
const ASCII_UMLAUT_REPLACEMENTS: [&str; 4] = ["ss", "ae", "oe", "ue"];

#[inline]
#[allow(clippy::ptr_arg)]
#[allow(clippy::needless_pass_by_value)]
fn str_contains_umlaut(string: String) -> bool {
    UMLAUTS.iter().any(
        #[inline]
        |ch| string.contains(*ch),
    )
}

#[allow(clippy::ptr_arg)]
#[allow(clippy::needless_pass_by_value)]
fn replace_umlauts(mut string: String) -> String {
    while let Some((idx, ch, repl_index)) =
        string.char_indices().find_map(|(idx, ch)| {
            UMLAUTS
                .iter()
                .position(|u| u == &ch)
                .map(|repl_idx| (idx, ch, repl_idx))
        })
    {
        string.replace_range(
            idx..idx + ch.len_utf8(),
            ASCII_UMLAUT_REPLACEMENTS
                .get(repl_index)
                .expect("Can find replacement"),
        );
    }
    string
}

const WORDS_DIR: &str = "./words/";

fn main() {
    let now = Instant::now();
    println!("cargo:warning=start main {:?}", now.elapsed());
    println!("cargo:rerun-if-changed={WORDS_DIR}");
    let paths = fs::read_dir(WORDS_DIR).unwrap();

    let mut words_vec: Vec<WordsData> = vec![];

    for dir_entry in paths {
        let p = dir_entry.unwrap().path();
        let path = p.as_path();
        if path.file_name().unwrap().to_str().unwrap() == "LICENSE" {
            continue;
        }
        println!("cargo:rerun-if-changed={}", path.display());

        let mut data: WordsData = WordsData::from_path(path);
        if data.read_lines().any(str_contains_umlaut) {
            words_vec
                .push(data.clone_with_lang(format!("{}_umlauts", data.lang)));
            data.conv = replace_umlauts;
        }
        words_vec.push(data);
    }

    words_vec.sort_by(|w1, w2| w1.lang.cmp(&w2.lang));

    let words_vec = words_vec;

    println!("cargo:warning=before write_words_data {:?}", now.elapsed());
    Parallel::new().each(&words_vec, write_words_data).run();
    println!("cargo:warning=after write_words_data {:?}", now.elapsed());

    let language_count = words_vec.len();

    fs::write(
        get_out_dir_joined(String::from("language.rs")),
        format!(
            r###"#[cfg_attr(feature = "pyo3", pyclass(eq))]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Language {{
    {}
}}

impl Language {{
    #[must_use]
    pub const fn read_words(self, length: usize) -> WordSequence {{
        let (padded_length, words): (NonZeroUsize, &'static str) = match self {{
            {}
        }};
        WordSequence::new(length, words, padded_length)
    }}

    #[inline]
    #[must_use]
    pub const fn all() -> [Self; {language_count}] {{
        [
            {}
        ]
    }}

    #[inline]
    #[must_use]
    pub fn from_string(string: &str) -> Option<Self> {{
        match string.to_lowercase().replace('-', "_").as_str() {{
            {},
            _ => None,
        }}
    }}

    #[allow(clippy::needless_pass_by_value)]
    #[allow(clippy::trivially_copy_pass_by_ref)]
    #[must_use]
    #[inline]
    pub const fn name(&self) -> &'static str {{
        match self {{
            {}
        }}
    }}
}}

#[cfg(feature = "pyo3")]
#[pymethods]
impl Language {{
    #[staticmethod]
    #[must_use]
    const fn values() -> [Self; {language_count}] {{
        Self::all()
    }}

    #[allow(clippy::trivially_copy_pass_by_ref)]
    #[getter]
    #[must_use]
    const fn value(&self) -> &'static str {{
        self.name()
    }}

    #[staticmethod]
    #[pyo3(signature = (name, default = None))]
    pub fn parse_string(name: &str, default: Option<Self>) -> PyResult<Self> {{
        Self::from_string(name)
            .or(default)
            .ok_or_else(|| UnknownLanguageError::new_err(name.to_owned()))
    }}
}}
"###,
            words_vec.iter().map(WordsData::enum_name).join(",\n"),
            words_vec
                .iter()
                .map(|data| format!(
                    "Self::{} => include!(concat!(env!(\"OUT_DIR\"), \"/{}\"))",
                    data.enum_name(),
                    data.out_file_name()
                ))
                .join("\n,"),
            words_vec
                .iter()
                .map(|data| format!("Self::{}", data.enum_name()))
                .join(", "),
            words_vec
                .iter()
                .map(|data| format!(
                    "\"{}\" => Some(Self::{})",
                    data.lang,
                    data.enum_name()
                ))
                .join(",\n"),
            words_vec
                .iter()
                .map(|data| format!(
                    "Self::{} => \"{}\"",
                    data.enum_name(),
                    data.lang
                ))
                .join(",\n"),
        ),
    )
    .unwrap();
    println!("cargo:warning=end main {:?}", now.elapsed());
}

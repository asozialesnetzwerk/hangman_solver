#![allow(clippy::unwrap_used)]

use std::env;
use std::fs;
use std::fs::File;
use std::io;
use std::io::BufRead;
use std::io::BufReader;
use std::path::{Path, PathBuf};

use inflector::Inflector;
use itertools::Itertools;

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

    fn read_lines(&self) -> Vec<String> {
        read_lines_of_file(Path::new(self.path.as_str()))
            .unwrap()
            .filter(|word| !word.is_empty())
            .map(|word| word.to_lowercase())
            .map(self.conv)
            .unique()
            .collect()
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
    let mut words: Vec<String> = words_data.read_lines();

    words.sort_unstable();
    words.sort_by_key(|word: &String| word.chars().count());

    let words: Vec<(usize, Vec<String>)> = words
        .into_iter()
        .unique()
        .into_group_map_by(|word: &String| word.chars().count())
        .into_iter()
        .sorted_by_key(|(length, _)| *length)
        .collect();

    let mut output = String::new();
    output += "match length {";
    for (length, words_group) in words {
        let max_real_str_length: usize = words_group
            .iter()
            .map(|word| word.as_str().len())
            .max()
            .unwrap();

        output += &*format!("{length} => ({max_real_str_length}, \"");

        let string_content: String = if max_real_str_length == length {
            words_group.iter().join("").replace('"', "\\\"")
        } else {
            println!("{} {length} {max_real_str_length}", words_data.lang);
            words_group
                .iter()
                .map(|word| {
                    ("\\0".repeat(max_real_str_length - word.as_str().len()))
                        + &word.clone()
                })
                .join("")
                .replace('"', "\\\"")
        };

        output += &string_content;

        output += "\"),\n";
    }
    output += "_ => (0, \"\")}";
    fs::write(words_data.dest_path(), output).unwrap();
}

#[allow(clippy::ptr_arg)]
fn str_contains_umlaut(string: &String) -> bool {
    string.contains('ß')
        || string.contains('ä')
        || string.contains('ö')
        || string.contains('ü')
}

#[allow(clippy::ptr_arg)]
#[allow(clippy::needless_pass_by_value)]
fn replace_umlauts(string: String) -> String {
    string
        .replace('ß', "ss")
        .replace('ä', "ae")
        .replace('ö', "oe")
        .replace('ü', "ue")
}

const WORDS_DIR: &str = "./words/";

fn main() {
    println!("cargo:rerun-if-changed={WORDS_DIR}");
    let paths = fs::read_dir(WORDS_DIR).unwrap();

    let pyo3: bool = env::var("CARGO_FEATURE_PYO3").is_ok();

    let mut words_vec: Vec<WordsData> = vec![];

    for dir_entry in paths {
        let p = dir_entry.unwrap().path();
        let path = p.as_path();
        if path.file_name().unwrap().to_str().unwrap() == "LICENSE" {
            continue;
        }
        println!("cargo:rerun-if-changed={}", path.display());

        let mut data: WordsData = WordsData::from_path(path);
        if data.read_lines().iter().any(str_contains_umlaut) {
            words_vec
                .push(data.clone_with_lang(format!("{}_umlauts", data.lang)));
            data.conv = replace_umlauts;
        }
        words_vec.push(data);
    }

    let words_vec: Vec<WordsData> = words_vec
        .iter()
        .sorted_by_key(|data| data.lang.as_str())
        .map(std::clone::Clone::clone)
        .collect();

    for word_data in &words_vec {
        write_words_data(word_data);
    }

    fs::write(
        get_out_dir_joined(String::from("language.rs")),
        format!(
            r###"{}
#[derive(Copy, Clone, Eq, PartialEq)]            
pub enum Language {{
    {}
}}

impl Language {{
    pub fn read_words(self, length: usize) -> StringChunkIter {{
        let (padded_length, words): (usize, &'static str) = match self {{
            {}
        }};
        StringChunkIter::new(length, words, padded_length)
    }}
    
    pub fn all() -> Vec<Self> {{
        vec![
            {}
        ]
    }}
    
    pub fn from_string(string: &str) -> Option<Self> {{
        match string.to_lowercase().replace('-', "_").as_str() {{
            {},
            _ => None,
        }}
    }}

    #[allow(clippy::needless_pass_by_value)]
    #[allow(clippy::trivially_copy_pass_by_ref)]
    pub fn name(&self) -> &'static str {{
        match self {{
            {}
        }}
    }}
}}
"###,
            if pyo3 { "#[pyclass]" } else { "" },
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
}

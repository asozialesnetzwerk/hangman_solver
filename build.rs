#![allow(clippy::unwrap_used)]

use inflector::Inflector;
use itertools::Itertools;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
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
    let lang = words_data.lang.as_str();
    let mut words: Vec<_> = words_data.read_lines().unique().collect();

    for word in &words {
        assert_eq!(
            word.as_str().graphemes(true).count(),
            word.chars().count(),
            "{lang}: {word} has graphemes",
        );
        assert_eq!(
            word.as_str().unicode_words().count(),
            1,
            "{lang}: {word} is multiple words",
        );
    }

    words.sort_unstable();
    words.sort_by_key(|word: &String| word.chars().count());

    let mut output = String::from("match length {");
    for (length, chunk) in
        &words.into_iter().chunk_by(|word| word.chars().count())
    {
        let words_group: Vec<_> = chunk.collect();
        let max_real_str_length: usize = words_group
            .iter()
            .map(|word| word.as_str().len())
            .max()
            .expect("word group needs to have max length");

        let start_of_case = format!("{length} => ({max_real_str_length}, \"");
        const END_OF_CASE: &'static str = "\"),\n";
        output.reserve(max_real_str_length * words_group.len() + start_of_case.len() + END_OF_CASE.len());
        output += &start_of_case;

        if max_real_str_length == length {
            words_group.into_iter().for_each(|word| output += &word.replace('"', "\\\""))
        } else {
            println!("{} {length} {max_real_str_length}", words_data.lang);
            words_group
                .into_iter()
                .map(|word| {
                    ("\\0".repeat(max_real_str_length - word.as_str().len()))
                        + &word.replace('"', "\\\"")
                })
                .for_each(|word| output += &word)
        };

        output += END_OF_CASE;
    }
    output += "_ => (0, \"\")}";
    fs::write(words_data.dest_path(), output).unwrap();
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
fn replace_umlauts(string: String) -> String {
    let mut result: String = String::with_capacity(string.len());
    for ch in string.chars() {
        if let Some(repl_index) = UMLAUTS.iter().position(|u| u == &ch) {
            result.push_str(ASCII_UMLAUT_REPLACEMENTS[repl_index]);
        } else {
            result.push(ch);
        };
    }
    result
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

    words_vec.sort_by(|w1, w2| w1.lang.as_str().cmp(w2.lang.as_str()));

    let words_vec = words_vec;

    println!("cargo:warning=before par_iter {:?}", now.elapsed());
    words_vec.par_iter().for_each(write_words_data);
    println!("cargo:warning=after par_iter {:?}", now.elapsed());

    let language_count = words_vec.len();

    fs::write(
        get_out_dir_joined(String::from("language.rs")),
        format!(
            r###"#[cfg_attr(feature = "pyo3", pyclass)]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Language {{
    {}
}}

impl Language {{
    #[must_use]
    pub const fn read_words(self, length: usize) -> StringChunkIter {{
        let (padded_length, words): (usize, &'static str) = match self {{
            {}
        }};
        StringChunkIter::new(length, words, padded_length)
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

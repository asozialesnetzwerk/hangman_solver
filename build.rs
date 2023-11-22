use std::env;
use std::fs;
use std::fs::File;
use std::io;
use std::io::BufRead;
use std::io::BufReader;
use std::path::{Path, PathBuf};

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
    fn new(path: String, lang: String, conv: StrConv) -> WordsData {
        WordsData { path, lang, conv }
    }

    fn from_path(path: &Path) -> WordsData {
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
            .map(self.conv)
            .unique()
            .collect()
    }

    fn dest_path(&self) -> PathBuf {
        let out_dir: &String = &env::var("OUT_DIR").unwrap();
        Path::new(out_dir)
            .join(format!("{}.txt.rs", self.lang))
            .to_owned()
    }
}

fn write_words_data(words_data: WordsData) {
    let mut words: Vec<String> = words_data.read_lines();

    words.sort_unstable();
    words.sort_by_key(|word: &String| word.len());

    let words: Vec<(usize, Vec<String>)> = words
        .into_iter()
        .unique()
        .into_group_map_by(|word: &String| word.len())
        .into_iter()
        .sorted_by_key(|(length, _)| *length)
        .collect();

    let mut output = String::new();
    output += "match length {";
    for (length, words_group) in words {
        output += &*format!("{length} => \"");

        output += &*(words_group.iter().join("").replace('"', "\\\""));

        output += "\",\n";
    }
    output += "_ => \"\"}";
    fs::write(words_data.dest_path(), output).unwrap();
}

fn main() {
    let paths = fs::read_dir("./words/").unwrap();

    let mut words_vec: Vec<WordsData> = vec![];

    for dir_entry in paths {
        let p = dir_entry.unwrap().path();
        let path = p.as_path();
        println!("cargo:rerun-if-changed={}", path.display());

        let mut data: WordsData = WordsData::from_path(path);
        if data.lang == "de" {
            data.conv = |word| {
                word.replace('ß', "ss")
                    .replace('ä', "ae")
                    .replace('ö', "oe")
                    .replace('ü', "ue")
            };
        }
        words_vec.push(data);
    }

    for word_data in words_vec {
        write_words_data(word_data);
    }
}

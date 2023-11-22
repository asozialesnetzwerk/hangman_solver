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
    fn clone_with_lang(&self, lang: String) -> WordsData {
        Self::new(self.path.clone(), lang, self.conv.clone())
    }

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

    fn enum_name(&self) -> String {
        self.lang
            .replace('-', "_")
            .as_str()
            .split("")
            .map(|string| string.to_lowercase().to_title_case())
            .join("")
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
    Path::new(out_dir).join(path).to_owned()
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
            words_vec.push(data.clone_with_lang(String::from("de_umlauts")));
            data.conv = |word| {
                word.replace('ß', "ss")
                    .replace('ä', "ae")
                    .replace('ö', "oe")
                    .replace('ü', "ue")
            };
        }
        words_vec.push(data);
    }

    for word_data in words_vec.clone() {
        write_words_data(word_data);
    }

    fs::write(
        get_out_dir_joined(String::from("language.rs")),
        format!(
            r###"
        #[derive(Copy, Clone, Eq, PartialEq)]
        pub enum Language {{
            {}
        }}
    
        impl Language {{
            #[must_use]
            pub fn from_string(string: &str) -> Option<Self> {{
                match string.to_lowercase().as_str() {{
                    {},
                    _ => None,
                }}
            }}
    
            pub fn read_words(self, length: usize) -> StringChunkIter<'static> {{
                let words: &'static str = match self {{
                    {}
                }};
                StringChunkIter::new(length, words)
            }}
        }}
    "###,
            words_vec.clone().iter().map(|data| data.enum_name()).join(",\n"),
            words_vec.clone().iter().map(|data| format!("\"{}\" => Some(Self::{})", data.lang, data.enum_name())).join(",\n"),
            words_vec.clone().iter().map(|data| format!("Self::{} => include!(concat!(env!(\"OUT_DIR\"), \"/{}\"))", data.enum_name(), data.out_file_name())).join("\n,")
        ),
    ).unwrap();
}

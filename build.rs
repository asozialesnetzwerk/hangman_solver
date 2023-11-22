use std::env;
use std::fs;
use std::fs::File;
use std::io;
use std::io::BufRead;
use std::io::BufReader;
use std::path::Path;

use itertools::Itertools;

fn read_lines_of_file(
    path: &Path,
) -> Result<impl Iterator<Item = String>, io::Error> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    Ok(reader.lines().map_while(Result::ok))
}

fn main() {
    let out_dir = &env::var("OUT_DIR").unwrap();

    let paths = fs::read_dir("./words/").unwrap();

    for dir_entry in paths {
        let p = dir_entry.unwrap().path();
        let path = p.as_path();
        println!("cargo:rerun-if-changed={}", path.display());
        let dest_path = Path::new(out_dir)
            .join(format!(
                "{}.rs",
                path.file_name().unwrap().to_str().unwrap()
            ))
            .to_owned();
        let mut words: Vec<String> = read_lines_of_file(path)
            .unwrap()
            .map(|word| {
                word.replace('ß', "ss")
                    .replace('ä', "ae")
                    .replace('ö', "oe")
                    .replace('ü', "ue")
            })
            .collect();

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
        fs::write(dest_path.clone(), output).unwrap();
    }
}

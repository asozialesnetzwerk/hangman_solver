// SPDX-License-Identifier: EUPL-1.2

#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

pub struct StringChunkIter<'a> {
    word_length: usize,
    index: usize,
    string: &'a str,
}

impl<'a> StringChunkIter<'a> {
    pub fn new(word_length: usize, string: &'a str) -> StringChunkIter<'a> {
        StringChunkIter {
            word_length,
            index: 0,
            string,
        }
    }
}

impl<'a> Iterator for StringChunkIter<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        let index = self.index;
        if index >= self.string.len() {
            return None;
        }
        self.index += self.word_length;

        Some(&self.string[index..self.index])
    }
}

include!(concat!(env!("OUT_DIR"), "/language.rs"));

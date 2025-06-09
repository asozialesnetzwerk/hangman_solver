// SPDX-License-Identifier: EUPL-1.2
use std::num::NonZeroUsize;

#[cfg(feature = "pyo3")]
use pyo3::prelude::*;
#[cfg(feature = "pyo3")]
use std::hash::{DefaultHasher, Hasher};

use super::StringChunkIter;

#[allow(dead_code)]
const EMPTY_WORD_SEQUENCE: WordSequence = WordSequence {
    word_length: 0,
    padded_word_byte_count: NonZeroUsize::MIN,
    data: "",
};

const _: () = assert!(EMPTY_WORD_SEQUENCE.is_empty());
const _: () = assert!(EMPTY_WORD_SEQUENCE.word_char_count() == 0);
const _: () = assert!(EMPTY_WORD_SEQUENCE.is_empty());

#[cfg_attr(feature = "pyo3", pyclass(frozen))]
pub struct WordSequence {
    word_length: usize,
    data: &'static str,
    padded_word_byte_count: NonZeroUsize,
}

impl WordSequence {
    #[inline]
    #[must_use]
    pub(crate) const fn new(
        word_length: usize,
        data: &'static str,
        padded_word_byte_count: NonZeroUsize,
    ) -> Self {
        Self {
            word_length,
            data,
            padded_word_byte_count,
        }
    }

    #[inline]
    #[must_use]
    pub const fn word_char_count(&self) -> usize {
        self.word_length
    }

    #[inline]
    #[must_use]
    pub const fn len(&self) -> usize {
        self.data.len() / self.padded_word_byte_count.get()
    }

    #[inline]
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    #[inline]
    #[must_use]
    pub fn contains(&self, word: &str) -> bool {
        if word.chars().count() == self.word_length {
            self.iter().any(|w| w == word)
        } else {
            false
        }
    }

    #[inline]
    #[must_use]
    pub fn iter(&self) -> StringChunkIter {
        self.into_iter()
    }

    #[inline]
    const fn const_convert_to_iter(&self) -> StringChunkIter {
        StringChunkIter {
            index: 0,
            is_ascii: self.word_length == self.padded_word_byte_count.get(),
            padded_word_byte_count: self.padded_word_byte_count,
            string: self.data,
        }
    }
}

impl IntoIterator for WordSequence {
    type Item = &'static str;

    type IntoIter = StringChunkIter;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.const_convert_to_iter()
    }
}

impl IntoIterator for &WordSequence {
    type Item = &'static str;

    type IntoIter = StringChunkIter;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.const_convert_to_iter()
    }
}

#[cfg(feature = "pyo3")]
#[derive(FromPyObject)]
pub enum ContainsArg {
    StringArg(String),
    Other(PyObject),
}

#[cfg(feature = "pyo3")]
#[pymethods]
impl WordSequence {
    #[must_use]
    pub fn __iter__(&self) -> StringChunkIter {
        self.into_iter()
    }

    #[must_use]
    pub const fn __len__(&self) -> usize {
        self.len()
    }

    #[must_use]
    pub fn __hash__(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        hasher.write(self.data.as_bytes());
        hasher.finish()
    }

    #[must_use]
    pub const fn __bool__(&self) -> bool {
        !self.is_empty()
    }

    #[must_use]
    pub fn __contains__(&self, arg: ContainsArg) -> bool {
        if let ContainsArg::StringArg(string) = arg {
            self.contains(&string)
        } else {
            false
        }
    }

    #[must_use]
    pub fn __repr__(&self) -> String {
        const START: &str = "['";
        const SEPARATOR: &str = "', '";
        const END: &str = "']";

        const _: () = assert!(SEPARATOR.len() == START.len() + END.len());

        let mut data = String::with_capacity(
            self.data.len() + SEPARATOR.len() * self.len(),
        );
        data.push_str(START);

        let mut iter = self.iter();
        while let Some(word) = iter.next() {
            data.push_str(word);
            if iter.__len__() > 0 {
                data.push_str(SEPARATOR);
            }
        }

        data.push_str(END);

        data.shrink_to_fit();
        data
    }
}

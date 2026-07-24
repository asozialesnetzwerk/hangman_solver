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
    pub fn get(&self, index: usize) -> Option<&str> {
        self.into_iter().nth(index)
    }

    #[inline]
    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    pub fn contains(&self, word: &str) -> bool {
        if word.chars().count() != self.word_char_count() {
            return false;
        }

        let length = self.len();

        if length == 0 {
            return false;
        }

        let mut low = 0usize;
        let mut high = length - 1;

        while low <= high {
            let mid = low + (high - low) / 2;

            let mid_value = self
                .get(mid)
                .expect("this can't fail if binary search is correct");
            match mid_value.cmp(word) {
                std::cmp::Ordering::Less => low = mid + 1,
                std::cmp::Ordering::Equal => return true,
                std::cmp::Ordering::Greater => high = mid - 1,
            }
        }

        false
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
        hasher.write_usize(self.word_length);
        hasher.write_usize(self.padded_word_byte_count.get());
        hasher.write(self.data.as_bytes());
        hasher.finish()
    }

    #[must_use]
    pub const fn __bool__(&self) -> bool {
        !self.is_empty()
    }

    #[must_use]
    pub fn __contains__(&self, arg: &Bound<'_, PyAny>) -> bool {
        let Ok(string) = arg.extract::<std::borrow::Cow<'_, str>>() else {
            return false;
        };

        self.contains(&string)
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

#[cfg(test)]
mod tests {
    use super::WordSequence;
    use crate::Language;

    #[test]
    fn test_word_sequence_words_len() {
        for lang in Language::all() {
            assert!(lang.read_words(4).len() > 100);
            assert!(lang.read_words(5).len() > 100);
            assert!(lang.read_words(6).len() > 100);

            for i in 0..100 {
                assert_eq!(
                    lang.read_words(i).len(),
                    lang.read_words(i).into_iter().count()
                );
            }
        }
    }

    #[test]
    fn test_word_sequence_words_have_same_length() {
        for lang in Language::all() {
            for i in 0..100 {
                assert_eq!(lang.read_words(i).word_char_count(), i);

                for word in lang.read_words(i) {
                    assert_eq!(word.chars().count(), i);
                }
            }
        }
    }

    #[test]
    fn test_word_sequence_is_sorted() {
        for lang in Language::all() {
            for i in 0..100 {
                assert!(lang.read_words(i).into_iter().is_sorted());
            }
        }
    }

    #[test]
    fn test_word_sequence_get() {
        for lang in Language::all() {
            assert_eq!(
                lang.read_words(10).get(0).expect("we have more than 1 ten letter word"),
                lang.read_words(10).into_iter().next().expect("we have more than 1 ten letter word"),
            );

            for i in 0..100 {
                let words: WordSequence = lang.read_words(i);

                assert!(
                    words
                        .iter()
                        .zip(
                            (0..words.len())
                                .map(|i| words.get(i).expect("i is in bounds"))
                        )
                        .all(|(a, b)| a == b)
                );
            }
        }
    }

    #[test]
    fn test_word_sequence_contains_with_real_words() {
        assert!(Language::De.read_words(4).contains("test"));

        for lang in Language::all() {
            for i in 0..100 {
                let words: WordSequence = lang.read_words(i);

                for word in &words {
                    assert!(words.contains(word));
                }
            }
        }
    }

    #[test]
    fn test_word_sequence_contains_with_broken_words() {
        assert!(!Language::De.read_words(4).contains("xxx"));
        assert!(!Language::De.read_words(4).contains("xxxx"));
        assert!(!Language::De.read_words(4).contains("xxxxx"));

        for lang in Language::all() {
            for i in 2..100 {
                let words: WordSequence = lang.read_words(i);

                assert!(!words.contains("x"));
                assert!(!words.contains("abcde"));
                assert!(!words.contains("mmmmmmmm"));
                assert!(!words.contains(&"x".repeat(i)));
            }
        }
    }
}

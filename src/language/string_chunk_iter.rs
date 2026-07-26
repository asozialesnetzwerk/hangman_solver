use std::iter::FusedIterator;
// SPDX-License-Identifier: EUPL-1.2
use std::num::NonZeroUsize;

#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

#[cfg_attr(feature = "pyo3", pyclass(skip_from_py_object))]
#[derive(Clone)]
pub struct StringChunkIter {
    pub(super) padded_word_byte_count: NonZeroUsize,
    pub(super) is_ascii: bool,
    pub(super) string: &'static str,
}

impl StringChunkIter {
    #[inline]
    const fn remaining_words(&self) -> usize {
        self.string.len() / self.padded_word_byte_count.get()
    }
}

impl FusedIterator for StringChunkIter {}

impl ExactSizeIterator for StringChunkIter {
    fn len(&self) -> usize {
        self.remaining_words()
    }
}

impl Iterator for StringChunkIter {
    type Item = &'static str;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.nth(0)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let length = self.remaining_words();

        (length, Some(length))
    }

    #[inline]
    fn count(self) -> usize {
        self.remaining_words()
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        let Some(advance_by) = self
            .padded_word_byte_count
            .get()
            .checked_mul(n.saturating_add(1))
        else {
            // we tried to advance more than possible
            self.string = "";
            return None;
        };
        let Some((value, new_string)) =
            self.string.split_at_checked(advance_by)
        else {
            debug_assert!(advance_by > self.string.len());
            self.string = "";
            return None;
        };
        self.string = new_string;
        debug_assert!(value.len() >= self.padded_word_byte_count.get());

        let result = if n == 0 {
            value
        } else {
            &value[value.len() - self.padded_word_byte_count.get()..value.len()]
        };

        debug_assert_eq!(result.len(), self.padded_word_byte_count.get());

        let result = if self.is_ascii {
            result
        } else {
            result.trim_start_matches('\0')
        };

        debug_assert!(!result.contains('\0'));
        Some(result)
    }
}

impl DoubleEndedIterator for StringChunkIter {
    fn next_back(&mut self) -> Option<Self::Item> {
        let pivot = self
            .string
            .len()
            .checked_sub(self.padded_word_byte_count.get())?;

        let result = self.string.get(pivot..self.string.len())?;
        self.string = &self.string[..pivot];

        debug_assert_eq!(result.len(), self.padded_word_byte_count.get());

        let result = if self.is_ascii {
            result
        } else {
            result.trim_start_matches('\0')
        };

        debug_assert!(!result.contains('\0'));
        Some(result)
    }
}

#[cfg(feature = "pyo3")]
#[pymethods]
impl StringChunkIter {
    #[must_use]
    const fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    #[must_use]
    fn __next__(&mut self) -> Option<&'static str> {
        self.next()
    }

    #[must_use]
    pub const fn __len__(&self) -> usize {
        self.remaining_words()
    }

    #[must_use]
    #[allow(clippy::needless_pass_by_value)]
    pub fn __reversed__(
        slf: PyRef<'_, Self>,
    ) -> super::reversed_string_chunk_iter::ReversedStringChunkIter {
        super::reversed_string_chunk_iter::ReversedStringChunkIter::from(
            slf.clone(),
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::Language;

    use super::StringChunkIter;

    #[test]
    fn test_string_chunk_iter() {
        const STRING: &str = "abcdefgh";

        let mut string_chunk_iter = StringChunkIter {
            padded_word_byte_count: 1.try_into().expect("1 is not 0"),
            is_ascii: true,
            string: STRING,
        };

        assert_eq!(STRING.len(), string_chunk_iter.clone().count());

        assert_eq!(string_chunk_iter.next(), Some("a"));
        assert_eq!(string_chunk_iter.nth(1), Some("c"));
        assert_eq!(string_chunk_iter.nth(usize::MAX), None);
        assert_eq!(string_chunk_iter.next(), None);

        let mut string_chunk_iter = StringChunkIter {
            padded_word_byte_count: 2.try_into().expect("2 is not 0"),
            is_ascii: false,
            string: STRING,
        };

        assert_eq!(string_chunk_iter.next(), Some("ab"));
        assert_eq!(string_chunk_iter.nth(1), Some("ef"));
        assert_eq!(string_chunk_iter.nth(usize::MAX - 100), None);
        assert_eq!(string_chunk_iter.next(), None);
    }

    #[test]
    fn test_string_chunk_iter_being_exact_sized() {
        let sequence = Language::En.read_words(5);
        let length = sequence.len();

        assert!(length > 100);

        let iterator: StringChunkIter = sequence.into_iter();

        assert_eq!(iterator.len(), length);
        assert_eq!(iterator.size_hint(), (length, Some(length)));
        assert_eq!(iterator.clone().count(), length);

        let mut c = 0;

        for _ in iterator {
            c += 1;
        }

        assert_eq!(c, length);
    }

    #[test]
    fn test_string_chunk_iter_being_fused() {
        let mut iterator: StringChunkIter =
            Language::DeUmlauts.read_words(6).iter();

        let start_length = iterator.len();
        assert!(start_length > 100);

        for _ in 0..25 {
            assert!(iterator.next().is_some());
        }
        for _ in 0..25 {
            assert!(iterator.next_back().is_some());
        }
        assert_eq!(iterator.len() + 50, start_length);
        assert!(iterator.nth(iterator.len() - 1).is_some());

        for _ in 0..100 {
            assert_eq!(iterator.next(), None);
            assert_eq!(iterator.next_back(), None);
        }
    }

    #[test]
    fn test_string_chunk_iter_being_double_ended() {
        let mut iterator: StringChunkIter =
            Language::DeUmlauts.read_words(10).iter();

        let mut last_hundred_words = vec![];

        let mut cloned_iterator = iterator.clone();
        while last_hundred_words.len() < 100
            && let Some(word) = cloned_iterator.next_back()
        {
            last_hundred_words.push(word);
        }

        assert!(iterator.nth(iterator.remaining_words() - 100 - 1).is_some());

        assert_eq!(iterator.len(), 100);

        let reversed_words = iterator.clone().rev().collect::<Vec<_>>();
        assert_eq!(reversed_words.len(), 100);

        assert_eq!(last_hundred_words, reversed_words);

        let mut words = iterator.collect::<Vec<_>>();
        assert_eq!(words.len(), 100);

        words.reverse();

        assert_eq!(last_hundred_words, words);
    }
}

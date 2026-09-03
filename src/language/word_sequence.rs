// SPDX-License-Identifier: EUPL-1.2
use std::hash::{Hash, Hasher};
use std::num::NonZeroUsize;
#[cfg(feature = "pyo3")]
use std::sync::LazyLock;

#[cfg(feature = "pyo3")]
use itertools::Itertools;
#[cfg(feature = "pyo3")]
use pyo3::{
    IntoPyObjectExt,
    exceptions::{PyIndexError, PyTypeError, PyValueError},
    prelude::*,
    types::PySlice,
};

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

#[cfg_attr(feature = "pyo3", pyclass(frozen, skip_from_py_object))]
#[cfg_attr(feature = "pyo3", cfg_attr(any(Py_3_14, all(Py_3_10, not(Py_LIMITED_API))), pyo3(immutable_type)))]
pub struct WordSequence {
    word_length: usize,
    data: &'static str,
    padded_word_byte_count: NonZeroUsize,
}

impl Hash for WordSequence {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        hasher.write_usize(self.len());

        for word in self {
            word.hash(hasher);
        }
    }
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
    fn index_of(&self, word: &str) -> Option<usize> {
        if word.chars().count() != self.word_char_count() {
            return None;
        }

        let length = self.len();

        if length == 0 {
            return None;
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
                std::cmp::Ordering::Equal => return Some(mid),
                std::cmp::Ordering::Greater => high = mid - 1,
            }
        }

        None
    }

    #[cfg(feature = "pyo3")]
    const fn convert_index(&self, index: isize) -> Option<usize> {
        if index < 0 {
            self.len().checked_add_signed(index)
        } else {
            0usize.checked_add_signed(index)
        }
    }

    #[inline]
    #[must_use]
    pub fn contains(&self, word: &str) -> bool {
        self.index_of(word).is_some()
    }

    #[inline]
    #[must_use]
    pub fn iter(&self) -> StringChunkIter {
        self.into_iter()
    }

    #[inline]
    const fn const_convert_to_iter(&self) -> StringChunkIter {
        StringChunkIter {
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
    pub fn __hash__(&self, py: Python<'_>) -> u64 {
        use std::hash::{BuildHasher as _, RandomState};

        static RANDOM: LazyLock<RandomState> = LazyLock::new(RandomState::new);

        py.detach(|| RANDOM.hash_one(self))
    }

    #[must_use]
    pub fn __eq__(&self, py: Python<'_>, other: &Self) -> bool {
        if self.len() != other.len() {
            return false;
        }
        py.detach(|| self.iter().zip_eq(other.iter()).all(|(a, b)| a == b))
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

    #[pyo3(signature=(value, start = 0, stop = None))]
    pub fn index(
        &self,
        value: &Bound<'_, PyAny>,
        start: isize,
        stop: Option<isize>,
    ) -> PyResult<usize> {
        let Ok(string) = value.extract::<std::borrow::Cow<'_, str>>() else {
            return Err(PyValueError::new_err(()));
        };
        let get_index = move || -> Option<usize> {
            let index = self.index_of(&string)?;

            if start != 0 {
                let start = self.convert_index(start)?;
                if start > index {
                    return None;
                }
            }
            if let Some(stop) = stop {
                let stop = self.convert_index(stop)?;
                if stop <= index {
                    return None;
                }
            }

            Some(index)
        };

        get_index()
            .ok_or_else(|| PyValueError::new_err("value not in sequence"))
    }

    #[must_use]
    pub fn count(&self, arg: &Bound<'_, PyAny>) -> u8 {
        let Ok(string) = arg.extract::<std::borrow::Cow<'_, str>>() else {
            return 0;
        };

        self.contains(&string).into()
    }

    #[allow(clippy::missing_panics_doc)]
    pub fn __getitem__(&self, arg: &Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
        if let Ok(index) = arg.extract::<isize>() {
            let index: usize = self
                .convert_index(index)
                .ok_or_else(|| PyIndexError::new_err("index out of range"))?;

            let value = self
                .get(index)
                .ok_or_else(|| PyIndexError::new_err("index out of range"))?;

            return value.into_py_any(arg.py());
        }
        if let Ok(slice) = arg.cast::<PySlice>() {
            if self.is_empty() {
                let value = Self {
                    word_length: self.word_length,
                    padded_word_byte_count: self.padded_word_byte_count,
                    data: "",
                };

                return value.into_py_any(arg.py());
            }

            let indices = slice.indices(self.len().try_into()?)?;

            if indices.slicelength == 0 {
                let value = Self {
                    word_length: self.word_length,
                    padded_word_byte_count: self.padded_word_byte_count,
                    data: "",
                };

                return value.into_py_any(arg.py());
            }

            if indices.step == 1 {
                let start: usize = indices
                    .start
                    .try_into()
                    .expect("start has to be positive if step is 1");
                let stop: usize = indices
                    .stop
                    .try_into()
                    .expect("stop has to be positive if step is 1");

                let range = start * self.padded_word_byte_count.get()
                    ..stop * self.padded_word_byte_count.get();

                let value = Self {
                    word_length: self.word_length,
                    padded_word_byte_count: self.padded_word_byte_count,
                    data: self.data.get(range).unwrap_or(""),
                };

                return value.into_py_any(arg.py());
            }

            let value: Vec<&str> = arg.py().detach(|| {
                PyResult::Ok(if indices.step > 0 {
                    let step: usize =
                        indices.step.try_into().expect("step is positive");
                    let start: usize = indices
                        .start
                        .try_into()
                        .expect("start has to be positive if step is positive");

                    self.iter()
                        .skip(start)
                        .step_by(step)
                        .take(indices.slicelength)
                        .collect()
                } else {
                    let start_from_end: usize = if indices.start < 0 {
                        indices.start.unsigned_abs()
                    } else {
                        // len can't be zero here
                        (self.len() - 1)
                            .checked_sub_signed(indices.start)
                            .ok_or_else(|| {
                                PyIndexError::new_err("index out of range")
                            })?
                    };

                    self.iter()
                        .rev()
                        .skip(start_from_end)
                        .step_by(indices.step.unsigned_abs())
                        .take(indices.slicelength)
                        .collect()
                })
            })?;

            return value.into_py_any(arg.py());
        }

        let name = arg.get_type().fully_qualified_name()?;
        let type_name = name.extract::<std::borrow::Cow<'_, str>>()?;
        Err(PyTypeError::new_err(format!(
            "indices must be integers or slices, not {type_name}"
        )))
    }

    #[must_use]
    fn __reversed__(
        &self,
    ) -> super::reversed_string_chunk_iter::ReversedStringChunkIter {
        super::reversed_string_chunk_iter::ReversedStringChunkIter::from(
            self.iter(),
        )
    }

    #[must_use]
    pub fn __repr__(&self, py: Python<'_>) -> String {
        const START: &str = "['";
        const SEPARATOR: &str = "', '";
        const END: &str = "']";

        const _: () = assert!(SEPARATOR.len() == START.len() + END.len());

        if self.is_empty() {
            return "[]".into();
        }

        py.detach(|| {
            let mut data = String::with_capacity(
                self.data.len()
                    + SEPARATOR.len() * self.len()
                    + const { END.len() + START.len() },
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
        })
    }
}

#[cfg(test)]
mod tests {
    use std::hash::{
        BuildHasher as _, BuildHasherDefault, DefaultHasher, RandomState,
    };

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
                lang.read_words(10)
                    .get(0)
                    .expect("we have more than 1 ten letter word"),
                lang.read_words(10)
                    .into_iter()
                    .next()
                    .expect("we have more than 1 ten letter word"),
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

    #[test]
    fn test_word_sequence_hash() {
        let random_state = RandomState::new();

        let empty_hash = random_state.hash_one(Language::De.read_words(0));

        for lang in Language::all() {
            for i in 0..100 {
                let words: WordSequence = lang.read_words(i);

                if words.is_empty() {
                    assert_eq!(random_state.hash_one(&words), empty_hash);
                }

                assert_eq!(
                    random_state.hash_one(&words),
                    random_state.hash_one(&words)
                );
                let random_state = BuildHasherDefault::<DefaultHasher>::new();
                assert_eq!(
                    random_state.hash_one(&words),
                    random_state.hash_one(words)
                );
            }
        }
    }
}

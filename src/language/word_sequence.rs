// SPDX-License-Identifier: EUPL-1.2
use std::num::NonZeroUsize;
use std::ops::{Div, Range};

#[cfg(feature = "pyo3")]
use pyo3::exceptions::*;
#[cfg(feature = "pyo3")]
use pyo3::prelude::*;
#[cfg(feature = "pyo3")]
use pyo3::types::*;
#[cfg(feature = "pyo3")]
use std::hash::{DefaultHasher, Hasher};

use super::StringChunkIter;

pub const EMPTY_WORD_SEQUENCE: WordSequence = WordSequence {
    word_length: NonZeroUsize::MIN,
    padded_word_byte_count: NonZeroUsize::MIN,
    data: "",
};

#[cfg_attr(feature = "pyo3", pyclass)]
pub struct WordSequence {
    word_length: NonZeroUsize,
    data: &'static str,
    padded_word_byte_count: NonZeroUsize,
}

impl WordSequence {
    #[inline]
    #[must_use]
    pub(crate) const fn new(
        word_length: NonZeroUsize,
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
        self.word_length.get()
    }

    #[inline]
    #[must_use]
    pub fn get<I>(&self, index: I) -> Option<I::Output>
    where
        I: WordSequenceIndex,
    {
        index.get(self)
    }

    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.data.len().div(self.padded_word_byte_count)
    }

    #[inline]
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    #[inline]
    #[must_use]
    pub fn iter(&self) -> StringChunkIter {
        self.into_iter()
    }
}

impl IntoIterator for WordSequence {
    type Item = &'static str;

    type IntoIter = StringChunkIter;

    #[inline]
    #[must_use]
    fn into_iter(self) -> Self::IntoIter {
        StringChunkIter {
            index: 0,
            padded_word_byte_count: self.padded_word_byte_count,
            string: self.data,
            is_ascii: self.padded_word_byte_count == self.word_length,
        }
    }
}

impl IntoIterator for &WordSequence {
    type Item = &'static str;

    type IntoIter = StringChunkIter;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        StringChunkIter {
            index: 0,
            is_ascii: self.word_length == self.padded_word_byte_count,
            padded_word_byte_count: self.padded_word_byte_count,
            string: self.data,
        }
    }
}

pub trait WordSequenceIndex {
    type Output;

    fn get(&self, word_sequence: &WordSequence) -> Option<Self::Output>;
}

impl WordSequenceIndex for usize {
    type Output = &'static str;

    #[inline]
    fn get(&self, word_sequence: &WordSequence) -> Option<Self::Output> {
        let data_index: Self =
            self.checked_mul(word_sequence.padded_word_byte_count.into())?;
        Some(
            word_sequence
                .data
                .get(
                    data_index
                        ..data_index.checked_add(
                            word_sequence.padded_word_byte_count.into(),
                        )?,
                )?
                .trim_start_matches('\0'),
        )
    }
}

impl WordSequenceIndex for Range<usize> {
    type Output = WordSequence;

    #[inline]
    fn get(&self, word_sequence: &WordSequence) -> Option<Self::Output> {
        Some(WordSequence {
            data: word_sequence.data.get(
                self.start
                    .checked_mul(word_sequence.padded_word_byte_count.into())?
                    ..self.end.checked_mul(
                        word_sequence.padded_word_byte_count.into(),
                    )?,
            )?,
            padded_word_byte_count: word_sequence.padded_word_byte_count,
            word_length: word_sequence.word_length,
        })
    }
}

#[cfg(feature = "pyo3")]
pub enum GetItemResult {
    Item(&'static str),
}

#[cfg(feature = "pyo3")]
impl IntoPy<PyObject> for GetItemResult {
    fn into_py(self, py: pyo3::Python<'_>) -> pyo3::Py<pyo3::PyAny> {
        match self {
            Self::Item(value) => value.into_py(py),
        }
    }
}

#[cfg(feature = "pyo3")]
#[derive(FromPyObject)]
pub enum GetItemArg {
    Int(isize),
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
    #[inline]
    pub fn __len__(&self) -> usize {
        self.len()
    }

    #[must_use]
    pub fn __getitem__(&self, index: GetItemArg) -> PyResult<GetItemResult> {
        match index {
            GetItemArg::Int(index) => {
                let uindex: Option<usize> = if index >= 0 {
                    usize::try_from(index).ok()
                } else {
                    self.len().checked_add_signed(index)
                };
                match uindex {
                    None => Err(PyIndexError::new_err("Invalid index")),
                    Some(index) => match self.get(index) {
                        None => {
                            Err(PyIndexError::new_err("Index out of range"))
                        }
                        Some(value) => Ok(GetItemResult::Item(value)),
                    },
                }
            }
        }
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
    #[inline]
    #[pyo3(signature = (arg, /))]
    pub fn index(&self, arg: ContainsArg) -> PyResult<usize> {
        match arg {
            ContainsArg::StringArg(string) => {
                if string.chars().count() == self.word_length.get() {
                    for (i, word) in self.into_iter().enumerate() {
                        if word == string {
                            return Ok(i);
                        }
                    }
                }
                Err(PyValueError::new_err("Word not in words."))
            }
            ContainsArg::Other(_) => Err(PyTypeError::new_err("Not a string.")),
        }
    }

    #[must_use]
    #[inline]
    pub fn __contains__(&self, arg: ContainsArg) -> bool {
        self.index(arg).is_ok()
    }

    #[must_use]
    #[inline]
    pub fn count(&self, arg: ContainsArg) -> u8 {
        u8::from(self.__contains__(arg))
    }

    // todo: __reversed__
    // https://users.rust-lang.org/t/solved-slice-protocol-and-custom-conversions-for-a-rust-object-exposed-to-python-via-pyo3/77633
}

// SPDX-License-Identifier: EUPL-1.2

use pyo3::{PyRef, pyclass, pymethods};

use super::StringChunkIter;

#[pyclass(skip_from_py_object)]
pub struct ReversedStringChunkIter {
    iter: StringChunkIter,
}

impl From<StringChunkIter> for ReversedStringChunkIter {
    fn from(value: StringChunkIter) -> Self {
        Self { iter: value }
    }
}

#[pymethods]
impl ReversedStringChunkIter {
    #[must_use]
    const fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    #[must_use]
    fn __next__(&mut self) -> Option<&'static str> {
        self.iter.next_back()
    }

    #[must_use]
    pub fn __len__(&self) -> usize {
        self.iter.len()
    }

    #[must_use]
    #[allow(clippy::needless_pass_by_value)]
    pub fn __reversed__(slf: PyRef<'_, Self>) -> StringChunkIter {
        slf.iter.clone()
    }
}

// SPDX-License-Identifier: EUPL-1.2

pub trait CharUtils {
    fn to_ascii_char(self) -> Option<u8>;
}

impl CharUtils for char {
    #[inline]
    fn to_ascii_char(self) -> Option<u8> {
        if self.is_ascii() {
            Some(self as u8)
        } else {
            None
        }
    }
}

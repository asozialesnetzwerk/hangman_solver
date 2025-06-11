#[cfg(feature = "wasm-bindgen")]
use js_sys::JsString;

pub trait InfallibleCharCollection {
    #[must_use]
    fn char_count(&self) -> usize;

    #[must_use]
    #[inline]
    fn first_char(&self) -> Option<char> {
        self.iter_chars().next()
    }

    #[must_use]
    fn iter_chars(&self) -> impl Iterator<Item = char> + '_;
}

impl<CC: InfallibleCharCollection + ?Sized> InfallibleCharCollection for &CC {
    #[inline]
    fn iter_chars(&self) -> impl Iterator<Item = char> + '_ {
        CC::iter_chars(self)
    }

    #[inline]
    fn char_count(&self) -> usize {
        CC::char_count(self)
    }

    #[inline]
    fn first_char(&self) -> Option<char> {
        CC::first_char(self)
    }
}

impl InfallibleCharCollection for String {
    #[inline]
    fn char_count(&self) -> usize {
        self.chars().count()
    }

    #[inline]
    fn iter_chars(&self) -> impl Iterator<Item = char> + '_ {
        self.chars()
    }
}

impl InfallibleCharCollection for str {
    #[inline]
    fn char_count(&self) -> usize {
        self.chars().count()
    }

    #[inline]
    fn iter_chars(&self) -> impl Iterator<Item = char> + '_ {
        self.chars()
    }
}

impl<const L: usize> InfallibleCharCollection for [char; L] {
    #[inline]
    fn char_count(&self) -> usize {
        L
    }

    #[inline]
    fn iter_chars(&self) -> impl Iterator<Item = char> + '_ {
        self.iter().copied()
    }

    #[inline]
    fn first_char(&self) -> Option<char> {
        if L > 0 { self.first().copied() } else { None }
    }
}

impl InfallibleCharCollection for [char] {
    #[inline]
    fn char_count(&self) -> usize {
        self.len()
    }

    #[inline]
    fn iter_chars(&self) -> impl Iterator<Item = char> + '_ {
        self.iter().copied()
    }
}

impl InfallibleCharCollection for Vec<char> {
    #[inline]
    fn char_count(&self) -> usize {
        self.len()
    }

    #[inline]
    fn iter_chars(&self) -> impl Iterator<Item = char> + '_ {
        self.iter().copied()
    }
}

impl InfallibleCharCollection for Box<[char]> {
    #[inline]
    fn char_count(&self) -> usize {
        self.len()
    }

    #[inline]
    fn iter_chars(&self) -> impl Iterator<Item = char> + '_ {
        self.iter().copied()
    }
}

#[cfg(feature = "wasm-bindgen")]
struct CodepointIterator<'js> {
    js_string: &'js JsString,
    utf16_idx: u32,
}

#[cfg(feature = "wasm-bindgen")]
impl Iterator for CodepointIterator<'_> {
    type Item = char;

    #[expect(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.utf16_idx >= self.js_string.length() {
            None
        } else {
            use wasm_bindgen::{JsValue, UnwrapThrowExt};

            let x: JsValue = self.js_string.code_point_at(self.utf16_idx);

            let ch = x.as_f64().unwrap_throw() as u32;

            if ch > u32::from(u16::MAX) {
                self.utf16_idx += 2;
            } else {
                self.utf16_idx += 1;
            }

            Some(char::try_from(ch).unwrap_throw())
        }
    }
}

#[cfg(feature = "wasm-bindgen")]
impl InfallibleCharCollection for JsString {
    #[inline]
    fn iter_chars(&self) -> impl Iterator<Item = char> + '_ {
        CodepointIterator {
            js_string: self,
            utf16_idx: 0,
        }
    }

    #[inline]
    fn char_count(&self) -> usize {
        self.iter_chars().count()
    }
}

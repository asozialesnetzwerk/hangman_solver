#[cfg(feature = "wasm-bindgen")]
use js_sys::JsString;

pub trait CharCollection {
    #[must_use]
    #[inline]
    fn char_count(&self) -> usize {
        self.iter_chars().count()
    }

    #[must_use]
    #[inline]
    fn first_char(&self) -> Option<char> {
        self.iter_chars().next()
    }

    #[must_use]
    fn iter_chars(&self) -> impl Iterator<Item = char> + '_;
}

impl CharCollection for String {
    #[inline]
    fn iter_chars(&self) -> impl Iterator<Item = char> + '_ {
        self.chars()
    }
}

impl CharCollection for str {
    #[inline]
    fn iter_chars(&self) -> impl Iterator<Item = char> + '_ {
        self.chars()
    }
}

impl CharCollection for &str {
    #[inline]
    fn iter_chars(&self) -> impl Iterator<Item = char> + '_ {
        self.chars()
    }
}

impl CharCollection for [char] {
    #[inline]
    fn char_count(&self) -> usize {
        self.len()
    }

    #[inline]
    fn iter_chars(&self) -> impl Iterator<Item = char> + '_ {
        self.iter().copied()
    }
}

impl CharCollection for Vec<char> {
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
impl CharCollection for JsString {
    fn iter_chars(&self) -> impl Iterator<Item = char> + '_ {
        self.iter().map(|u: u16| {
            char::from_u32(u32::from(u)).expect("failed to parse char")
        })
    }
}

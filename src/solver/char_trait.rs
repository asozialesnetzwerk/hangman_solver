// SPDX-License-Identifier: EUPL-1.2

pub trait ControlChars: PartialEq + Sized {
    const WILDCARD: Self;
    const WILDCARD_ALIASES: [Self; 2];
    const RESERVED: [Self; 5];

    #[inline]
    #[must_use]
    #[expect(clippy::wrong_self_convention)]
    fn is_wildcard(self) -> bool {
        Self::WILDCARD_ALIASES.contains(&self) || Self::WILDCARD == self
    }

    #[inline]
    #[must_use]
    #[expect(clippy::wrong_self_convention)]
    fn is_normalised_wildcard(self) -> bool {
        self == Self::WILDCARD
    }

    #[inline]
    #[must_use]
    #[allow(dead_code)]
    fn is_reserved(&self) -> bool {
        Self::RESERVED.contains(self)
    }

    #[inline]
    #[must_use]
    #[allow(dead_code)]
    fn normalise_wildcard(self) -> Self {
        if Self::WILDCARD_ALIASES.contains(&self) {
            Self::WILDCARD
        } else {
            self
        }
    }
}

const WILDCARD_CHAR: char = '_';
const WILDCARD_ALIASES: [char; 2] = ['#', '?'];
const RESERVED_CHARS: [char; 5] = ['#', '?', WILDCARD_CHAR, '\0', '\n'];
const WILDCARD_U8: u8 = WILDCARD_CHAR as u8;
const WILDCARD_ALIASES_U8: [u8; 2] = [b'#', b'?'];
const RESERVED_U8S: [u8; 5] = [b'#', b'?', WILDCARD_U8, b'\0', b'\n'];

impl ControlChars for char {
    const WILDCARD: Self = WILDCARD_CHAR;
    const WILDCARD_ALIASES: [Self; 2] = WILDCARD_ALIASES;
    const RESERVED: [Self; 5] = RESERVED_CHARS;
}

impl ControlChars for u8 {
    const WILDCARD: Self = WILDCARD_U8;
    const WILDCARD_ALIASES: [Self; 2] = WILDCARD_ALIASES_U8;
    const RESERVED: [Self; 5] = RESERVED_U8S;
}

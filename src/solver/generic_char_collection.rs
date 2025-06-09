use crate::solver::char_collection::CharCollection;

pub trait GenericCharCollection<Char> {
    #[must_use]
    fn count(&self) -> usize;

    #[must_use]
    fn first(&self) -> Option<Char>;

    #[must_use]
    fn char_iter(&self) -> impl Iterator<Item = Char> + '_;

    #[must_use]
    fn iter_lowercased(&self) -> impl Iterator<Item = Char> + '_;
}

impl<T: CharCollection> GenericCharCollection<u8> for T {
    #[inline]
    fn first(&self) -> Option<u8> {
        self.first_ascii_char()
    }

    #[inline]
    fn char_iter(&self) -> impl Iterator<Item = u8> + '_ {
        self.iter_ascii_chars()
    }

    #[inline]
    fn iter_lowercased(&self) -> impl Iterator<Item = u8> + '_ {
        self.iter_ascii_chars().map(|ch| ch.to_ascii_lowercase())
    }

    #[inline]
    fn count(&self) -> usize {
        self.char_count()
    }
}

impl<T: CharCollection + ?Sized> GenericCharCollection<char> for T {
    #[inline]
    fn first(&self) -> Option<char> {
        CharCollection::first_char(self)
    }

    #[inline]
    fn char_iter(&self) -> impl Iterator<Item = char> + '_ {
        CharCollection::iter_chars(self)
    }

    #[inline]
    fn iter_lowercased(&self) -> impl Iterator<Item = char> + '_ {
        self.iter_chars().flat_map(char::to_lowercase)
    }

    #[inline]
    fn count(&self) -> usize {
        self.char_count()
    }
}

use crate::solver::Language;
use crate::solver::char_collection::CharCollection as _;
use crate::solver::char_trait::ControlChars as _;
use crate::solver::char_utils::CharUtils;
use crate::solver::hangman_result::HangmanResult;
use crate::solver::pattern::{AsciiPattern, Pattern};

#[allow(dead_code)]
pub fn compile_pattern(
    pattern: &str,
    invalid_letters: &[char],
    letters_in_pattern_have_no_other_occurrences: bool,
) -> Box<dyn PatternTrait> {
    if pattern.all_chars_are_ascii() && invalid_letters.all_chars_are_ascii() {
        Box::new(AsciiPattern::new(
            pattern,
            invalid_letters,
            letters_in_pattern_have_no_other_occurrences,
        ))
    } else {
        Box::new(Pattern::new(
            pattern,
            invalid_letters,
            letters_in_pattern_have_no_other_occurrences,
        ))
    }
}

#[allow(dead_code)]
#[derive(PartialEq, Eq, Debug)]
pub enum PatternLetter {
    Wildcard,
    Char(char),
}

#[allow(dead_code)]
pub trait PatternTrait {
    #[must_use]
    fn length(&self) -> usize;
    #[must_use]
    fn get_letter(&self, index: usize) -> Option<PatternLetter>;
    #[must_use]
    fn is_allowed_letter(&self, ch: char) -> bool;
    #[must_use]
    fn into_pattern(self: Box<Self>) -> Pattern;
    #[must_use]
    fn solve(
        &self,
        language: Language,
        max_words_to_collect: Option<usize>,
    ) -> HangmanResult;

    #[must_use]
    fn known_letters_count(&self) -> usize;
}

impl PatternTrait for Pattern {
    fn length(&self) -> usize {
        self.pattern.char_count()
    }

    fn get_letter(&self, index: usize) -> Option<PatternLetter> {
        let letter = self.pattern.get(index)?;
        if letter.is_normalised_wildcard() {
            Some(PatternLetter::Wildcard)
        } else {
            Some(PatternLetter::Char(*letter))
        }
    }

    fn is_allowed_letter(&self, ch: char) -> bool {
        !self.invalid_letters.contains(&ch)
    }

    fn into_pattern(self: Box<Self>) -> Pattern {
        *self
    }

    fn solve(
        &self,
        language: Language,
        max_words_to_collect: Option<usize>,
    ) -> HangmanResult {
        Self::solve(self, language, max_words_to_collect)
    }

    fn known_letters_count(&self) -> usize {
        self.known_letters_count
    }
}

impl PatternTrait for AsciiPattern {
    fn length(&self) -> usize {
        self.pattern.len()
    }

    fn get_letter(&self, index: usize) -> Option<PatternLetter> {
        let letter = self.pattern.get(index)?;
        if letter.is_normalised_wildcard() {
            Some(PatternLetter::Wildcard)
        } else {
            Some(PatternLetter::Char(char::from(*letter)))
        }
    }

    fn is_allowed_letter(&self, ch: char) -> bool {
        ch.is_ascii() && !self.invalid_letters.contains(&(ch as u8))
    }

    fn into_pattern(self: Box<Self>) -> Pattern {
        Pattern {
            invalid_letters: self
                .invalid_letters
                .into_iter()
                .map(CharUtils::to_char)
                .collect(),
            pattern: self.pattern.into_iter().map(CharUtils::to_char).collect(),
            first_letter: self.first_letter.to_char(),
            letters_in_pattern_have_no_other_occurrences: self
                .letters_in_pattern_have_no_other_occurrences,
            known_letters_count: self.known_letters_count,
        }
    }

    fn solve(
        &self,
        language: Language,
        max_words_to_collect: Option<usize>,
    ) -> HangmanResult {
        Self::solve(self, language, max_words_to_collect)
    }

    fn known_letters_count(&self) -> usize {
        self.known_letters_count
    }
}

#[cfg(test)]
mod test {
    use crate::{Language, solver::pattern_trait::PatternLetter};

    use super::compile_pattern;

    #[test]
    pub fn test_compile_pattern() {
        let pattern = compile_pattern("_____r_ü_", &['i'], true);

        assert_eq!(pattern.get_letter(0), Some(PatternLetter::Wildcard));
        assert_eq!(pattern.get_letter(5), Some(PatternLetter::Char('r')));
        assert_eq!(pattern.length(), 9);
        assert_eq!(pattern.known_letters_count(), 2);
        assert!(!pattern.is_allowed_letter('i'));
        assert!(!pattern.is_allowed_letter('r'));
        assert!(pattern.is_allowed_letter('a'));

        let hr = pattern.solve(Language::DeBasicUmlauts, None);

        assert_eq!(hr.language, Language::DeBasicUmlauts);
        assert_eq!(&hr.input, "_____r_ü_");
        assert_eq!(hr.invalid, vec!['i']);
        assert_eq!(hr.matching_words_count, 1);
        assert_eq!(hr.possible_words, vec!["zuckersüß"]);

        let pattern = pattern.into_pattern();
        assert_eq!(pattern.invalid_letters, vec!['r', 'ü', 'i']);
    }

    #[test]
    #[cfg(not(debug_assertions))] // TODO: fix stack over flow
    pub fn test_compile_pattern_ascii() {
        let pattern = compile_pattern("______n_s__r_____n", &['e'], true);

        assert_eq!(pattern.get_letter(0), Some(PatternLetter::Wildcard));
        assert_eq!(pattern.get_letter(6), Some(PatternLetter::Char('n')));
        assert_eq!(pattern.length(), 18);
        assert_eq!(pattern.known_letters_count(), 4);
        assert!(!pattern.is_allowed_letter('e'));
        assert!(!pattern.is_allowed_letter('n'));
        assert!(pattern.is_allowed_letter('a'));

        let hr = pattern.solve(Language::DeUmlauts, None);

        assert_eq!(hr.language, Language::DeUmlauts);
        assert_eq!(&hr.input, "______n_s__r_____n");
        assert_eq!(hr.invalid, vec!['e']);
        assert_eq!(hr.matching_words_count, 1);
        assert_eq!(hr.possible_words, vec!["zwillingsparadoxon"]);
    }
}

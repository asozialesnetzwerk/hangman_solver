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
    fn into_pattern(self) -> Pattern;
    #[must_use]
    fn solve(
        &self,
        language: Language,
        max_words_to_collect: Option<usize>,
    ) -> HangmanResult;
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

    fn into_pattern(self) -> Pattern {
        self
    }

    fn solve(
        &self,
        language: Language,
        max_words_to_collect: Option<usize>,
    ) -> HangmanResult {
        self.solve(language, max_words_to_collect)
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

    fn into_pattern(self) -> Pattern {
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
        }
    }

    fn solve(
        &self,
        language: Language,
        max_words_to_collect: Option<usize>,
    ) -> HangmanResult {
        self.solve(language, max_words_to_collect)
    }
}

from collections.abc import Iterable, Mapping
from typing import ClassVar


__all__ = [
    'solve',
    'read_words_with_length',
    'UnknownLanguageError',
    'HangmanResult',
    'Language',
]


def read_words_with_length(language: Language, word_length: int) -> Iterable[str]:
    pass

def solve(pattern_string: str, invalid_letters: list[str], language: Language) -> HangmanResult:
    pass

class HangmanResult:
    def letter_frequency(self) -> Mapping[str, int]:
        pass

    input: str

    invalid: list[str]

    language: Language

    words: Iterable[str]



class Language:
    def parse_string(self, name: str, default: Language = None) -> Language:
        pass

    @staticmethod
    def values() -> list[Language]:
        pass

    def __eq__(self, *args, **kwargs) -> bool:
        """ Return self==value. """
        pass
    def __ne__(self, *args, **kwargs) -> bool:
        """ Return self!=value. """
        pass

    def __repr__(self) -> str:
        pass

    value: str


    De: ClassVar[Language]
    DeBasic: ClassVar[Language]
    DeBasicUmlauts: ClassVar[Language]
    DeUmlauts: ClassVar[Language]
    En: ClassVar[Language]


class UnknownLanguageError(ValueError):
    pass

from collections.abc import Collection, Sequence, Mapping
from typing import Final, Never


__all__ = (
    "solve",
    "solve_crossword",
    "read_words_with_length",
    "UnknownLanguageError",
    "HangmanResult",
    "Language",
)

class HangmanResult:
    @property
    def input(self, /) -> str: ...
    @property
    def matching_words_count(self, /) -> int: ...
    @property
    def invalid(self, /) -> list[str]: ...
    @property
    def language(self, /) -> Language: ...
    @property
    def words(self, /) -> list[str]: ...
    @property
    def letter_frequency(self, /) -> list[tuple[str, int]]: ...


class Language:
    @staticmethod
    def parse_string(name: str, /, default: Language = None) -> Language:
        pass

    @staticmethod
    def values() -> list[Language]:
        pass

    def __eq__(self, other: Language, /) -> bool:
        pass
    def __ne__(self, other: Language, /) -> bool:
        pass

    def __repr__(self, /) -> str:
        pass

    @property
    def value(self, /) -> str: ...

    De: Final[Language]
    DeBasic: Final[Language]
    DeBasicUmlauts: Final[Language]
    DeUmlauts: Final[Language]
    En: Final[Language]


class UnknownLanguageError(ValueError):
    pass


def read_words_with_length(language: Language, word_length: int, /) -> Collection[str]:
    pass

def solve(
    pattern_string: str,
    invalid_letters: Sequence[str],
    language: Language,
    max_words_to_collect: int
) -> HangmanResult:
    pass

def solve_crossword(
    pattern_string: str,
    invalid_letters: Sequence[str],
    language: Language,
    max_words_to_collect: int
) -> HangmanResult:
    pass

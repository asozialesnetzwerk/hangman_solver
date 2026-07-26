from ._solver import (
    solve,
    solve_crossword,
    read_words_with_length,
    UnknownLanguageError,
    HangmanResult,
    Language,
)

__all__ = (
    "solve",
    "solve_crossword",
    "read_words_with_length",
    "UnknownLanguageError",
    "HangmanResult",
    "Language",
)

from collections.abc import Sequence
try:
    Sequence.register(type(read_words_with_length(Language.De, 67)))
except Exception:
    pass
finally:
    del Sequence

#!/usr/bin/env python3

from collections.abc import Iterable, Collection
import sys
from hangman_solver import solve, HangmanResult, Language

def solve_line(line: str, language: Language) -> HangmanResult:
    invalid_letters: tuple[str, ...]

    match line.split(" "):
        case [pattern, invalid]:
            invalid_letters = tuple(invalid)
        case [pattern]:
            invalid_letters = ()
        case _:
            raise Exception("Expected at most one space")

    return solve(pattern, invalid_letters, language, max_words_to_collect=40)


def join_with_max_length(strings: Collection[str], sep: str, max_len: int) -> str:
    last_index = len(strings) - 1
    string = ""
    for i, item in enumerate(strings):
        current_sep = "" if i == 0 else sep
        min_next_len = 0 if i == last_index else len(sep) + 3
        if len(string) + len(current_sep) + len(item) + min_next_len > max_len:
            string += current_sep + "..."
            break
        string += current_sep + item

    assert len(string) <= max_len
    return string


def print_result(result: HangmanResult, max_width: int) -> None:
    print(f"Found {result.matching_words_count} words (input: {result.input}, invalid: {''.join(result.invalid)})")
    if not result.words:
        return
    words_list =  join_with_max_length(result.words, sep=", ", max_len=max_width - len(" words:   "))
    print(f" words:   {words_list}")

    if not result.letter_frequency:
        return

    letters_list = join_with_max_length([f"{ch}: {f}" for (ch, f) in result.letter_frequency], sep=", ", max_len=max_width - len(" letters: "))
    print(f" letters: {letters_list}")



def main() -> int | str:
    [_prog, language] = sys.argv

    lang = Language.parse_string(language)

    for line in sys.stdin:
        result = solve_line(line.strip(), lang)
        print_result(result, max_width=80)

    return 0


if __name__ == "__main__":
    sys.exit(main())

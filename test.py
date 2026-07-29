#!/usr/bin/env python3

from itertools import permutations
from concurrent.futures import wait
from collections.abc import Sequence

from hangman_solver import Language, read_words_with_length

german_words = read_words_with_length(Language.DeUmlauts, 4)
english_words = read_words_with_length(Language.En, 4)

only_test_de = german_words[german_words.index("test") : german_words.index("test") + 1]
only_test_en = english_words[english_words.index("test") : english_words.index("test") + 1]

assert type(german_words) is type(english_words) is type(only_test_de) is type(only_test_en)

assert "test" in only_test_de
assert "test" in only_test_en
assert repr(only_test_de) == repr(only_test_en) == "['test']"
assert list(only_test_de) == list(only_test_en) == ["test"]

def assert_hashable_sequences_equal(a: Sequence[str], b: Sequence[str]) -> None:
    assert id(a) != id(b)

    assert len(a) == len(b) == sum(1 for _ in a) == sum(1 for _ in b)
    assert hash(a) == hash(b)
    assert a == b
    for i, j in zip(a, b):
        assert i == j
    for i, j in zip(reversed(a), reversed(b)):
        assert i == j


assert_hashable_sequences_equal(only_test_de, only_test_en)
assert_hashable_sequences_equal(german_words[:0], english_words[:0])

empty_sequence = german_words[:0]
assert repr(empty_sequence) == "[]"
assert not empty_sequence
assert len(empty_sequence) == 0

sequences = set()

for _ in (1, 2):
    for lang in Language.values():
        for i in range(200 * _):
            words = read_words_with_length(lang, i)
            sequences.add(words)

            for w, word in enumerate(words):
                assert "" not in words
                assert words.count("") == 0
                assert word in words
                assert word not in words[:w]
                assert word in words[w:]
                assert word in words[w:w+1]
                assert list(words[w:w+1]) == [word]
                assert words.count(word) == 1
                assert words.index(word) == w
                assert words[w] == word
                assert len(word) == i

            if words:
                assert words[0] == next(iter(words))
                assert words[-1] == next(reversed(words)) == words[len(words) - 1]

            assert_hashable_sequences_equal(words, read_words_with_length(lang, i))
            assert_hashable_sequences_equal(empty_sequence, words and words[:0])

    assert len(sequences) == 206

counter = 0
for words in sequences:
    if len(words) > 1000:
        continue
    counter += 1
    for s in [slice(0, 100), slice(-100, None)]:
        words_slice = words[s]
        assert type(words_slice) is type(words)
        assert isinstance(words_slice, Sequence)
        word_list = list(words_slice)

        if len(words) >= 100:
            assert len(word_list) == 100

        for (start, stop, step) in permutations(range(-55, 55), 3):
            if step == 0:
                continue
            _slice = words_slice[start:stop:step]
            assert isinstance(_slice, Sequence)
            if step == 1:
                assert type(_slice) is type(words_slice)
                assert _slice == words_slice[start:stop:step]
            assert word_list[start:stop:step] == list(_slice), f"{word_list[start:stop:step]} != {_slice}; [{start}:{stop}:{step}]"


assert counter == 96, f"{counter} != 96"

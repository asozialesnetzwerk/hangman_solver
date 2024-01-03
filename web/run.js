#!/bin/env -S deno run --allow-net=github.asozial.org,asozial.org,deno.land
import { parse } from "https://deno.land/std@0.207.0/flags/mod.ts";
import { exit } from "https://deno.land/x/exit/mod.ts";
import init, { solve_hangman } from "https://github.asozial.org/hangman_solver/hangman_solver_lib.js";


const flags = parse(Deno.args, {
    boolean: ["crossword"],
    string: ["input", "invalid", "language"],
    number: ["maxwords"],
    default: { crossword: false, maxwords: 10, language: "de_umlauts" },
});

const response = await fetch(
    `https://asozial.org/hangman-loeser/worte/${flags.language}/${flags.input?.length}.txt`,
);
if (!response.ok) {
    console.error(`${response.url} returned ${response.status} ${response.statusText}`);
    exit(1);
}
const words = (await response.text()).split("\n");

await init();

const result = solve_hangman(
    words,
    flags.input,
    flags.invalid,
    flags.maxwords,
    flags.crossword,
);

console.log({
    input: result.input,
    invalid: result.invalid,
    matchingWords: result.matching_words_count,
});

if (result.matching_words_count) {
    console.log(`Letter frequency: ${result.letter_frequency}`);
    console.log(`Words (${result.possible_words.length}/${result.matching_words_count}): ${result.possible_words.join(", ")}`);
} else {
    console.log("Nothing found");
}

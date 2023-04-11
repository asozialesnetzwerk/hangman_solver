#!/bin/sh

CARGO_ARGS="--release --package hangman_solver --bin hangman_solver"

cargo build ${CARGO_ARGS}

run_with_input()
{
  FILE="test_inputs/${1}.txt"
  START=$(date +%s%N)
  cargo run -q ${CARGO_ARGS} < "${FILE}" > /dev/null
  END=$(date +%s%N)
  ELAPSED=$((END-START))
  LINES=$(wc -l "${FILE}"  | cut -d " " -f1)
  echo "$((ELAPSED/1000000))ms (${1}, lines: ${LINES}, per line: $((ELAPSED/LINES/1000000))ms)"
}

run_all()
{
  run_with_input biergarten
  run_with_input ersatzteilplattform
  run_with_input wohnungsbaukaufmann
  run_with_input zweitwohnsitz
  run_with_input zwillingsparadoxon
  run_with_input _
  run_with_input _-e
}

run_all

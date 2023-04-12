#!/bin/sh

CARGO_ARGS="--release --package hangman_solver --bin hangman_solver"

cargo build ${CARGO_ARGS}

run_with_input()
{
  if [ -z "${2}" ] ; then
    OUTPUT_FILE="/dev/null"
  else
    OUTPUT_FILE="${2}"
  fi
  FILE="test_inputs/${1}.txt"
  START=$(date +%s%N)
  cargo run -q ${CARGO_ARGS} < "${FILE}" >> "${OUTPUT_FILE}"
  END=$(date +%s%N)
  ELAPSED=$((END-START))
  LINES=$(wc -l "${FILE}"  | cut -d " " -f1)
  echo "$((ELAPSED/1000000))ms (${1}, lines: ${LINES}, per line: $((ELAPSED/LINES/1000000))ms)"
}

run_all()
{
  TEMP_FILE=$(mktemp /tmp/hangman_solver-benchmark.XXXXXX)
  run_with_input biergarten "${TEMP_FILE}"
  run_with_input ersatzteilplattform "${TEMP_FILE}"
  run_with_input wohnungsbaukaufmann "${TEMP_FILE}"
  run_with_input zweitwohnsitz "${TEMP_FILE}"
  run_with_input zwillingsparadoxon "${TEMP_FILE}"
  run_with_input _ "${TEMP_FILE}"
  run_with_input _-e "${TEMP_FILE}"
  sha256sum "${TEMP_FILE}"
}

run_all
echo "Expected: 99de6a21e8ee1fc326503e504dc8fa5735d62837842d3e4421d3f24ad1d5e7be"
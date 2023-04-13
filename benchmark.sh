#!/bin/sh

TEST_INPUTS_DIR="test_inputs"
CARGO_ARGS="--release --package hangman_solver --bin hangman_solver"

cargo build ${CARGO_ARGS}


run_with_input()
{
  if [ -z "${2}" ] ; then
    OUTPUT_FILE="/dev/null"
  else
    OUTPUT_FILE="${2}/${1}.output"
  fi
  FILE="${TEST_INPUTS_DIR}/${1}.txt"
  START=$(date +%s%N)
  cargo run -q ${CARGO_ARGS} < "${FILE}" > "${OUTPUT_FILE}"
  END=$(date +%s%N)
  ELAPSED=$((END-START))
  LINES=$(wc -l "${FILE}"  | cut -d " " -f1)
  echo "$((ELAPSED/1000000))ms (${1}, lines: ${LINES}, per line: $((ELAPSED/LINES/1000000))ms)"
  if [ -n "${2}" ] ; then
    diff --color=auto "${TEST_INPUTS_DIR}/${1}.output" "${OUTPUT_FILE}"
  fi
}

run_all()
{
  SAVED_HASH=$(cat "${TEST_INPUTS_DIR}"/*output | sha256sum - | cut -d " " -f1)

  TMP_DIR=$(mktemp --directory)
  run_with_input biergarten "${TMP_DIR}"
  run_with_input ersatzteilplattform "${TMP_DIR}"
  run_with_input wohnungsbaukaufmann "${TMP_DIR}"
  run_with_input zweitwohnsitz "${TMP_DIR}"
  run_with_input zwillingsparadoxon "${TMP_DIR}"
  run_with_input _ "${TMP_DIR}"
  run_with_input _-e "${TMP_DIR}"

  OUTPUT_HASH=$(cat "${TMP_DIR}"/*output | sha256sum - | cut -d " " -f1)
  if [ "${OUTPUT_HASH}" = "${SAVED_HASH}" ] ; then
    echo "Hash: ${OUTPUT_HASH}"
  else
    echo "Hash: ${OUTPUT_HASH} != ${SAVED_HASH}"
    exit 1
  fi
}

run_all

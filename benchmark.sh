#!/bin/sh

set -eu

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
    diff --color=auto "${TEST_INPUTS_DIR}/${1}.output" "${OUTPUT_FILE}" >&2
  fi
}

run_all()
{
  SAVED_HASH=$(cat "${TEST_INPUTS_DIR}"/*output | sha256sum - | cut -d " " -f1)

  DIR=${1}

  ls -1 "${TEST_INPUTS_DIR}" | cut -d "." -f1 | sort -firu  | while read -r LINE ; do run_with_input "${LINE}" "${DIR}" ; done

  OUTPUT_HASH=$(cat "${DIR}"/*output | sha256sum - | cut -d " " -f1)
  if [ "${OUTPUT_HASH}" = "${SAVED_HASH}" ] ; then
    echo "Hash: ${OUTPUT_HASH}" >&2
  else
    echo "Hash: ${OUTPUT_HASH} != ${SAVED_HASH}"
    return 1
  fi
}

if [ -z "${1:-}" ] ; then
  run_all "$(mktemp -d)" || exit "${?}"
elif [ "--write-out" = "${1}" ] ; then
  run_all "${TEST_INPUTS_DIR}" || echo "Updated output"
else
  echo "Unexpected argument '${1}'" >&2
  exit 2
fi

#!/bin/sh

set -eu

TEST_INPUTS_DIR="test_inputs"
if [ -z "${CARGO_ARGS:-}" ] ; then
    CARGO_ARGS="--release"
fi
CARGO_ARGS="${CARGO_ARGS} --package hangman_solver --bin hangman_solver"
cargo build ${CARGO_ARGS}

run_with_input()
{
  if [ -n "${2:-}" ] ; then
    LANGUAGE="${2}"
  else
    LANGUAGE="de"
  fi
  FILE="${TEST_INPUTS_DIR}/${LANGUAGE}/${1}.txt"
  if [ -n "${HYPERFINE:-}" ] ; then
    cargo run -q ${CARGO_ARGS} "${LANGUAGE}" < "${FILE}"
    return 0
  fi
  START=$(date +%s%N)
  if [ -z "${3:-}" ] ; then
    cargo run -q ${CARGO_ARGS} "${LANGUAGE}" < "${FILE}"
  else
    OUTPUT_FILE="${3}/${1}.output"
    cargo run -q ${CARGO_ARGS} "${LANGUAGE}" < "${FILE}" 2>&1 | tee "${OUTPUT_FILE}" > /dev/null
  fi
  END=$(date +%s%N)
  ELAPSED=$((END-START))
  LINES=$(wc -l "${FILE}"  | cut -d " " -f1)
  echo "$((ELAPSED/1000000))ms (${1}, lines: ${LINES}, per line: $((ELAPSED/LINES/1000000))ms)"
  if [ -n "${3:-}" ] ; then
    git -c pager.diff=cat diff --no-index --color=auto "${TEST_INPUTS_DIR}/${LANGUAGE}/${1}.output" "${OUTPUT_FILE}" >&2
  fi
}

run_all()
{
  LANGUAGE="${1}"
  DIR="${2}/${LANGUAGE}"

  if [ -z "${HYPERFINE:-}" ] ; then
    SAVED_HASH=$(cat "${TEST_INPUTS_DIR}/${LANGUAGE}"/*output | sha256sum - | cut -d " " -f1)
  fi

  mkdir -p "${DIR}"
  ls -1 "${TEST_INPUTS_DIR}/${LANGUAGE}" | cut -d "." -f1 | sort -firu  | while read -r LINE ; do run_with_input "${LINE}" "${LANGUAGE}" "${DIR}" ; done

  if [ -n "${HYPERFINE:-}" ] ; then
    return 0
  fi

  OUTPUT_HASH=$(cat "${DIR}"/*output | sha256sum - | cut -d " " -f1)
  if [ "${OUTPUT_HASH}" = "${SAVED_HASH}" ] ; then
    echo "Hash: ${OUTPUT_HASH}" >&2
  else
    echo "Hash: ${OUTPUT_HASH} != ${SAVED_HASH}"
    return 1
  fi
}

if [ -z "${1:-}" ] ; then
  run_all "de" "$(mktemp -d)" || exit "${?}"
  run_all "de_umlauts" "$(mktemp -d)" || exit "${?}"
  run_all "de_basic" "$(mktemp -d)" || exit "${?}"
  run_all "de_basic_umlauts" "$(mktemp -d)" || exit "${?}"
  run_all "en" "$(mktemp -d)" || exit "${?}"
elif [ "--write-out" = "${1}" ] ; then
  run_all "de" "${TEST_INPUTS_DIR}" || echo "Updated output de"
  run_all "de_umlauts" "${TEST_INPUTS_DIR}" || echo "Updated output de_umlauts"
  run_all "de_basic" "${TEST_INPUTS_DIR}" || echo "Updated output de_basic"
  run_all "de_basic_umlauts" "${TEST_INPUTS_DIR}" || echo "Updated output de_basic_umlauts"
  run_all "en" "${TEST_INPUTS_DIR}" || echo "Updated output en"
else
  run_with_input "${@?}"
fi

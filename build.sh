#!/bin/sh

set -euo pipefail

TMP_DIR="$(mktemp -d)"

podman run --rm -v .:/io ghcr.io/pyo3/maturin:v1.8.7 build --release --compatibility manylinux_2_17 2>&1 | tee "${TMP_DIR}/build.log"
RESULT1="$(tail -n 1 "${TMP_DIR}/build.log")"

podman run --arch arm64 --rm -v .:/io ghcr.io/pyo3/maturin:v1.8.7 build --release --compatibility manylinux_2_17 2>&1 | tee "${TMP_DIR}/build.log"
RESULT2="$(tail -n 1 "${TMP_DIR}/build.log")"

pyproject-build

echo "$RESULT1"
echo "$RESULT2"
rm -fr "$TMP_DIR"

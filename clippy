#!/bin/sh
cargo clippy $1 -- \
    -D warnings \
    -W clippy::pedantic \
    -W clippy::nursery \
    -W clippy::unwrap_used \
    -W clippy::expect_used \
    -A clippy::missing-const-for-fn \
    -A clippy::cast-possible-wrap \
    -A clippy::cast-possible-truncation \
    -A clippy::option-if-let-else

#!/usr/bin/env bash

set -eu

rustfmt src/*.rs
cargo clippy --all -- \
    -W clippy::all \
    -W clippy::complexity \
    -W clippy::correctness \
    -W clippy::nursery \
    -W clippy::pedantic \
    -W clippy::perf \
    -W clippy::suspicious \
    -A clippy::derive_partial_eq_without_eq \
    -D warnings

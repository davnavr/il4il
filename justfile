#!/usr/bin/env -S just --justfile

test:
    cargo test --all --all-features

fmt:
    cargo fmt --all

clippy:
    cargo clippy --all --all-features

check: fmt clippy test

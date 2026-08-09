default: ci

ci: fmt clippy test

fmt:
    cargo +nightly fmt --check

clippy:
    cargo clippy --all-features -- -D warnings

test:
    cargo test --all-features --workspace
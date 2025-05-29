list:
    just --list

test:
    cargo test-all-features --workspace --all-targets
    cargo test --doc --all-features

bench *ARGS:
    cargo bench --all-features {{ARGS}}

# Publish all crates of this library
publish:
    cd stecs-derive && cargo publish
    cargo publish

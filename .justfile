list:
    just --list

# Publish all crates of this library
publish:
    cd stecs-derive && cargo publish
    cargo publish

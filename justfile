# https://just.systems

default:
    echo 'Hello, world!'

unify:
    cargo rail unify --check

unify-explain:
    cargo rail unify --check --explain

check:
    cargo check --all

clippy:
    cargo clippy --all

test:
    cargo test --all

release:
    cargo b --release

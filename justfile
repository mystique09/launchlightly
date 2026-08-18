# https://just.systems

set dotenv-load := true

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
    topcoat asset bundle --bin launchlight-server --release

db-up:
    docker compose up -d postgres

db-down:
    docker compose down

db-migrate:
    cargo run -p launchlightly-infra-postgresql --bin migrate

db-seed:
    cargo run -p launchlightly-infra-postgresql --bin seed

run:
    topcoat asset bundle --bin launchlight-server
    cargo run -p launchlight-server

dev:
    topcoat dev --bin launchlight-server

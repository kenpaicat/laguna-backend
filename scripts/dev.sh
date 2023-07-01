#!/usr/bin/env bash

DATABASE_URL=postgres://postgres:postgres@127.0.0.1:5432/laguna_db RUST_BACKTRACE=full RUST_LOG=debug HOST=127.0.0.1 PORT=8080 cargo watch -x run

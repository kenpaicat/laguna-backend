# Table of contents

1. [Requirements](#requirements)
2. [Setup](#setup)
3. [Project structure](#project-structure)
4. [Testing](#testing)
5. [Generating documentation](#generating-documentation)
6. [Fixing warnings](#fixing-warnings)
7. [Migrations and model changes](#migrations-and-model-changes)
8. [Running](#running)

# Requirements

1. Rust (https://www.rust-lang.org/tools/install)
2. Postgres (https://www.postgresql.org/download/)

It is recommended to run Linux (either WSL or VM if you are not on Linux) for development.
This is because `scripts/*` are written in bash and because backend will be deployed on linux.
In the future we will add powershell scripts for Windows.

# Setup

1. Clone this repo and `cd` into it.
2. Run `cargo install sqlx-cli`.
3. Run `cargo install cargo-watch`.
4. Make sure Postgres daemon is running, then do `scripts/dbsetup.sh laguna_db` to create `laguna_db` local DB with tables.
5. [Running](#running) the server.

# Project structure

> **Note**
> Only important files and dirs are listed here.

- `.github/` contains GitHub Actions definitions for CI/CD.
  - `dependabot.yml` contains config for automatic dependency updates.
  - `workflows/` contains CI/CD workflows.
    - `rust.yml` contains CI workflow for Rust.
- `.cargo/config.toml` contains GLOBAL project config for Cargo and Rust. This is because we have a [Cargo Workspace](https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html) and its easier to have global config.
- `/` root directory contains root Cargo Crate `laguna-backend` and definition of [Cargo Workspace](https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html).
- `src/main.rs` is server entry point.
- `crates/` contains Cargo Workspace members (sub-Crates) of the project.
  - `laguna-backend-internal/` is a crate that contains re-exports of all other `crates/*` and is used by `laguna-backend` (root crate) to access all other crates.
    - `laguna-backend-internal/src/lib.rs` re-exporting can be seen here.
  - `laguna-backend-api/` contains API endpoints and DTOs (data-transfer-objects) used by [laguna-frontend](https://github.com/SloveniaEngineering/laguna-frontend).
  - `laguna-backend-model/` contains DB models and relations.
  - `laguna-backend-middleware/` contains application logic from API to DB.
- `migrations/` contains SQL migrations for DB.
- `scripts/` contains Bash scripts for development, testing and deploy.

Each Cargo Crate has its own structure defined by Rust.
Generally they contain `src/` directory with _source code and unit tests_.
And also `tests/` directory with _integration tests_.

Crate structure in Rust: https://doc.rust-lang.org/cargo/guide/project-layout.html.

# Testing

1. Run `scripts/test.sh` to run all tests in release mode.

> **Warning**
> all tests use same test DB, so they should not be run in parallel hence `--tests-threads=1`.

In the future we will fix this by using different test DBs for each batch of tests.

# Generating documentation

1. Run `scripts/doc.sh` to generate useful documentation.

How to write doc comments in Rust: https://doc.rust-lang.org/rustdoc/how-to-write-documentation.html.

# Fixing warnings and checking for errors

1. Run `scripts/fix.sh` to fix most warnings automatically.

or if you want to check for errors in external crates:

1. `cargo check`

# Migrations and model changes

Here is a scenario.

1. You change a model in `laguna-backend-model` crate.
2. You also change the corresponding migration in `migrations` crate.
3. Your migration is now out of sync with DB.
4. Run `scripts/dbreset.sh laguna_db` which drops current `laguna_db`, recreates new `laguna_db` and runs all migrations.

Here is another way to do it (without dropping DB):

1. You change a model in `laguna-backend-model` crate.
2. You create a new migration in `migrations` crate using `sqlx migrate add <migration_name>` which contains some `ALTER` statements which probably have `DEFAULT`s.
3. You run `sqlx migrate run` to run all migrations.

It is also possible to create "reversible" migrations with `sqlx migrate add -r <migration_name>`
which will create an `up` and `down` migration files and can be reverted with `sqlx migrate revert`.

# Running

1. Run `scripts/dev.sh` to run the server in development mode. You can override the following environment variables:
   1. `DATABASE_URL` - URL of the database to connect to. Defaults to `postgres://postgres:postgres@localhost:5432/laguna_db`.
   2. `RUST_LOG` - Logging level. Defaults to `info`.
   3. `RUST_BACKTRACE` - Backtrace level. Defaults to `full`.
   4. `HOST` - Logging style. Defaults to `127.0.0.1` or `localhost`.
   5. `PORT` - Port to listen on. Defaults to `8080`.

For example, if your database is on a different host or port, or maybe it has a different name, you can override `DATABASE_URL` env var to point at your DB.
This is however not recommended.

#### On Linux:

```bash
DATABASE_URL=postgres://user:password@localhost:37201/my_db_name scripts/dev.sh
```

#### On Windows:

> **Warning**
> Not implemented yet.

```bash
$env:DATABASE_URL="postgres://user:password@localhost:37201/my_db_name"; scripts/dev-win.ps1
```
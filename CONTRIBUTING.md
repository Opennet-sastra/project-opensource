# Contributing to opennet

## Workflow

1. Fork the repository and create your feature branch from `main`.
2. Keep pull requests focused on a single change.
3. Pull requests are squash-merged into `main`.

## Commit Messages

Use conventional commit style:
- `feat: add marks breakdown screen`
- `fix: handle empty captcha response`
- `docs: update build guide`
- `refactor: simplify attendance parsing`

## Layer Dependency Rules

Never break these architectural rules:
- `scraper` and `storage` depend on `domain` only, never on each other.
- `app` depends on `storage` and `sync` only, never on `scraper`.
- `domain` contains no I/O and no heavy external dependencies.
- `sync` is the only crate that wires concrete portal sources into `storage`.

## Pre-commit Verification

Run the verification suite before opening a PR:
```sh
just check
```
Or manually:
```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

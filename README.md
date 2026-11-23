# opennet

An open-source, Android-first student portal client built in Rust and Dioxus.

## Status

Alpha / MVP — core architecture and login/portal scrapers established across a multi-crate workspace.

## Architecture

The project strictly follows a layered dependency architecture:

```
[app (UI)]
   │     ╲
   ▼      ▼
[storage] ◄── [sync]
   │          ╱  │  ╲
   │  ┌──────┘   │   └──────┐
   ▼  ▼          ▼          ▼
[domain]     [scraper]  [captcha]
```

- **`domain`**: Core domain types, traits, and error definitions. No I/O or heavy dependencies.
- **`scraper`**: Network client and HTML parsing for portal endpoints. Depends on `domain` only.
- **`captcha`**: OCR solving and captcha image handling.
- **`storage`**: Thread-safe in-memory session and cached data store. Depends on `domain` only.
- **`sync`**: Coordinator wiring scraper, captcha, and storage into a unified service.
- **`app`**: Cross-platform Dioxus frontend. Depends only on `storage` and `sync`.

## How to Build

### Prerequisites
- Pinned Rust stable toolchain (`rust-toolchain.toml`)
- Dioxus CLI (`dx`) for Android packaging
- Android NDK (r26 or r27)

### Commands
- Check all crates: `cargo check --workspace`
- Lint: `cargo clippy --workspace`
- Test: `cargo test --workspace`
- Build Android APK: `cd crates/app && dx build --android` (or `just apk`)

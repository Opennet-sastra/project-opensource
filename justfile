default: check

# Run formatting check, clippy with warnings denied, and tests across all workspace crates
check:
    cargo fmt --all --check
    cargo clippy --workspace --all-targets -- -D warnings
    cargo test --workspace

# Build debug Android APK for the app
apk:
    cd crates/app && dx build --android

# Build Windows Desktop .exe (optimized release)
desktop:
    cargo build -p app --release

# Run Windows Desktop app locally
run:
    cargo run -p app

# Check dependencies with cargo-deny
deny:
    cargo deny check

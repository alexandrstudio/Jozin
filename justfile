# Build alles
build:
    cargo build --workspace

# Nur CLI (schnell iterieren)
cli:
    cargo run -p jozin -- Scan ./Photos --dry-run

# Release Binaries (mac, linux je nach Runner)
release:
    cargo build --workspace --release

# App (Tauri)
app-dev:
    cd app && npm i && npx tauri dev

# Tests
test:
    cargo test --workspace

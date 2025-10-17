# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**Jožin** is a privacy-focused, local-first photo organizer written in **Rust** with a **Tauri + React** desktop interface. It scans local photo directories, extracts EXIF metadata, computes BLAKE3 hashes, detects duplicates and faces, and stores all derived information in **JSON sidecar files** adjacent to the original photos. All processing happens locally—no cloud uploads, no external APIs, complete user control.

**Core Philosophy:**
- Immutable originals (read-only, never modified)
- Local-first design (100% offline capable)
- Schema-driven metadata (versioned JSON sidecars)
- Modular monolith (single Rust binary, no microservices)

## Architecture

### Workspace Structure

This is a **Cargo workspace** with three members:

```
jozin/
├─ core/              # Rust library (jozin-core) - all photo processing logic
│  └─ src/
│     ├─ lib.rs       # Module exports & core API
│     ├─ scan.rs      # Directory walking, EXIF reading, hash computation
│     ├─ cleanup.rs   # Remove Jožin-generated files
│     ├─ verify.rs    # Sidecar validation & staleness detection
│     ├─ migrate.rs   # Schema version migrations
│     ├─ faces.rs     # Face detection (feature-gated)
│     ├─ tags.rs      # ML/rule-based tagging (feature-gated)
│     └─ thumbs.rs    # Thumbnail generation (feature-gated)
├─ cli/               # CLI binary (jozin) - thin wrapper around core
│  └─ src/main.rs
└─ app/               # Tauri + React desktop app
   └─ src-tauri/      # Tauri backend (bridges core to JS)
      └─ src/
```

### Core Modules

The `jozin-core` library is structured into **7 core modules**:

1. **scan** - Directory traversal, EXIF extraction, BLAKE3 hashing, sidecar generation
2. **cleanup** - Remove Jožin-generated files (sidecars, thumbnails, backups, cache)
3. **verify** - Validates sidecar integrity, schema versions, detects staleness
4. **migrate** - Handles schema version upgrades with backup rotation
5. **faces** - Face detection & identification (optional feature)
6. **tags** - ML-based and rule-based automatic tagging (optional feature)
7. **thumbs** - Multi-size thumbnail generation (optional feature)

All modules share a common **parameter structure** and return machine-readable JSON.

### Key Data Structures

- **Sidecar** - JSON file stored adjacent to each photo (e.g., `IMG_1234.JPG.json`)
- **PipelineSignature** - Tracks schema version, producer version, and algorithms used
- **api module** - Public-facing functions (`scan_path`, `verify_path`, `rescan_reason`)

## Common Development Commands

### Building

```bash
# Build entire workspace
just build
# or
cargo build --workspace

# Build release binaries
just release
# or
cargo build --workspace --release
```

### Running the CLI

```bash
# Quick CLI test with dry-run
just cli
# or
cargo run -p jozin -- scan ./Photos --dry-run

# Run specific commands
cargo run -p jozin -- scan ~/Pictures --dry-run
cargo run -p jozin -- cleanup ~/Pictures --only-sidecars --dry-run
cargo run -p jozin -- verify ~/Pictures
cargo run -p jozin -- migrate ~/Pictures --to v2
```

### Running the Desktop App

```bash
just app-dev
# or
cd app && npm install && npx tauri dev
```

### Testing

```bash
just test
# or
cargo test --workspace
```

## CLI Command Structure

All CLI commands follow this pattern:

```
jozin <module> [options]
```

### Common Parameters Across All Modules

- `--dry-run` - Print intended actions without writing files
- `--recursive` - Process directories recursively
- `--include <globs>` - File patterns to include (e.g., `*.jpg,*.png`)
- `--exclude <globs>` - File patterns to exclude (e.g., `**/.jozin/**`)
- `--max-threads <N>` - Bounded parallelism (default: min(2×CPU, 8))

### Module-Specific Parameters

**scan:**
- `--hash-mode <file|pixel|both>` - Hash computation strategy (Phase 2+)

**cleanup:**
- `--only-sidecars` - Remove only JSON sidecar files
- `--only-thumbnails` - Remove only thumbnail files
- `--only-backups` - Remove only backup files (*.bak1/2/3)
- `--only-cache` - Remove only cache directories (.jozin/*)

**faces:**
- `--model <name>` - Face detection model (e.g., `arcface-1.4`)
- `--identify` - Match embeddings to known persons
- `--train <json>` - Train on labeled faces
- `--min-score <f32>` - Confidence threshold (default: 0.8)

**tags:**
- `--mode <ml|rules|both>` - Tagging strategy
- `--model <name>` - ML model to use
- `--min-score <f32>` - Confidence threshold (default: 0.6)
- `--append` - Keep existing user labels

**thumbs:**
- `--sizes <list>` - Comma-separated sizes (e.g., `256,512`)
- `--format <jpg|webp>` - Output format
- `--quality <1-100>` - Compression quality
- `--overwrite` - Replace existing thumbnails

**verify:**
- `--fix` - Attempt auto-repair of minor issues
- `--strict` - Treat warnings as errors
- `--pipeline-signature <sig>` - Override pipeline signature

**migrate:**
- `--from <version>` - Source schema version (auto-detect if omitted)
- `--to <version>` - Target schema version
- `--backup` - Create `.bakN` backup files

## Exit Codes

- `0` - Success
- `1` - User error (invalid arguments, missing files)
- `2` - I/O error
- `3` - Validation error
- `4` - Internal error

## File Layout & Sidecar Policy

Jožin stores metadata **adjacent to original files** (no hidden `.jozin/` trees):

```
/photos/
├─ 2020/
│  ├─ IMG_1234.JPG
│  ├─ IMG_1234.JPG.json        # Sidecar with scan + tags + faces
│  ├─ IMG_1234_256.jpg         # Optional thumbnails
│  └─ IMG_1234_512.webp
└─ jozin.journal.ndjson         # Optional operation log (Phase 2)
```

### Sidecar Writing Strategy

- Atomic writes: `.tmp` → `fsync` → `rename`
- Backup rotation: `.bak1`, `.bak2`, `.bak3`
- Never modify originals

### Cache & Temporary Files

- Cache location: OS temp directory (e.g., `/tmp/jozin-*`)
- Auto-cleaned before and after each run
- Models stored in app data dir: `~/Library/Application Support/Jožin/models` (macOS)

## Development Phases

The project follows a **phased plan** (see `TASK+PHASE_PLAN.md`):

- **Phase 0** - Parameter parsing & CLI wiring (help text, validation, tests)
- **Phase 1** - Minimal functional core (each module implements basic operations)
- **Phase 2** - Robustness & performance (parallelism, journaling, progress API)

### Current State

The codebase is in **Phase 1**:
- Workspace structure is set up
- CLI fully implements `scan` and `cleanup` subcommands
- `verify` and `migrate` subcommands are stubs (return parameters as JSON)
- Phase 2+ modules (`faces`, `tags`, `thumbs`) are feature-gated stubs
- All core tests passing (58 tests: 23 CLI + 24 core + 11 doc)
- Zero compiler warnings
- Production-ready build

## Key Technical Decisions

### Why Rust?
- Performance & safety for large photo libraries
- Single binary, no runtime dependencies
- Perfect fit for Tauri (same language on both sides)

### Why Tauri + React?
- Native desktop experience (no Docker, no terminal required)
- Direct Rust integration (no HTTP/IPC overhead)
- Cross-platform (macOS, Windows, Linux)

### Why JSON Sidecars?
- Human-readable, easily inspectable
- Schema versioning with clean migrations
- Reversible (can regenerate from originals)

### Why BLAKE3?
- Ultra-fast parallel hashing
- Cryptographically secure
- Superior to SHA256 and MD5 for file deduplication

## Testing Strategy

All modules must handle:
- **Paths:** Deep trees, Unicode, spaces, symlinks, network mounts
- **Files:** Large JPEG/PNG/HEIC/RAW, corrupt headers, missing EXIF
- **OS:** macOS, Windows, Linux (CRLF, permissions)
- **Concurrency:** `--max-threads 1` vs many, no starvation
- **Determinism:** Same inputs → same outputs (fixed seeds for ML)

## AI-Assisted Development Context

This project is **intentionally built using AI coding assistants** (GPT, Claude, Gemini) as a teaching example. Development follows a **spec-first → code → test** workflow. When implementing new features:

1. Check `SCOPE.md` for architectural constraints
2. Consult `TASK+PHASE_PLAN.md` for module parameters & acceptance criteria
3. Maintain deterministic, testable outputs
4. Use structured JSON for all CLI output

## Privacy & Security Principles

- Never modify original files
- No telemetry, analytics, or network calls
- All ML models run locally (ONNX)
- Face embeddings can be hashed/salted
- JSON sidecars contain only user-approved metadata

## Important Notes

- The project is **proprietary software** (© 2025 5 Leaves s.r.o.)
- Use `just` task runner for common operations (see `justfile`)
- Core library uses **Cargo features** for optional modules (`faces`, `tags`, `thumbs`)
- All operations must return timing data (`started_at`, `finished_at`, `duration_ms`)

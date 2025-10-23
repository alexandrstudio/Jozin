# Jožin — Local-First Photo Organizer

> **The friendly monster that organizes your photo swamp.**
>
> Local-first. AI-assisted. 100% private.

**Jožin** is a privacy-focused photo organizer written in **Rust** with a **Tauri + React** desktop interface. It scans local photo directories, extracts EXIF metadata, computes BLAKE3 hashes, detects duplicates and faces, and stores all derived information in **JSON sidecar files** adjacent to the original photos. All processing happens locally—no cloud uploads, no external APIs, complete user control.

---

## 🚀 Quick Start (Developers)

```bash
# Clone and build
git clone <repo-url>
cd jozin
cargo build --workspace

# Run CLI tests
cargo run -p jozin -- scan ./Photos --dry-run
cargo run -p jozin -- cleanup ./Photos --only-sidecars --dry-run

# Run all tests
cargo test --workspace

# Build release binaries
cargo build --workspace --release
```

**Current Build Status:**
- ✅ `cargo build --workspace` - Zero warnings
- ✅ `cargo test --workspace` - 59 tests passing
- ✅ Production-ready CLI binary

---

## 📊 Current Development Status

**Phase:** Phase 1 - Near Completion (see [TASKMASTER_PLAN.md](TASKMASTER_PLAN.md) for details)

### ✅ Fully Implemented

| Module | Status | Features |
|--------|--------|----------|
| **scan** | ✅ Complete | Directory traversal, EXIF extraction, BLAKE3 hashing, sidecar generation |
| **cleanup** | ✅ Complete | Selective cleanup (sidecars, thumbnails, backups, cache) |
| **CLI** | ✅ Complete | Full parameter validation, help text, structured JSON output |
| **Core Infrastructure** | ✅ Complete | Error types, timing metadata, pipeline signatures |

### ⚠️ In Progress (Phase 1)

| Module | Status | Next Steps |
|--------|--------|------------|
| **verify** | 10-line stub | Task 1-2: Implement sidecar validation, staleness detection |
| **migrate** | 10-line stub | Task 3-4: Implement schema migrations with backup rotation |

### 🔒 Planned (Phase 2+)

| Module | Status | Features |
|--------|--------|----------|
| **faces** | Feature-gated stub | Face detection and identification (local ONNX models) |
| **tags** | Feature-gated stub | ML-based and rule-based automatic tagging |
| **thumbs** | Feature-gated stub | Multi-size thumbnail generation |
| **Tauri App** | Basic structure | Desktop UI with React (Tasks 6-7) |
| **Parallelism** | Not started | Bounded thread pools, progress events API |
| **Journaling** | Not started | Resumable scans, operation history |

**See [TASKMASTER_PLAN.md](TASKMASTER_PLAN.md) for detailed task breakdown (Tasks 1-7).**

---

## 🏗️ Architecture

Jožin is a **Cargo workspace** with three members:

```
jozin/
├─ core/              # Rust library (jozin-core) - all photo processing logic
│  └─ src/
│     ├─ lib.rs       # Module exports & core API
│     ├─ scan.rs      # ✅ Directory walking, EXIF, hashing
│     ├─ cleanup.rs   # ✅ Selective cleanup
│     ├─ verify.rs    # ⚠️ Sidecar validation (stub)
│     ├─ migrate.rs   # ⚠️ Schema migrations (stub)
│     ├─ faces.rs     # 🔒 Face detection (feature-gated)
│     ├─ tags.rs      # 🔒 ML tagging (feature-gated)
│     └─ thumbs.rs    # 🔒 Thumbnails (feature-gated)
├─ cli/               # CLI binary (jozin) - thin wrapper around core
│  └─ src/main.rs     # ✅ Full implementation
└─ app/               # Tauri + React desktop app
   └─ src-tauri/      # 🔒 Basic structure (not implemented)
      └─ src/
```

### Core Principles

- **Immutable originals** - Photos are read-only, never modified
- **Local-first design** - 100% offline capable, no telemetry
- **Schema-driven metadata** - Versioned JSON sidecars with migrations
- **Modular monolith** - Single Rust binary, no microservices

---

## 📦 Module Overview

### scan - Photo Scanning & Metadata Extraction

**Status:** ✅ Fully Implemented (core/src/scan.rs)

**Features:**
- Directory traversal (recursive or non-recursive)
- Glob pattern filtering (include/exclude)
- EXIF metadata extraction (camera, GPS, timestamp)
- BLAKE3 file hashing (ultra-fast parallel hashing)
- JSON sidecar generation (atomic writes with fsync)
- Dry-run mode (preview without writing files)

**CLI Usage:**
```bash
jozin scan ~/Photos --recursive --dry-run
jozin scan ~/Photos --include "*.jpg,*.png" --exclude "**/tmp/**"
```

**Output:** JSON with timing metadata and per-file results

---

### cleanup - Remove Generated Files

**Status:** ✅ Fully Implemented (core/src/cleanup.rs)

**Features:**
- Selective cleanup modes (sidecars, thumbnails, backups, cache)
- Dry-run mode (preview deletions)
- Recursive directory processing
- Glob pattern filtering
- Safe deletion (only removes Jožin-generated files)

**CLI Usage:**
```bash
jozin cleanup ~/Photos --only-sidecars --dry-run
jozin cleanup ~/Photos --only-thumbnails --recursive
jozin cleanup ~/Photos --recursive  # Remove all generated files
```

**Output:** JSON with deleted file paths and timing metadata

---

### verify - Sidecar Validation

**Status:** ⚠️ Minimal Stub (core/src/verify.rs) - **Task 1-2 in TASKMASTER_PLAN.md**

**Planned Features:**
- Sidecar existence check (detect missing `.json` files)
- JSON parsing and validation (handle corrupt/malformed JSON)
- Schema version compatibility check
- Hash staleness detection (file modified after scan)
- Action recommendations (noop/rescan/migrate)
- Fix mode (auto-repair minor issues)

**Planned CLI Usage:**
```bash
jozin verify ~/Photos --recursive
jozin verify ~/Photos --fix --strict
```

**Planned Output:** JSON with validation status (ok/stale/missing/corrupt) and suggested actions

---

### migrate - Schema Version Upgrades

**Status:** ⚠️ Minimal Stub (core/src/migrate.rs) - **Task 3-4 in TASKMASTER_PLAN.md**

**Planned Features:**
- Auto-detect source schema version
- Version upgrade transformations (e.g., split camera field)
- Backup rotation (`.bak1`, `.bak2`, `.bak3`)
- Dry-run mode (preview changes)
- Idempotent migrations (safe to run multiple times)
- Atomic writes (`.tmp` → fsync → rename)

**Planned CLI Usage:**
```bash
jozin migrate ~/Photos --to v2 --dry-run
jozin migrate ~/Photos --from v1 --to v2 --backup
```

**Planned Output:** JSON with migrated paths, version changes, backup locations

---

### faces - Face Detection & Identification

**Status:** 🔒 Feature-Gated Stub (core/src/faces.rs) - **Phase 2+**

**Planned Features:**
- Local ONNX model execution (no cloud API calls)
- Face detection in photos
- Face embedding generation (for identification)
- Person identification (match to known faces)
- Training on labeled faces
- Confidence scoring and thresholds

**Planned CLI Usage:**
```bash
jozin faces ~/Photos --recursive --min-score 0.8
jozin faces ~/Photos --identify --train faces.json
```

---

### tags - Automatic Tagging

**Status:** 🔒 Feature-Gated Stub (core/src/tags.rs) - **Phase 2+**

**Planned Features:**
- ML-based keyword detection (local models)
- Rule-based tagging (EXIF context, filename patterns)
- Tag confidence scoring
- Append mode (keep existing user labels)
- Batch processing with progress events

**Planned CLI Usage:**
```bash
jozin tags ~/Photos --mode ml --min-score 0.6
jozin tags ~/Photos --mode rules --append
```

---

### thumbs - Thumbnail Generation

**Status:** 🔒 Feature-Gated Stub (core/src/thumbs.rs) - **Phase 2+**

**Planned Features:**
- Multi-size thumbnail generation (256, 512, 1024, etc.)
- Multiple output formats (JPEG, WebP)
- Quality control (compression settings)
- Overwrite mode (replace existing thumbnails)
- Optimized for batch processing

**Planned CLI Usage:**
```bash
jozin thumbs ~/Photos --sizes 256,512 --format webp
jozin thumbs ~/Photos --quality 85 --overwrite
```

---

## 💻 Development Setup

### Prerequisites

- **Rust** ≥ 1.75 ([rustup.rs](https://rustup.rs))
- **Node.js** ≥ 20 (for Tauri app)
- **just** task runner (optional): `cargo install just`

### Build & Test

```bash
# Build all workspace members
cargo build --workspace

# Build with release optimizations
cargo build --workspace --release

# Run all tests (59 tests)
cargo test --workspace

# Run specific module tests
cargo test --package jozin-core scan
cargo test --package jozin cleanup

# Check for linting issues
cargo clippy --workspace

# Build documentation
cargo doc --workspace --no-deps --open
```

### Using the CLI

```bash
# Run from source (debug build)
cargo run -p jozin -- --help
cargo run -p jozin -- scan ~/Photos --dry-run

# Build and install release binary
cargo build --workspace --release
sudo cp target/release/jozin /usr/local/bin/
jozin --version

# Using just (shortcut)
just cli  # Runs: cargo run -p jozin -- scan ./Photos --dry-run
```

### Common Development Commands (just)

```bash
just build        # Build workspace
just release      # Build release binaries
just test         # Run all tests
just cli          # Quick CLI test (scan --dry-run)
just app-dev      # Launch Tauri app in dev mode
```

See `justfile` for all available commands.

---

## 🖥️ Tauri App Development

**Status:** Basic structure exists, not yet implemented (Task 6)

### Setup

```bash
cd app
npm install
npm run tauri dev
```

### Planned Features

- Folder picker (drag-and-drop or button)
- Real-time progress during operations
- JSON result display (pretty-printed)
- Tabs for Scan / Verify / Cleanup / Tags / Faces
- Dark mode support
- Native desktop integration (macOS, Windows, Linux)

**See Task 6 in [TASKMASTER_PLAN.md](TASKMASTER_PLAN.md) for implementation plan.**

---

## 🧪 Testing Strategy

### Test Coverage (59 tests passing)

- **23 CLI tests** - Parameter validation, help text, exit codes
- **24 Core tests** - Scan, cleanup, core infrastructure
- **12 Doc tests** - Code examples in documentation

### Test Requirements

All modules must handle:
- **Paths:** Deep trees, Unicode, spaces, symlinks
- **Files:** Large JPEG/PNG/HEIC/RAW, corrupt headers, missing EXIF
- **OS:** macOS, Windows, Linux (CRLF, permissions)
- **Concurrency:** Single-threaded vs multi-threaded (no race conditions)
- **Determinism:** Same inputs → same outputs

### Running Tests

```bash
# All tests
cargo test --workspace

# Specific module
cargo test --package jozin-core scan::tests

# Show test output
cargo test --workspace -- --nocapture

# Single test
cargo test --package jozin-core test_scan_directory
```

---

## 📋 File Layout & Sidecar Policy

Jožin stores metadata **adjacent to original files** (no hidden `.jozin/` trees):

```
/photos/
├─ 2020/
│  ├─ IMG_1234.JPG
│  ├─ IMG_1234.JPG.json        # Sidecar with metadata
│  ├─ IMG_1234.JPG.json.bak1   # Backup (after migration)
│  ├─ IMG_1234_256.jpg         # Optional thumbnail
│  └─ IMG_1234_512.webp        # Optional thumbnail
└─ jozin.journal.ndjson         # Optional operation log (Phase 2)
```

### Sidecar Writing Strategy

- **Atomic writes:** `.tmp` → `fsync` → `rename`
- **Backup rotation:** `.bak1`, `.bak2`, `.bak3` (keeps 3 versions)
- **Never modify originals:** Photos are read-only

---

## 🗺️ Roadmap

### Phase 1 - Basic Functionality (Near Complete)

- [x] Workspace structure
- [x] CLI with parameter validation
- [x] scan module (full implementation)
- [x] cleanup module (full implementation)
- [ ] verify module (Tasks 1-2)
- [ ] migrate module (Tasks 3-4)
- [ ] Phase 1 validation (Task 5)

### Phase 2 - Robustness & Performance

- [ ] Bounded parallelism (`--max-threads`)
- [ ] Journal support (resumable scans)
- [ ] Progress events API
- [ ] Perceptual hash (duplicate detection)
- [ ] Pixel hash (cross-format duplicates)

### Phase 2+ - Advanced Features

- [ ] Face detection & identification
- [ ] ML-based tagging
- [ ] Thumbnail generation
- [ ] Tauri desktop UI (Tasks 6-7)

**See [TASKMASTER_PLAN.md](TASKMASTER_PLAN.md) for detailed task breakdown.**

---

## 🛡️ Privacy & Security

- **Originals are read-only** - Never modified
- **No telemetry** - No analytics, no network calls
- **100% local** - All computation on your machine
- **Local ML models** - Face recognition uses ONNX (no cloud APIs)
- **User control** - JSON sidecars contain only metadata you approve

---

## 🔧 Technology Stack

| Layer | Technology | Purpose |
|-------|------------|---------|
| Core | Rust | High-performance, memory-safe processing |
| Hashing | BLAKE3 | Ultra-fast parallel file fingerprinting |
| CLI | Clap | Comprehensive argument parsing |
| UI | Tauri + React | Lightweight cross-platform desktop app |
| ML | ONNX Runtime | Local face and tag recognition (planned) |
| Schema | JSON | Versioned sidecar metadata |
| Tasks | Just / Cargo | Build and test automation |

---

## 🤖 AI-Assisted Development

Jožin is built using **AI coding assistants** (GPT, Claude, Gemini) as a teaching example of **human-AI collaboration** in software engineering.

**Development workflow:**
1. **Spec-first** - Define requirements in markdown (SCOPE.md, TASK+PHASE_PLAN.md)
2. **AI-generated code** - Implement modules with AI assistance
3. **Test-driven validation** - Verify with comprehensive test suites
4. **Iterative refinement** - Review, refine, and iterate

**Key documents:**
- `SCOPE.md` - Architectural constraints and design principles
- `TASK+PHASE_PLAN.md` - Module parameters and acceptance criteria
- `TASKMASTER_PLAN.md` - Current task breakdown (Tasks 1-7)
- `CLAUDE.md` - AI assistant guidance for this codebase

---

## ❤️ Philosophy

> **Data belongs to its creator.**
>
> Jožin exists to help people organize their memories without surrendering them to a cloud.

---

## 📄 Copyright & Ownership

© 2025 **5 Leaves s.r.o.** — All Rights Reserved.

"Jožin" and all related materials are proprietary software and intellectual property of **5 Leaves (5LVES.com)**.

Unauthorized copying, modification, distribution, or disclosure of this software or its documentation, in whole or in part, is strictly prohibited without prior written consent from 5 Leaves s.r.o.

For licensing or partnership inquiries, please contact: **info@5lves.com**

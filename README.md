# Jožin — The Friendly Monster That Organizes Your Photo Swamp 🐊

> **Local-first. AI-assisted. 100 % private.**
>
> Jožin scans your messy photo folders, detects duplicates and faces, creates intelligent JSON sidecars — and never uploads a single byte.

---

## Overview

**Jožin** is a privacy-focused, local photo organizer written in **Rust** with a **Tauri + React** desktop interface.
It brings order to years of scattered photos without touching your originals or requiring any cloud service.

Jožin reads EXIF data, computes fast **BLAKE3** hashes, detects faces and duplicates, and stores structured metadata as **sidecar JSON files**.
Everything runs entirely on your machine — safe, transparent, and reversible.

---

## Key Features

- **Local-first design:** Works entirely offline. No upload, no account, no telemetry.
- **Immutable originals:** Photos are read-only. All metadata lives beside them in JSON sidecars.
- **Fast scanning:** Parallel I/O with Rust and BLAKE3 hashing.
- **Smart organization:** Detects duplicates, near-duplicates, bursts, faces, and series.
- **Modular architecture:** One binary with clear internal modules — easy to extend, no external services.
- **Tauri + React UI:** Clean, native desktop experience for macOS, Windows, and Linux.
- **AI-assisted development:** The project itself is built using modern AI coding assistants (GPT, Claude, Gemini, etc.) as an educational example of human-AI collaboration.

---

## Architecture
```bash
jozin/
├─ core/            # Rust library – scan, cleanup, faces, tags, thumbs, verify, migrate
│  └─ src/
├─ cli/             # CLI binary built on top of the core library
│  └─ src/main.rs
├─ app/             # Tauri + React desktop application
│  ├─ src-tauri/
│  └─ src/
├─ schemas/         # JSON schema definitions for sidecars
├─ justfile         # Dev commands (build, run, test)
└─ README.md
```

### Components

- **Core (Rust Library)** – the heart of Jožin.
  Provides scanning, hashing, cleanup, face detection, tagging, verification, and migration logic.

- **CLI** – a minimal command-line interface built on the core.
  Ideal for automation and testing.

```bash
  jozin scan ~/Photos --dry-run
  jozin cleanup ~/Photos --only-sidecars
  jozin verify ~/Photos
```

•	App (Tauri + React) – the desktop UI, directly embedding the same Rust core.
No Docker, no local server — just a native app with full control and visibility.

### Example Folder Layout

```bash
/photos/
├─ 2020/
│  ├─ IMG_1234.JPG
│  ├─ IMG_1234.JPG.json        # Sidecar with analysis + tags + faces
│  └─ IMG_1234_thumb.jpg       # Optional generated thumbnail
└─ .jozin/
   ├─ _journal.ndjson          # Operation history
   ├─ _cache/                  # Temporary data and hashes
   ├─ _trash/                  # Quarantined or deleted files
   └─ _models/                 # Local ML models for tagging and faces
```

---

## Development Setup

### Prerequisites
- **Rust** ≥ 1.75
- **Node.js** ≥ 20
- **Tauri CLI** (cargo install tauri-cli)
- (Optional) Just task runner (cargo install just)

### Clone & Build
```bash
git clone https://github.com/yourname/jozin.git
cd jozin
cargo build --workspace
```

The build will complete without warnings. All Phase 2+ modules (faces, tags, thumbs) are defined as feature flags with stub implementations ready for future development.

### Run the CLI
```bash
# Quick test
just cli

# Or run directly
cargo run -p jozin -- scan ~/Pictures --dry-run

# Build release binary
cargo build --workspace --release
./target/release/jozin scan ~/Pictures --recursive
```

### Current Implementation Status

**✅ Phase 1 Complete:**
- **scan** module - Directory traversal, BLAKE3 hashing, glob filtering
- **cleanup** module - Remove Jožin-generated files (sidecars, thumbnails, backups, cache)
- **verify** module - Sidecar validation (stub)
- **migrate** module - Schema migration (stub)
- CLI with comprehensive argument parsing
- Unit and integration tests (58 tests passing)

**🔜 Phase 2+ Planned:**
- **faces** module - Face detection and identification
- **tags** module - ML-based and rule-based tagging
- **thumbs** module - Multi-size thumbnail generation

All Phase 2+ modules are declared as Cargo features and can be enabled when implemented:
```bash
# Future: enable face detection
cargo build --features faces

# Future: enable all features
cargo build --all-features
```

### Run the Desktop App
```bash
cd app
npm install
npm run tauri dev
```

**Note:** The Tauri app is in early development. For now, use the CLI for testing and development.

### 🎯 Build Status

  ✅ cargo build --workspace          # Zero warnings
  ✅ cargo build --workspace --release # Zero warnings
  ✅ cargo test --workspace           # 58 tests passing
  ✅ ./target/release/jozin --version # Binary works

### Common CLI Commands

```bash
# Scan photos and generate sidecars
jozin scan ~/Photos --recursive --dry-run
jozin scan ~/Photos --recursive

# Cleanup Jožin-generated files
jozin cleanup ~/Photos --dry-run             # Preview what will be deleted
jozin cleanup ~/Photos --only-sidecars       # Remove only JSON sidecars
jozin cleanup ~/Photos --only-thumbnails -r  # Remove only thumbnails recursively
jozin cleanup ~/Photos --recursive           # Remove all generated files
```

---

## Privacy & Security

-	Originals are read-only and never modified.
-	No analytics, telemetry, or background network connections.
-	All computation happens locally — no data ever leaves your computer.
-	Face recognition runs with local ONNX models; embeddings can be hashed or salted for privacy.
-	JSON sidecars contain only metadata you explicitly approve.

---

## Technology Stack

| Layer | Technology | Purpose |
|-------|------------|---------|
| Core | Rust | High-performance, memory-safe processing |
| UI | Tauri + React | Lightweight, cross-platform desktop app |
| Hash | BLAKE3 | Ultra-fast, parallel file fingerprinting |
| ML | ONNX Runtime (planned) | Local face and tag recognition |
| Schema | JSON Schema | Stable data contracts for sidecars |
| Tasks | Just / Cargo | Build, test, and run automation |

---

## Roadmap

-	Project workspace structure
-	CLI prototype (scan, verify)
-	JSON sidecar v1 schema
-	Tauri desktop UI prototype
-	Face and tag recognition (local models)
-	Thumbnail generation pipeline
-	Sidecar migration + versioning system
-	Backup/export features
-	Public beta release

---

## AI-Assisted Development

Jožin is also an experiment in **AI-augmented software engineering**.
Development steps follow a **spec-first → code → test** workflow guided by multiple AI systems (GPT, Claude, Gemini).
The project serves as both a real application and a teaching example for **how to safely and effectively build software with AI tools**.

---

## ❤️ Philosophy

> **Data belongs to its creator.**
>
> Jožin exists to help people organize their memories without surrendering them to a cloud.

---

## 🛡️ Copyright & Ownership

© 2025 **5 Leaves s.r.o.** — All Rights Reserved.
“Jožin” and all related materials are proprietary software and intellectual property of **5 Leaves (5LVES.com)**.

Unauthorized copying, modification, distribution, or disclosure of this software or its documentation, in whole or in part, is strictly prohibited without prior written consent from 5 Leaves s.r.o.

For licensing or partnership inquiries, please contact:
**info@5lves.com**

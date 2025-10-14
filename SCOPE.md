# Jožin — SCOPE

> **Jožin — the friendly monster that organizes your photo swamp.**

## Purpose (Global)

Jožin organizes large, messy local photo libraries **without ever uploading or altering originals**.
It scans local directories, reads EXIF metadata, fingerprints images, detects duplicates and faces, and stores all derived information in **JSON sidecar files** directly next to the photos.
All processing happens locally — the user keeps full control over both data and computation.
The resulting sidecars enable powerful local search, filtering, tagging, and curation features.

---

## Non-Goals

- No cloud backend or online sync.
- No destructive operations on original files.
- No training or telemetry using user data.
- No dependencies on external servers or APIs.

---

## Core Principles

- **Local-first & reversible:** Everything happens on the user’s machine. Any derived data (JSON, thumbnails, caches) can be rebuilt anytime from the originals.
- **Single process, modular design:** Jožin is one Rust application with well-separated modules for scanning, tagging, face recognition, thumbnails, migration, and verification.
- **Schema-driven:** A single, versioned JSON schema defines the sidecar structure. It guarantees forward compatibility and clean migrations.
- **Performance & safety:** Rust, parallel I/O, predictable memory bounds, atomic file writes, resumable scans.
- **Privacy & transparency:** No data leaves the computer. Models for face or tag detection are stored and executed locally. The user decides when and what to rescan, update, or delete.
- **Minimal footprint:** No hidden folders, no global caches, no external state — everything is explainable and user-visible.
- **AI-assisted development:** The project itself is built using modern AI coding assistants (GPT, Claude, Gemini, etc.) to demonstrate collaborative, iterative software creation.

---

## Architecture (High-Level Overview)

### 1. Jožin Core (Rust Library + CLI)
A self-contained Rust crate providing all photo analysis and file operations.

- **Modules:**
  - `scan`: walks directories, reads EXIF, computes BLAKE3 hashes, writes JSON sidecars.
  - `faces`: detects and identifies faces using local ONNX models or custom embeddings.
  - `tags`: proposes tags based on ML keyword detection or EXIF context.
  - `thumbs`: creates thumbnails and previews.
  - `verify`: validates JSON integrity, schema versions, and hash consistency.
  - `migrate`: updates sidecars between schema versions.

- **CLI binary (`jozin`):** built on top of the core library for quick testing and batch operation.
  Example:
  ```bash
  jozin scan ~/Photos --dry-run
  jozin verify ~/Photos
  ```

### 2. Jožin App (Tauri + React)
A lightweight desktop interface embedding the same Rust core directly inside Tauri.

- **Rust–JS bridge:** Exposes core functions as Tauri commands (e.g., `scan_path`, `verify_path`).
- **React UI:** Provides intuitive controls for scanning folders, viewing progress, inspecting duplicates, editing tags, and reviewing detected faces.
- **Cross-platform:** macOS, Windows, Linux — no Docker, no server setup, no terminal required.
- **Distribution:** Single-file installer; runs fully offline.

### 3. Optional CLI Mode
The CLI can run standalone for developers or power users — ideal for automation, integration testing, or batch processing.

---

## Local File Structure

Jožin keeps your photo directories clean and predictable.
All metadata lives **next to** the original files — no hidden `.jozin/` folders, no detached sidecar trees.

```bash
/photos/
├─ 2020/
│  ├─ IMG_1234.JPG
│  ├─ IMG_1234.JPG.json       # sidecar (scan + tags + faces)
│  ├─ IMG_1234_512.jpg        # optional thumbnail
│  └─ IMG_1234_256.webp       # optional thumbnail
└─ jozin.journal.ndjson        # optional operation log (if enabled)
```

---

## Project Layout
```bash
jozin/
├─ core/                       # Rust library + API (workspace member)
│  ├─ src/
│  │  ├─ lib.rs
│  │  ├─ scan.rs
│  │  ├─ faces.rs
│  │  ├─ tags.rs
│  │  ├─ thumbs.rs
│  │  ├─ verify.rs
│  │  └─ migrate.rs
│  └─ Cargo.toml
├─ cli/                        # CLI binary (depends on core)
│  ├─ src/main.rs
│  └─ Cargo.toml
├─ app/                        # Tauri + React desktop application
│  ├─ src-tauri/
│  │  ├─ main.rs
│  │  └─ commands.rs
│  ├─ src/                     # React UI
│  ├─ package.json
│  └─ tauri.conf.json
├─ schemas/
│  └─ image-sidecar.schema.json
├─ justfile                    # build/test/run shortcuts
├─ SCOPE.md                    # this document
├─ README.md
└─ .gitignore
```

⸻

### Folder and Cache Policy

| Component | Default Location | Notes |
|------------|------------------|-------|
| **Sidecars** | Adjacent to each image | Written atomically (`.tmp` → fsync → rename); backups `.bak1..bak3` |
| **Thumbnails** | Adjacent to each image | Optional sizes (e.g., `_256`, `_512`) |
| **Cache** | OS temporary directory (e.g. `/tmp/jozin-*`) | Ephemeral; auto-cleaned before and after each run |
| **Journal** | Optional single file at root (`jozin.journal.ndjson`) | Contains per-file operation logs; rotates at 50 MB |
| **Models** | App data directory (e.g. `~/Library/Application Support/Jožin/models`) | Stores ONNX or other model binaries; referenced in sidecar only by ID |
| **Trash** | *Not used* | Jožin never deletes or modifies original files |

---

## Design Rationale

### A modular monolith — by choice
Jožin intentionally avoids a multi-service setup:
- Maximum control by the user
- Minimal operational complexity
- Consistent schema and deterministic behavior
- Easier debugging and development
- Seamless embedding in a desktop environment

### Why Rust
- **Performance & safety:** Ideal for handling large image libraries, EXIF parsing, and hashing at scale.
- **Single binary:** No runtime dependencies, portable across macOS, Windows, and Linux.
- **Perfect fit for Tauri:** Same language on both sides (Rust core + Tauri backend).

### Why Tauri + React
- **Approachable for everyone:** Users get a native app experience with drag-and-drop folders, progress bars, and clear visual feedback.
- **Direct Rust integration:** The Tauri backend can call Jožin Core functions directly, without HTTP or IPC overhead.
- **Cross-platform simplicity:** One build, three OS targets.
- **Offline-friendly:** No Docker, no terminal, no network required.

---

## Privacy & Security

- Original image files are **read-only**; all derived data is stored separately.
- No telemetry, no analytics, no hidden API calls.
- JSON sidecars contain no private data unless the user explicitly adds labels.
- Face embeddings can be hashed or salted for additional privacy.
- All computation happens locally; no data leaves the device.

---

## Future Outlook

- Multiple hash layers (file hash, pixel hash, perceptual hash).
- Plugin architecture (WASM/Dylib) for custom analyzers.
- Visual diff and sidecar-merge tools.
- Portable metadata import/export for backups.
- Optional face-model manager for local retraining.
- Benchmark and AI-assistant integration examples for teaching purposes.

> **Summary**
> Jožin is a local-first photo intelligence platform, built as a Rust core + Tauri app hybrid.
> It’s privacy-respecting, fully offline, and accessible to everyone — from AI engineers to casual users who simply want order in their photo chaos.

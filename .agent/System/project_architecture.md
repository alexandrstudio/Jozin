# Project Architecture - Jožin Photo Organizer

**Last Updated**: 2025-10-21
**Project Phase**: Phase 1 - Near Completion
**Build Status**: ✅ 59 tests passing, zero compiler warnings

---

## Project Goal

Jožin is a **privacy-focused, local-first photo organizer** that helps users manage large, messy photo libraries without uploading or modifying original files. It scans directories, extracts EXIF metadata, computes BLAKE3 hashes, detects duplicates and faces, and stores all derived information in **JSON sidecar files** adjacent to photos. All processing happens locally with complete user control.

### Core Philosophy

1. **Immutable Originals** - Photos are read-only, never modified
2. **Local-First Design** - 100% offline capable, no cloud uploads, no telemetry
3. **Schema-Driven Metadata** - Versioned JSON sidecars with clean migrations
4. **Modular Monolith** - Single Rust binary with well-separated modules
5. **AI-Assisted Development** - Built using modern AI coding assistants as a teaching example

---

## Project Structure

### Cargo Workspace Layout

```
jozin/
├─ core/                      # Rust library (jozin-core)
│  ├─ src/
│  │  ├─ lib.rs               # ✅ Module exports, types, errors
│  │  ├─ scan.rs              # ✅ Directory walking, EXIF, hashing, sidecar generation
│  │  ├─ cleanup.rs           # ✅ Remove generated files (sidecars, thumbnails, backups)
│  │  ├─ verify.rs            # ⚠️ Sidecar validation (10-line stub, Task 1-2)
│  │  ├─ migrate.rs           # ⚠️ Schema migrations (10-line stub, Task 3-4)
│  │  ├─ faces.rs             # 🔒 Face detection (feature-gated, Phase 2+)
│  │  ├─ tags.rs              # 🔒 ML tagging (feature-gated, Phase 2+)
│  │  └─ thumbs.rs            # 🔒 Thumbnails (feature-gated, Phase 2+)
│  ├─ Cargo.toml              # Features: faces, tags, thumbs (Phase 2+)
│  └─ tests/
├─ cli/                       # CLI binary (jozin)
│  ├─ src/
│  │  └─ main.rs              # ✅ Full implementation with clap
│  ├─ tests/
│  │  └─ cli_basic.rs         # ✅ 23 CLI tests
│  └─ Cargo.toml
├─ app/                       # Tauri + React desktop app
│  ├─ src-tauri/              # 🔒 Basic structure (Task 6)
│  │  └─ src/main.rs
│  ├─ src/                    # React UI (not implemented)
│  └─ package.json
├─ .agent/                    # Engineering documentation
│  ├─ System/                 # Architecture, tech stack
│  ├─ Tasks/                  # PRDs and implementation plans
│  ├─ SOP/                    # Best practices
│  └─ README.md               # Documentation index
├─ Cargo.toml                 # Workspace root
├─ justfile                   # Build commands (build, cli, test, release, app-dev)
├─ SCOPE.md                   # Architectural constraints
├─ TASKMASTER_PLAN.md         # Current task breakdown (Tasks 1-7)
├─ CLAUDE.md                  # AI assistant guidance
└─ README.md                  # Developer onboarding

Legend:
  ✅ Fully Implemented
  ⚠️ Minimal Stub (Phase 1 incomplete)
  🔒 Planned (Phase 2+)
```

### Workspace Members

| Member | Package Name | Purpose | Status |
|--------|--------------|---------|--------|
| **core/** | `jozin-core` | Photo processing library | ✅ Partial (scan, cleanup done) |
| **cli/** | `jozin` | Command-line interface | ✅ Complete |
| **app/** | (not in workspace yet) | Tauri desktop app | 🔒 Planned (Task 6) |

---

## Technology Stack

### Core Technologies

| Layer | Technology | Version | Purpose |
|-------|------------|---------|---------|
| **Language** | Rust | 1.75+ | High-performance, memory-safe processing |
| **Workspace** | Cargo | - | Multi-package monorepo |
| **CLI** | Clap | 4.x | Argument parsing with derive macros |
| **Serialization** | Serde | 1.x | JSON sidecar serialization |
| **Hashing** | BLAKE3 | 1.x | Ultra-fast parallel file hashing |
| **Time** | time | 0.3.x | RFC3339 timestamp formatting |
| **File Walking** | walkdir | 2.x | Recursive directory traversal |
| **Glob Matching** | globset | 0.4.x | Include/exclude pattern matching |

### Planned Technologies (Phase 2+)

| Layer | Technology | Purpose |
|-------|------------|---------|
| **Parallelism** | rayon | Bounded thread pools for file processing |
| **ML Runtime** | ONNX Runtime | Local face detection and tagging |
| **Image Decoding** | image | Thumbnail generation, pixel hashing |
| **EXIF Parsing** | kamadak-exif | EXIF metadata extraction |
| **UI Framework** | Tauri + React | Cross-platform desktop app |

### Development Tools

| Tool | Purpose |
|------|---------|
| **just** | Task runner (build, test, release shortcuts) |
| **assert_cmd** | CLI integration testing |
| **tempfile** | Temporary directories for tests |
| **clippy** | Linting and code quality |
| **cargo doc** | API documentation generation |

---

## Module Architecture

### Core Library (jozin-core)

#### Fully Implemented Modules

##### 1. scan - Photo Scanning & Metadata Extraction

**Location**: `core/src/scan.rs` (✅ 600+ lines, fully implemented)

**Purpose**: Directory traversal, EXIF extraction, BLAKE3 hashing, sidecar generation

**Key Functions**:
- `scan_path(path, recursive, include, exclude, dry_run, max_threads, hash_mode, progress_callback) -> Result<OperationResponse<Vec<ScannedFile>>>`
- `scan_file(path) -> Result<ScannedFile>`

**Features**:
- Recursive and non-recursive directory traversal
- Glob pattern filtering (include/exclude)
- EXIF metadata extraction (camera, GPS, timestamp)
- BLAKE3 file hashing (parallel, ultra-fast)
- Atomic sidecar writes (`.tmp` → fsync → rename)
- Dry-run mode (preview without writing)
- Progress callbacks for real-time UI updates

**Data Structures**:
- `ScannedFile` - Per-file scan result
- `ScanAction` - Action taken: created, updated, skipped
- `ScanResult` - Aggregate scan statistics

**Exit Codes**:
- 0: Success
- 1: User error (invalid path, bad glob)
- 2: I/O error (file not found, permission denied)
- 4: Internal error (hash computation failed)

##### 2. cleanup - Remove Generated Files

**Location**: `core/src/cleanup.rs` (✅ 400+ lines, fully implemented)

**Purpose**: Selective removal of Jožin-generated files

**Key Functions**:
- `cleanup_path(path, options) -> Result<OperationResponse<CleanupResult>>`

**Features**:
- Selective cleanup modes:
  - `--only-sidecars`: Remove `*.json` sidecar files
  - `--only-thumbnails`: Remove `*_256.jpg`, `*_512.webp`, etc.
  - `--only-backups`: Remove `*.bak1`, `*.bak2`, `*.bak3`
  - `--only-cache`: Remove `.jozin/` cache directories (deprecated design)
- Dry-run mode (preview deletions)
- Recursive directory processing
- Glob pattern filtering
- Safe deletion (only removes files matching Jožin patterns)

**Data Structures**:
- `CleanupOptions` - Cleanup configuration
- `CleanupResult` - Deleted file paths and statistics
- `DeletedFile` - Per-file deletion result
- `FileType` - Enum: Sidecar, Thumbnail, Backup, Cache

**Exit Codes**:
- 0: Success
- 1: User error (invalid path)
- 2: I/O error (deletion failed)

#### Minimal Stubs (Phase 1 Incomplete)

##### 3. verify - Sidecar Validation

**Location**: `core/src/verify.rs` (⚠️ 10-line stub, **Task 1-2**)

**Purpose**: Validate sidecar integrity, detect staleness, recommend actions

**Planned Features** (see TASKMASTER_PLAN.md Task 1):
- Sidecar existence check (detect missing `.json` files)
- JSON parsing and validation (handle corrupt/malformed JSON)
- Schema version compatibility check
- Hash staleness detection (file modified after scan)
- File modification detection (compare mtime)
- Action recommendations: `noop`, `rescan`, `migrate`
- Fix mode (auto-repair minor issues)

**Planned Data Structures**:
```rust
pub struct VerifyResult {
    pub path: String,
    pub status: VerifyStatus,  // ok, stale, missing, corrupt
    pub reasons: Vec<String>,
    pub suggested: String,      // noop, rescan, migrate
}
```

**Exit Codes**:
- 0: All sidecars valid
- 2: I/O error
- 3: Validation error (corrupt/stale sidecars)

**Implementation Status**: Task 1 (core functionality) + Task 2 (comprehensive tests)

##### 4. migrate - Schema Version Upgrades

**Location**: `core/src/migrate.rs` (⚠️ 10-line stub, **Task 3-4**)

**Purpose**: Upgrade sidecar schema versions with backup rotation

**Planned Features** (see TASKMASTER_PLAN.md Task 3):
- Auto-detect source schema version
- Version upgrade transformations (e.g., split `camera` → `camera_make` + `camera_model`)
- Backup rotation (`.bak1`, `.bak2`, `.bak3`)
- Dry-run mode (preview changes)
- Idempotent migrations (safe to run multiple times)
- Atomic writes (`.tmp` → fsync → rename)

**Planned Data Structures**:
```rust
pub struct MigrationResult {
    pub path: String,
    pub migrated: bool,
    pub from: String,
    pub to: String,
    pub backup_path: Option<String>,
}
```

**Backup Rotation Strategy**:
```
IMG_1234.JPG.json       ← New version
IMG_1234.JPG.json.bak1  ← Previous version
IMG_1234.JPG.json.bak2  ← Two versions ago
IMG_1234.JPG.json.bak3  ← Three versions ago (oldest kept)
```

**Exit Codes**:
- 0: All migrations successful
- 1: User error (invalid version format)
- 2: I/O error
- 3: Validation error (corrupt sidecar)

**Implementation Status**: Task 3 (core functionality) + Task 4 (comprehensive tests)

#### Feature-Gated Modules (Phase 2+)

##### 5. faces - Face Detection & Identification

**Location**: `core/src/faces.rs` (🔒 Stub, feature-gated)

**Purpose**: Local face detection and identification using ONNX models

**Planned Features**:
- Face detection in photos
- Face embedding generation
- Person identification (match to known faces)
- Training on labeled faces
- Confidence scoring and thresholds

**Cargo Feature**: `faces = ["dep:ort", "dep:ndarray"]`

##### 6. tags - Automatic Tagging

**Location**: `core/src/tags.rs` (🔒 Stub, feature-gated)

**Purpose**: ML-based and rule-based automatic tagging

**Planned Features**:
- ML keyword detection (local models)
- Rule-based tagging (EXIF context, filename patterns)
- Tag confidence scoring
- Append mode (keep existing user labels)

**Cargo Feature**: `tags = ["dep:ort", "dep:tokenizers"]`

##### 7. thumbs - Thumbnail Generation

**Location**: `core/src/thumbs.rs` (🔒 Stub, feature-gated)

**Purpose**: Multi-size thumbnail generation

**Planned Features**:
- Multiple sizes (256, 512, 1024, etc.)
- Multiple formats (JPEG, WebP)
- Quality control
- Overwrite mode

**Cargo Feature**: `thumbs = ["dep:image"]`

---

## Data Structures

### Core Types (core/src/lib.rs)

#### Error Handling

```rust
pub enum JozinError {
    UserError { message: String },       // Exit code 1
    IoError { message: String },         // Exit code 2
    ValidationError { message: String }, // Exit code 3
    InternalError { message: String },   // Exit code 4
}

pub type Result<T> = std::result::Result<T, JozinError>;
```

#### Operation Response Wrapper

```rust
pub struct OperationResponse<T> {
    pub started_at: String,    // RFC3339 timestamp
    pub finished_at: String,   // RFC3339 timestamp
    pub duration_ms: u64,
    pub data: T,
}
```

All CLI and Tauri commands return this structure for consistent timing metadata.

#### Pipeline Signature (Schema Versioning)

```rust
pub struct PipelineSignature {
    pub schema_version: String,        // e.g., "1.0.0"
    pub producer_version: String,      // e.g., "0.1.0"
    pub hash_algorithm: String,        // e.g., "blake3"
    pub face_model: Option<String>,    // e.g., "arcface-1.4"
    pub tag_model: Option<String>,     // e.g., "clip-vit-b32"
    pub created_at: String,            // RFC3339
}
```

Used by verify module to detect stale sidecars requiring rescanning.

#### Sidecar Metadata Structure

```rust
pub struct Sidecar {
    pub schema_version: String,
    pub producer_version: String,
    pub created_at: String,
    pub updated_at: String,
    pub pipeline_signature: PipelineSignature,
    pub source: SourceInfo,
    pub image: Option<ImageInfo>,
    pub faces: Vec<FaceDetection>,
    pub tags: Vec<Tag>,
    pub thumbnails: Vec<ThumbnailInfo>,
}

pub struct SourceInfo {
    pub file_path: String,
    pub file_size_bytes: u64,
    pub file_hash_b3: String,       // BLAKE3 hash
    pub file_modified_at: String,   // RFC3339
}

pub struct ImageInfo {
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub format: Option<String>,        // "JPEG", "PNG", "HEIC", "RAW"
    pub orientation: Option<u8>,       // 1-8
    pub datetime_original: Option<String>,  // RFC3339
    pub camera_make: Option<String>,
    pub camera_model: Option<String>,
    pub gps_latitude: Option<f64>,
    pub gps_longitude: Option<f64>,
}
```

### Progress Events

```rust
pub enum ProgressEvent {
    FileStarted { path: String },
    FileCompleted {
        path: String,
        success: bool,
        error: Option<String>,
        size_bytes: Option<u64>,
    },
}
```

Used for real-time progress in CLI and Tauri UI.

---

## CLI Architecture

### Command Structure

```
jozin <subcommand> [options]

Subcommands:
  scan      - Scan photos and generate sidecars
  cleanup   - Remove Jožin-generated files
  verify    - Validate sidecar integrity
  migrate   - Upgrade sidecar schema versions
  faces     - Face detection and identification (Phase 2+)
  tags      - Automatic tagging (Phase 2+)
  thumbs    - Thumbnail generation (Phase 2+)
```

### Common Parameters (All Subcommands)

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `--dry-run` | bool | false | Preview actions without writing files |
| `--recursive` / `-r` | bool | false | Process directories recursively |
| `--include` | String[] | `["*.jpg", "*.jpeg", "*.png", "*.heic"]` | File patterns to include |
| `--exclude` | String[] | - | File patterns to exclude |
| `--max-threads` | u32 | min(2×CPU, 8) | Bounded parallelism (Phase 2+) |
| `--json` | bool | auto-detect | Force JSON output (vs human-readable) |

### Module-Specific Parameters

**scan**:
- `--hash-mode <file|pixel|both>` - Hash computation strategy (Phase 2+)

**cleanup**:
- `--only-sidecars` - Remove only JSON sidecar files
- `--only-thumbnails` - Remove only thumbnail files
- `--only-backups` - Remove only backup files (*.bak1/2/3)
- `--only-cache` - Remove only cache directories (.jozin/*)

**verify** (Task 1-2):
- `--fix` - Attempt auto-repair of minor issues
- `--strict` - Treat warnings as errors

**migrate** (Task 3-4):
- `--from <version>` - Source schema version (auto-detect if omitted)
- `--to <version>` - Target schema version (required)
- `--backup` - Create .bakN backup files (default: true)

### Output Formats

**Human Mode** (TTY detected or --human):
```
Processing: /photos/IMG_1234.JPG
  ✓ Hashed in 12ms
  ✓ EXIF extracted
  ✓ Sidecar written

Scanned 150 files in 3.2s
  Created: 120
  Updated: 25
  Skipped: 5
```

**JSON Mode** (piped or --json):
```json
{
  "started_at": "2025-10-21T14:30:00Z",
  "finished_at": "2025-10-21T14:30:05Z",
  "duration_ms": 5000,
  "data": {
    "scanned": 150,
    "created": 120,
    "updated": 25,
    "skipped": 5,
    "files": [...]
  }
}
```

### Exit Codes

| Code | Meaning | Usage |
|------|---------|-------|
| 0 | Success | All operations completed successfully |
| 1 | User Error | Invalid arguments, bad input |
| 2 | I/O Error | File not found, permission denied |
| 3 | Validation Error | Schema mismatch, corrupt sidecar |
| 4 | Internal Error | Unexpected panics, logic errors |

---

## File Layout & Sidecar Policy

### Metadata Storage Strategy

Jožin stores metadata **adjacent to original files** (no hidden `.jozin/` trees):

```
/photos/
├─ 2020/
│  ├─ IMG_1234.JPG
│  ├─ IMG_1234.JPG.json        # Sidecar with metadata
│  ├─ IMG_1234.JPG.json.bak1   # Backup (after migration)
│  ├─ IMG_1234.JPG.json.bak2
│  ├─ IMG_1234.JPG.json.bak3
│  ├─ IMG_1234_256.jpg         # Optional thumbnail
│  └─ IMG_1234_512.webp        # Optional thumbnail
└─ jozin.journal.ndjson         # Optional operation log (Phase 2)
```

### Sidecar Writing Strategy

1. **Atomic Writes**: `.tmp` → `fsync` → `rename` (prevents corruption)
2. **Backup Rotation**: Keep 3 previous versions (`.bak1`, `.bak2`, `.bak3`)
3. **Never Modify Originals**: Photos are read-only

### Cache & Temporary Files

| Location | Purpose | Lifetime |
|----------|---------|----------|
| OS temp dir (`/tmp/jozin-*`) | Temporary computation cache | Auto-cleaned before/after each run |
| `~/Library/Application Support/Jožin/models` | ONNX model storage | Persistent |

**Note**: Earlier designs used `.jozin/` directories adjacent to photos. This was changed to keep all metadata in sidecars for transparency.

---

## Integration Points

### CLI → Core Library

```rust
// cli/src/main.rs
use jozin_core::{scan_path, cleanup_path, Result, OperationResponse};

fn main() -> Result<()> {
    let matches = Cli::parse();

    match matches.command {
        Commands::Scan(args) => {
            let result = scan_path(
                &args.path,
                args.recursive,
                args.include,
                args.exclude,
                args.dry_run,
                args.max_threads,
                None,  // hash_mode (Phase 2+)
                None,  // progress_callback (Phase 2+)
            )?;

            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        // ... other commands
    }
    Ok(())
}
```

### Tauri App → Core Library (Planned, Task 6)

```rust
// app/src-tauri/src/commands.rs
#[tauri::command]
async fn scan_path(
    path: String,
    recursive: bool,
    dry_run: bool,
) -> Result<serde_json::Value, String> {
    jozin_core::scan_path(&path, recursive, None, None, dry_run, 4, None, None)
        .map(|result| serde_json::to_value(result).unwrap())
        .map_err(|e| e.to_string())
}
```

### Progress Callbacks (Phase 2)

```rust
// Real-time progress for CLI and UI
let progress_callback = |event: ProgressEvent| {
    match event {
        ProgressEvent::FileStarted { path } => {
            println!("Processing: {}", path);
        }
        ProgressEvent::FileCompleted { path, success, .. } => {
            println!("{} ... {}", path, if success { "✓" } else { "✗" });
        }
    }
};

scan_path(&path, true, None, None, false, 4, None, Some(&progress_callback))?;
```

---

## Build & Development Workflow

### Common Commands (justfile)

```bash
# Build entire workspace
just build
# or
cargo build --workspace

# Quick CLI test
just cli
# or
cargo run -p jozin -- scan ./Photos --dry-run

# Run all tests (59 tests)
just test
# or
cargo test --workspace

# Build release binaries
just release
# or
cargo build --workspace --release

# Launch Tauri app in dev mode (Task 6)
just app-dev
# or
cd app && npm install && npx tauri dev
```

### Testing Strategy

**Test Coverage** (59 tests passing):
- 23 CLI tests (cli/tests/cli_basic.rs)
- 24 Core tests (core/src/scan.rs, core/src/cleanup.rs)
- 12 Doc tests (code examples in core/src/lib.rs)

**Test Requirements**:
- Handle Unicode paths, spaces, symlinks
- Support large files (10GB+ RAW images)
- Cross-platform (macOS, Windows, Linux)
- Concurrent execution (no race conditions)
- Deterministic outputs (same inputs → same outputs)

### Continuous Integration

**Build Health** (as of 2025-10-21):
- ✅ `cargo build --workspace` - Zero warnings
- ✅ `cargo test --workspace` - 59 tests passing
- ✅ `cargo clippy --workspace` - No issues
- ✅ `cargo doc --workspace --no-deps` - Builds without errors

---

## Development Phases

### Phase 0 - Parameter Parsing & CLI Wiring (✅ Complete)

- CLI argument parsing with clap
- Help text and validation
- Exit code specification
- JSON output formatting

### Phase 1 - Minimal Functional Core (⚠️ Near Complete)

**Completed**:
- ✅ scan module (full implementation)
- ✅ cleanup module (full implementation)
- ✅ CLI with comprehensive parameter validation
- ✅ Core infrastructure (errors, timing, signatures)
- ✅ Test suite (59 tests passing)

**Remaining**:
- ⚠️ verify module (Tasks 1-2)
- ⚠️ migrate module (Tasks 3-4)
- Phase 1 validation (Task 5)

### Phase 2 - Robustness & Performance (🔒 Planned)

- Bounded parallelism (`--max-threads`, rayon/tokio)
- Journal support (resumable scans, `jozin.journal.ndjson`)
- Progress events API (real-time CLI/UI updates)
- Perceptual hash (pHash for duplicate detection)
- Pixel hash (cross-format duplicate detection)

### Phase 2+ - Advanced Features (🔒 Planned)

- Face detection & identification (local ONNX models)
- ML-based tagging (local models)
- Thumbnail generation (multi-size, multi-format)
- Tauri desktop UI (Tasks 6-7)

---

## Current Development Status

**Active Tasks** (see TASKMASTER_PLAN.md):

| Task | Description | Status | Priority |
|------|-------------|--------|----------|
| Task 1 | Implement verify module core functionality | Pending | Critical |
| Task 2 | Add comprehensive tests for verify module | Pending | Critical |
| Task 3 | Implement migrate module core functionality | Pending | Critical |
| Task 4 | Add comprehensive tests for migrate module | Pending | Critical |
| Task 5 | Validate Phase 1 completion and create Phase 2 roadmap | Pending | High |
| Task 6 | Initialize Tauri app foundation | Pending | Medium |
| Task 7 | Plan Phase 2 implementation strategy | Pending | Low |

**Next Milestone**: Complete Tasks 1-4 to finish Phase 1

---

## Related Documentation

- **SCOPE.md** - Architectural constraints and design principles
- **TASKMASTER_PLAN.md** - Detailed task breakdown (Tasks 1-7)
- **README.md** - Developer onboarding and quick start
- **CLAUDE.md** - AI assistant guidance for this codebase
- **BUILD_STATUS.md** - Detailed build and test status
- **.agent/Tasks/** - Individual task PRDs and implementation plans
- **.agent/SOP/** - Best practices for common development tasks

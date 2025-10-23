# Jožin Development Plan — Taskmaster Roadmap

**Generated**: 2025-10-21
**Request ID**: req-1
**Total Tasks**: 7
**Current Phase**: Phase 1 completion → Phase 2 preparation

---

## 📊 Project Status Overview

### Current State Analysis

**Phase 1 — Completed Components:**
- ✅ **scan module** (core/src/scan.rs) — Full implementation with BLAKE3 hashing, EXIF extraction, sidecar generation
- ✅ **cleanup module** (core/src/cleanup.rs) — Full implementation with selective cleanup modes
- ✅ **CLI binary** (cli/src/main.rs) — All subcommands wired, help text, parameter validation
- ✅ **Core infrastructure** (core/src/lib.rs) — Error types, timing metadata, pipeline signatures
- ✅ **Test suite** — 59 tests passing (23 CLI + 24 core + 12 doc tests)
- ✅ **Build health** — Zero compiler warnings, production-ready

**Phase 1 — Incomplete Components:**
- ⚠️ **verify module** (core/src/verify.rs) — Minimal 10-line stub only
- ⚠️ **migrate module** (core/src/migrate.rs) — Minimal 10-line stub only

**Phase 2+ — Future Work:**
- 🔒 **faces module** (feature-gated stub)
- 🔒 **tags module** (feature-gated stub)
- 🔒 **thumbs module** (feature-gated stub)
- 🔒 **Tauri app** (basic structure exists, not implemented)
- 🔒 **Parallelism, journaling, progress API** (not started)

---

## 🎯 Strategic Goals

1. **Complete Phase 1** by implementing verify and migrate modules
2. **Validate acceptance criteria** from TASK+PHASE_PLAN.md
3. **Prepare Tauri foundation** for desktop UI development
4. **Plan Phase 2 architecture** for parallelism and advanced features

---

## 📋 Task Breakdown

### Task 1: Implement verify module core functionality
**ID**: task-1
**Status**: Pending
**Priority**: Critical (blocks Phase 1 completion)
**Estimated Effort**: Medium

#### Description
Build the verify module to validate sidecar JSON structure, check schema versions, detect file hash staleness, and recommend actions (noop/rescan/migrate). Should handle invalid JSON, missing fields, schema mismatches, and hash changes. Must return structured output with status: ok|stale|missing|corrupt and suggested actions.

#### Technical Specifications

**Module Location**: `core/src/verify.rs`

**Core Functionality Required**:
1. **Sidecar existence check** — Detect missing `.json` files
2. **JSON parsing** — Handle corrupt/malformed JSON gracefully
3. **Schema validation** — Verify required fields exist (schema_version, producer_version, source, etc.)
4. **Version compatibility** — Use `PipelineSignature::is_compatible_with()` to check schema compatibility
5. **Hash staleness detection** — Compare `source.file_hash_b3` with current file hash
6. **File modification detection** — Compare `source.file_modified_at` with current mtime
7. **Action recommendation** — Return one of: "noop", "rescan", "migrate"

**Output Structure** (per TASK+PHASE_PLAN.md):
```json
{
  "started_at": "2025-10-21T14:30:00Z",
  "finished_at": "2025-10-21T14:30:05Z",
  "duration_ms": 5000,
  "data": [
    {
      "path": "/photos/IMG_1234.JPG",
      "status": "ok|stale|missing|corrupt",
      "reasons": ["file_hash_changed", "schema_version_mismatch"],
      "suggested": "noop|rescan|migrate"
    }
  ]
}
```

**Status Values**:
- `ok` — Sidecar is valid and up-to-date
- `stale` — Sidecar exists but needs updating (hash changed, old schema)
- `missing` — No sidecar file found
- `corrupt` — Sidecar exists but JSON is malformed/invalid

**Suggested Actions**:
- `noop` — No action needed (status: ok)
- `rescan` — File content changed, run `jozin scan` again
- `migrate` — Schema version changed, run `jozin migrate`

**Parameters** (from TASK+PHASE_PLAN.md):
- `path`: File or directory to verify
- `recursive`: Process directories recursively
- `fix`: Attempt auto-repair of minor issues (Phase 2+)
- `strict`: Treat warnings as errors (Phase 2+)
- `pipeline_signature`: Override pipeline signature for comparison (Phase 2+)

**Error Handling**:
- Return `JozinError::IoError` for file access failures (exit code 2)
- Return `JozinError::ValidationError` for schema issues (exit code 3)
- Use `OperationResponse<Vec<VerifyResult>>` wrapper for timing metadata

**Dependencies**:
- Use existing `Sidecar` struct from lib.rs
- Use existing `PipelineSignature::is_compatible_with()` logic
- Reuse BLAKE3 hashing from scan module

#### Acceptance Criteria
- [ ] Can detect missing sidecar files
- [ ] Can parse valid JSON sidecars without errors
- [ ] Can detect corrupt/malformed JSON
- [ ] Can identify missing required fields
- [ ] Can detect file hash changes (staleness)
- [ ] Returns correct status for each scenario
- [ ] Provides actionable recommendations
- [ ] Handles both single files and directories
- [ ] Respects `--recursive` flag
- [ ] Returns timing metadata in `OperationResponse`

---

### Task 2: Add comprehensive tests for verify module
**ID**: task-2
**Status**: Pending
**Priority**: Critical (validates Task 1)
**Estimated Effort**: Medium
**Dependencies**: Task 1 must be completed first

#### Description
Create unit tests covering all validation scenarios: valid sidecars, corrupt JSON, missing required fields, schema version mismatches, stale hashes (file modified), missing sidecars, and pipeline signature compatibility checks. Tests should verify exit codes and JSON output structure.

#### Test Coverage Requirements

**Test File Location**: `core/src/verify.rs` (in `#[cfg(test)] mod tests`)

**Required Test Cases**:
1. ✅ Valid sidecar (status: ok, suggested: noop)
2. ✅ Missing sidecar file (status: missing, suggested: rescan)
3. ✅ Corrupt JSON (status: corrupt, suggested: rescan)
4. ✅ Missing required field `schema_version` (status: corrupt)
5. ✅ Missing required field `source.file_hash_b3` (status: corrupt)
6. ✅ File hash changed (status: stale, suggested: rescan)
7. ✅ Schema version mismatch (status: stale, suggested: migrate)
8. ✅ Pipeline signature incompatible (status: stale, suggested: migrate)
9. ✅ Directory verification (recursive vs non-recursive)
10. ✅ Multiple files with mixed statuses

**Test Data Setup**:
- Create temporary test directories with sample images
- Generate valid sidecars using scan module
- Create corrupt sidecars (invalid JSON, missing fields)
- Modify files to trigger staleness detection

**Assertions**:
- Verify correct status for each scenario
- Verify suggested action matches expected
- Verify reasons array contains appropriate messages
- Verify timing metadata is present
- Verify error exit codes match specification

#### Acceptance Criteria
- [ ] All 10+ test scenarios pass
- [ ] Tests use temporary directories (cleaned up after)
- [ ] Tests are deterministic (no flaky behavior)
- [ ] `cargo test --package jozin-core verify` passes
- [ ] Test coverage includes edge cases (Unicode paths, symlinks)

---

### Task 3: Implement migrate module core functionality
**ID**: task-3
**Status**: Pending
**Priority**: Critical (blocks Phase 1 completion)
**Estimated Effort**: Medium-Large

#### Description
Build the migrate module to detect sidecar schema versions, perform version upgrades with transformations, create backup files (.bak1/.bak2/.bak3 rotation), support dry-run mode, and handle migration failures gracefully. Must support idempotent migrations (v1→v1) and sample migration (e.g., split camera field).

#### Technical Specifications

**Module Location**: `core/src/migrate.rs`

**Core Functionality Required**:
1. **Schema detection** — Parse `schema_version` from existing sidecar
2. **Version validation** — Ensure source → target migration path exists
3. **Migration logic** — Transform sidecar structure according to version changes
4. **Backup rotation** — Create `.bak1`, `.bak2`, `.bak3` before modifying files
5. **Atomic writes** — Use `.tmp` → fsync → rename pattern
6. **Dry-run mode** — Show intended changes without writing files
7. **Idempotency** — Running same migration twice produces same result

**Output Structure** (per TASK+PHASE_PLAN.md):
```json
{
  "started_at": "2025-10-21T14:30:00Z",
  "finished_at": "2025-10-21T14:30:05Z",
  "duration_ms": 5000,
  "data": [
    {
      "path": "/photos/IMG_1234.JPG.json",
      "migrated": true,
      "from": "1.0.0",
      "to": "1.1.0",
      "backup_path": "/photos/IMG_1234.JPG.json.bak1"
    }
  ]
}
```

**Parameters** (from TASK+PHASE_PLAN.md):
- `path`: File or directory to migrate
- `recursive`: Process directories recursively
- `from`: Source schema version (auto-detect if omitted)
- `to`: Target schema version (required)
- `dry_run`: Show changes without writing
- `backup`: Create backup files (default: true)

**Migration Examples**:

**No-op migration (v1.0.0 → v1.0.0)**:
- Validate sidecar is already at target version
- Return `migrated: false`
- No file changes

**Sample migration (v1.0.0 → v1.1.0)**:
- Example: Split `image.camera` into `image.camera_make` and `image.camera_model`
- Read existing sidecar
- Apply transformation function
- Update `schema_version` to "1.1.0"
- Write to file with backup

**Backup Rotation Strategy**:
```
IMG_1234.JPG.json       ← New version
IMG_1234.JPG.json.bak1  ← Previous version (before this migration)
IMG_1234.JPG.json.bak2  ← Two versions ago
IMG_1234.JPG.json.bak3  ← Three versions ago (oldest kept)
```

**Error Handling**:
- Return `JozinError::UserError` for invalid version format (exit code 1)
- Return `JozinError::IoError` for file access failures (exit code 2)
- Return `JozinError::ValidationError` for corrupt sidecars (exit code 3)

**Architecture**:
```rust
// Migration registry (Phase 1: simple match statement; Phase 2+: plugin system)
fn migrate_v1_to_v1_1(sidecar: &mut Sidecar) -> Result<()> {
    // Example transformation
    if let Some(ref mut image) = sidecar.image {
        // Split camera field logic
    }
    sidecar.schema_version = "1.1.0".to_string();
    Ok(())
}

fn apply_migration(from: &str, to: &str, sidecar: &mut Sidecar) -> Result<()> {
    match (from, to) {
        ("1.0.0", "1.0.0") => Ok(()), // No-op
        ("1.0.0", "1.1.0") => migrate_v1_to_v1_1(sidecar),
        _ => Err(JozinError::UserError {
            message: format!("No migration path from {} to {}", from, to)
        })
    }
}
```

#### Acceptance Criteria
- [ ] Auto-detects schema version from existing sidecars
- [ ] Supports no-op migration (v1.0.0 → v1.0.0)
- [ ] Implements sample migration with schema transformation
- [ ] Creates backup files with rotation (.bak1/.bak2/.bak3)
- [ ] Dry-run mode shows changes without writing
- [ ] Idempotent (running twice produces same result)
- [ ] Handles missing source version gracefully
- [ ] Returns structured JSON output with timing

---

### Task 4: Add comprehensive tests for migrate module
**ID**: task-4
**Status**: Pending
**Priority**: Critical (validates Task 3)
**Estimated Effort**: Medium
**Dependencies**: Task 3 must be completed first

#### Description
Create unit tests covering: no-op migration (v1→v1), version upgrades with schema changes, backup rotation (verify .bak1/.bak2/.bak3), dry-run mode (no file writes), migration failure handling, and idempotency (running twice produces same result).

#### Test Coverage Requirements

**Test File Location**: `core/src/migrate.rs` (in `#[cfg(test)] mod tests`)

**Required Test Cases**:
1. ✅ No-op migration (v1.0.0 → v1.0.0, no changes)
2. ✅ Version upgrade (v1.0.0 → v1.1.0, schema transformed)
3. ✅ Backup rotation (verify .bak1/.bak2/.bak3 files created)
4. ✅ Dry-run mode (no files written, shows intended changes)
5. ✅ Migration failure (invalid source version)
6. ✅ Migration failure (unknown target version)
7. ✅ Idempotency (run twice, verify same result)
8. ✅ Backup rotation overflow (4th migration rotates oldest .bak3 out)
9. ✅ Directory migration (recursive vs non-recursive)
10. ✅ Auto-detect source version (from omitted)

**Test Data Setup**:
- Create temporary test directories
- Generate sidecars with different schema versions
- Pre-create existing .bak1/.bak2 files to test rotation
- Create corrupt sidecars to test error handling

**Assertions**:
- Verify schema_version field updated correctly
- Verify backup files created in correct order
- Verify dry-run produces no file changes
- Verify idempotent migrations (hash before == hash after on second run)
- Verify error messages for invalid migrations

#### Acceptance Criteria
- [ ] All 10+ test scenarios pass
- [ ] Tests verify backup file content matches original
- [ ] Tests verify schema transformations applied correctly
- [ ] `cargo test --package jozin-core migrate` passes
- [ ] Tests clean up temporary files

---

### Task 5: Validate Phase 1 completion and create Phase 2 roadmap
**ID**: task-5
**Status**: Pending
**Priority**: High (gates transition to Phase 2)
**Estimated Effort**: Small
**Dependencies**: Tasks 1-4 must be completed first

#### Description
Run full test suite, verify all acceptance criteria from TASK+PHASE_PLAN.md are met, ensure zero compiler warnings, validate CLI --help output for all commands, and document what's needed for Phase 2 (bounded parallelism, journaling, progress API, perceptual/pixel hashing).

#### Validation Checklist

**Phase 1 Acceptance Criteria** (from TASK+PHASE_PLAN.md):
- [ ] CLI callable for all modules (scan, cleanup, verify, migrate)
- [ ] Each parameter validated and returns JSON response
- [ ] Inner functional structure for each task implemented
- [ ] All modules handle paths, globs, dry-run correctly
- [ ] Exit codes match specification (0/1/2/3/4)

**Build Health**:
- [ ] `cargo build --workspace` succeeds with zero warnings
- [ ] `cargo test --workspace` passes all tests
- [ ] `cargo clippy --workspace` reports no issues
- [ ] `cargo doc --workspace --no-deps` builds without errors

**CLI Validation**:
- [ ] `jozin --help` shows all subcommands
- [ ] `jozin scan --help` shows all options with defaults
- [ ] `jozin cleanup --help` shows all options
- [ ] `jozin verify --help` shows all options
- [ ] `jozin migrate --help` shows all options
- [ ] Help text includes examples

**Documentation**:
- [ ] Update CRUSH.md with Phase 1 completion status
- [ ] Document current test count and coverage
- [ ] List known limitations and edge cases

#### Phase 2 Roadmap Document

**Create**: `PHASE2_ROADMAP.md`

**Contents**:
1. **Bounded Parallelism**
   - Implement `--max-threads` parameter across all modules
   - Use thread pool (rayon or tokio) for concurrent file processing
   - Ensure no unbounded memory growth on 100k+ file libraries
   - Target: stable throughput, predictable memory usage

2. **Journal Support**
   - Design `jozin.journal.ndjson` schema
   - Implement append-only operation logging
   - Add rotation at 50 MB threshold
   - Enable resumable scans (kill mid-scan, rerun continues safely)

3. **Progress Events API**
   - Design callback interface for real-time progress
   - Emit `FileStarted`, `FileCompleted` events
   - Wire to CLI progress bars and Tauri UI
   - Include file count, bytes processed, ETA

4. **Advanced Hashing**
   - Implement perceptual hash (pHash) for duplicate detection
   - Implement pixel hash (normalized decode) for near-duplicate detection
   - Add `--hash-mode file|pixel|both` parameter
   - Store in sidecar `source` section

5. **Test Matrix Expansion**
   - Unicode paths, spaces, symlinks
   - Network mounts (NFS, SMB)
   - Large files (10GB+ RAW images)
   - Corrupt headers, missing EXIF
   - Cross-platform (macOS, Windows, Linux)
   - Concurrency edge cases (--max-threads 1 vs many)

#### Acceptance Criteria
- [ ] All Phase 1 acceptance criteria verified
- [ ] Build health confirmed (zero warnings)
- [ ] All CLI help text validated
- [ ] PHASE2_ROADMAP.md created with detailed plan
- [ ] Team alignment on Phase 2 priorities

---

### Task 6: Initialize Tauri app foundation
**ID**: task-6
**Status**: Pending
**Priority**: Medium (enables desktop UI development)
**Estimated Effort**: Medium

#### Description
Set up the Tauri application with package.json, tauri.conf.json, basic Rust commands bridge, and minimal React UI scaffold. Implement basic commands: scan_path, verify_path, cleanup_path. Create just app-dev workflow.

#### Technical Specifications

**Directory Structure**:
```
app/
├── src/                    # React frontend
│   ├── App.tsx
│   ├── main.tsx
│   └── components/
│       ├── Scanner.tsx
│       ├── Verifier.tsx
│       └── Cleaner.tsx
├── src-tauri/              # Tauri backend
│   ├── src/
│   │   ├── main.rs
│   │   └── commands.rs     # Tauri command handlers
│   ├── Cargo.toml
│   └── tauri.conf.json
├── package.json
├── tsconfig.json
└── vite.config.ts
```

**Tauri Commands to Implement**:

```rust
// app/src-tauri/src/commands.rs

#[tauri::command]
async fn scan_path(
    path: String,
    recursive: bool,
    dry_run: bool,
) -> Result<serde_json::Value, String> {
    // Bridge to jozin_core::scan::scan_path
}

#[tauri::command]
async fn verify_path(
    path: String,
    recursive: bool,
) -> Result<serde_json::Value, String> {
    // Bridge to jozin_core::verify module
}

#[tauri::command]
async fn cleanup_path(
    path: String,
    only_sidecars: bool,
    dry_run: bool,
) -> Result<serde_json::Value, String> {
    // Bridge to jozin_core::cleanup::cleanup_path
}
```

**React UI Requirements**:
- Folder picker (drag-and-drop or button)
- Progress indicator during operations
- JSON result display (pretty-printed)
- Tabs for Scan / Verify / Cleanup
- Dark mode toggle
- Responsive layout

**Configuration Files**:

**package.json**:
```json
{
  "name": "jozin-app",
  "version": "0.1.0",
  "scripts": {
    "dev": "vite",
    "build": "vite build",
    "tauri": "tauri"
  },
  "dependencies": {
    "react": "^18",
    "react-dom": "^18"
  },
  "devDependencies": {
    "@tauri-apps/cli": "^2",
    "@tauri-apps/api": "^2",
    "vite": "^5",
    "typescript": "^5"
  }
}
```

**tauri.conf.json**:
- Set app identifier: `com.5leaves.jozin`
- Enable file system access
- Configure window size (1200x800)
- Set app name and version

**justfile Integration**:
```bash
# Add to existing justfile
app-dev:
    cd app && npm install && npx tauri dev

app-build:
    cd app && npm install && npx tauri build
```

#### Acceptance Criteria
- [ ] `npm install` succeeds in app/
- [ ] `just app-dev` launches Tauri app
- [ ] All three commands (scan/verify/cleanup) callable from UI
- [ ] UI displays JSON results
- [ ] Progress indicator works during long operations
- [ ] App compiles for macOS (primary target)
- [ ] No console errors in dev mode

---

### Task 7: Plan Phase 2 implementation strategy
**ID**: task-7
**Status**: Pending
**Priority**: Low (planning only)
**Estimated Effort**: Small

#### Description
Document detailed implementation plan for Phase 2 features: bounded parallelism (--max-threads), journal support (jozin.journal.ndjson), progress events API for UI, perceptual hash (pHash), and pixel hash (normalized decode). Define acceptance criteria for each feature.

#### Deliverables

**Create**: `PHASE2_IMPLEMENTATION.md`

**Structure**:

1. **Feature 1: Bounded Parallelism**
   - Dependency: `rayon` or `tokio::task`
   - API: `--max-threads N` (default: min(2×CPU, 8))
   - Implementation: Thread pool for file processing
   - Tests: Verify no unbounded memory growth on 100k files
   - Acceptance: Stable throughput, predictable memory

2. **Feature 2: Journal Support**
   - File format: NDJSON (newline-delimited JSON)
   - Location: `{root}/jozin.journal.ndjson`
   - Schema: `{ timestamp, operation, path, status, duration_ms }`
   - Rotation: At 50 MB, create `jozin.journal.1.ndjson`
   - Tests: Verify resumable scans (kill mid-run, restart)
   - Acceptance: Operations can resume without re-processing

3. **Feature 3: Progress Events API**
   - API: Callback function `Option<&dyn Fn(ProgressEvent)>`
   - Events: `FileStarted { path }`, `FileCompleted { path, success, error, size_bytes }`
   - CLI: Wire to progress bar (indicatif crate)
   - Tauri: Wire to real-time UI updates
   - Tests: Verify events emitted for all files
   - Acceptance: CLI shows progress bar, UI shows real-time count

4. **Feature 4: Perceptual Hash (pHash)**
   - Dependency: `image` crate + pHash algorithm
   - Add `source.perceptual_hash` field to sidecar
   - Parameter: `--hash-mode pixel|perceptual|both`
   - Use case: Detect near-duplicates (cropped, resized, filtered images)
   - Tests: Verify identical pHash for visually similar images
   - Acceptance: Can detect duplicates with 95% similarity

5. **Feature 5: Pixel Hash**
   - Decode image to normalized pixel data
   - Hash decoded pixels (not file bytes)
   - Add `source.pixel_hash` field to sidecar
   - Use case: Detect duplicates across different encodings (JPEG→PNG)
   - Tests: Verify same pixel hash for re-encoded images
   - Acceptance: Can detect duplicates across formats

**Priority Order** (for implementation):
1. Bounded parallelism (biggest performance win)
2. Progress events API (best UX improvement)
3. Journal support (enables resumability)
4. Perceptual hash (advanced duplicate detection)
5. Pixel hash (advanced duplicate detection)

**Risk Assessment**:
- Parallelism: Risk of race conditions, memory issues
- Journal: Risk of file corruption, disk space management
- Progress API: Risk of performance overhead from callbacks
- pHash: Risk of false positives, CPU-intensive computation

**Mitigation Strategies**:
- Extensive testing on large libraries (100k+ files)
- Benchmarking before/after each feature
- Feature flags to enable/disable risky features
- User documentation for troubleshooting

#### Acceptance Criteria
- [ ] PHASE2_IMPLEMENTATION.md created
- [ ] All 5 features documented with specs
- [ ] Acceptance criteria defined for each
- [ ] Priority order established
- [ ] Risks and mitigations documented

---

## 📅 Execution Strategy

### Task Dependencies
```
Task 1 (verify impl) ──→ Task 2 (verify tests) ──┐
                                                   │
Task 3 (migrate impl) ─→ Task 4 (migrate tests) ──┤
                                                   ├──→ Task 5 (Phase 1 validation)
                                                   │
Task 6 (Tauri app) ────────────────────────────────┤
                                                   │
Task 7 (Phase 2 plan) ─────────────────────────────┘
```

### Recommended Order
1. **Day 1-2**: Task 1 + Task 2 (verify module)
2. **Day 3-4**: Task 3 + Task 4 (migrate module)
3. **Day 5**: Task 5 (Phase 1 validation)
4. **Day 6-7**: Task 6 (Tauri app foundation)
5. **Day 8**: Task 7 (Phase 2 planning)

### Success Metrics
- All 59+ tests passing (will grow to 80+ after Tasks 2 and 4)
- Zero compiler warnings
- All Phase 1 acceptance criteria met
- Tauri app launches and works
- Clear Phase 2 roadmap documented

---

## 🚀 Next Steps

**Immediate Action**: Review this plan and approve to begin Task 1.

**Questions to Consider**:
1. Should verify module support `--fix` mode in Phase 1 or defer to Phase 2?
2. What sample migration should we implement (camera field split is proposed)?
3. Which Phase 2 feature should be highest priority?
4. Should Tauri app support Windows/Linux in Phase 1 or macOS-only first?

**After Approval**: Execute Task 1 by implementing core verify module functionality in `core/src/verify.rs`.

---

**End of Taskmaster Plan**

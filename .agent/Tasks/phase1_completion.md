# Phase 1 Completion Tasks

**Created**: 2025-10-21
**Status**: In Progress (4 of 7 tasks remaining)
**Target**: Complete Phase 1 core functionality

---

## Overview

Phase 1 aims to deliver minimal functional implementations of all core modules. Currently, `scan` and `cleanup` are fully implemented, but `verify` and `migrate` are minimal 10-line stubs that need completion.

**Source**: TASKMASTER_PLAN.md

---

## Task 1: Implement verify Module Core Functionality

**Status**: 🔴 Pending
**Priority**: Critical (blocks Phase 1 completion)
**Estimated Effort**: Medium
**Assignee**: TBD

### Purpose

Build the verify module to validate sidecar JSON structure, check schema versions, detect file hash staleness, and recommend actions (noop/rescan/migrate).

### Requirements

**Module Location**: `core/src/verify.rs`

**Core Functionality**:
1. Sidecar existence check - Detect missing `.json` files
2. JSON parsing - Handle corrupt/malformed JSON gracefully
3. Schema validation - Verify required fields exist
4. Version compatibility - Use `PipelineSignature::is_compatible_with()`
5. Hash staleness detection - Compare `source.file_hash_b3` with current hash
6. File modification detection - Compare `source.file_modified_at` with current mtime
7. Action recommendation - Return: `noop`, `rescan`, or `migrate`

### API Signature

```rust
pub fn verify_path(
    path: &Path,
    recursive: bool,
    fix: bool,           // Phase 2+
    strict: bool,        // Phase 2+
) -> Result<OperationResponse<Vec<VerifyResult>>>

pub struct VerifyResult {
    pub path: String,
    pub status: VerifyStatus,  // ok, stale, missing, corrupt
    pub reasons: Vec<String>,
    pub suggested: String,      // noop, rescan, migrate
}

pub enum VerifyStatus {
    Ok,       // Sidecar valid and up-to-date
    Stale,    // Needs updating (hash changed, old schema)
    Missing,  // No sidecar file found
    Corrupt,  // JSON malformed/invalid
}
```

### Output Format

```json
{
  "started_at": "2025-10-21T14:30:00Z",
  "finished_at": "2025-10-21T14:30:05Z",
  "duration_ms": 5000,
  "data": [
    {
      "path": "/photos/IMG_1234.JPG",
      "status": "stale",
      "reasons": ["file_hash_changed", "schema_version_mismatch"],
      "suggested": "rescan"
    }
  ]
}
```

### Error Handling

- `JozinError::IoError` for file access failures (exit code 2)
- `JozinError::ValidationError` for schema issues (exit code 3)
- Use `OperationResponse<Vec<VerifyResult>>` wrapper for timing metadata

### Acceptance Criteria

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

### Implementation Notes

- Reuse existing `Sidecar` struct from `lib.rs`
- Reuse BLAKE3 hashing from scan module
- Use `PipelineSignature::is_compatible_with()` for schema checks
- Phase 1: Defer `--fix` and `--strict` to Phase 2

---

## Task 2: Add Comprehensive Tests for verify Module

**Status**: 🔴 Pending
**Priority**: Critical (validates Task 1)
**Estimated Effort**: Medium
**Dependencies**: Task 1 must be completed first

### Purpose

Create unit tests covering all validation scenarios to ensure verify module behaves correctly across all edge cases.

### Test Coverage Requirements

**Test File Location**: `core/src/verify.rs` (in `#[cfg(test)] mod tests`)

**Required Test Cases**:
1. Valid sidecar (status: ok, suggested: noop)
2. Missing sidecar file (status: missing, suggested: rescan)
3. Corrupt JSON (status: corrupt, suggested: rescan)
4. Missing required field `schema_version` (status: corrupt)
5. Missing required field `source.file_hash_b3` (status: corrupt)
6. File hash changed (status: stale, suggested: rescan)
7. Schema version mismatch (status: stale, suggested: migrate)
8. Pipeline signature incompatible (status: stale, suggested: migrate)
9. Directory verification (recursive vs non-recursive)
10. Multiple files with mixed statuses

### Test Data Setup

- Create temporary test directories with sample images
- Generate valid sidecars using scan module
- Create corrupt sidecars (invalid JSON, missing fields)
- Modify files to trigger staleness detection

### Assertions

- Verify correct status for each scenario
- Verify suggested action matches expected
- Verify reasons array contains appropriate messages
- Verify timing metadata is present
- Verify error exit codes match specification

### Acceptance Criteria

- [ ] All 10+ test scenarios pass
- [ ] Tests use temporary directories (cleaned up after)
- [ ] Tests are deterministic (no flaky behavior)
- [ ] `cargo test --package jozin-core verify` passes
- [ ] Test coverage includes edge cases (Unicode paths, symlinks)

---

## Task 3: Implement migrate Module Core Functionality

**Status**: 🔴 Pending
**Priority**: Critical (blocks Phase 1 completion)
**Estimated Effort**: Medium-Large

### Purpose

Build the migrate module to detect sidecar schema versions, perform version upgrades with transformations, create backup files, and handle migration failures gracefully.

### Requirements

**Module Location**: `core/src/migrate.rs`

**Core Functionality**:
1. Schema detection - Parse `schema_version` from existing sidecar
2. Version validation - Ensure source → target migration path exists
3. Migration logic - Transform sidecar structure according to version changes
4. Backup rotation - Create `.bak1`, `.bak2`, `.bak3` before modifying
5. Atomic writes - Use `.tmp` → fsync → rename pattern
6. Dry-run mode - Show intended changes without writing
7. Idempotency - Running same migration twice produces same result

### API Signature

```rust
pub fn migrate_path(
    path: &Path,
    from: Option<&str>,  // Auto-detect if None
    to: &str,            // Required
    recursive: bool,
    dry_run: bool,
    backup: bool,        // Default: true
) -> Result<OperationResponse<Vec<MigrationResult>>>

pub struct MigrationResult {
    pub path: String,
    pub migrated: bool,
    pub from: String,
    pub to: String,
    pub backup_path: Option<String>,
}
```

### Migration Examples

**No-op migration (v1.0.0 → v1.0.0)**:
- Validate sidecar is already at target version
- Return `migrated: false`
- No file changes

**Sample migration (v1.0.0 → v1.1.0)**:
- Example: Split `image.camera` into `image.camera_make` + `image.camera_model`
- Read existing sidecar
- Apply transformation function
- Update `schema_version` to "1.1.0"
- Write to file with backup

### Backup Rotation Strategy

```
IMG_1234.JPG.json       ← New version
IMG_1234.JPG.json.bak1  ← Previous version (before this migration)
IMG_1234.JPG.json.bak2  ← Two versions ago
IMG_1234.JPG.json.bak3  ← Three versions ago (oldest kept)
```

When creating 4th backup:
1. Delete `.bak3` (oldest)
2. Rename `.bak2` → `.bak3`
3. Rename `.bak1` → `.bak2`
4. Create new `.bak1` from current file

### Output Format

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

### Error Handling

- `JozinError::UserError` for invalid version format (exit code 1)
- `JozinError::IoError` for file access failures (exit code 2)
- `JozinError::ValidationError` for corrupt sidecars (exit code 3)

### Acceptance Criteria

- [ ] Auto-detects schema version from existing sidecars
- [ ] Supports no-op migration (v1.0.0 → v1.0.0)
- [ ] Implements sample migration with schema transformation
- [ ] Creates backup files with rotation (.bak1/.bak2/.bak3)
- [ ] Dry-run mode shows changes without writing
- [ ] Idempotent (running twice produces same result)
- [ ] Handles missing source version gracefully
- [ ] Returns structured JSON output with timing

---

## Task 4: Add Comprehensive Tests for migrate Module

**Status**: 🔴 Pending
**Priority**: Critical (validates Task 3)
**Estimated Effort**: Medium
**Dependencies**: Task 3 must be completed first

### Purpose

Create unit tests covering all migration scenarios including no-op, version upgrades, backup rotation, dry-run mode, failures, and idempotency.

### Test Coverage Requirements

**Test File Location**: `core/src/migrate.rs` (in `#[cfg(test)] mod tests`)

**Required Test Cases**:
1. No-op migration (v1.0.0 → v1.0.0, no changes)
2. Version upgrade (v1.0.0 → v1.1.0, schema transformed)
3. Backup rotation (verify .bak1/.bak2/.bak3 files created)
4. Dry-run mode (no files written, shows intended changes)
5. Migration failure (invalid source version)
6. Migration failure (unknown target version)
7. Idempotency (run twice, verify same result)
8. Backup rotation overflow (4th migration rotates oldest .bak3 out)
9. Directory migration (recursive vs non-recursive)
10. Auto-detect source version (from omitted)

### Test Data Setup

- Create temporary test directories
- Generate sidecars with different schema versions
- Pre-create existing .bak1/.bak2 files to test rotation
- Create corrupt sidecars to test error handling

### Assertions

- Verify schema_version field updated correctly
- Verify backup files created in correct order
- Verify dry-run produces no file changes
- Verify idempotent migrations (hash before == hash after on second run)
- Verify error messages for invalid migrations

### Acceptance Criteria

- [ ] All 10+ test scenarios pass
- [ ] Tests verify backup file content matches original
- [ ] Tests verify schema transformations applied correctly
- [ ] `cargo test --package jozin-core migrate` passes
- [ ] Tests clean up temporary files

---

## Task 5: Validate Phase 1 Completion

**Status**: 🔴 Pending
**Priority**: High (gates transition to Phase 2)
**Estimated Effort**: Small
**Dependencies**: Tasks 1-4 must be completed first

### Purpose

Run full test suite, verify all acceptance criteria are met, ensure zero compiler warnings, validate CLI help output, and document Phase 2 requirements.

### Validation Checklist

**Phase 1 Acceptance Criteria**:
- [ ] CLI callable for all modules (scan, cleanup, verify, migrate)
- [ ] Each parameter validated and returns JSON response
- [ ] Inner functional structure for each task implemented
- [ ] All modules handle paths, globs, dry-run correctly
- [ ] Exit codes match specification (0/1/2/3/4)

**Build Health**:
- [ ] `cargo build --workspace` succeeds with zero warnings
- [ ] `cargo test --workspace` passes all tests (target: 80+ tests)
- [ ] `cargo clippy --workspace` reports no issues
- [ ] `cargo doc --workspace --no-deps` builds without errors

**CLI Validation**:
- [ ] `jozin --help` shows all subcommands
- [ ] `jozin scan --help` shows all options with defaults
- [ ] `jozin cleanup --help` shows all options
- [ ] `jozin verify --help` shows all options
- [ ] `jozin migrate --help` shows all options
- [ ] Help text includes examples

**Documentation Updates**:
- [ ] Update TASKMASTER_PLAN.md with completion status
- [ ] Update README.md with current test count
- [ ] Update .agent/ documentation
- [ ] Document known limitations and edge cases

### Phase 2 Readiness Assessment

Create `PHASE2_ROADMAP.md` documenting:
1. Bounded Parallelism (--max-threads, rayon)
2. Journal Support (jozin.journal.ndjson)
3. Progress Events API (real-time callbacks)
4. Advanced Hashing (pHash, pixel hash)
5. Test Matrix Expansion (Unicode, symlinks, large files, cross-platform)

### Acceptance Criteria

- [ ] All Phase 1 acceptance criteria verified
- [ ] Build health confirmed (zero warnings)
- [ ] All CLI help text validated
- [ ] Test count increased to 80+ tests
- [ ] PHASE2_ROADMAP.md created

---

## Task Dependencies

```
Task 1 (verify impl) ──→ Task 2 (verify tests) ──┐
                                                   │
Task 3 (migrate impl) ─→ Task 4 (migrate tests) ──┼──→ Task 5 (Phase 1 validation)
```

---

## Execution Strategy

### Recommended Order

1. **Day 1-2**: Task 1 + Task 2 (verify module)
2. **Day 3-4**: Task 3 + Task 4 (migrate module)
3. **Day 5**: Task 5 (Phase 1 validation)

### Success Metrics

- All 80+ tests passing (current: 59)
- Zero compiler warnings
- All Phase 1 acceptance criteria met
- Clear Phase 2 roadmap documented

---

## Related Documentation

- **TASKMASTER_PLAN.md** - Detailed task specifications (Tasks 1-7)
- **.agent/System/project_architecture.md** - Technical architecture
- **SCOPE.md** - Architectural constraints
- **README.md** - Developer onboarding
- **CLAUDE.md** - AI assistant guidance

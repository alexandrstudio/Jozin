# Jožin — Tasks & Phased Plan

## Modules
- scan
- faces
- tags
- thumbs
- verify
- migrate

## Global conventions
- **CLI shape**: jozin <module> [subcmd] [options]
- **Tauri bridge**: mirror CLI with invoke("<module>_<action>", params)
- **Return type**: machine-readable JSON on stdout (CLI) / structured object (Tauri)
- **Time/Perf**: every op returns { started_at, finished_at, duration_ms }
- **Dry-run**: --dry-run never writes; prints intended actions + reasons
- **Paths**: accept files OR directories; --recursive for deep scan
- **Globs**: --include "*.jpg,*.png", --exclude "**/.jozin/**"
- **Concurrency**: --max-threads N (bounded), default = min(2×CPU, 8)
- **Exit codes**: 0 ok, 1 user error, 2 IO, 3 validation, 4 internal

## Parameters (by module)

### scan
- **path**: string (file/dir)
- **recursive**: bool
- **include**: string[] (globs)
- **exclude**: string[] (globs)
- **dry_run**: bool
- **max_threads**: u16
- **hash_mode**: "file"|"pixel"|"both" (phase 2+)

Output: array of { path, actions: ["write"|"skip"|"update"], duration_ms }

### faces
- **path**: string (file/dir)
- **recursive**: bool
- **model**: string (e.g. arcface-1.4)
- **identify**: bool (match embeddings to known persons)
- **train**: Array<{ person: string, images: string[] }>?
- **min_score**: f32 (default 0.8)
- **dry_run**: bool
- **max_threads**: u16

Output: per file { faces: [{bbox, score, embedding_hash, person?}] }

### tags
- **path**: string (file/dir)
- **recursive**: bool
- **mode**: "ml"|"rules"|"both"
- **model**: string?
- **min_score**: f32 (default 0.6)
- **append**: bool (keep user labels; don’t overwrite)
- **dry_run**: bool

Output: per file { added: string[], scores: Record<string, number> }

### thumbs
- **path**: string (file/dir)
- **recursive**: bool
- **sizes**: string (e.g. "256,512")
- **format**: "jpg"|"webp"
- **quality**: u8 (1–100)
- **overwrite**: bool
- **dry_run**: bool
- **max_threads**: u16

Output: per file { generated: string[], skipped: string[] }

### verify
- **path**: string (file/dir)
- **recursive**: bool
- **fix**: bool (attempt auto-repair of minor issues)
- **strict**: bool (treat warnings as errors)
- **pipeline_signature**: string? (override)

Output: per file { status: "ok"|"stale"|"missing"|"corrupt", reasons: string[], suggested: "noop"|"rescan"|"migrate" }

### migrate
- **path**: string (file/dir)
- **recursive**: bool
- **from**: string? (detect if absent)
- **to**: string (target schema)
- **dry_run**: bool
- **backup**: bool (write .bakN)

Output: per file { migrated: bool, from, to, backup_path? }

## Phases

### Phase 0 — Init & Parameter Handling

#### Goal: Wire CLI + Tauri commands. Functions exist and print structured “called with params”.

#### Tasks (all modules):
	1.	CLI subcommands + --help with examples.
	2.	Parse & validate params.
	3.	Unit tests: param parsing (valid/invalid), help text, exit codes.

#### Acceptance:
- jozin scan --help shows all options with defaults.
- jozin scan <dir> --dry-run prints JSON with parsed params.

### Phase 1 — Minimal Functional Core (by module)

#### scan
- Task 1: Walk directory (respect include/exclude), count files, print duration.
- Task 2: Compute BLAKE3 file hash; add source.file_hash_b3.
- Task 3: Read EXIF (datetime, camera, orientation, gps); add to image{}.
- Tests: large dir, unreadable file, long paths, unicode, symlink loops.

#### faces
- Task 1: Stub detection → returns empty faces array + timing.
- Tests: images with/ohne faces; thresholds; corrupted files.

#### tags
- Task 1: Rules mode (cheap heuristics: filename, EXIF context).
- Task 2: ML mode (local model), merge with user labels if append.
- Tests: merge policy, scores, deterministic results.

#### thumbs
- Task 1: Generate single size (default 512 jpg 85).
- Task 2: Multiple sizes, webp, overwrite policy, atomic write.
- Tests: gamma, orientation handling, color profile passthrough.

#### verify
- Task 1: Validate schema shape & required fields.
- Task 2: Staleness rules: compare schema_version, producer_version, pipeline_signature, and file_hash_b3.
- Tests: each reason path → suggested action.

#### migrate
- Task 1: No-op migrator (v1→v1).
- Task 2: Add sample migration (e.g., split camera into make/model).
- Tests: idempotency, backup rotation, dry-run diff.

### Phase 2 — Robustness & Performance
- Bounded parallelism across modules; --max-threads.
- Journal append (.jozin/_journal.ndjson) for ops.
- Progress events API (for UI progress bars).
- Perceptual hash (pHash) in scan (optional).
- Pixel hash (normalized decode) in scan (optional).

#### Acceptance:
- On 100k files: no unbounded memory growth; stable throughput.
- Killing mid-scan is recoverable; rerun continues safely.

###  Test Matrix (minimum)
- Paths: deep trees, unicode, spaces, symlinks, network mounts.
- Files: large JPEG/PNG/HEIC/RAW; corrupt headers; missing EXIF.
- OS: macOS, Windows, Linux (CRLF/permissions cases).
- Concurrency: --max-threads 1 vs many; starvation checks.
- Determinism: same inputs → same outputs (hashes, tags with fixed seeds).

### Acceptance Criteria for - Phase 1
- CLI callable app.
- Each parameter is validated and output a response that it has been called.
- Inner functional structure for each task.

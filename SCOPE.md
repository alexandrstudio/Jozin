# Jožin — SCOPE

> **Jožin — the friendly monster that organizes your photo swamp.**

## Purpose (Global)
Organize large, messy local photo libraries **without ever touching originals**. Jožin scans, fingerprints, and groups images into meaningful structures (duplicates, near‑duplicates, series, people), stores facts in JSON sidecars and a database, and exposes them to a UI/API for search, curation, and optional thumbnail‑only sharing.

## Non‑Goals
- No cloud upload of original files.
- No destructive edits; no lossy batch conversions.
- No training on user data.

## Core Principles
- **Local‑first & reversible:** Originals are read‑only. All derived data (JSON, thumbs) can be rebuilt.
- **Worker composition:** Small, focused processes with strict interfaces.
- **Contract‑driven:** JSON Schemas & API specs are the source of truth.
- **Performance by design:** Rust workers, constant memory bounds, parallel I/O.
- **Privacy by default:** Faces → embeddings; sharing → thumbnails/metadata only.

## High‑Level Architecture
- **Workers (local, Rust)**
  - `worker-scan`: discover files, extract EXIF, compute hashes, produce sidecars.
  - `worker-series`: graph‑based clustering for bursts, brackets, focus stacks, panoramas, timelapses.
  - `worker-face` (pluggable): detect, align, embed faces (Python service v1; Rust+ONNX later).
  - `worker-thumbs`: create previews/thumbnails for UI/search.
- **Server (optional, Docker)**
  - DB (SQLite/Postgres), API (Next.js/Laravel/Hono), HNSW index, thumbnail server.
  - Self‑hosted or cloud; **never** stores originals—only thumbs/metadata.

## Folder Layout (Monorepo)
```bash
jozin/
├─ SCOPE/
│  ├─ SCOPE.md                   # this file (global scope)
│  ├─ worker-scan.md             # per‑worker scope docs (to be added)
│  ├─ worker-series.md
│  ├─ worker-face.md
│  └─ worker-thumbs.md
├─ contracts/
│  ├─ jsonschema/                # machine‑readable data contracts
│  │  ├─ image-sidecar.schema.json
│  │  ├─ face-embedding.schema.json
│  │  └─ series-record.schema.json
│  └─ api/
│     └─ face-service.openapi.yaml
├─ workers/                      # Rust workspace: small, focused binaries
│  ├─ scan/
│  ├─ series/
│  ├─ face/
│  └─ thumbs/
├─ services/
│  └─ face-service/              # Python FastAPI (v1), Dockerized
├─ apps/
│  ├─ web/                       # optional UI (Next.js)
│  └─ api/                       # optional API (Laravel/Hono)
├─ deploy/
│  ├─ docker-compose.yml
│  └─ k8s/                       # optional later
├─ Cargo.toml                    # Rust workspace
├─ justfile or Makefile          # cross‑language tasks (build/test/run)
├─ .editorconfig
├─ .gitattributes
└─ .gitignore
```

## Deliberate Choice
> **No Turborepo:** Cargo workspace + Make/Just + Docker Compose keep it lean and polyglot.

### Why Rust for Workers
- **Performance & safety:** Low-latency I/O and hashing without GC.
- **Single binary shipping:** One executable per worker, zero user-side deps.
- **Portability:** macOS/Windows/Linux friendly; easy to sandbox.

### Data Contracts (overview)
- **Image sidecar:** intrinsic dims, EUS, EXIF, hashes `{sha256, phash64, dhash64, bmh64, cmh192, tiled?}`, sync flags.
- **Series record:** list of image IDs, type (burst/bracket/focus/pano/timelapse/generic), span, representative, metadata.
- **Face embedding:** `{bbox, landmarks?, embedding[512], model, version}`.

> (Full JSON Schemas live under `/contracts/jsonschema`.)

## Supported Formats (initial)
**Raster:** JPEG, PNG, GIF, BMP, TIFF
**RAW (roadmap):** CR2, CR3, NEF, NRW, ARW, RAF, RW2, ORF, X3F, X3I, DNG, HEIC/HEIF, GPR, PEF

## Similarity & Dedup (summary)
- **Exact duplicates:** Canonicalize (EXIF orientation → sRGB → fixed mode) → SHA‑256 of pixels.
- **Near-duplicates:** pHash(64) + dHash(64) + BlockMean(64) + ColorMoment(192/256); match if ≥3/4 within thresholds.
- **Crops/overlays:** Tiled pHash/dHash; tile-overlap ≥ 0.65.
- **Indexing:** BK‑tree per 64‑bit hash; pre-filter by `(width,height)` and/or EUS.
- **Similar scenes:** Optional CLIP/DINO embeddings + HNSW; cosine ≥ 0.9 as soft gate.

## Face Recognition (summary)
- Detect → align → embed (e.g., ArcFace 512‑D).
  Store embeddings in sidecars/DB; originals untouched. Matching by cosine/Euclidean.
  Python service v1; Rust+ONNX later.

## Privacy & Security
- Originals are read-only and never leave the machine.
- Sharing only publishes thumbnails/metadata.
- Embeddings can be quantized and salted for identity-only matching.

## Performance Targets (reference machine)
- Scan ≥ 150 images/sec (SSD) with hashing & EXIF.
- Series clustering O(N log N) via aggressive prefilters.
- Face inference ≤ 40 ms/face (CPU) or ≤ 8 ms (GPU) when available.

## CLI & Ops (root)
```bash
just build   # cargo build --workspace --release
just test    # run Rust tests (+ Python unit tests if present)
just up      # docker compose up -d
just face    # start local face-service
```

## Roadmap
1. Implement worker-scan + sidecars/DB ingest.
2. Add worker-series clustering & representatives.
3. Stand up services/face-service (Python v1).
4. Implement worker-thumbs + UI search.
5. Optional: replace face-service with Rust+ONNX.

## Index of Worker Scopes

- [jozan-worker: Scans and fingerprints local image libraries, generating detailed JSON sidecars for every file.](./SCOPE/jozan-worker.md)
- [jozync-worker: Synchronizes JSON sidecars with the central database — imports scans, exports user changes, and keeps everything in perfect - balance.](./SCOPE/jozync-worker.md)
- [joztag-worker: Face detection, recognition, and tagging.](./SCOPE/joztag-worker.md)
- [jozimg-worker: Thumbnails and image adjustments.](./SCOPE/jozimg-worker.md)

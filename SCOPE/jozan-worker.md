# Jožin Worker — Scan (jozan)

## Purpose
The **jozan worker** is the foundation of Jožin’s ecosystem. It scans local directories for supported image files, analyzes them, and generates metadata-rich JSON sidecars that describe each image without modifying the originals. The data collected by jozan becomes the core input for all other workers.

---

## Responsibilities
1. Traverse user-defined directories recursively.
2. Identify supported image formats (JPEG, PNG, GIF, BMP, TIFF, RAW variants).
3. Extract key file and image attributes:
   - Filename and path
   - File size (bytes)
   - Image dimensions (pixels)
   - Format / MIME type
   - Creation, modification, and EXIF timestamps
   - EXIF and GPS metadata
   - Camera information (make, model, lens)
   - Exposure metadata (ISO, aperture, shutter speed)
   - OCR text (optional)
4. Compute image fingerprints and metrics:
   - SHA‑256 (exact duplicate ID)
   - pHash, dHash, BlockMean, ColorMoment, and optionally Wavelet hash
   - Equivalent Uncompressed Size (EUS)
5. Store analysis results as a sidecar JSON file next to the image with the same filename (e.g., `IMG_1234.jpg.json`).
6. Optionally send the parsed data to the central database or API endpoint for indexing.

---

## Data Output (JSON Sidecar)
Example:
```json
{
  "imageId": "sha256:abcd1234...",
  "path": "/photos/2025/IMG_1234.JPG",
  "format": "jpeg",
  "bytes": 3849216,
  "dims": {"w": 4032, "h": 3024},
  "exif": {
    "make": "Canon",
    "model": "EOS R5",
    "focal": 50.0,
    "aperture": 1.8,
    "iso": 100,
    "exposure": "1/200",
    "ts": "2025-05-10T18:22:15Z",
    "gps": [50.083, 14.417]
  },
  "ocr": null,
  "hashes": {
    "sha256": "abcd1234...",
    "phash64": "0xA1B2C3...",
    "dhash64": "0xD1E2F3...",
    "bmh64": "0xB1B2B3...",
    "cmh192": "0xC1C2C3..."
  },
  "eus": 36578304,
  "sync": {"written": true, "dirty": false, "lastWrite": "2025-10-13T15:05:00Z"},
  "schema_version": "1.0.0",
  "worker_version": "0.1.0"
}
```

## CLI Design

```
jozan-scan [options] <path>

Options:
  -r, --recursive          Scan subdirectories recursively
  -o, --output <dir>       Output directory for JSON sidecars (default: next to image)
  -f, --formats <list>     Comma-separated list of formats to include
  -e, --exclude <pattern>  Glob pattern to exclude
  -j, --jobs <n>           Parallel jobs (default: CPU cores)
  -u, --upload             Send data to API endpoint
  -v, --verbose            Verbose output
  -q, --quiet              Minimal output
  -h, --help               Show usage
```

---

## Performance Targets
- ≥150 images/second on SSD (JPEG mix)
- Memory footprint ≤100 MB for 100k image scans
- Parallelized using Rayon (CPU cores auto‑detected)

---

## Future Extensions
- OCR module for embedded text recognition (Tesseract integration)
- Hash‑only mode for quick duplicate detection
- File change watcher for incremental updates
- Metadata diff engine to flag inconsistencies

---

## Acceptance Criteria
- Correctly handles all supported image formats and produces valid JSON sidecars.
- Ignores unsupported or corrupted files gracefully.
- JSON validated against image-sidecar.schema.json.
- Parallel execution yields deterministic results.
- 95% unit test coverage on parsing and hashing functions.

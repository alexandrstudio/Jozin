//! Scan module - Directory traversal, EXIF extraction, BLAKE3 hashing
//!
//! This module is responsible for:
//! - Walking directory trees
//! - Reading image files
//! - Extracting EXIF metadata (Phase 1: basic file info only)
//! - Computing BLAKE3 hashes
//! - Writing JSON sidecar files atomically
//!
//! ## Phase 1 Implementation
//!
//! This implementation provides:
//! - File metadata extraction (size, modified time)
//! - BLAKE3 hash computation
//! - Atomic sidecar writing with backup rotation
//! - Directory traversal with recursive option
//! - Glob pattern filtering (include/exclude)
//! - Sequential processing (parallelism in Phase 2)
//! - Basic error handling
//!
//! Future phases will add:
//! - EXIF parsing (Phase 1+)
//! - Parallel processing with rayon (Phase 2)
//! - Progress reporting (Phase 2)
//!
//! ## Usage
//!
//! For most use cases, use [`scan_path()`] which handles both files and directories:
//!
//! ```no_run
//! use jozin_core::scan::scan_path;
//! use std::path::Path;
//!
//! // Scan a single file
//! let result = scan_path(
//!     Path::new("/photos/IMG_1234.JPG"),
//!     false,  // recursive
//!     None,   // include patterns
//!     None,   // exclude patterns
//!     false,  // dry_run
//!     4,      // max_threads (unused in Phase 1)
//!     Some("file"), // hash_mode
//!     None,   // progress_callback
//! )?;
//! println!("Scanned {} files", result.successful);
//!
//! // Scan a directory recursively with filtering
//! let result = scan_path(
//!     Path::new("/photos"),
//!     true,   // recursive
//!     Some(&[String::from("*.jpg"), String::from("*.png")]),
//!     Some(&[String::from("**/.jozin/**")]),
//!     false,  // dry_run
//!     8,      // max_threads
//!     Some("file"),
//!     None,   // progress_callback
//! )?;
//! # Ok::<(), jozin_core::JozinError>(())
//! ```
//!
//! For low-level single-file operations, use [`scan_file()`] directly.

use crate::{JozinError, PipelineSignature, Result, Sidecar, SourceInfo};
use globset::{Glob, GlobSetBuilder};
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use time::OffsetDateTime;
use walkdir::WalkDir;

// ============================================================================
// Constants
// ============================================================================

/// Supported image file extensions (lowercase).
///
/// This list covers common image formats per TASK+PHASE_PLAN.md line 149.
/// Phase 2+ may add magic byte detection for more robust identification.
///
/// Categories:
/// - JPEG variants: jpg, jpeg
/// - PNG: png
/// - HEIC/HEIF (Apple formats): heic, heif
/// - RAW formats: raw (generic), cr2 (Canon), nef (Nikon), arw (Sony), dng (Adobe)
/// - TIFF: tiff, tif
/// - WebP: webp
const SUPPORTED_EXTENSIONS: &[&str] = &[
    "jpg", "jpeg", "png", "heic", "heif", "raw", "cr2", "nef", "arw", "dng", "tiff", "tif",
    "webp",
];

// ============================================================================
// Public Types
// ============================================================================

/// Result of scanning files or directories.
///
/// This struct provides detailed results for CLI/Tauri JSON output per
/// TASK+PHASE_PLAN.md line 33. It includes both summary counts and detailed
/// per-file information.
///
/// # Fields
///
/// - `scanned_files`: Detailed results for each file processed
/// - `total_files`: Total number of files discovered (including skipped/failed)
/// - `successful`: Number of files successfully scanned with sidecars written
/// - `failed`: Number of files that failed to scan (errors)
/// - `skipped`: Number of files skipped by filters or dry_run
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    pub scanned_files: Vec<ScannedFile>,
    pub total_files: usize,
    pub successful: usize,
    pub failed: usize,
    pub skipped: usize,
}

/// Detailed information about a single scanned file.
///
/// This struct represents the result of scanning one file, including
/// the action taken, paths, and metadata.
///
/// # Fields
///
/// - `path`: Absolute or relative path to the file that was scanned
/// - `action`: What happened during scanning (written/skipped/failed)
/// - `sidecar_path`: Path to the generated sidecar (only if action is Written)
/// - `error`: Error message (only if action is Failed)
/// - `hash`: BLAKE3 hash of file contents (only if successful)
/// - `size_bytes`: File size in bytes (only if successful)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScannedFile {
    pub path: String,
    pub action: ScanAction,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sidecar_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size_bytes: Option<u64>,
}

/// Action taken when scanning a file.
///
/// This enum represents the outcome of scanning a single file.
///
/// # Variants
///
/// - `Written`: Sidecar was successfully created or updated
/// - `Skipped`: File was skipped (dry_run mode or filtered out)
/// - `Failed`: Scan failed with an error
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ScanAction {
    Written,
    Skipped,
    Failed,
}

// ============================================================================
// Public API
// ============================================================================

/// Scans a path (file or directory) and generates sidecar metadata.
///
/// This is the main entry point for the scan module, handling both single files
/// and directory trees. It supports recursive traversal, glob pattern filtering,
/// and dry-run mode.
///
/// # Arguments
///
/// * `path` - File or directory to scan
/// * `recursive` - Enable recursive directory traversal (ignored for files)
/// * `include` - Glob patterns to include (e.g., `["*.jpg", "*.png"]`). If specified,
///               only files matching at least one pattern are scanned.
/// * `exclude` - Glob patterns to exclude (e.g., `["**/.jozin/**"]`). Files matching
///               any exclude pattern are skipped. Applied before include patterns.
/// * `dry_run` - Preview mode: compute metadata but don't write sidecars
/// * `max_threads` - Parallelism limit (Phase 1: unused, processes sequentially.
///                   Phase 2: will use rayon for parallel processing)
/// * `hash_mode` - Hash computation strategy: "file", "pixel", or "both"
///                 (Phase 1: only "file" is supported)
/// * `progress_callback` - Optional callback for real-time progress reporting
///
/// # Returns
///
/// Returns a [`ScanResult`] containing:
/// - List of all processed files with their outcomes
/// - Summary counts (total, successful, failed, skipped)
///
/// # Errors
///
/// - `JozinError::IoError` if path doesn't exist or cannot be accessed
/// - `JozinError::ValidationError` if path is neither a file nor a directory
///
/// Individual file scan failures do not fail the entire operation; they are
/// captured in the `failed` count and returned in `scanned_files` with error details.
///
/// # Examples
///
/// ```no_run
/// use jozin_core::scan::scan_path;
/// use std::path::Path;
///
/// // Scan a single file
/// let result = scan_path(
///     Path::new("/photos/IMG_1234.JPG"),
///     false, None, None, false, 4, Some("file"), None
/// )?;
/// assert_eq!(result.total_files, 1);
///
/// // Scan directory with filtering
/// let result = scan_path(
///     Path::new("/photos"),
///     true,
///     Some(&[String::from("*.jpg")]),
///     Some(&[String::from("**/.jozin/**")]),
///     false,
///     8,
///     Some("file"),
///     None
/// )?;
/// println!("Scanned {} files", result.successful);
/// # Ok::<(), jozin_core::JozinError>(())
/// ```
pub fn scan_path(
    path: &Path,
    recursive: bool,
    include: Option<&[String]>,
    exclude: Option<&[String]>,
    dry_run: bool,
    _max_threads: u16, // Unused in Phase 1, will be used in Phase 2 for rayon
    _hash_mode: Option<&str>, // Phase 1: only "file" mode is supported
    progress_callback: Option<&dyn Fn(crate::ProgressEvent)>,
) -> Result<ScanResult> {
    // Validate path exists
    if !path.exists() {
        return Err(JozinError::IoError {
            message: format!("Path not found: {}", path.display()),
        });
    }

    // Handle single file
    if path.is_file() {
        // Validate it's an image file
        if !is_image_file(path) {
            return Err(JozinError::ValidationError {
                message: format!("Not an image file: {}", path.display()),
            });
        }

        // Scan the file
        match scan_file(path, dry_run) {
            Ok(sidecar) => {
                let action = if dry_run {
                    ScanAction::Skipped
                } else {
                    ScanAction::Written
                };
                let scanned_file = ScannedFile {
                    path: path.display().to_string(),
                    action,
                    sidecar_path: if dry_run {
                        None
                    } else {
                        Some(get_sidecar_path(path).display().to_string())
                    },
                    error: None,
                    hash: Some(sidecar.source.file_hash_b3),
                    size_bytes: Some(sidecar.source.file_size_bytes),
                };

                Ok(ScanResult {
                    scanned_files: vec![scanned_file],
                    total_files: 1,
                    successful: if dry_run { 0 } else { 1 },
                    failed: 0,
                    skipped: if dry_run { 1 } else { 0 },
                })
            }
            Err(e) => {
                let scanned_file = ScannedFile {
                    path: path.display().to_string(),
                    action: ScanAction::Failed,
                    sidecar_path: None,
                    error: Some(e.to_string()),
                    hash: None,
                    size_bytes: None,
                };

                Ok(ScanResult {
                    scanned_files: vec![scanned_file],
                    total_files: 1,
                    successful: 0,
                    failed: 1,
                    skipped: 0,
                })
            }
        }
    }
    // Handle directory
    else if path.is_dir() {
        scan_directory(path, recursive, include, exclude, dry_run, progress_callback)
    }
    // Path exists but is neither file nor directory (e.g., socket, pipe)
    else {
        Err(JozinError::ValidationError {
            message: format!("Path is neither a file nor a directory: {}", path.display()),
        })
    }
}

/// Scans a single file and generates its sidecar metadata.
///
/// This function:
/// 1. Reads file metadata (size, modification time)
/// 2. Computes BLAKE3 hash of file contents
/// 3. Creates a Sidecar struct with current pipeline signature
/// 4. Writes sidecar atomically to `<file_path>.json`
///
/// # Arguments
///
/// * `file_path` - Path to the image file to scan
/// * `dry_run` - If true, don't write the sidecar (just compute metadata)
///
/// # Returns
///
/// Returns the generated `Sidecar` structure.
///
/// # Errors
///
/// - `JozinError::IoError` if file cannot be read or sidecar cannot be written
/// - `JozinError::ValidationError` if file path is invalid
///
/// # Example
///
/// ```no_run
/// use jozin_core::scan::scan_file;
/// use std::path::Path;
///
/// let sidecar = scan_file(Path::new("/photos/IMG_1234.JPG"), false)?;
/// println!("Hash: {}", sidecar.source.file_hash_b3);
/// # Ok::<(), jozin_core::JozinError>(())
/// ```
pub fn scan_file(file_path: &Path, dry_run: bool) -> Result<Sidecar> {
    // Validate path
    if !file_path.exists() {
        return Err(JozinError::IoError {
            message: format!("File not found: {}", file_path.display()),
        });
    }

    if !file_path.is_file() {
        return Err(JozinError::ValidationError {
            message: format!("Path is not a file: {}", file_path.display()),
        });
    }

    // Read file metadata
    let metadata = fs::metadata(file_path)?;
    let file_size_bytes = metadata.len();

    // Get modification time
    let modified = metadata.modified()?;
    let modified_offset = OffsetDateTime::from(modified);
    let file_modified_at = modified_offset
        .format(&time::format_description::well_known::Rfc3339)
        .map_err(|e| JozinError::InternalError {
            message: format!("Failed to format modification time: {}", e),
        })?;

    // Compute BLAKE3 hash
    let file_hash_b3 = compute_blake3_hash(file_path)?;

    // Create pipeline signature
    let now = OffsetDateTime::now_utc();
    let created_at = now
        .format(&time::format_description::well_known::Rfc3339)
        .map_err(|e| JozinError::InternalError {
            message: format!("Failed to format timestamp: {}", e),
        })?;

    let pipeline_signature = PipelineSignature {
        schema_version: "1.0.0".to_string(),
        producer_version: env!("CARGO_PKG_VERSION").to_string(),
        hash_algorithm: "blake3".to_string(),
        face_model: None,
        tag_model: None,
        created_at: created_at.clone(),
    };

    // Build sidecar
    let sidecar = Sidecar {
        schema_version: "1.0.0".to_string(),
        producer_version: env!("CARGO_PKG_VERSION").to_string(),
        created_at: created_at.clone(),
        updated_at: created_at,
        pipeline_signature,
        source: SourceInfo {
            file_path: file_path.display().to_string(),
            file_size_bytes,
            file_hash_b3,
            file_modified_at,
        },
        image: None, // EXIF parsing to be added in Phase 1+
        faces: Vec::new(),
        tags: Vec::new(),
        thumbnails: Vec::new(),
    };

    // Write sidecar atomically (unless dry_run)
    if !dry_run {
        write_sidecar(file_path, &sidecar)?;
    }

    Ok(sidecar)
}

// ============================================================================
// Internal Helpers
// ============================================================================

/// Scans a directory and generates sidecar metadata for all image files.
///
/// This function orchestrates directory traversal with filtering. In Phase 1,
/// it processes files sequentially. Phase 2 will add parallel processing using rayon.
///
/// The filtering logic is applied in this order for performance:
/// 1. Skip directories (only process files)
/// 2. Apply exclude patterns (early rejection)
/// 3. Apply include patterns (if specified)
/// 4. Check image file extension
/// 5. Scan the file
///
/// # Arguments
///
/// * `dir_path` - Directory to scan
/// * `recursive` - Traverse subdirectories
/// * `include` - Include glob patterns (e.g., `["*.jpg", "*.png"]`)
/// * `exclude` - Exclude glob patterns (e.g., `["**/.jozin/**"]`)
/// * `dry_run` - Preview mode without writing sidecars
///
/// # Returns
///
/// Returns a [`ScanResult`] with all files processed, including successes,
/// failures, and skipped files.
///
/// # Error Handling
///
/// Individual file scan failures do not stop the entire operation. Errors are
/// collected in the result structure rather than failing fast. This ensures
/// that one corrupted or unreadable file doesn't prevent scanning the rest.
fn scan_directory(
    dir_path: &Path,
    recursive: bool,
    include: Option<&[String]>,
    exclude: Option<&[String]>,
    dry_run: bool,
    progress_callback: Option<&dyn Fn(crate::ProgressEvent)>,
) -> Result<ScanResult> {
    // Initialize result
    let mut result = ScanResult {
        scanned_files: Vec::new(),
        total_files: 0,
        successful: 0,
        failed: 0,
        skipped: 0,
    };

    // Build glob matchers
    let exclude_matcher = if let Some(patterns) = exclude {
        Some(build_glob_matcher(patterns)?)
    } else {
        None
    };

    let include_matcher = if let Some(patterns) = include {
        Some(build_glob_matcher(patterns)?)
    } else {
        None
    };

    // Configure directory walker
    let walker = if recursive {
        WalkDir::new(dir_path)
    } else {
        WalkDir::new(dir_path).max_depth(1)
    };

    // Iterate through directory entries
    for entry in walker {
        // Handle walkdir errors (permission denied, etc.)
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                // Log error but continue scanning
                eprintln!("Warning: Failed to access entry: {}", e);
                continue;
            }
        };

        let path = entry.path();

        // Skip directories, only process files
        if !path.is_file() {
            continue;
        }

        // Apply exclude patterns first (early rejection for performance)
        if let Some(ref matcher) = exclude_matcher {
            if matcher.is_match(path) {
                result.total_files += 1;
                result.skipped += 1;
                result.scanned_files.push(ScannedFile {
                    path: path.display().to_string(),
                    action: ScanAction::Skipped,
                    sidecar_path: None,
                    error: Some("Excluded by pattern".to_string()),
                    hash: None,
                    size_bytes: None,
                });
                continue;
            }
        }

        // Apply include patterns (if specified, file must match at least one)
        if let Some(ref matcher) = include_matcher {
            if !matcher.is_match(path) {
                result.total_files += 1;
                result.skipped += 1;
                result.scanned_files.push(ScannedFile {
                    path: path.display().to_string(),
                    action: ScanAction::Skipped,
                    sidecar_path: None,
                    error: Some("Not included by pattern".to_string()),
                    hash: None,
                    size_bytes: None,
                });
                continue;
            }
        }

        // Check if file is an image by extension
        if !is_image_file(path) {
            result.total_files += 1;
            result.skipped += 1;
            result.scanned_files.push(ScannedFile {
                path: path.display().to_string(),
                action: ScanAction::Skipped,
                sidecar_path: None,
                error: Some("Not an image file (unsupported extension)".to_string()),
                hash: None,
                size_bytes: None,
            });
            continue;
        }

        // Scan the file
        result.total_files += 1;

        // Emit FileStarted event
        if let Some(callback) = progress_callback {
            callback(crate::ProgressEvent::FileStarted {
                path: path.display().to_string(),
            });
        }

        match scan_file(path, dry_run) {
            Ok(sidecar) => {
                let action = if dry_run {
                    ScanAction::Skipped
                } else {
                    ScanAction::Written
                };
                result.scanned_files.push(ScannedFile {
                    path: path.display().to_string(),
                    action,
                    sidecar_path: if dry_run {
                        None
                    } else {
                        Some(get_sidecar_path(path).display().to_string())
                    },
                    error: None,
                    hash: Some(sidecar.source.file_hash_b3),
                    size_bytes: Some(sidecar.source.file_size_bytes),
                });
                if dry_run {
                    result.skipped += 1;
                } else {
                    result.successful += 1;
                }

                // Emit FileCompleted event for success
                if let Some(callback) = progress_callback {
                    callback(crate::ProgressEvent::FileCompleted {
                        path: path.display().to_string(),
                        success: true,
                        error: None,
                        size_bytes: Some(sidecar.source.file_size_bytes),
                    });
                }
            }
            Err(e) => {
                result.scanned_files.push(ScannedFile {
                    path: path.display().to_string(),
                    action: ScanAction::Failed,
                    sidecar_path: None,
                    error: Some(e.to_string()),
                    hash: None,
                    size_bytes: None,
                });
                result.failed += 1;

                // Emit FileCompleted event for failure
                if let Some(callback) = progress_callback {
                    callback(crate::ProgressEvent::FileCompleted {
                        path: path.display().to_string(),
                        success: false,
                        error: Some(e.to_string()),
                        size_bytes: None,
                    });
                }
            }
        }
    }

    Ok(result)
}

/// Builds a GlobSet matcher from a list of glob patterns.
///
/// This function compiles multiple glob patterns into a single efficient
/// matcher for use with include/exclude filtering. Invalid patterns are
/// logged as warnings and treated as non-matching.
///
/// # Glob Syntax
///
/// Supports standard glob syntax per TASK+PHASE_PLAN.md line 18:
/// - `*`: Matches any characters except path separators
/// - `**`: Matches any characters including path separators (recursive)
/// - `?`: Matches a single character
/// - `[abc]`: Matches any character in the set
///
/// # Arguments
///
/// * `patterns` - List of glob pattern strings (e.g., `["*.jpg", "**/.jozin/**"]`)
///
/// # Returns
///
/// Returns a compiled `GlobSet` that can efficiently match paths against
/// all patterns.
///
/// # Errors
///
/// Returns `JozinError::ValidationError` if any pattern is invalid.
fn build_glob_matcher(patterns: &[String]) -> Result<globset::GlobSet> {
    let mut builder = GlobSetBuilder::new();

    for pattern in patterns {
        match Glob::new(pattern) {
            Ok(glob) => {
                builder.add(glob);
            }
            Err(e) => {
                return Err(JozinError::ValidationError {
                    message: format!("Invalid glob pattern '{}': {}", pattern, e),
                });
            }
        }
    }

    builder.build().map_err(|e| JozinError::InternalError {
        message: format!("Failed to build glob matcher: {}", e),
    })
}

/// Checks if a file is an image based on its extension.
///
/// This function filters files by extension to identify images. The extension
/// list covers common formats from TASK+PHASE_PLAN.md line 149:
/// - JPEG variants: .jpg, .jpeg
/// - PNG: .png
/// - HEIC/HEIF (Apple formats): .heic, .heif
/// - RAW formats: .raw, .cr2 (Canon), .nef (Nikon), .arw (Sony), .dng (Adobe)
/// - TIFF: .tiff, .tif
/// - WebP: .webp
///
/// Phase 2+ may add magic byte detection for more robust identification.
///
/// # Arguments
///
/// * `file_path` - Path to check
///
/// # Returns
///
/// Returns `true` if the file has a supported image extension (case-insensitive),
/// `false` otherwise.
fn is_image_file(file_path: &Path) -> bool {
    if let Some(extension) = file_path.extension() {
        let ext_lower = extension.to_string_lossy().to_lowercase();
        SUPPORTED_EXTENSIONS.contains(&ext_lower.as_str())
    } else {
        false
    }
}

/// Computes BLAKE3 hash of a file.
///
/// Reads the entire file and computes its hash using the BLAKE3 algorithm.
/// Returns the hash as a lowercase hexadecimal string.
fn compute_blake3_hash(file_path: &Path) -> Result<String> {
    let mut file = File::open(file_path)?;
    let mut hasher = blake3::Hasher::new();
    let mut buffer = [0u8; 8192];

    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    let hash = hasher.finalize();
    Ok(hash.to_hex().to_string())
}

/// Writes a sidecar file atomically with backup rotation.
///
/// Atomic write strategy:
/// 1. Write to temporary file: `<file_path>.json.tmp`
/// 2. Fsync to ensure data is on disk
/// 3. Rename to final location: `<file_path>.json`
///
/// Backup rotation strategy:
/// - If `<file_path>.json` exists, rotate to `.bak1`
/// - If `.bak1` exists, rotate to `.bak2`
/// - If `.bak2` exists, rotate to `.bak3`
/// - `.bak3` is overwritten (oldest backup is lost)
///
/// # Errors
///
/// Returns `JozinError::IoError` if writing or renaming fails.
fn write_sidecar(file_path: &Path, sidecar: &Sidecar) -> Result<()> {
    let sidecar_path = get_sidecar_path(file_path);
    let tmp_path = get_tmp_sidecar_path(file_path);

    // Rotate backups if sidecar already exists
    if sidecar_path.exists() {
        rotate_backups(&sidecar_path)?;
    }

    // Serialize to JSON
    let json = serde_json::to_string_pretty(sidecar)?;

    // Write to temporary file
    let mut tmp_file = File::create(&tmp_path)?;
    tmp_file.write_all(json.as_bytes())?;
    tmp_file.sync_all()?; // Ensure data is on disk

    // Atomic rename
    fs::rename(&tmp_path, &sidecar_path)?;

    Ok(())
}

/// Returns the sidecar path for a given file: `<file_path>.json`
fn get_sidecar_path(file_path: &Path) -> PathBuf {
    let mut path = file_path.to_path_buf();
    let current_name = path.file_name().unwrap().to_string_lossy().to_string();
    path.set_file_name(format!("{}.json", current_name));
    path
}

/// Returns the temporary sidecar path: `<file_path>.json.tmp`
fn get_tmp_sidecar_path(file_path: &Path) -> PathBuf {
    let mut path = get_sidecar_path(file_path);
    let current_name = path.file_name().unwrap().to_string_lossy().to_string();
    path.set_file_name(format!("{}.tmp", current_name));
    path
}

/// Rotates backup files: .json → .bak1 → .bak2 → .bak3
///
/// This ensures we keep up to 3 backups of the sidecar file.
fn rotate_backups(sidecar_path: &Path) -> Result<()> {
    let bak3 = sidecar_path.with_extension("json.bak3");
    let bak2 = sidecar_path.with_extension("json.bak2");
    let bak1 = sidecar_path.with_extension("json.bak1");

    // Rotate .bak2 → .bak3 (overwrite .bak3 if exists)
    if bak2.exists() {
        fs::rename(&bak2, &bak3)?;
    }

    // Rotate .bak1 → .bak2
    if bak1.exists() {
        fs::rename(&bak1, &bak2)?;
    }

    // Rotate .json → .bak1
    if sidecar_path.exists() {
        fs::rename(sidecar_path, &bak1)?;
    }

    Ok(())
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use tempfile::TempDir;

    /// Helper to create a temporary test image file
    fn create_test_image(dir: &Path, filename: &str, content: &[u8]) -> PathBuf {
        let path = dir.join(filename);
        let mut file = fs::File::create(&path).unwrap();
        file.write_all(content).unwrap();
        path
    }

    #[test]
    fn test_is_image_file() {
        // Supported extensions
        assert!(is_image_file(Path::new("test.jpg")));
        assert!(is_image_file(Path::new("test.JPG")));
        assert!(is_image_file(Path::new("test.jpeg")));
        assert!(is_image_file(Path::new("test.png")));
        assert!(is_image_file(Path::new("test.heic")));
        assert!(is_image_file(Path::new("test.raw")));
        assert!(is_image_file(Path::new("test.cr2")));
        assert!(is_image_file(Path::new("test.nef")));
        assert!(is_image_file(Path::new("test.arw")));
        assert!(is_image_file(Path::new("test.dng")));
        assert!(is_image_file(Path::new("test.tiff")));
        assert!(is_image_file(Path::new("test.webp")));

        // Unsupported extensions
        assert!(!is_image_file(Path::new("test.txt")));
        assert!(!is_image_file(Path::new("test.pdf")));
        assert!(!is_image_file(Path::new("test.mp4")));
        assert!(!is_image_file(Path::new("test")));
    }

    #[test]
    fn test_scan_path_single_file() {
        let temp_dir = TempDir::new().unwrap();
        let image_path = create_test_image(temp_dir.path(), "test.jpg", b"fake image data");

        let result = scan_path(&image_path, false, None, None, false, 4, Some("file"), None).unwrap();

        assert_eq!(result.total_files, 1);
        assert_eq!(result.successful, 1);
        assert_eq!(result.failed, 0);
        assert_eq!(result.skipped, 0);
        assert_eq!(result.scanned_files.len(), 1);

        let scanned = &result.scanned_files[0];
        assert!(matches!(scanned.action, ScanAction::Written));
        assert!(scanned.sidecar_path.is_some());
        assert!(scanned.hash.is_some());
        assert!(scanned.size_bytes.is_some());

        // Verify sidecar was created
        let sidecar_path = get_sidecar_path(&image_path);
        assert!(sidecar_path.exists());
    }

    #[test]
    fn test_scan_path_single_file_dry_run() {
        let temp_dir = TempDir::new().unwrap();
        let image_path = create_test_image(temp_dir.path(), "test.jpg", b"fake image data");

        let result = scan_path(&image_path, false, None, None, true, 4, Some("file"), None).unwrap();

        assert_eq!(result.total_files, 1);
        assert_eq!(result.successful, 0);
        assert_eq!(result.failed, 0);
        assert_eq!(result.skipped, 1);

        let scanned = &result.scanned_files[0];
        assert!(matches!(scanned.action, ScanAction::Skipped));
        assert!(scanned.hash.is_some());

        // Verify sidecar was NOT created
        let sidecar_path = get_sidecar_path(&image_path);
        assert!(!sidecar_path.exists());
    }

    #[test]
    fn test_scan_path_directory_recursive() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        // Create test files
        create_test_image(root, "image1.jpg", b"image 1");
        create_test_image(root, "image2.png", b"image 2");

        // Create subdirectory with more images
        let subdir = root.join("subdir");
        fs::create_dir(&subdir).unwrap();
        create_test_image(&subdir, "image3.jpg", b"image 3");

        let result = scan_path(root, true, None, None, false, 4, Some("file"), None).unwrap();

        assert_eq!(result.total_files, 3);
        assert_eq!(result.successful, 3);
        assert_eq!(result.failed, 0);
        assert_eq!(result.skipped, 0);
    }

    #[test]
    fn test_scan_path_directory_non_recursive() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        // Create test files
        create_test_image(root, "image1.jpg", b"image 1");
        create_test_image(root, "image2.png", b"image 2");

        // Create subdirectory with more images (should be skipped)
        let subdir = root.join("subdir");
        fs::create_dir(&subdir).unwrap();
        create_test_image(&subdir, "image3.jpg", b"image 3");

        let result = scan_path(root, false, None, None, false, 4, Some("file"), None).unwrap();

        assert_eq!(result.total_files, 2);
        assert_eq!(result.successful, 2);
        assert_eq!(result.failed, 0);
        assert_eq!(result.skipped, 0);
    }

    #[test]
    fn test_scan_path_include_patterns() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        create_test_image(root, "image1.jpg", b"image 1");
        create_test_image(root, "image2.png", b"image 2");
        create_test_image(root, "image3.jpg", b"image 3");

        let include = vec![String::from("*.jpg")];
        let result = scan_path(root, false, Some(&include), None, false, 4, Some("file"), None).unwrap();

        assert_eq!(result.total_files, 3);
        assert_eq!(result.successful, 2); // Only .jpg files
        assert_eq!(result.skipped, 1); // .png file skipped
    }

    #[test]
    fn test_scan_path_exclude_patterns() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        create_test_image(root, "image1.jpg", b"image 1");

        // Create subdirectory to exclude
        let excluded_dir = root.join(".jozin");
        fs::create_dir(&excluded_dir).unwrap();
        create_test_image(&excluded_dir, "image2.jpg", b"image 2");

        let exclude = vec![String::from("**/.jozin/**")];
        let result = scan_path(root, true, None, Some(&exclude), false, 4, Some("file"), None).unwrap();

        assert_eq!(result.total_files, 2);
        assert_eq!(result.successful, 1); // Only root image
        assert_eq!(result.skipped, 1); // Excluded dir image
    }

    #[test]
    fn test_scan_path_non_image_files_skipped() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        create_test_image(root, "image.jpg", b"image");
        create_test_image(root, "document.txt", b"not an image");

        let result = scan_path(root, false, None, None, false, 4, Some("file"), None).unwrap();

        assert_eq!(result.total_files, 2);
        assert_eq!(result.successful, 1); // Only .jpg
        assert_eq!(result.skipped, 1); // .txt skipped
    }

    #[test]
    fn test_scan_path_nonexistent_path() {
        let result = scan_path(Path::new("/nonexistent/path"), false, None, None, false, 4, Some("file"), None);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), JozinError::IoError { .. }));
    }

    #[test]
    fn test_scan_path_invalid_image_extension() {
        let temp_dir = TempDir::new().unwrap();
        let text_file = create_test_image(temp_dir.path(), "test.txt", b"not an image");

        let result = scan_path(&text_file, false, None, None, false, 4, Some("file"), None);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), JozinError::ValidationError { .. }));
    }

    #[test]
    fn test_build_glob_matcher_valid_patterns() {
        let patterns = vec![String::from("*.jpg"), String::from("**/*.png")];
        let matcher = build_glob_matcher(&patterns).unwrap();

        assert!(matcher.is_match("test.jpg"));
        assert!(matcher.is_match("dir/subdir/test.png"));
        assert!(!matcher.is_match("test.txt"));
    }

    #[test]
    fn test_build_glob_matcher_invalid_pattern() {
        let patterns = vec![String::from("[invalid")];
        let result = build_glob_matcher(&patterns);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), JozinError::ValidationError { .. }));
    }
}

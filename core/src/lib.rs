//! # Jožin Core Library
//!
//! This is the core library for Jožin, a privacy-focused, local-first photo organizer.
//! It provides all photo processing functionality including scanning, verification,
//! migration, face detection, tagging, and thumbnail generation.
//!
//! ## Architecture
//!
//! Jožin follows a **modular monolith** design with seven core modules:
//!
//! - **scan** - Directory traversal, EXIF extraction, BLAKE3 hashing, sidecar generation
//! - **verify** - Validates sidecar integrity, schema versions, detects staleness
//! - **migrate** - Handles schema version upgrades with backup rotation
//! - **cleanup** - Removes Jožin-generated files (sidecars, thumbnails, backups, cache)
//! - **faces** - Face detection & identification (optional feature)
//! - **tags** - ML-based and rule-based automatic tagging (optional feature)
//! - **thumbs** - Multi-size thumbnail generation (optional feature)
//!
//! ## Core Philosophy
//!
//! - **Immutable originals** - Original photos are never modified
//! - **Local-first design** - 100% offline capable, no cloud uploads
//! - **Schema-driven metadata** - Versioned JSON sidecars stored adjacent to photos
//! - **Modular monolith** - Single Rust binary, no microservices
//!
//! For more details, see `SCOPE.md` and `TASK+PHASE_PLAN.md` in the repository.

use serde::{Deserialize, Serialize};
use std::fmt;
use time::OffsetDateTime;

// Module declarations
pub mod scan;
pub mod verify;
pub mod migrate;
pub mod cleanup;

// Re-export commonly used types for convenience
pub use scan::{scan_file, scan_path, ScanAction, ScanResult, ScannedFile};
pub use cleanup::{cleanup_path, CleanupOptions, CleanupResult, DeletedFile, FileType};

// Phase 2+ modules (feature-gated)
#[cfg(feature = "faces")]
pub mod faces;
#[cfg(feature = "tags")]
pub mod tags;
#[cfg(feature = "thumbs")]
pub mod thumbs;

// ============================================================================
// Type Aliases
// ============================================================================

/// Standard result type for all Jožin operations.
///
/// All public module functions should return this type for consistent error handling
/// and propagation throughout the codebase.
///
/// # Example
///
/// ```
/// use jozin_core::Result;
///
/// fn process_image(path: &str) -> Result<()> {
///     // ... implementation
///     Ok(())
/// }
/// ```
pub type Result<T> = std::result::Result<T, JozinError>;

/// RFC3339-formatted timestamp string.
///
/// All timestamps in Jožin use ISO 8601 / RFC3339 format (e.g., `2025-01-15T14:30:00Z`)
/// for consistency and international compatibility. Use the `time` crate to generate
/// these values:
///
/// ```
/// use time::OffsetDateTime;
/// let timestamp = OffsetDateTime::now_utc()
///     .format(&time::format_description::well_known::Rfc3339)
///     .unwrap();
/// ```
pub type Timestamp = String;

// ============================================================================
// Progress Event for Real-Time Callbacks
// ============================================================================

/// Progress event emitted during operations for real-time progress tracking.
///
/// Used by CLI and UI to display progress while operations are running.
/// Core library functions accept an optional callback that receives these events.
///
/// # Example Usage
///
/// ```
/// use jozin_core::ProgressEvent;
/// use std::path::Path;
///
/// let callback = |event: ProgressEvent| {
///     match event {
///         ProgressEvent::FileStarted { path } => {
///             println!("Processing: {}", path);
///         }
///         ProgressEvent::FileCompleted { path, success, error, .. } => {
///             if success {
///                 println!("{} ... ✓", path);
///             } else {
///                 println!("{} ... ✗ {}", path, error.as_deref().unwrap_or("error"));
///             }
///         }
///     }
/// };
///
/// // Pass callback to core function
/// // scan_path(&Path::new("./photos"), true, None, None, false, 4, None, Some(&callback))?;
/// # Ok::<(), jozin_core::JozinError>(())
/// ```
#[derive(Debug, Clone)]
pub enum ProgressEvent {
    /// File processing started
    FileStarted {
        /// Path to the file being processed
        path: String,
    },
    /// File processing completed
    FileCompleted {
        /// Path to the file that was processed
        path: String,
        /// Whether processing succeeded
        success: bool,
        /// Error message if processing failed
        error: Option<String>,
        /// File size in bytes (if available)
        size_bytes: Option<u64>,
    },
}

// ============================================================================
// Common Response Type with Timing Metadata
// ============================================================================

/// Generic wrapper for operation results that includes timing metadata.
///
/// Per the requirements in `TASK+PHASE_PLAN.md`, all CLI and Tauri operations
/// must return consistent timing information. This struct wraps any operation
/// result with `started_at`, `finished_at`, and `duration_ms` fields.
///
/// # Fields
///
/// - `started_at`: RFC3339 timestamp when the operation began
/// - `finished_at`: RFC3339 timestamp when the operation completed
/// - `duration_ms`: Duration in milliseconds (automatically calculated)
/// - `data`: The actual operation result
///
/// # Example
///
/// ```
/// use jozin_core::OperationResponse;
/// use time::OffsetDateTime;
///
/// let start = OffsetDateTime::now_utc();
/// // ... perform operation ...
/// let result = vec!["file1.jpg", "file2.jpg"];
/// let end = OffsetDateTime::now_utc();
///
/// let response = OperationResponse::new(result, start, end);
/// println!("{}", serde_json::to_string_pretty(&response).unwrap());
/// ```
///
/// # JSON Output
///
/// ```json
/// {
///   "started_at": "2025-01-15T14:30:00Z",
///   "finished_at": "2025-01-15T14:30:05Z",
///   "duration_ms": 5000,
///   "data": ["file1.jpg", "file2.jpg"]
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationResponse<T> {
    /// RFC3339 timestamp indicating when the operation started
    pub started_at: Timestamp,

    /// RFC3339 timestamp indicating when the operation finished
    pub finished_at: Timestamp,

    /// Duration of the operation in milliseconds
    pub duration_ms: u64,

    /// The actual operation result
    pub data: T,
}

impl<T> OperationResponse<T> {
    /// Creates a new operation response with automatic duration calculation.
    ///
    /// RFC3339 formatting from valid `OffsetDateTime` instances should never fail in practice.
    /// If formatting somehow fails, this returns an internal error rather than panicking.
    ///
    /// # Arguments
    ///
    /// - `data`: The operation result to wrap
    /// - `started_at`: Timestamp when the operation began
    /// - `finished_at`: Timestamp when the operation completed
    ///
    /// # Errors
    ///
    /// Returns `JozinError::InternalError` if RFC3339 timestamp formatting fails
    /// (this should never happen with valid `OffsetDateTime` values).
    ///
    /// # Example
    ///
    /// ```
    /// use jozin_core::OperationResponse;
    /// use time::OffsetDateTime;
    ///
    /// let start = OffsetDateTime::now_utc();
    /// let result = "success";
    /// let end = OffsetDateTime::now_utc();
    ///
    /// let response = OperationResponse::new(result, start, end)?;
    /// # Ok::<(), jozin_core::JozinError>(())
    /// ```
    pub fn new(
        data: T,
        started_at: OffsetDateTime,
        finished_at: OffsetDateTime,
    ) -> Result<Self> {
        let duration_ms = (finished_at - started_at).whole_milliseconds().max(0) as u64;

        let started_at_str = started_at
            .format(&time::format_description::well_known::Rfc3339)
            .map_err(|e| JozinError::InternalError {
                message: format!("Failed to format started_at timestamp: {}", e),
            })?;

        let finished_at_str = finished_at
            .format(&time::format_description::well_known::Rfc3339)
            .map_err(|e| JozinError::InternalError {
                message: format!("Failed to format finished_at timestamp: {}", e),
            })?;

        Ok(Self {
            started_at: started_at_str,
            finished_at: finished_at_str,
            duration_ms,
            data,
        })
    }
}

// ============================================================================
// Structured Error Type with Exit Codes
// ============================================================================

/// Structured error type for all Jožin operations.
///
/// This enum maps to the exit code specification in `TASK+PHASE_PLAN.md`:
///
/// - Exit code 0: Success (not represented here)
/// - Exit code 1: User error (invalid arguments, bad input)
/// - Exit code 2: I/O error (file not found, permission denied)
/// - Exit code 3: Validation error (schema mismatch, corrupt sidecar)
/// - Exit code 4: Internal error (unexpected panics, logic errors)
///
/// # Serialization
///
/// This type implements `Serialize` and `Deserialize` for JSON error responses.
/// When serialized, it produces a structured JSON object with `kind`, `message`,
/// and `exit_code` fields.
///
/// # Examples
///
/// ```
/// use jozin_core::JozinError;
///
/// // User provided invalid parameter
/// let err = JozinError::UserError {
///     message: "--max-threads must be positive".to_string()
/// };
/// assert_eq!(err.exit_code(), 1);
///
/// // File not found (via From trait)
/// let err: JozinError = std::io::Error::from(std::io::ErrorKind::NotFound).into();
/// assert_eq!(err.exit_code(), 2);
///
/// // Corrupt sidecar JSON
/// let err = JozinError::ValidationError {
///     message: "Invalid schema version".to_string()
/// };
/// assert_eq!(err.exit_code(), 3);
///
/// // Unexpected internal error
/// let err = JozinError::InternalError {
///     message: "Hash computation failed".to_string()
/// };
/// assert_eq!(err.exit_code(), 4);
///
/// // Serialize to JSON
/// let json = serde_json::to_string(&err).unwrap();
/// // {"kind":"internal","message":"Hash computation failed"}
/// ```
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum JozinError {
    /// User error (exit code 1) - Invalid parameters, bad input, user mistake
    ///
    /// Use this for:
    /// - Invalid command-line arguments (e.g., "--max-threads must be positive")
    /// - Missing required files specified by user
    /// - Invalid glob patterns
    /// - Bad configuration values
    #[serde(rename = "user")]
    UserError {
        /// Error message describing what went wrong
        message: String,
    },

    /// I/O error (exit code 2) - File system operations failed
    ///
    /// Use this for:
    /// - File not found
    /// - Permission denied
    /// - Disk full
    /// - Read/write failures
    #[serde(rename = "io")]
    IoError {
        /// Error message describing the I/O failure
        message: String,
    },

    /// Validation error (exit code 3) - Schema or data integrity issues
    ///
    /// Use this for:
    /// - Sidecar schema version mismatch
    /// - Corrupt JSON sidecar files
    /// - Invalid EXIF data
    /// - Hash mismatches during verification
    #[serde(rename = "validation")]
    ValidationError {
        /// Error message describing the validation failure
        message: String,
    },

    /// Internal error (exit code 4) - Unexpected failures in Jožin logic
    ///
    /// Use this for:
    /// - Unexpected panics caught and converted to errors
    /// - Logic errors in algorithms
    /// - Assertion failures
    /// - Unhandled edge cases
    #[serde(rename = "internal")]
    InternalError {
        /// Error message describing the internal failure
        message: String,
    },
}

impl fmt::Display for JozinError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JozinError::UserError { message } => write!(f, "User error: {}", message),
            JozinError::IoError { message } => write!(f, "I/O error: {}", message),
            JozinError::ValidationError { message } => write!(f, "Validation error: {}", message),
            JozinError::InternalError { message } => write!(f, "Internal error: {}", message),
        }
    }
}

impl std::error::Error for JozinError {}

impl From<std::io::Error> for JozinError {
    fn from(err: std::io::Error) -> Self {
        JozinError::IoError {
            message: err.to_string(),
        }
    }
}

impl From<serde_json::Error> for JozinError {
    fn from(err: serde_json::Error) -> Self {
        JozinError::ValidationError {
            message: format!("JSON error: {}", err),
        }
    }
}

impl From<walkdir::Error> for JozinError {
    fn from(err: walkdir::Error) -> Self {
        JozinError::IoError {
            message: format!("Directory traversal error: {}", err),
        }
    }
}

impl JozinError {
    /// Returns the appropriate exit code for this error.
    ///
    /// Maps error variants to exit codes as specified in `TASK+PHASE_PLAN.md`:
    /// - UserError → 1
    /// - IoError → 2
    /// - ValidationError → 3
    /// - InternalError → 4
    pub fn exit_code(&self) -> i32 {
        match self {
            JozinError::UserError { .. } => 1,
            JozinError::IoError { .. } => 2,
            JozinError::ValidationError { .. } => 3,
            JozinError::InternalError { .. } => 4,
        }
    }
}

// ============================================================================
// Pipeline Signature for Schema Versioning
// ============================================================================

/// Records the pipeline configuration that produced a sidecar.
///
/// The pipeline signature enables the verify module to detect stale sidecars
/// that need rescanning. When algorithms, models, or schema versions change,
/// the signature changes, triggering rescan recommendations.
///
/// Per `TASK+PHASE_PLAN.md`, the verify module compares the current pipeline
/// signature against the signature stored in each sidecar to determine if
/// rescanning is needed.
///
/// # Fields
///
/// - `schema_version`: Sidecar schema version (e.g., "1.0.0")
/// - `producer_version`: Jožin binary version that created this sidecar
/// - `hash_algorithm`: Hash algorithm used (e.g., "blake3")
/// - `face_model`: Face detection model used, if any (e.g., "arcface-1.4")
/// - `tag_model`: Tagging model used, if any (e.g., "clip-vit-b32")
/// - `created_at`: When this signature was created (RFC3339)
///
/// # Example
///
/// ```json
/// {
///   "schema_version": "1.0.0",
///   "producer_version": "0.1.0",
///   "hash_algorithm": "blake3",
///   "face_model": "arcface-1.4",
///   "tag_model": null,
///   "created_at": "2025-01-15T14:30:00Z"
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PipelineSignature {
    /// Sidecar schema version (e.g., "1.0.0"). Changed when sidecar structure evolves.
    pub schema_version: String,

    /// Jožin binary version that produced this sidecar (e.g., "0.1.0").
    /// Updated with each Jožin release.
    pub producer_version: String,

    /// Hash algorithm used for file deduplication (e.g., "blake3").
    /// Changed when hash algorithm is upgraded.
    pub hash_algorithm: String,

    /// Face detection model used, if faces module was run (e.g., "arcface-1.4").
    /// None if faces were not processed. Changed when user switches models.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub face_model: Option<String>,

    /// Tagging model used, if tags module was run (e.g., "clip-vit-b32").
    /// None if tags were not processed. Changed when user switches models.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag_model: Option<String>,

    /// RFC3339 timestamp when this signature was created
    pub created_at: Timestamp,
}

impl PipelineSignature {
    /// Checks if two pipeline signatures are compatible.
    ///
    /// Signatures are compatible if they have the same `schema_version` and
    /// `hash_algorithm`. Different producer versions or model versions are
    /// considered compatible if the core schema hasn't changed.
    ///
    /// This is used by the verify module to determine if a sidecar needs
    /// rescanning due to fundamental pipeline changes.
    ///
    /// # Example
    ///
    /// ```
    /// use jozin_core::PipelineSignature;
    ///
    /// let sig1 = PipelineSignature {
    ///     schema_version: "1.0.0".to_string(),
    ///     producer_version: "0.1.0".to_string(),
    ///     hash_algorithm: "blake3".to_string(),
    ///     face_model: None,
    ///     tag_model: None,
    ///     created_at: "2025-01-15T14:30:00Z".to_string(),
    /// };
    ///
    /// let sig2 = PipelineSignature {
    ///     schema_version: "1.0.0".to_string(),
    ///     producer_version: "0.2.0".to_string(), // Different version
    ///     hash_algorithm: "blake3".to_string(),
    ///     face_model: None,
    ///     tag_model: None,
    ///     created_at: "2025-01-16T10:00:00Z".to_string(),
    /// };
    ///
    /// assert!(sig1.is_compatible_with(&sig2)); // Compatible despite version difference
    /// ```
    pub fn is_compatible_with(&self, other: &PipelineSignature) -> bool {
        self.schema_version == other.schema_version
            && self.hash_algorithm == other.hash_algorithm
    }
}

// ============================================================================
// Helper Structs for Nested Sidecar Sections
// ============================================================================

/// Face detection result stored in sidecar.
///
/// This struct is populated by the faces module (Phase 2+) and stores bounding
/// box coordinates, confidence scores, and optional identification information.
///
/// # Fields
///
/// - `bbox`: Bounding box [x, y, width, height] normalized to 0-1 range
/// - `score`: Detection confidence score (0-1)
/// - `embedding_hash`: Optional hash of face embedding vector for privacy
/// - `person`: Optional identified person name (if `--identify` was used)
///
/// # Example
///
/// ```json
/// {
///   "bbox": [0.25, 0.30, 0.15, 0.20],
///   "score": 0.95,
///   "embedding_hash": "a3f2c1...",
///   "person": "John Doe"
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaceDetection {
    /// Bounding box coordinates [x, y, width, height] normalized to 0-1 range.
    /// Coordinates are relative to image dimensions (e.g., x=0.5 means center).
    pub bbox: [f32; 4],

    /// Detection confidence score (0-1). Higher values indicate more confident detections.
    /// Typical threshold is 0.8 (controlled by `--min-score` parameter).
    pub score: f32,

    /// Optional BLAKE3 hash of the face embedding vector.
    /// Used for privacy-preserving face matching without storing raw embeddings.
    /// Populated when faces module runs with embedding generation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embedding_hash: Option<String>,

    /// Optional identified person name.
    /// Populated when `--identify` flag is used and embedding matches a known person.
    /// None if face was detected but not identified.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub person: Option<String>,
}

/// Tag source type indicating how a tag was assigned.
///
/// Used to differentiate automatic vs manual tags and handle `--append` mode
/// in the tags module.
///
/// # Variants
///
/// - `Ml`: Tag assigned by machine learning model
/// - `Rules`: Tag assigned by rule-based logic
/// - `User`: Tag manually added by user
///
/// # JSON Representation
///
/// Serializes to lowercase strings: "ml", "rules", "user"
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TagSource {
    /// Tag assigned by machine learning model
    Ml,
    /// Tag assigned by rule-based logic
    Rules,
    /// Tag manually added by user
    User,
}

/// Tag label with optional confidence score.
///
/// This struct is populated by the tags module (Phase 2+) and stores labels
/// assigned to images through ML models, rules, or user input.
///
/// # Fields
///
/// - `label`: Tag text (e.g., "sunset", "beach", "portrait")
/// - `score`: Optional confidence score (0-1) for ML-assigned tags
/// - `source`: Tag source: "ml" (ML model), "rules" (rule-based), or "user" (manually added)
///
/// # Example
///
/// ```json
/// [
///   { "label": "sunset", "score": 0.89, "source": "ml" },
///   { "label": "vacation", "score": null, "source": "user" }
/// ]
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    /// Tag label text (e.g., "sunset", "beach", "portrait").
    /// Should be lowercase for consistency.
    pub label: String,

    /// Optional confidence score (0-1) for ML-assigned tags.
    /// None for user-assigned or rule-based tags.
    /// Typical threshold is 0.6 (controlled by `--min-score` parameter).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub score: Option<f32>,

    /// Tag source: "ml" (ML model), "rules" (rule-based), or "user" (manually added).
    /// Used to differentiate automatic vs manual tags and handle `--append` mode.
    pub source: TagSource,
}

/// Thumbnail file information.
///
/// This struct is populated by the thumbs module (Phase 2+) and records the
/// location and properties of generated thumbnail files.
///
/// # Fields
///
/// - `path`: Relative or absolute path to thumbnail file
/// - `size`: Thumbnail size in pixels (e.g., 256, 512)
/// - `format`: Image format ("jpg" or "webp")
///
/// # Example
///
/// ```json
/// [
///   { "path": "IMG_1234_256.jpg", "size": 256, "format": "jpg" },
///   { "path": "IMG_1234_512.webp", "size": 512, "format": "webp" }
/// ]
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThumbnailInfo {
    /// Path to thumbnail file (relative or absolute).
    /// Typically stored adjacent to original with size suffix (e.g., "IMG_1234_256.jpg").
    pub path: String,

    /// Thumbnail size in pixels (e.g., 256, 512).
    /// Controlled by `--sizes` parameter in thumbs module.
    pub size: u32,

    /// Image format: "jpg" or "webp".
    /// Controlled by `--format` parameter in thumbs module.
    pub format: String,
}

// ============================================================================
// Complete Sidecar Struct
// ============================================================================

/// Canonical sidecar metadata structure stored as JSON adjacent to photos.
///
/// This is the main metadata structure for Jožin. Each photo gets a `.json`
/// sidecar file (e.g., `IMG_1234.JPG.json`) containing all derived information.
///
/// Per `SCOPE.md`, sidecars are written atomically using `.tmp` → fsync → rename
/// strategy, with backup rotation (`.bak1`, `.bak2`, `.bak3`) to prevent data loss.
///
/// # Structure
///
/// The sidecar contains:
/// - Top-level metadata (schema version, timestamps, pipeline signature)
/// - `source` section: Original file information (path, hash, size)
/// - `image` section: EXIF metadata (dimensions, camera, GPS)
/// - `faces` section: Face detection results (Phase 2+)
/// - `tags` section: ML and user-assigned labels (Phase 2+)
/// - `thumbnails` section: Generated thumbnail information (Phase 2+)
///
/// # Example JSON
///
/// ```json
/// {
///   "schema_version": "1.0.0",
///   "producer_version": "0.1.0",
///   "created_at": "2025-01-15T14:30:00Z",
///   "updated_at": "2025-01-15T14:30:00Z",
///   "pipeline_signature": {
///     "schema_version": "1.0.0",
///     "producer_version": "0.1.0",
///     "hash_algorithm": "blake3",
///     "created_at": "2025-01-15T14:30:00Z"
///   },
///   "source": {
///     "file_path": "/photos/2020/IMG_1234.JPG",
///     "file_size_bytes": 2048576,
///     "file_hash_b3": "a3f2c1d4e5f6...",
///     "file_modified_at": "2020-06-15T10:30:00Z"
///   },
///   "image": {
///     "width": 4032,
///     "height": 3024,
///     "format": "JPEG",
///     "orientation": 1,
///     "datetime_original": "2020-06-15T10:30:00Z",
///     "camera_make": "Apple",
///     "camera_model": "iPhone 12",
///     "gps_latitude": 37.7749,
///     "gps_longitude": -122.4194
///   },
///   "faces": [],
///   "tags": [],
///   "thumbnails": []
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sidecar {
    /// Sidecar schema version (e.g., "1.0.0").
    /// Updated when sidecar structure changes. Used by migrate module to upgrade schemas.
    pub schema_version: String,

    /// Jožin binary version that created this sidecar (e.g., "0.1.0").
    pub producer_version: String,

    /// RFC3339 timestamp when this sidecar was first created
    pub created_at: Timestamp,

    /// RFC3339 timestamp when this sidecar was last updated
    pub updated_at: Timestamp,

    /// Pipeline signature tracking algorithms and models used
    pub pipeline_signature: PipelineSignature,

    /// Original file information
    pub source: SourceInfo,

    /// EXIF and image metadata (optional, populated by scan module)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub image: Option<ImageInfo>,

    /// Face detection results (populated by faces module, Phase 2+)
    #[serde(default)]
    pub faces: Vec<FaceDetection>,

    /// Tags and labels (populated by tags module, Phase 2+)
    #[serde(default)]
    pub tags: Vec<Tag>,

    /// Generated thumbnails (populated by thumbs module, Phase 2+)
    #[serde(default)]
    pub thumbnails: Vec<ThumbnailInfo>,
}

/// Original file information section of sidecar.
///
/// Contains immutable properties of the source photo file used for integrity
/// verification and duplicate detection.
///
/// # Fields
///
/// - `file_path`: Path to original file (relative or absolute)
/// - `file_size_bytes`: File size in bytes
/// - `file_hash_b3`: BLAKE3 hash in hexadecimal format
/// - `file_modified_at`: File system modification timestamp (RFC3339)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceInfo {
    /// Path to original photo file (relative or absolute).
    /// Stored as provided during scan.
    pub file_path: String,

    /// File size in bytes.
    /// Used for quick change detection before computing full hash.
    pub file_size_bytes: u64,

    /// BLAKE3 hash of file contents in hexadecimal format.
    /// Used for duplicate detection and integrity verification.
    pub file_hash_b3: String,

    /// File system modification timestamp (RFC3339).
    /// Used to detect if file has changed since last scan.
    pub file_modified_at: Timestamp,
}

/// EXIF and image metadata section of sidecar.
///
/// Contains information extracted from image file headers and EXIF metadata.
/// All fields are optional since not all images have complete metadata.
///
/// Populated by the scan module in Phase 1+.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageInfo {
    /// Image width in pixels. None if not available or not yet extracted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<u32>,

    /// Image height in pixels. None if not available or not yet extracted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<u32>,

    /// Image format (e.g., "JPEG", "PNG", "HEIC", "RAW").
    /// Detected from file extension and magic bytes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,

    /// EXIF orientation value (1-8).
    /// Used to correctly display rotated images without modifying original.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub orientation: Option<u8>,

    /// Original capture date/time from EXIF DateTimeOriginal tag (RFC3339).
    /// More reliable than file modification time.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub datetime_original: Option<String>,

    /// Camera manufacturer from EXIF Make tag (e.g., "Apple", "Canon").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub camera_make: Option<String>,

    /// Camera model from EXIF Model tag (e.g., "iPhone 12", "EOS 5D Mark IV").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub camera_model: Option<String>,

    /// GPS latitude from EXIF GPSLatitude tag (decimal degrees).
    /// Positive values indicate North, negative indicate South.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gps_latitude: Option<f64>,

    /// GPS longitude from EXIF GPSLongitude tag (decimal degrees).
    /// Positive values indicate East, negative indicate West.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gps_longitude: Option<f64>,
}


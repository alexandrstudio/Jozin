//! # Jožin CLI
//!
//! Command-line interface for Jožin photo organizer.
//!
//! This CLI provides access to all seven core modules:
//! - **scan**: Directory traversal, EXIF extraction, hash computation
//! - **cleanup**: Remove Jožin-generated files (sidecars, thumbnails, backups, cache)
//! - **faces**: Face detection and identification
//! - **tags**: ML and rule-based tagging
//! - **thumbs**: Thumbnail generation
//! - **verify**: Sidecar validation and staleness detection
//! - **migrate**: Schema version upgrades
//!
//! All commands output JSON to stdout for machine readability.
//! Errors are printed to stderr with appropriate exit codes (1-4).
//!
//! See `TASK+PHASE_PLAN.md` for complete parameter specifications.

use clap::{Args, Parser, Subcommand, ValueEnum};
use jozin_core::{JozinError, Result, OperationResponse};
use serde::Serialize;
use std::path::PathBuf;
use std::process::exit;
use time::OffsetDateTime;

// ============================================================================
// Output Format
// ============================================================================

/// Output format mode for CLI commands
#[derive(Clone, Copy, Debug)]
enum OutputFormat {
    /// Human-readable progress output with real-time feedback
    Human,
    /// JSON output (silent until completion)
    Json,
}

/// Determines output format based on --json flag and TTY detection
///
/// If --json is explicitly set, use JSON mode.
/// Otherwise, auto-detect: Human if stdout is a TTY, JSON if piped.
fn determine_output_format(json_flag: bool) -> OutputFormat {
    if json_flag {
        OutputFormat::Json
    } else if atty::is(atty::Stream::Stdout) {
        OutputFormat::Human
    } else {
        OutputFormat::Json
    }
}

// ============================================================================
// Value Enums for Type-Safe Options
// ============================================================================

/// Hash computation mode for scan module
#[derive(Clone, Debug, ValueEnum, Serialize)]
#[serde(rename_all = "lowercase")]
enum HashMode {
    /// Hash file contents only
    File,
    /// Hash decoded pixel data (Phase 2+)
    Pixel,
    /// Hash both file and pixel data (Phase 2+)
    Both,
}

impl HashMode {
    #[allow(dead_code)] // Will be used in Phase 2
    fn as_str(&self) -> &'static str {
        match self {
            HashMode::File => "file",
            HashMode::Pixel => "pixel",
            HashMode::Both => "both",
        }
    }
}

/// Tagging mode for tags module
#[derive(Clone, Debug, ValueEnum, Serialize)]
#[serde(rename_all = "lowercase")]
enum TagMode {
    /// Use ML models only
    Ml,
    /// Use rule-based heuristics only
    Rules,
    /// Use both ML and rules
    Both,
}

impl TagMode {
    fn as_str(&self) -> &'static str {
        match self {
            TagMode::Ml => "ml",
            TagMode::Rules => "rules",
            TagMode::Both => "both",
        }
    }
}

/// Thumbnail output format for thumbs module
#[derive(Clone, Debug, ValueEnum, Serialize)]
#[serde(rename_all = "lowercase")]
enum ThumbFormat {
    /// JPEG format
    Jpg,
    /// WebP format
    Webp,
}

impl ThumbFormat {
    fn as_str(&self) -> &'static str {
        match self {
            ThumbFormat::Jpg => "jpg",
            ThumbFormat::Webp => "webp",
        }
    }
}

// ============================================================================
// Top-Level CLI Structure
// ============================================================================

/// Jožin - Local-first photo organizer
///
/// Processes photos locally with complete privacy:
/// - Immutable originals (never modified)
/// - Local-first design (100% offline capable)
/// - Schema-driven metadata (versioned JSON sidecars)
/// - Modular monolith (single Rust binary, no microservices)
#[derive(Parser)]
#[command(
    name = "jozin",
    version,
    about = "Local photo organizer",
    long_about = "Jožin is a privacy-focused, local-first photo organizer.\n\n\
                  It scans local directories, extracts EXIF metadata, computes BLAKE3 hashes,\n\
                  detects duplicates and faces, and stores all derived information in JSON\n\
                  sidecar files adjacent to originals. All processing happens locally—no cloud\n\
                  uploads, no external APIs, complete user control.",
    after_help = "EXAMPLES:\n  \
                  jozin scan ~/Photos --recursive --dry-run\n  \
                  jozin faces ~/Photos --model arcface-1.4 --min-score 0.8\n  \
                  jozin verify ~/Photos --fix --strict\n\n\
                  For detailed help on a subcommand, run: jozin <SUBCOMMAND> --help"
)]
struct CliArgs {
    #[command(subcommand)]
    cmd: Cmd,
}

/// Available subcommands
#[derive(Subcommand)]
enum Cmd {
    /// Scan directories and generate JSON sidecars
    Scan(ScanArgs),
    /// Remove Jožin-generated files (sidecars, thumbnails, backups, cache)
    Cleanup(CleanupArgs),
    /// Detect and identify faces in images
    Faces(FacesArgs),
    /// Generate tags for images using ML and rules
    Tags(TagsArgs),
    /// Generate thumbnails for images
    Thumbs(ThumbsArgs),
    /// Verify sidecar integrity and detect staleness
    Verify(VerifyArgs),
    /// Migrate sidecars between schema versions
    Migrate(MigrateArgs),
}

// ============================================================================
// Scan Subcommand
// ============================================================================

/// Scan directories and generate JSON sidecars
///
/// Walks directories recursively (if --recursive), reads EXIF metadata from images,
/// computes BLAKE3 file hashes, and generates JSON sidecar files adjacent to each image.
/// Supports glob patterns for filtering files.
#[derive(Args)]
#[command(
    about = "Scan directories and generate JSON sidecars",
    long_about = "Walks directories recursively (if --recursive), reads EXIF metadata from images,\n\
                  computes BLAKE3 file hashes, and generates JSON sidecar files adjacent to each image.\n\
                  Supports glob patterns for filtering files.\n\n\
                  The scan module is the foundation of Jožin's metadata extraction pipeline.",
    after_help = "EXAMPLES:\n  \
                  # Scan a directory recursively\n  \
                  jozin scan ~/Photos --recursive\n\n  \
                  # Dry run to preview actions\n  \
                  jozin scan ~/Photos --dry-run\n\n  \
                  # Include only JPEG files\n  \
                  jozin scan ~/Photos --include \"*.jpg,*.jpeg\"\n\n  \
                  # Exclude hidden directories\n  \
                  jozin scan ~/Photos --exclude \"**/.*/**\"\n\n  \
                  # Limit parallelism\n  \
                  jozin scan ~/Photos --max-threads 4"
)]
struct ScanArgs {
    /// File or directory path to scan
    path: PathBuf,

    /// Enable recursive directory traversal
    #[arg(short = 'r', long)]
    recursive: bool,

    /// Comma-separated glob patterns to include (e.g., "*.jpg,*.png")
    #[arg(long, value_name = "PATTERNS")]
    include: Option<String>,

    /// Comma-separated glob patterns to exclude (e.g., "**/.jozin/**")
    #[arg(long, value_name = "PATTERNS")]
    exclude: Option<String>,

    /// Print intended actions without writing files
    #[arg(long)]
    dry_run: bool,

    /// Maximum number of parallel threads (default: min(2×CPU, 8))
    #[arg(long, value_name = "N", value_parser = parse_threads)]
    max_threads: Option<u16>,

    /// Hash computation mode: file, pixel, or both (Phase 2+)
    #[arg(long)]
    hash_mode: Option<HashMode>,

    /// Output JSON format (default: auto-detect based on TTY)
    #[arg(long)]
    json: bool,
}

// ============================================================================
// Cleanup Subcommand
// ============================================================================

/// Remove Jožin-generated files (sidecars, thumbnails, backups, cache)
///
/// Safely removes files created by Jožin without touching original photos.
/// By default removes all generated files (sidecars, thumbnails, backups, cache).
/// Use --only-* flags to selectively remove specific file types.
#[derive(Args)]
#[command(
    about = "Remove Jožin-generated files",
    long_about = "Safely removes files created by Jožin without touching original photos.\n\
                  By default removes all generated files (sidecars, thumbnails, backups, cache).\n\
                  Use --only-* flags to selectively remove specific file types.\n\n\
                  Pattern detection ensures only Jožin files are removed:\n\
                  - Sidecars: *.json (adjacent to images)\n\
                  - Backups: *.json.bak1/bak2/bak3\n\
                  - Thumbnails: *_<digits>.jpg/webp\n\
                  - Cache: .jozin/* directories",
    after_help = "EXAMPLES:\n  \
                  # Remove all Jožin files (dry-run first)\n  \
                  jozin cleanup ~/Photos --recursive --dry-run\n\n  \
                  # Remove only sidecar JSON files\n  \
                  jozin cleanup ~/Photos --only-sidecars\n\n  \
                  # Remove only thumbnails\n  \
                  jozin cleanup ~/Photos --only-thumbnails --recursive\n\n  \
                  # Remove only backups\n  \
                  jozin cleanup ~/Photos --only-backups\n\n  \
                  # Remove cache directories\n  \
                  jozin cleanup ~/Photos --only-cache"
)]
struct CleanupArgs {
    /// File or directory path to clean
    path: PathBuf,

    /// Enable recursive directory traversal
    #[arg(short = 'r', long)]
    recursive: bool,

    /// Print intended actions without deleting files
    #[arg(long)]
    dry_run: bool,

    /// Remove only sidecar JSON files
    #[arg(long, conflicts_with_all = ["only_thumbnails", "only_backups", "only_cache"])]
    only_sidecars: bool,

    /// Remove only thumbnail files
    #[arg(long, conflicts_with_all = ["only_sidecars", "only_backups", "only_cache"])]
    only_thumbnails: bool,

    /// Remove only backup files (*.bak1/2/3)
    #[arg(long, conflicts_with_all = ["only_sidecars", "only_thumbnails", "only_cache"])]
    only_backups: bool,

    /// Remove only cache directories (.jozin/*)
    #[arg(long, conflicts_with_all = ["only_sidecars", "only_thumbnails", "only_backups"])]
    only_cache: bool,

    /// Output JSON format (default: auto-detect based on TTY)
    #[arg(long)]
    json: bool,
}

// ============================================================================
// Faces Subcommand
// ============================================================================

/// Detect and identify faces in images
///
/// Detects faces using local ONNX models, generates embeddings for face matching,
/// and can identify faces against known persons. Supports training mode for adding
/// new persons. All processing happens locally (privacy-focused).
#[derive(Args)]
#[command(
    about = "Detect and identify faces in images",
    long_about = "Detects faces using local ONNX models, generates embeddings for face matching,\n\
                  and can identify faces against known persons. Supports training mode for adding\n\
                  new persons. All processing happens locally (privacy-focused).\n\n\
                  This is a Phase 2+ feature requiring the 'faces' cargo feature.",
    after_help = "EXAMPLES:\n  \
                  # Detect faces with default model\n  \
                  jozin faces ~/Photos --recursive\n\n  \
                  # Use specific model with custom threshold\n  \
                  jozin faces ~/Photos --model arcface-1.4 --min-score 0.9\n\n  \
                  # Identify faces against known persons\n  \
                  jozin faces ~/Photos --identify\n\n  \
                  # Train on new person\n  \
                  jozin faces ~/Photos --train '{\"person\":\"John\",\"images\":[\"john1.jpg\",\"john2.jpg\"]}'"
)]
struct FacesArgs {
    /// File or directory path to process
    path: PathBuf,

    /// Enable recursive directory traversal
    #[arg(short = 'r', long)]
    recursive: bool,

    /// Face detection model identifier (e.g., "arcface-1.4")
    #[arg(long, value_name = "MODEL")]
    model: Option<String>,

    /// Match embeddings to known persons
    #[arg(long)]
    identify: bool,

    /// JSON string with training data: {"person": "name", "images": ["path1", "path2"]}
    #[arg(long, value_name = "JSON")]
    train: Option<String>,

    /// Minimum confidence threshold (0.0-1.0, default: 0.8)
    #[arg(long, value_name = "SCORE", value_parser = parse_score)]
    min_score: Option<f32>,

    /// Print intended actions without writing files
    #[arg(long)]
    dry_run: bool,

    /// Maximum number of parallel threads (default: min(2×CPU, 8))
    #[arg(long, value_name = "N", value_parser = parse_threads)]
    max_threads: Option<u16>,

    /// Output JSON format (default: auto-detect based on TTY)
    #[arg(long)]
    json: bool,
}

// ============================================================================
// Tags Subcommand
// ============================================================================

/// Generate tags for images using ML and rules
///
/// ML mode uses local models for keyword detection. Rules mode uses heuristics
/// based on filename and EXIF context. Both mode combines ML and rules.
/// Append mode preserves user-assigned tags.
#[derive(Args)]
#[command(
    about = "Generate tags for images using ML and rules",
    long_about = "ML mode uses local models for keyword detection. Rules mode uses heuristics\n\
                  based on filename and EXIF context. Both mode combines ML and rules.\n\
                  Append mode preserves user-assigned tags.\n\n\
                  This is a Phase 2+ feature requiring the 'tags' cargo feature.",
    after_help = "EXAMPLES:\n  \
                  # Tag with both ML and rules\n  \
                  jozin tags ~/Photos --recursive\n\n  \
                  # ML-only with custom threshold\n  \
                  jozin tags ~/Photos --mode ml --min-score 0.7\n\n  \
                  # Rules-only (fast, no ML)\n  \
                  jozin tags ~/Photos --mode rules\n\n  \
                  # Append to existing tags\n  \
                  jozin tags ~/Photos --append"
)]
struct TagsArgs {
    /// File or directory path to process
    path: PathBuf,

    /// Enable recursive directory traversal
    #[arg(short = 'r', long)]
    recursive: bool,

    /// Tagging mode: ml, rules, or both (default: both)
    #[arg(long)]
    mode: Option<TagMode>,

    /// ML model identifier
    #[arg(long, value_name = "MODEL")]
    model: Option<String>,

    /// Minimum confidence threshold (0.0-1.0, default: 0.6)
    #[arg(long, value_name = "SCORE", value_parser = parse_score)]
    min_score: Option<f32>,

    /// Keep existing user labels, don't overwrite
    #[arg(long)]
    append: bool,

    /// Print intended actions without writing files
    #[arg(long)]
    dry_run: bool,

    /// Output JSON format (default: auto-detect based on TTY)
    #[arg(long)]
    json: bool,
}

// ============================================================================
// Thumbs Subcommand
// ============================================================================

/// Generate thumbnails for images
///
/// Generates multiple thumbnail sizes in JPEG or WebP format.
/// Preserves orientation and color profiles. Uses atomic writes to prevent corruption.
#[derive(Args)]
#[command(
    about = "Generate thumbnails for images",
    long_about = "Generates multiple thumbnail sizes in JPEG or WebP format.\n\
                  Preserves orientation and color profiles. Uses atomic writes to prevent corruption.\n\n\
                  This is a Phase 2+ feature requiring the 'thumbs' cargo feature.",
    after_help = "EXAMPLES:\n  \
                  # Generate default 512px thumbnails\n  \
                  jozin thumbs ~/Photos --recursive\n\n  \
                  # Multiple sizes in WebP format\n  \
                  jozin thumbs ~/Photos --sizes \"256,512,1024\" --format webp\n\n  \
                  # High quality JPEG thumbnails\n  \
                  jozin thumbs ~/Photos --quality 95\n\n  \
                  # Overwrite existing thumbnails\n  \
                  jozin thumbs ~/Photos --overwrite"
)]
struct ThumbsArgs {
    /// File or directory path to process
    path: PathBuf,

    /// Enable recursive directory traversal
    #[arg(short = 'r', long)]
    recursive: bool,

    /// Comma-separated sizes in pixels (e.g., "256,512", default: "512")
    #[arg(long, value_name = "SIZES", value_parser = parse_sizes_for_clap)]
    sizes: Option<Vec<u32>>,

    /// Output format: jpg or webp (default: jpg)
    #[arg(long)]
    format: Option<ThumbFormat>,

    /// Compression quality 1-100 (default: 85)
    #[arg(long, value_name = "QUALITY", value_parser = parse_quality)]
    quality: Option<u8>,

    /// Overwrite existing thumbnails
    #[arg(long)]
    overwrite: bool,

    /// Print intended actions without writing files
    #[arg(long)]
    dry_run: bool,

    /// Maximum number of parallel threads (default: min(2×CPU, 8))
    #[arg(long, value_name = "N", value_parser = parse_threads)]
    max_threads: Option<u16>,

    /// Output JSON format (default: auto-detect based on TTY)
    #[arg(long)]
    json: bool,
}

// ============================================================================
// Verify Subcommand
// ============================================================================

/// Verify sidecar integrity and detect staleness
///
/// Validates JSON schema structure, checks required fields presence,
/// detects stale sidecars (schema version mismatch), compares pipeline signatures,
/// verifies file hash consistency, and suggests actions: noop, rescan, migrate.
#[derive(Args)]
#[command(
    about = "Verify sidecar integrity and detect staleness",
    long_about = "Validates JSON schema structure, checks required fields presence,\n\
                  detects stale sidecars (schema version mismatch), compares pipeline signatures,\n\
                  verifies file hash consistency, and suggests actions: noop, rescan, migrate.\n\n\
                  Use --fix to attempt auto-repair of minor issues.\n\
                  Use --strict to treat warnings as errors.",
    after_help = "EXAMPLES:\n  \
                  # Verify all sidecars\n  \
                  jozin verify ~/Photos --recursive\n\n  \
                  # Auto-fix minor issues\n  \
                  jozin verify ~/Photos --fix\n\n  \
                  # Strict mode (warnings = errors)\n  \
                  jozin verify ~/Photos --strict\n\n  \
                  # Override pipeline signature\n  \
                  jozin verify ~/Photos --pipeline-signature '{\"schema_version\":\"1.0.0\"}'"
)]
struct VerifyArgs {
    /// File or directory path to verify
    path: PathBuf,

    /// Enable recursive directory traversal
    #[arg(short = 'r', long)]
    recursive: bool,

    /// Attempt auto-repair of minor issues
    #[arg(long)]
    fix: bool,

    /// Treat warnings as errors
    #[arg(long)]
    strict: bool,

    /// Override pipeline signature for comparison (JSON string)
    #[arg(long, value_name = "JSON")]
    pipeline_signature: Option<String>,

    /// Output JSON format (default: auto-detect based on TTY)
    #[arg(long)]
    json: bool,
}

// ============================================================================
// Migrate Subcommand
// ============================================================================

/// Migrate sidecars between schema versions
///
/// Upgrades sidecars to new schema versions. Auto-detects source version if not specified.
/// Creates backup rotation (.bak1, .bak2, .bak3). Uses atomic writes to prevent corruption.
/// Idempotent (safe to run multiple times).
struct MigrateArgs {
    /// File or directory path to migrate
    path: PathBuf,

    /// Enable recursive directory traversal
    recursive: bool,

    /// Source schema version (auto-detect if omitted)
    from: Option<String>,

    /// Target schema version (required)
    to: String,

    /// Print intended actions without writing files
    dry_run: bool,

    /// Create .bakN backup files (default: true, use --no-backup to disable)
    backup: bool,

    /// Output JSON format (default: auto-detect based on TTY)
    json: bool,
}

impl clap::FromArgMatches for MigrateArgs {
    fn from_arg_matches(matches: &clap::ArgMatches) -> std::result::Result<Self, clap::Error> {
        // Determine backup value: --backup sets true, --no-backup sets false, default is true
        let backup = if matches.get_flag("no_backup") {
            false
        } else if matches.get_flag("backup") {
            true
        } else {
            true // default
        };

        Ok(Self {
            path: matches.get_one::<PathBuf>("path").expect("required").clone(),
            recursive: matches.get_flag("recursive"),
            from: matches.get_one::<String>("from").cloned(),
            to: matches.get_one::<String>("to").expect("required").clone(),
            dry_run: matches.get_flag("dry_run"),
            backup,
            json: matches.get_flag("json"),
        })
    }

    fn update_from_arg_matches(&mut self, matches: &clap::ArgMatches) -> std::result::Result<(), clap::Error> {
        if let Some(path) = matches.get_one::<PathBuf>("path") {
            self.path = path.clone();
        }
        if matches.contains_id("recursive") {
            self.recursive = matches.get_flag("recursive");
        }
        if matches.contains_id("from") {
            self.from = matches.get_one::<String>("from").cloned();
        }
        if let Some(to) = matches.get_one::<String>("to") {
            self.to = to.clone();
        }
        if matches.contains_id("dry_run") {
            self.dry_run = matches.get_flag("dry_run");
        }
        // Update backup based on which flag was provided
        if matches.contains_id("no_backup") {
            self.backup = !matches.get_flag("no_backup");
        } else if matches.contains_id("backup") {
            self.backup = matches.get_flag("backup");
        }
        if matches.contains_id("json") {
            self.json = matches.get_flag("json");
        }
        Ok(())
    }
}

impl clap::Args for MigrateArgs {
    fn augment_args(cmd: clap::Command) -> clap::Command {
        cmd
            .about("Migrate sidecars between schema versions")
            .long_about("Upgrades sidecars to new schema versions. Auto-detects source version if not specified.\n\
                         Creates backup rotation (.bak1, .bak2, .bak3). Uses atomic writes to prevent corruption.\n\
                         Idempotent (safe to run multiple times).\n\n\
                         Use --dry-run to preview changes without writing.\n\
                         Use --no-backup to skip creating backup files.")
            .after_help("EXAMPLES:\n  \
                         # Migrate to version 2.0.0 (auto-detect source)\n  \
                         jozin migrate ~/Photos --to 2.0.0 --recursive\n\n  \
                         # Explicit source and target versions\n  \
                         jozin migrate ~/Photos --from 1.0.0 --to 2.0.0\n\n  \
                         # Dry run to preview changes\n  \
                         jozin migrate ~/Photos --to 2.0.0 --dry-run\n\n  \
                         # Migrate without backups\n  \
                         jozin migrate ~/Photos --to 2.0.0 --no-backup")
            .arg(clap::Arg::new("path")
                .required(true)
                .value_name("PATH")
                .value_parser(clap::value_parser!(PathBuf))
                .help("File or directory path to migrate"))
            .arg(clap::Arg::new("recursive")
                .short('r')
                .long("recursive")
                .action(clap::ArgAction::SetTrue)
                .help("Enable recursive directory traversal"))
            .arg(clap::Arg::new("from")
                .long("from")
                .value_name("VERSION")
                .help("Source schema version (auto-detect if omitted)"))
            .arg(clap::Arg::new("to")
                .long("to")
                .value_name("VERSION")
                .required(true)
                .help("Target schema version (required)"))
            .arg(clap::Arg::new("dry_run")
                .long("dry-run")
                .action(clap::ArgAction::SetTrue)
                .help("Print intended actions without writing files"))
            .arg(clap::Arg::new("backup")
                .long("backup")
                .action(clap::ArgAction::SetTrue)
                .overrides_with_all(["backup", "no_backup"])
                .help("Create .bakN backup files"))
            .arg(clap::Arg::new("no_backup")
                .long("no-backup")
                .action(clap::ArgAction::SetTrue)
                .overrides_with_all(["backup", "no_backup"])
                .help("Skip creating .bakN backup files"))
            .arg(clap::Arg::new("json")
                .long("json")
                .action(clap::ArgAction::SetTrue)
                .help("Output JSON format (default: auto-detect based on TTY)"))
    }

    fn augment_args_for_update(cmd: clap::Command) -> clap::Command {
        Self::augment_args(cmd)
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Returns the default max_threads value: min(2×CPU, 8)
///
/// This implements the default parallelism strategy from TASK+PHASE_PLAN.md.
fn default_max_threads() -> u16 {
    std::cmp::min(num_cpus::get() * 2, 8) as u16
}

/// Custom value parser for clap to parse comma-separated sizes
fn parse_sizes_for_clap(sizes_str: &str) -> std::result::Result<Vec<u32>, String> {
    sizes_str
        .split(',')
        .map(|s| {
            let trimmed = s.trim();
            let val = trimmed.parse::<u32>().map_err(|_| {
                format!("Invalid size value: '{}' - must be a positive integer", trimmed)
            })?;
            if val == 0 {
                return Err("size values must be positive integers (> 0)".to_string());
            }
            Ok(val)
        })
        .collect()
}

/// Custom value parser for score validation (0.0-1.0)
fn parse_score(s: &str) -> std::result::Result<f32, String> {
    let score: f32 = s.parse().map_err(|_| "not a valid number")?;
    if (0.0..=1.0).contains(&score) {
        Ok(score)
    } else {
        Err("score must be between 0.0 and 1.0".to_string())
    }
}

/// Custom value parser for thread count (must be > 0)
fn parse_threads(s: &str) -> std::result::Result<u16, String> {
    let threads: u16 = s.parse().map_err(|_| "not a valid number")?;
    if threads > 0 {
        Ok(threads)
    } else {
        Err("max_threads must be greater than 0".to_string())
    }
}

/// Custom value parser for quality (1-100)
fn parse_quality(s: &str) -> std::result::Result<u8, String> {
    let quality: u8 = s.parse().map_err(|_| "not a valid number")?;
    if (1..=100).contains(&quality) {
        Ok(quality)
    } else {
        Err("quality must be between 1 and 100".to_string())
    }
}

/// Parses comma-separated patterns into a vector of strings
fn parse_patterns(patterns_str: &str) -> Vec<String> {
    patterns_str
        .split(',')
        .map(|s| s.trim().to_string())
        .collect()
}

// ============================================================================
// Validation Functions
// ============================================================================
//
// NOTE: Most validation is now handled by clap's value_parser attributes.
// These functions only handle complex validation that can't be expressed
// declaratively in clap (e.g., JSON validation, non-empty patterns).

/// Validates scan command arguments
///
/// Clap handles: max_threads > 0, hash_mode enum validation
/// This function handles: non-empty glob patterns
fn validate_scan_args(args: &ScanArgs) -> Result<()> {
    // Validate glob patterns are non-empty
    if let Some(ref patterns) = args.include {
        if patterns.trim().is_empty() {
            return Err(JozinError::UserError {
                message: "include patterns cannot be empty".to_string(),
            });
        }
    }
    if let Some(ref patterns) = args.exclude {
        if patterns.trim().is_empty() {
            return Err(JozinError::UserError {
                message: "exclude patterns cannot be empty".to_string(),
            });
        }
    }
    Ok(())
}

/// Validates cleanup command arguments
///
/// Clap handles: --only-* flag conflicts
/// No additional validation needed.
fn validate_cleanup_args(_args: &CleanupArgs) -> Result<()> {
    Ok(())
}

/// Validates faces command arguments
///
/// Clap handles: min_score range (0.0-1.0), max_threads > 0
/// This function handles: train JSON validation
fn validate_faces_args(args: &FacesArgs) -> Result<()> {
    // Validate train JSON if provided
    if let Some(ref train_json) = args.train {
        serde_json::from_str::<serde_json::Value>(train_json).map_err(|e| {
            JozinError::UserError {
                message: format!("Invalid train JSON: {}", e),
            }
        })?;
    }
    Ok(())
}

/// Validates tags command arguments
///
/// Clap handles: min_score range (0.0-1.0), mode enum validation
/// No additional validation needed.
fn validate_tags_args(_args: &TagsArgs) -> Result<()> {
    Ok(())
}

/// Validates thumbs command arguments
///
/// Clap handles: quality range (1-100), sizes validation, format enum, max_threads > 0
/// No additional validation needed.
fn validate_thumbs_args(_args: &ThumbsArgs) -> Result<()> {
    Ok(())
}

/// Validates verify command arguments
///
/// Enforces parameter constraints:
/// - pipeline_signature must be valid JSON
fn validate_verify_args(args: &VerifyArgs) -> Result<()> {
    // Validate pipeline_signature JSON
    if let Some(ref sig_json) = args.pipeline_signature {
        serde_json::from_str::<serde_json::Value>(sig_json).map_err(|e| {
            JozinError::UserError {
                message: format!("Invalid pipeline_signature JSON: {}", e),
            }
        })?;
    }

    Ok(())
}

/// Validates migrate command arguments
///
/// Enforces parameter constraints:
/// - to is required and non-empty
/// - from and to are valid semver strings (basic check)
fn validate_migrate_args(args: &MigrateArgs) -> Result<()> {
    // Validate 'to' is non-empty
    if args.to.trim().is_empty() {
        return Err(JozinError::UserError {
            message: "to version cannot be empty".to_string(),
        });
    }

    // Basic semver validation (just check format: X.Y.Z)
    let validate_version = |v: &str| -> Result<()> {
        let parts: Vec<&str> = v.split('.').collect();
        if parts.len() != 3 {
            return Err(JozinError::UserError {
                message: format!("Invalid version format '{}': expected X.Y.Z", v),
            });
        }
        for part in parts {
            if part.parse::<u32>().is_err() {
                return Err(JozinError::UserError {
                    message: format!("Invalid version format '{}': parts must be numbers", v),
                });
            }
        }
        Ok(())
    };

    validate_version(&args.to)?;
    if let Some(ref from) = args.from {
        validate_version(from)?;
    }

    Ok(())
}

// ============================================================================
// Progress Printing Helpers
// ============================================================================

/// Prints a progress event in human-readable format.
///
/// This function is used as a callback during operations to provide real-time
/// progress feedback. It displays:
/// - File path relative to base directory
/// - Success indicator (✓) or error indicator (✗) with error message
///
/// # Arguments
///
/// * `base_path` - Base path to strip from file paths for cleaner output
/// * `event` - The progress event to print
fn print_progress(base_path: &std::path::Path, event: jozin_core::ProgressEvent) {
    match event {
        jozin_core::ProgressEvent::FileStarted { .. } => {
            // Don't print anything on start, wait for completion
        }
        jozin_core::ProgressEvent::FileCompleted { path, success, error, .. } => {
            // Calculate relative path for cleaner display
            let display_path = std::path::Path::new(&path)
                .strip_prefix(base_path)
                .unwrap_or(std::path::Path::new(&path));

            if success {
                println!("{} ... ✓", display_path.display());
            } else {
                let error_msg = error.as_deref().unwrap_or("unknown error");
                println!("{} ... ✗ {}", display_path.display(), error_msg);
            }
        }
    }
}

// ============================================================================
// Command Handlers (Phase 1 Stubs)
// ============================================================================

/// Response structure for Phase 1 stubs
///
/// Contains the module name and parsed parameters.
/// In Phase 2+, this will be replaced with actual operation results.
#[derive(Serialize)]
struct StubResponse {
    module: String,
    parameters: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    dry_run: Option<bool>,
}

/// Handles scan command
///
/// Phase 1: Implements file and directory scanning with hash computation.
/// Phase 2: Supports both human-readable progress and JSON output modes.
fn handle_scan(args: ScanArgs) -> Result<()> {
    let start = OffsetDateTime::now_utc();

    // Parse glob patterns from comma-separated strings
    let include = args.include.as_ref().map(|s| parse_patterns(s));
    let exclude = args.exclude.as_ref().map(|s| parse_patterns(s));

    // Get max_threads with default
    let max_threads = args.max_threads.unwrap_or_else(default_max_threads);

    // Get hash_mode with default to "file"
    let hash_mode = args.hash_mode.as_ref().map(|m| m.as_str()).or(Some("file"));

    // Determine output format
    let output_format = determine_output_format(args.json);

    // Call scan_path with appropriate callback based on output format
    let result = match output_format {
        OutputFormat::Human => {
            // Clone path for closure
            let base_path = args.path.clone();

            // Call scan_path with progress callback
            jozin_core::scan_path(
                &args.path,
                args.recursive,
                include.as_deref(),
                exclude.as_deref(),
                args.dry_run,
                max_threads,
                hash_mode,
                Some(&|event| print_progress(&base_path, event)),
            )?
        }
        OutputFormat::Json => {
            // Call scan_path without callback (silent mode)
            jozin_core::scan_path(
                &args.path,
                args.recursive,
                include.as_deref(),
                exclude.as_deref(),
                args.dry_run,
                max_threads,
                hash_mode,
                None,
            )?
        }
    };

    let end = OffsetDateTime::now_utc();

    // Print output based on format
    match output_format {
        OutputFormat::Human => {
            // Print summary
            let duration_secs = (end - start).whole_milliseconds() as f64 / 1000.0;
            println!("\nProcessed {} files in {:.2}s", result.total_files, duration_secs);
            println!("  Successful: {}", result.successful);
            println!("  Failed: {}", result.failed);
            println!("  Skipped: {}", result.skipped);
        }
        OutputFormat::Json => {
            // Print JSON to stdout
            let response = OperationResponse::new(result, start, end)?;
            println!("{}", serde_json::to_string_pretty(&response)?);
        }
    }

    Ok(())
}

/// Handles cleanup command
///
/// Removes Jožin-generated files based on user selection.
/// By default removes all types; --only-* flags select specific types.
fn handle_cleanup(args: CleanupArgs) -> Result<()> {
    let start = OffsetDateTime::now_utc();

    // Determine cleanup options based on flags
    let options = if args.only_sidecars {
        jozin_core::CleanupOptions::sidecars_only()
    } else if args.only_thumbnails {
        jozin_core::CleanupOptions::thumbnails_only()
    } else if args.only_backups {
        jozin_core::CleanupOptions::backups_only()
    } else if args.only_cache {
        jozin_core::CleanupOptions::cache_only()
    } else {
        // Default: remove all
        jozin_core::CleanupOptions::all()
    };

    // Determine output format
    let output_format = determine_output_format(args.json);

    // Call cleanup_path with appropriate callback based on output format
    let result = match output_format {
        OutputFormat::Human => {
            // Clone path for closure
            let base_path = args.path.clone();

            // Call cleanup_path with progress callback
            jozin_core::cleanup_path(
                &args.path,
                args.recursive,
                options,
                args.dry_run,
                Some(&|event| print_progress(&base_path, event)),
            )?
        }
        OutputFormat::Json => {
            // Call cleanup_path without callback (silent mode)
            jozin_core::cleanup_path(
                &args.path,
                args.recursive,
                options,
                args.dry_run,
                None,
            )?
        }
    };

    let end = OffsetDateTime::now_utc();

    // Print output based on format
    match output_format {
        OutputFormat::Human => {
            // Print summary
            let duration_secs = (end - start).whole_milliseconds() as f64 / 1000.0;
            println!("\nDeleted {} files ({} bytes) in {:.2}s", result.total_files, result.total_bytes, duration_secs);
            if result.failed > 0 {
                println!("  Failed: {}", result.failed);
            }
        }
        OutputFormat::Json => {
            // Print JSON to stdout
            let response = OperationResponse::new(result, start, end)?;
            println!("{}", serde_json::to_string_pretty(&response)?);
        }
    }

    Ok(())
}

/// Handles faces command
///
/// Phase 1 stub: prints parsed parameters as JSON.
/// Phase 2+: will call jozin_core::faces functions.
fn handle_faces(args: FacesArgs) -> Result<()> {
    let start = OffsetDateTime::now_utc();

    let module = if args.dry_run { "DRY RUN: faces".to_string() } else { "faces".to_string() };
    let data = StubResponse {
        module,
        parameters: serde_json::json!({
            "path": args.path.display().to_string(),
            "recursive": args.recursive,
            "model": args.model,
            "identify": args.identify,
            "train": args.train,
            "min_score": args.min_score.unwrap_or(0.8),
            "max_threads": args.max_threads.unwrap_or_else(default_max_threads),
        }),
        dry_run: if args.dry_run { Some(true) } else { None },
    };

    let end = OffsetDateTime::now_utc();
    let response = OperationResponse::new(data, start, end)?;

    println!("{}", serde_json::to_string_pretty(&response)?);

    Ok(())
}

/// Handles tags command
///
/// Phase 1 stub: prints parsed parameters as JSON.
/// Phase 2+: will call jozin_core::tags functions.
fn handle_tags(args: TagsArgs) -> Result<()> {
    let start = OffsetDateTime::now_utc();

    let module = if args.dry_run { "DRY RUN: tags".to_string() } else { "tags".to_string() };
    let data = StubResponse {
        module,
        parameters: serde_json::json!({
            "path": args.path.display().to_string(),
            "recursive": args.recursive,
            "mode": args.mode.as_ref().map(|m| m.as_str()).unwrap_or("both"),
            "model": args.model,
            "min_score": args.min_score.unwrap_or(0.6),
            "append": args.append,
        }),
        dry_run: if args.dry_run { Some(true) } else { None },
    };

    let end = OffsetDateTime::now_utc();
    let response = OperationResponse::new(data, start, end)?;

    println!("{}", serde_json::to_string_pretty(&response)?);

    Ok(())
}

/// Handles thumbs command
///
/// Phase 1 stub: prints parsed parameters as JSON.
/// Phase 2+: will call jozin_core::thumbs functions.
fn handle_thumbs(args: ThumbsArgs) -> Result<()> {
    let start = OffsetDateTime::now_utc();

    let module = if args.dry_run { "DRY RUN: thumbs".to_string() } else { "thumbs".to_string() };
    let data = StubResponse {
        module,
        parameters: serde_json::json!({
            "path": args.path.display().to_string(),
            "recursive": args.recursive,
            "sizes": args.sizes.unwrap_or_else(|| vec![512]),
            "format": args.format.as_ref().map(|f| f.as_str()).unwrap_or("jpg"),
            "quality": args.quality.unwrap_or(85),
            "overwrite": args.overwrite,
            "max_threads": args.max_threads.unwrap_or_else(default_max_threads),
        }),
        dry_run: if args.dry_run { Some(true) } else { None },
    };

    let end = OffsetDateTime::now_utc();
    let response = OperationResponse::new(data, start, end)?;

    println!("{}", serde_json::to_string_pretty(&response)?);

    Ok(())
}

/// Handles verify command
///
/// Phase 1 stub: prints parsed parameters as JSON.
/// Phase 2+: will call jozin_core::verify functions.
fn handle_verify(args: VerifyArgs) -> Result<()> {
    let start = OffsetDateTime::now_utc();

    let data = StubResponse {
        module: "verify".to_string(),
        parameters: serde_json::json!({
            "path": args.path.display().to_string(),
            "recursive": args.recursive,
            "fix": args.fix,
            "strict": args.strict,
            "pipeline_signature": args.pipeline_signature,
        }),
        dry_run: None,
    };

    let end = OffsetDateTime::now_utc();
    let response = OperationResponse::new(data, start, end)?;

    println!("{}", serde_json::to_string_pretty(&response)?);

    Ok(())
}

/// Handles migrate command
///
/// Phase 1 stub: prints parsed parameters as JSON.
/// Phase 2+: will call jozin_core::migrate functions.
fn handle_migrate(args: MigrateArgs) -> Result<()> {
    let start = OffsetDateTime::now_utc();

    let module = if args.dry_run { "DRY RUN: migrate".to_string() } else { "migrate".to_string() };
    let data = StubResponse {
        module,
        parameters: serde_json::json!({
            "path": args.path.display().to_string(),
            "recursive": args.recursive,
            "from": args.from,
            "to": args.to,
            "backup": args.backup,
        }),
        dry_run: if args.dry_run { Some(true) } else { None },
    };

    let end = OffsetDateTime::now_utc();
    let response = OperationResponse::new(data, start, end)?;

    println!("{}", serde_json::to_string_pretty(&response)?);

    Ok(())
}

// ============================================================================
// Main Entry Point
// ============================================================================

/// Executes the requested command with validation and error handling
///
/// Routes each command to its validation function and handler.
/// Returns JozinError with appropriate exit code on failure.
fn run_command(cmd: Cmd) -> Result<()> {
    match cmd {
        Cmd::Scan(args) => {
            validate_scan_args(&args)?;
            handle_scan(args)
        }
        Cmd::Cleanup(args) => {
            validate_cleanup_args(&args)?;
            handle_cleanup(args)
        }
        Cmd::Faces(args) => {
            validate_faces_args(&args)?;
            handle_faces(args)
        }
        Cmd::Tags(args) => {
            validate_tags_args(&args)?;
            handle_tags(args)
        }
        Cmd::Thumbs(args) => {
            validate_thumbs_args(&args)?;
            handle_thumbs(args)
        }
        Cmd::Verify(args) => {
            validate_verify_args(&args)?;
            handle_verify(args)
        }
        Cmd::Migrate(args) => {
            validate_migrate_args(&args)?;
            handle_migrate(args)
        }
    }
}

/// Main entry point
///
/// Parses CLI arguments, runs the requested command, and handles errors.
/// Errors are printed to stderr as JSON with appropriate exit codes (1-4).
fn main() {
    let args = CliArgs::parse();

    // Run the command and handle errors
    if let Err(e) = run_command(args.cmd) {
        let exit_code = e.exit_code();

        // Try to serialize JozinError directly; fall back to custom object on failure
        let error_output = serde_json::to_string_pretty(&e)
            .unwrap_or_else(|_| {
                // Fallback if JozinError serialization fails
                let fallback = serde_json::json!({
                    "error": e.to_string(),
                    "exit_code": exit_code,
                });
                serde_json::to_string_pretty(&fallback)
                    .unwrap_or_else(|_| format!("{{\"error\":\"{}\"}}", e))
            });

        eprintln!("{}", error_output);

        // Exit with appropriate code
        exit(exit_code);
    }
}

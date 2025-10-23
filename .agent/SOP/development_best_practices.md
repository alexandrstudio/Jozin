# Development Best Practices - Jožin

**Last Updated**: 2025-10-21

---

## How to Add a New Module

### 1. Create Module File

**Location**: `core/src/your_module.rs`

```rust
//! # Your Module
//!
//! Brief description of what this module does.
//!
//! See TASK+PHASE_PLAN.md for parameter specifications.

use crate::{JozinError, Result, OperationResponse, ProgressEvent};
use std::path::Path;
use time::OffsetDateTime;

/// Your module's main function
///
/// # Arguments
///
/// * `path` - File or directory to process
/// * `recursive` - Process directories recursively
/// * `dry_run` - Preview without writing files
///
/// # Returns
///
/// `OperationResponse` with timing metadata and results
pub fn your_module_path(
    path: &Path,
    recursive: bool,
    dry_run: bool,
) -> Result<OperationResponse<YourResult>> {
    let start = OffsetDateTime::now_utc();

    // Your implementation here
    let data = YourResult {
        // ...
    };

    let end = OffsetDateTime::now_utc();
    OperationResponse::new(data, start, end)
}

/// Result structure for your module
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct YourResult {
    // Your fields
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_your_module() {
        let dir = tempdir().unwrap();
        // Your test implementation
    }
}
```

### 2. Export Module in lib.rs

**Location**: `core/src/lib.rs`

```rust
// Add module declaration
pub mod your_module;

// Re-export commonly used types
pub use your_module::{your_module_path, YourResult};
```

### 3. Wire CLI Command

**Location**: `cli/src/main.rs`

Add subcommand to `Commands` enum:

```rust
#[derive(Subcommand)]
enum Commands {
    // ... existing commands
    YourModule(YourModuleArgs),
}

#[derive(Args)]
struct YourModuleArgs {
    /// Path to file or directory
    path: PathBuf,

    /// Process directories recursively
    #[arg(short, long)]
    recursive: bool,

    /// Preview without writing files
    #[arg(long)]
    dry_run: bool,
}
```

Add command handler:

```rust
match cli.command {
    // ... existing handlers
    Commands::YourModule(args) => {
        let result = jozin_core::your_module_path(
            &args.path,
            args.recursive,
            args.dry_run,
        )?;
        output_json(&result)?;
    }
}
```

### 4. Add Tests

**CLI Test** (`cli/tests/cli_basic.rs`):

```rust
#[test]
fn test_your_module_help() {
    Command::cargo_bin("jozin")
        .unwrap()
        .arg("your-module")
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn test_your_module_dry_run() {
    let dir = tempdir().unwrap();
    Command::cargo_bin("jozin")
        .unwrap()
        .arg("your-module")
        .arg(dir.path())
        .arg("--dry-run")
        .assert()
        .success();
}
```

---

## How to Add Schema Migration

### 1. Define Migration Function

**Location**: `core/src/migrate.rs`

```rust
fn migrate_v1_0_to_v1_1(sidecar: &mut Sidecar) -> Result<()> {
    // Example: Split camera field
    if let Some(ref mut image) = sidecar.image {
        if let Some(camera) = &image.camera {
            // Parse "Apple iPhone 12" into make + model
            let parts: Vec<&str> = camera.split_whitespace().collect();
            if parts.len() >= 2 {
                image.camera_make = Some(parts[0].to_string());
                image.camera_model = Some(parts[1..].join(" "));
            }
        }
    }

    // Update schema version
    sidecar.schema_version = "1.1.0".to_string();
    Ok(())
}
```

### 2. Register in Migration Dispatcher

```rust
fn apply_migration(from: &str, to: &str, sidecar: &mut Sidecar) -> Result<()> {
    match (from, to) {
        ("1.0.0", "1.0.0") => Ok(()), // No-op
        ("1.0.0", "1.1.0") => migrate_v1_0_to_v1_1(sidecar),
        // Add your migration here
        _ => Err(JozinError::UserError {
            message: format!("No migration path from {} to {}", from, to)
        })
    }
}
```

### 3. Add Tests

```rust
#[test]
fn test_migrate_v1_0_to_v1_1() {
    let mut sidecar = Sidecar {
        schema_version: "1.0.0".to_string(),
        image: Some(ImageInfo {
            camera: Some("Apple iPhone 12".to_string()),
            camera_make: None,
            camera_model: None,
            // ...
        }),
        // ...
    };

    migrate_v1_0_to_v1_1(&mut sidecar).unwrap();

    assert_eq!(sidecar.schema_version, "1.1.0");
    assert_eq!(sidecar.image.unwrap().camera_make, Some("Apple".to_string()));
    assert_eq!(sidecar.image.unwrap().camera_model, Some("iPhone 12".to_string()));
}
```

### 4. Create Backup Before Migration

Migration module automatically handles backup rotation:

```
IMG_1234.JPG.json       ← New version (v1.1.0)
IMG_1234.JPG.json.bak1  ← Previous version (v1.0.0)
IMG_1234.JPG.json.bak2  ← Two versions ago
IMG_1234.JPG.json.bak3  ← Three versions ago
```

---

## How to Run Tests

### Run All Tests

```bash
cargo test --workspace
```

### Run Specific Module Tests

```bash
# Core library tests
cargo test --package jozin-core scan
cargo test --package jozin-core cleanup
cargo test --package jozin-core verify
cargo test --package jozin-core migrate

# CLI tests
cargo test --package jozin
```

### Run Single Test

```bash
cargo test --package jozin-core test_scan_directory
```

### Show Test Output

```bash
cargo test --workspace -- --nocapture
```

### Run Tests with Verbose Output

```bash
cargo test --workspace -- --nocapture --test-threads=1
```

---

## How to Handle Errors

### Error Types

Use `JozinError` enum for all errors:

```rust
use crate::{JozinError, Result};

pub fn your_function() -> Result<()> {
    // User error (invalid arguments)
    if invalid_parameter {
        return Err(JozinError::UserError {
            message: "Invalid parameter value".to_string()
        });
    }

    // I/O error (file operations)
    std::fs::read_to_string(path)?;  // Automatically converts via From trait

    // Validation error (schema issues)
    if schema_invalid {
        return Err(JozinError::ValidationError {
            message: "Schema version mismatch".to_string()
        });
    }

    // Internal error (unexpected failures)
    if unexpected_condition {
        return Err(JozinError::InternalError {
            message: "Unexpected hash computation failure".to_string()
        });
    }

    Ok(())
}
```

### Exit Codes

| JozinError Variant | Exit Code | Usage |
|-------------------|-----------|-------|
| `UserError` | 1 | Invalid arguments, bad input |
| `IoError` | 2 | File not found, permission denied |
| `ValidationError` | 3 | Schema mismatch, corrupt sidecar |
| `InternalError` | 4 | Unexpected panics, logic errors |

### CLI Error Handling

```rust
// cli/src/main.rs
fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        std::process::exit(e.exit_code());
    }
}

fn run() -> Result<()> {
    // Your implementation
    Ok(())
}
```

---

## How to Write Atomic File Operations

### Atomic Write Pattern

Always use `.tmp` → fsync → rename for atomicity:

```rust
use std::fs::{File, rename};
use std::io::Write;
use std::path::Path;

fn write_sidecar_atomic(path: &Path, sidecar: &Sidecar) -> Result<()> {
    let tmp_path = path.with_extension("json.tmp");

    // 1. Write to temporary file
    let mut file = File::create(&tmp_path)?;
    let json = serde_json::to_string_pretty(sidecar)?;
    file.write_all(json.as_bytes())?;

    // 2. Sync to disk
    file.sync_all()?;

    // 3. Atomic rename
    rename(tmp_path, path)?;

    Ok(())
}
```

### Backup Rotation

```rust
fn create_backup(path: &Path) -> Result<()> {
    let bak3 = path.with_extension("json.bak3");
    let bak2 = path.with_extension("json.bak2");
    let bak1 = path.with_extension("json.bak1");

    // Rotate backups
    if bak2.exists() {
        rename(&bak2, &bak3)?;
    }
    if bak1.exists() {
        rename(&bak1, &bak2)?;
    }

    // Create new backup
    std::fs::copy(path, &bak1)?;

    Ok(())
}
```

---

## How to Add Progress Callbacks

### Define Progress Callback

```rust
use crate::ProgressEvent;

pub fn process_with_progress<F>(
    files: &[PathBuf],
    callback: Option<&F>
) -> Result<()>
where
    F: Fn(ProgressEvent),
{
    for file in files {
        // Emit start event
        if let Some(cb) = callback {
            cb(ProgressEvent::FileStarted {
                path: file.display().to_string(),
            });
        }

        // Process file
        let result = process_file(file);

        // Emit completion event
        if let Some(cb) = callback {
            cb(ProgressEvent::FileCompleted {
                path: file.display().to_string(),
                success: result.is_ok(),
                error: result.err().map(|e| e.to_string()),
                size_bytes: std::fs::metadata(file).ok().map(|m| m.len()),
            });
        }
    }

    Ok(())
}
```

### Use in CLI

```rust
// cli/src/main.rs
let callback = |event: ProgressEvent| {
    match event {
        ProgressEvent::FileStarted { path } => {
            println!("Processing: {}", path);
        }
        ProgressEvent::FileCompleted { path, success, .. } => {
            println!("{} ... {}", path, if success { "✓" } else { "✗" });
        }
    }
};

process_with_progress(&files, Some(&callback))?;
```

---

## How to Add Cargo Features

### 1. Define Feature in Cargo.toml

**Location**: `core/Cargo.toml`

```toml
[features]
default = []
your_feature = ["dep:your-crate", "dep:another-crate"]

[dependencies]
your-crate = { version = "1.0", optional = true }
another-crate = { version = "2.0", optional = true }
```

### 2. Feature-Gate Code

**Location**: `core/src/lib.rs`

```rust
#[cfg(feature = "your_feature")]
pub mod your_module;
```

### 3. Build with Feature

```bash
# Build without feature
cargo build --package jozin-core

# Build with feature
cargo build --package jozin-core --features your_feature

# Build with all features
cargo build --package jozin-core --all-features
```

---

## How to Format JSON Output

### Pretty-Print JSON

```rust
use serde_json;

let result = OperationResponse::new(data, start, end)?;
let json = serde_json::to_string_pretty(&result)?;
println!("{}", json);
```

### Compact JSON

```rust
let json = serde_json::to_string(&result)?;
println!("{}", json);
```

### Conditional Output

```rust
fn output_json<T: Serialize>(data: &T, pretty: bool) -> Result<()> {
    let json = if pretty {
        serde_json::to_string_pretty(data)?
    } else {
        serde_json::to_string(data)?
    };
    println!("{}", json);
    Ok(())
}
```

---

## How to Use justfile

### Common Commands

```bash
# Build entire workspace
just build

# Quick CLI test (scan with dry-run)
just cli

# Run all tests
just test

# Build release binaries
just release

# Launch Tauri app in dev mode
just app-dev
```

### Add New Command

**Location**: `justfile`

```makefile
# Your custom command
your-command:
    cargo run -p jozin -- your-module ./Photos --dry-run
```

---

## Common Pitfalls to Avoid

### 1. Modifying Original Files

**❌ NEVER DO THIS:**
```rust
std::fs::write(original_photo_path, modified_data)?;
```

**✅ CORRECT:**
```rust
// Only write to sidecar files (*.json)
let sidecar_path = original_photo_path.with_extension("JPG.json");
std::fs::write(sidecar_path, sidecar_json)?;
```

### 2. Non-Atomic Writes

**❌ NEVER DO THIS:**
```rust
std::fs::write(sidecar_path, json)?;  // Not atomic!
```

**✅ CORRECT:**
```rust
// Use tmp → sync → rename pattern
let tmp = sidecar_path.with_extension("json.tmp");
let mut file = File::create(&tmp)?;
file.write_all(json.as_bytes())?;
file.sync_all()?;
rename(tmp, sidecar_path)?;
```

### 3. Missing Timing Metadata

**❌ NEVER DO THIS:**
```rust
pub fn scan_path() -> Result<Vec<ScannedFile>> {
    // Missing timing metadata
}
```

**✅ CORRECT:**
```rust
pub fn scan_path() -> Result<OperationResponse<Vec<ScannedFile>>> {
    let start = OffsetDateTime::now_utc();
    // ... implementation
    let end = OffsetDateTime::now_utc();
    OperationResponse::new(data, start, end)
}
```

### 4. Hardcoded Exit Codes

**❌ NEVER DO THIS:**
```rust
std::process::exit(2);  // Magic number!
```

**✅ CORRECT:**
```rust
if let Err(e) = run() {
    eprintln!("Error: {}", e);
    std::process::exit(e.exit_code());  // Uses JozinError::exit_code()
}
```

### 5. Non-Deterministic Tests

**❌ AVOID THIS:**
```rust
#[test]
fn test_scan() {
    let timestamp = OffsetDateTime::now_utc();  // Non-deterministic!
    // Assertions may fail randomly
}
```

**✅ CORRECT:**
```rust
#[test]
fn test_scan() {
    // Use fixed timestamps for reproducibility
    let timestamp = OffsetDateTime::from_unix_timestamp(1609459200).unwrap();
    // Assertions always pass
}
```

---

## Related Documentation

- **.agent/System/project_architecture.md** - Technical architecture
- **.agent/Tasks/phase1_completion.md** - Current tasks
- **TASKMASTER_PLAN.md** - Detailed task specifications
- **SCOPE.md** - Architectural constraints
- **README.md** - Developer onboarding

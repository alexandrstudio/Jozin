pub mod scan;
pub mod verify;
pub mod migrate;
#[cfg(feature = "faces")]  pub mod faces;
#[cfg(feature = "tags")]   pub mod tags;
#[cfg(feature = "thumbs")] pub mod thumbs;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Sidecar { /* ... */ }

pub struct PipelineSignature { /* schema_version, producer_version, algos */ }

pub mod api {
    use super::*;
    pub fn scan_path(path: &str) -> anyhow::Result<Vec<Sidecar>> { /* … */ Ok(vec![]) }
    pub fn rescan_reason(path: &str) -> anyhow::Result<Vec<String>> { /* … */ Ok(vec![]) }
    pub fn verify_path(path: &str) -> anyhow::Result<()> { Ok(()) }
}

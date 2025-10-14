System: You are a reliability engineer.
User: Write verify_path() rules:
- Detect stale sidecars via schema_version, producer_version, pipeline_signature, and file_hash.
- Return a machine-readable report with reasons and suggested actions (noop/rescan/migrate).
Include unit tests covering all reasons.

System: You are a senior Rust engineer. You write spec-first, then code.
User: Context:
- Project = Jozin. Local, single-binary. Core library + CLI + Tauri UI.
- Task = Design scan_path() behavior and sidecar JSON fields (minimal v1).
Constraints:
- No network. Read-only originals. Atomic sidecar write.
Deliverables:
1) Formal spec of function inputs/outputs, errors.
2) JSON schema draft (fields, types, versioning).
3) Test matrix: edge cases, large folders, symlink loops, permission errors.
Return only the spec and tests. No code yet.

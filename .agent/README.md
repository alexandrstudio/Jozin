# Jožin Engineering Documentation

**Last Updated**: 2025-10-21
**Purpose**: Comprehensive technical documentation for engineers working on Jožin

---

## Quick Navigation

| I want to... | Read this |
|--------------|-----------|
| Understand the overall project architecture | [System/project_architecture.md](#system-documentation) |
| Know the current development tasks | [Tasks/phase1_completion.md](#task-documentation) |
| Learn best practices for adding features | [SOP/development_best_practices.md](#sop-documentation) |
| Get started as a new developer | [../README.md](../README.md) |
| Understand project scope and constraints | [../SCOPE.md](../SCOPE.md) |
| See detailed task breakdown | [../TASKMASTER_PLAN.md](../TASKMASTER_PLAN.md) |
| Get AI assistant guidance | [../CLAUDE.md](../CLAUDE.md) |

---

## Documentation Structure

```
.agent/
├── README.md                          # This file (documentation index)
├── System/                            # Architecture and technical design
│   └── project_architecture.md        # Complete technical architecture
├── Tasks/                             # PRDs and implementation plans
│   └── phase1_completion.md           # Current Phase 1 tasks (Tasks 1-5)
└── SOP/                               # Standard Operating Procedures
    └── development_best_practices.md  # How-to guides for common tasks
```

---

## System Documentation

### [System/project_architecture.md](System/project_architecture.md)

**Purpose**: Complete technical architecture documentation

**Contents**:
- Project goal and core philosophy
- Cargo workspace structure (core, cli, app)
- Technology stack (Rust, Clap, Serde, BLAKE3, etc.)
- Module architecture (scan, cleanup, verify, migrate, faces, tags, thumbs)
- Data structures (Sidecar, PipelineSignature, errors)
- CLI architecture and output formats
- File layout and sidecar policy
- Integration points (CLI → Core, Tauri → Core)
- Build and development workflow
- Current development status (Phase 1 near completion)

**Use this when**:
- Onboarding new engineers
- Understanding how modules interact
- Planning new features
- Debugging integration issues
- Reviewing architecture decisions

**Key Sections**:
- **Module Status**:
  - ✅ Fully Implemented: scan, cleanup
  - ⚠️ Minimal Stubs: verify, migrate
  - 🔒 Planned: faces, tags, thumbs
- **Data Structures**: Sidecar, PipelineSignature, JozinError, OperationResponse
- **Exit Codes**: 0 (success), 1 (user error), 2 (I/O error), 3 (validation), 4 (internal)
- **File Layout**: Adjacent sidecars, atomic writes, backup rotation

---

## Task Documentation

### [Tasks/phase1_completion.md](Tasks/phase1_completion.md)

**Purpose**: Detailed breakdown of current Phase 1 completion tasks

**Contents**:
- **Task 1**: Implement verify module core functionality
- **Task 2**: Add comprehensive tests for verify module
- **Task 3**: Implement migrate module core functionality
- **Task 4**: Add comprehensive tests for migrate module
- **Task 5**: Validate Phase 1 completion and create Phase 2 roadmap

**Use this when**:
- Starting work on verify or migrate modules
- Writing tests for new functionality
- Planning Phase 1 completion
- Understanding acceptance criteria
- Tracking task dependencies

**Key Information**:
- API signatures for verify and migrate modules
- Test coverage requirements (10+ test cases each)
- Output format specifications (JSON with timing metadata)
- Acceptance criteria (clear checklists)
- Task dependencies and execution order

**Status** (as of 2025-10-21):
- Task 1-4: 🔴 Pending (Critical priority)
- Task 5: 🔴 Pending (High priority)
- Estimated: 5 days total (2 days verify, 2 days migrate, 1 day validation)

---

## SOP Documentation

### [SOP/development_best_practices.md](SOP/development_best_practices.md)

**Purpose**: How-to guides for common development tasks

**Contents**:
- How to add a new module
- How to add schema migration
- How to run tests
- How to handle errors
- How to write atomic file operations
- How to add progress callbacks
- How to add Cargo features
- How to format JSON output
- How to use justfile
- Common pitfalls to avoid

**Use this when**:
- Adding a new core module (like faces, tags, thumbs)
- Implementing schema migrations
- Writing tests
- Debugging errors
- Ensuring atomic file writes
- Adding progress indicators
- Creating feature flags
- Avoiding common mistakes

**Quick References**:

**Add New Module**:
1. Create `core/src/your_module.rs`
2. Export in `core/src/lib.rs`
3. Wire CLI command in `cli/src/main.rs`
4. Add tests in `cli/tests/cli_basic.rs`

**Error Handling**:
- `JozinError::UserError` → Exit code 1
- `JozinError::IoError` → Exit code 2
- `JozinError::ValidationError` → Exit code 3
- `JozinError::InternalError` → Exit code 4

**Atomic Writes**:
```rust
// 1. Write to .tmp
// 2. Sync to disk
// 3. Atomic rename
```

**Common Commands**:
```bash
just build      # Build workspace
just cli        # Quick CLI test
just test       # Run all tests
just release    # Build release binaries
just app-dev    # Launch Tauri app
```

---

## Related Documentation (Root Level)

### [../README.md](../README.md)

**Purpose**: Developer onboarding and quick start

**Contents**:
- Quick start commands (clone, build, test)
- Current development status (Phase 1 near completion)
- Architecture overview (workspace structure)
- Module overview (detailed feature lists per module)
- Development setup (prerequisites, build, test)
- Tauri app development (planned, Task 6)
- Testing strategy (59 tests passing)
- File layout and sidecar policy
- Roadmap (Phase 1, Phase 2, Phase 2+)
- Technology stack table
- AI-assisted development context

**Use this when**:
- First time setting up development environment
- Understanding current implementation status
- Looking for quick CLI examples
- Checking build status
- Understanding roadmap

---

### [../SCOPE.md](../SCOPE.md)

**Purpose**: Architectural constraints and design principles

**Contents**:
- Project purpose (local-first photo organizer)
- Non-goals (no cloud, no destructive operations)
- Core principles (local-first, reversible, modular, schema-driven)
- Architecture overview (Core library, Tauri app, CLI)
- Local file structure (adjacent sidecars, no .jozin/ trees)
- Project layout (workspace structure)
- Folder and cache policy
- Design rationale (modular monolith, why Rust, why Tauri)
- Privacy and security
- Future outlook (plugin architecture, visual diff, etc.)

**Use this when**:
- Understanding architectural constraints
- Making design decisions
- Reviewing pull requests for compliance
- Planning new features
- Explaining project philosophy to stakeholders

**Key Principles**:
1. **Local-first & reversible** - Everything on user's machine
2. **Single process, modular design** - One Rust binary
3. **Schema-driven** - Versioned JSON sidecars
4. **Performance & safety** - Rust, parallel I/O, atomic writes
5. **Privacy & transparency** - No telemetry, local ML models
6. **Minimal footprint** - No hidden folders, user-visible state

---

### [../TASKMASTER_PLAN.md](../TASKMASTER_PLAN.md)

**Purpose**: Detailed task breakdown with specifications (Tasks 1-7)

**Contents**:
- Project status overview (Phase 1 near completion)
- Strategic goals (complete Phase 1, validate acceptance criteria)
- Task 1-7 detailed specifications:
  - Task 1: Implement verify module
  - Task 2: Test verify module
  - Task 3: Implement migrate module
  - Task 4: Test migrate module
  - Task 5: Validate Phase 1 completion
  - Task 6: Initialize Tauri app foundation
  - Task 7: Plan Phase 2 implementation strategy
- Task dependencies and execution strategy
- Success metrics (80+ tests, zero warnings)

**Use this when**:
- Starting work on any of Tasks 1-7
- Understanding detailed API specifications
- Checking acceptance criteria
- Planning implementation timeline
- Tracking task dependencies

**Key Differences from .agent/Tasks/phase1_completion.md**:
- TASKMASTER_PLAN.md: All 7 tasks (including Tauri app and Phase 2 planning)
- .agent/Tasks/phase1_completion.md: Focus on Phase 1 completion (Tasks 1-5)

---

### [../CLAUDE.md](../CLAUDE.md)

**Purpose**: AI assistant guidance for working with this codebase

**Contents**:
- Project overview (Jožin is local-first photo organizer)
- Architecture (workspace structure, modules)
- Key data structures (Sidecar, PipelineSignature)
- Common development commands (just, cargo)
- CLI command structure and parameters
- Exit codes (0/1/2/3/4)
- File layout and sidecar policy
- Development phases (Phase 0/1/2)
- Current state (Phase 1 near completion)
- Testing strategy (paths, files, OS, concurrency)
- AI-assisted development context
- Task planning and tracking (reference to TASKMASTER_PLAN.md)

**Use this when**:
- Working with AI coding assistants (Claude, GPT, etc.)
- Understanding context for AI pair programming
- Teaching AI about project conventions
- Ensuring AI follows project patterns

---

### [../BUILD_STATUS.md](../BUILD_STATUS.md)

**Purpose**: Detailed build and test status (if exists)

**Note**: Check if this file exists. If it does, it likely contains:
- Current build status (✅ or ❌)
- Test counts and breakdown
- Compiler warnings (should be zero)
- Clippy issues (should be none)
- Documentation build status

---

## Documentation Maintenance

### When to Update Documentation

| Event | Update These Files |
|-------|-------------------|
| New module added | `.agent/System/project_architecture.md`, `README.md`, `.agent/SOP/development_best_practices.md` |
| Task completed | `.agent/Tasks/phase1_completion.md`, `TASKMASTER_PLAN.md`, `README.md` |
| Schema changed | `.agent/System/project_architecture.md`, `.agent/SOP/development_best_practices.md` |
| New SOP needed | `.agent/SOP/development_best_practices.md`, `.agent/README.md` |
| Phase transition | All `.agent/` files, `README.md`, `TASKMASTER_PLAN.md` |
| Test count changed | `.agent/System/project_architecture.md`, `README.md` |
| Build status changed | `README.md`, `BUILD_STATUS.md` (if exists) |

### Documentation Ownership

| File | Primary Maintainer | Update Frequency |
|------|-------------------|------------------|
| `.agent/System/project_architecture.md` | Tech Lead | After major changes |
| `.agent/Tasks/phase1_completion.md` | Project Manager | Weekly |
| `.agent/SOP/development_best_practices.md` | Senior Engineers | As needed |
| `.agent/README.md` | Documentation Lead | Monthly |
| `README.md` | All Contributors | After each PR |
| `TASKMASTER_PLAN.md` | Project Manager | Daily during active sprints |
| `SCOPE.md` | Product Owner | Rarely (foundational) |
| `CLAUDE.md` | AI Pair Programming Lead | After architecture changes |

---

## Getting Help

### Common Questions

**Q: I'm a new engineer. Where do I start?**
A: Read these in order:
1. [../README.md](../README.md) - Developer onboarding
2. [System/project_architecture.md](System/project_architecture.md) - Technical architecture
3. [../SCOPE.md](../SCOPE.md) - Core principles
4. [SOP/development_best_practices.md](SOP/development_best_practices.md) - How-to guides

**Q: I want to implement Task 1 (verify module). Where are the specs?**
A: Read these:
1. [Tasks/phase1_completion.md](Tasks/phase1_completion.md#task-1-implement-verify-module-core-functionality) - Detailed requirements
2. [../TASKMASTER_PLAN.md](../TASKMASTER_PLAN.md) - Full task specification
3. [SOP/development_best_practices.md](SOP/development_best_practices.md#how-to-add-a-new-module) - Implementation guide

**Q: How do I add a schema migration?**
A: See [SOP/development_best_practices.md](SOP/development_best_practices.md#how-to-add-schema-migration)

**Q: What are the exit codes?**
A: See [System/project_architecture.md](System/project_architecture.md#exit-codes) or [SOP/development_best_practices.md](SOP/development_best_practices.md#error-types)

**Q: How do I run tests?**
A: See [SOP/development_best_practices.md](SOP/development_best_practices.md#how-to-run-tests)

**Q: Where is the database schema?**
A: Jožin doesn't use a traditional database. All metadata is stored in JSON sidecar files. See [System/project_architecture.md](System/project_architecture.md#data-structures) for sidecar schema.

---

## Contributing to Documentation

### Documentation Standards

1. **Use Markdown** - All docs are `.md` files
2. **Keep it Updated** - Update docs immediately after code changes
3. **Link Between Docs** - Use relative links to connect related docs
4. **Use Tables** - For comparing options or showing status
5. **Use Code Examples** - Show actual Rust code, not pseudocode
6. **Use Emojis for Status** - ✅ (done), ⚠️ (in progress), 🔒 (planned), 🔴 (pending)
7. **Include "Last Updated"** - Date stamp at top of each doc
8. **Add "Related Documentation"** - Links at bottom of each doc

### Documentation Review Checklist

Before committing documentation changes:
- [ ] Updated "Last Updated" timestamp
- [ ] Checked all links work
- [ ] Code examples compile
- [ ] Tables are aligned
- [ ] Status emojis are accurate
- [ ] Related docs are cross-referenced
- [ ] No sensitive information included
- [ ] Grammar and spelling checked

---

## Document Index

### .agent/ Directory

| File | Purpose | Last Updated |
|------|---------|--------------|
| `.agent/README.md` | This file (documentation index) | 2025-10-21 |
| `.agent/System/project_architecture.md` | Complete technical architecture | 2025-10-21 |
| `.agent/Tasks/phase1_completion.md` | Current Phase 1 tasks (1-5) | 2025-10-21 |
| `.agent/SOP/development_best_practices.md` | How-to guides for common tasks | 2025-10-21 |

### Root Level Documentation

| File | Purpose | Last Updated |
|------|---------|--------------|
| `README.md` | Developer onboarding | 2025-10-21 |
| `SCOPE.md` | Architectural constraints | Recent |
| `TASKMASTER_PLAN.md` | Task breakdown (1-7) | 2025-10-21 |
| `CLAUDE.md` | AI assistant guidance | 2025-10-21 |
| `BUILD_STATUS.md` | Build and test status | Check if exists |

---

**For questions or suggestions about documentation, please contact the Documentation Lead.**

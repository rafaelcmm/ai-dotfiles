# Source implementation guide

This document describes how the Rust implementation works, module boundaries, and behavior guarantees.

## Scope

- CLI command model and execution flow.
- Embedded file mapping and metadata generation.
- Managed file lifecycle (install, update, debloat).
- Safety guarantees and test coverage.

## High-level architecture

1. The binary entrypoint parses CLI arguments and enforces runtime safety checks.
2. The library orchestrates install/update/debloat operations against a target HOME.
3. Embedded static files are transformed into versioned managed destinations.
4. Filesystem helpers manage only package-owned content, without traversing symlinked directories.

## Module map

- `main.rs`
  - Defines CLI commands and flags.
  - Validates HOME boundary before running operations.
  - Triggers self-update flow on `update` (unless disabled).

- `lib.rs`
  - Library entrypoint.
  - Exposes `Command` and `run` to execute operations.

- `constants.rs`
  - Shared constants and enums.
  - Defines managed roots: `agents`, `rules`, `instructions`, `skills`.
  - Defines managed filename prefix: `rafaelcmm-`.

- `operations.rs`
  - Core behavior for `install`, `update`, and `debloat`.
  - Applies non-destructive managed-file rules.

- `embedded.rs`
  - Reads embedded files from `static/` (via `include_dir`).
  - Merges platform-specific and shared trees.
  - Applies destination path transformation and version prefixing.
  - Adds generated `_meta.md` into desired outputs.

- `meta.rs`
  - Renders `_meta.md` from `static/_meta_template.md`.
  - Extracts installed version from existing metadata.

- `fs_ops.rs`
  - Collects existing managed files for cleanup/update.
  - Removes empty managed directories after cleanup.
  - Treats symlinks as leaf entries and never traverses them.

- `self_update.rs`
  - Checks latest GitHub release tag.
  - Optionally installs newer binary via `cargo-binstall`.
  - Verifies `SHA256SUMS` contains expected target asset entry before install.
  - Re-executes `update` with `--no-self-update` after successful upgrade.

- `tests.rs`
  - Integration-style tests for install/update/debloat safety and behavior.

## Command behavior details

### Install

- Creates managed files only when no prior metadata is found.
- If an installation is already present, returns a message instructing to use `update`.
- Writes versioned managed files across supported platform roots.

### Update

- If all installed platform metadata versions match current package version, exits as up to date.
- If not installed yet, bootstraps managed files as a fresh synchronization.
- Otherwise:
  - Computes desired managed set for current version.
  - Removes only stale managed files.
  - Writes only changed/new files.
  - Preserves unmanaged user files.

### Debloat

- Removes only managed files that match managed prefix/rules.
- Removes `_meta.md` only when it appears tool-generated based on a marker phrase check.
- Leaves custom unmanaged files untouched.

## Managed path model

Static source layout:

- Platform-specific: `static/.claude`, `static/.copilot`, `static/.cursor`
- Shared: `static/__shared__`

Destination model:

- Content under managed roots is prefixed as `rafaelcmm-<version>-...`.
- `_meta.md` is written directly under each platform root.

Example mapping (version `1.0.0`):

- `static/.claude/agents/rust-specialist.md` -> `~/.claude/agents/rafaelcmm-1.0.0-rust-specialist.md`
- `static/__shared__/skills/clean-code/SKILL.md` -> `~/.copilot/skills/rafaelcmm-1.0.0-clean-code/SKILL.md`

## Safety and security guarantees

- CLI operations are constrained to HOME unless explicit override is provided via hidden flags.
- Symlinked directories are never traversed during update/debloat cleanup.
- Self-update is fail-open: network/install errors do not block config synchronization.
- Release checksum list is verified for expected asset presence before self-update install.

## Development and validation

Build:

```bash
cargo build
```

Run commands locally:

```bash
cargo run -- install
cargo run -- update
cargo run -- debloat
```

Run tests:

```bash
cargo test
```

## Extending behavior

When adding new managed configuration content:

1. Add files under one of the static trees.
2. Keep content under managed roots (`agents`, `rules`/`instructions`, `skills`).
3. Validate behavior with `cargo test`.
4. Confirm generated destination paths still follow managed prefix/versioning rules.

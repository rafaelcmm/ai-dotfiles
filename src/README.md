# Source implementation guide

This document describes how the Rust implementation works, module boundaries, and behavior guarantees.

## Scope

- CLI command model and execution flow.
- Embedded file mapping and metadata generation.
- External skill source resolution and cache behavior.
- Managed file lifecycle (install, update, debloat).
- Safety guarantees and test coverage.

## High-level architecture

1. The binary entrypoint parses CLI arguments and enforces runtime safety checks.
2. The library orchestrates install/update/debloat operations against a target HOME.
3. Embedded and external sources resolve to canonical platform-relative destinations.
4. Each platform keeps one `_meta.md` whose YAML frontmatter is the authoritative inventory of managed files and directories.
5. Filesystem helpers manage only tracked package-owned content, without traversing symlinked directories.

## Module map

- `main.rs`
  - Defines CLI commands and flags.
  - Validates HOME boundary before running operations.
  - Triggers self-update flow on `update` unless disabled.

- `lib.rs`
  - Library entrypoint.
  - Exposes `Command` and `run` to execute operations.

- `constants.rs`
  - Shared constants and enums.
  - Defines managed roots: `agents`, `rules`, `instructions`, `skills`.
  - Defines the managed filename prefix and external skill cache location.

- `operations.rs`
  - Core behavior for `install`, `update`, and `debloat`.
  - Builds desired canonical file trees, reconciles tracked paths from `_meta.md`, and rewrites `_meta.md` last.
  - Applies non-destructive managed-file rules.

- `embedded.rs`
  - Reads embedded files from `static/` via `include_dir`.
  - Merges platform-specific and shared trees.
  - Emits canonical platform-relative destinations and rejects duplicate embedded outputs.

- `external_skills.rs`
  - Loads `static/external-skills.toml` for external source definitions.
  - Source contract: `id`, `repository`, `commit`, `path`, optional `platforms`, optional `enabled`, optional `checksum`.
  - Validates source safety constraints such as non-empty `id` and `path`, slash-free `id`, and full SHA commit pinning.
  - Filters by platform using `all` or normalized platform names (`claude`, `copilot`, `cursor`).
  - Requires `SKILL.md` under the configured source path.
  - Downloads enabled sources from pinned GitHub commits and caches files under `~/.cache/ai-dotfiles/external-skills`.
  - On source fetch or parse errors, emits a warning and continues with other sources.
  - Maps external skill files into canonical destinations under `skills/<id>/...`.

- `meta.rs`
  - Renders `_meta.md` from `static/_meta_template.md` plus YAML frontmatter.
  - Parses structured manifests from existing metadata.
  - Falls back to legacy version-only metadata parsing for migration.

- `fs_ops.rs`
  - Resolves tracked files and directories from `_meta.md`.
  - Handles legacy prefixed path discovery used only during migration.
  - Removes empty tracked directories while preserving symlinks.

- `self_update.rs`
  - Checks the latest GitHub release tag.
  - Downloads and installs newer binaries directly from GitHub Releases.
  - Verifies `SHA256SUMS` contains the expected target asset entry before install.
  - Re-executes `update` with `--no-self-update` after a successful upgrade.

- `tests.rs`
  - Integration-style tests for canonical installs, metadata lifecycle, migration, and symlink safety.

## Command behavior details

### Install

- Creates managed files only when no prior metadata is found.
- If an installation is already present, returns a message instructing the user to run `update`.
- Writes canonical embedded and external files across supported platform roots.
- Writes `_meta.md` last so the manifest reflects the final successful state.

### Update

- Bootstraps managed files as a fresh synchronization when no installation exists yet.
- Otherwise:
  - Computes the desired canonical managed set for the current package version from embedded and external sources.
  - Removes stale tracked files from the current manifest.
  - Writes only changed and new files.
  - Rebuilds `_meta.md` from the final desired state.
  - Preserves unmanaged user files.
- Returns `Configuration is already up to date.` only when no tracked paths changed and no migration work was needed.

### Debloat

- Removes only files and directories tracked in `_meta.md`.
- Removes generated `_meta.md` after tracked content is cleaned up.
- Leaves custom unmanaged files untouched.

## Managed path model

Static source layout:

- Platform-specific: `static/.claude`, `static/.copilot`, `static/.cursor`
- Shared: `static/__shared__`

Destination model:

- Managed content is installed as-is under canonical names.
- `_meta.md` is written directly under each platform root and stores the authoritative inventory used by `update` and `debloat`.

Example mapping:

- `static/.claude/agents/rust-specialist.md` -> `~/.claude/agents/rust-specialist.md`
- `static/__shared__/skills/clean-code/SKILL.md` -> `~/.copilot/skills/clean-code/SKILL.md`
- External source `react-best-practices/SKILL.md` -> `~/.claude/skills/react-best-practices/SKILL.md`

## Safety and security guarantees

- CLI operations are constrained to HOME unless explicit override is provided via hidden flags.
- Symlinked directories are never traversed during update or debloat cleanup.
- Self-update is fail-open: network or install errors do not block config synchronization.
- Release checksum lists are verified for expected asset presence before self-update install.
- External cache state remains separate from installed ownership state.

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
2. Keep content under managed roots such as `agents`, `rules` or `instructions`, and `skills`.
3. Validate behavior with `cargo test`.
4. Confirm the new content installs under the intended canonical path and appears in `_meta.md`.

### Cursor global policy template

The repository includes a non-runtime template at `static/.cursor/instructions/user-rules-template.md`.

It is installed to `~/.cursor/instructions/user-rules-template.md` so users can copy it into Cursor Settings -> Rules -> User Rules.
This preserves a personal/global workflow policy without requiring project-level `.cursor/rules` files.
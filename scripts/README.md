# Scripts guide

This document covers repository scripts, when to use them, and how they behave.

## Available scripts

- `release.sh`: Release automation for semantic version tags.
- `smoke-release.sh`: Host-platform smoke validation for one release asset.
- `update.sh`: Thin wrapper around `rafaelcmm-ai-dotfiles update`.

## Prerequisites

Common requirements:

- Bash shell.
- Git configured with repository access.
- Rust toolchain installed (`cargo` in `PATH`). Install guide: https://www.rust-lang.org/tools/install
- Optional: `cargo-binstall` for faster binary install and self-update flows. Install guide: https://github.com/cargo-bins/cargo-binstall#installation

For `release.sh`:

- Clean git working tree.
- Current branch set to `main`.
- Rust toolchain (`cargo fmt`, `cargo clippy`, `cargo test`).
- Push permission to `origin`.

For `smoke-release.sh`:

- `git`, `curl`, `grep`, `sha256sum`.
- `tar` for `tar.gz` assets.
- `unzip` for `zip` assets.

For `update.sh`:

- `rafaelcmm-ai-dotfiles` available in `PATH`.

## Script details

### release.sh

Usage:

```bash
scripts/release.sh <patch|minor|major|X.Y.Z>
scripts/release.sh <patch|minor|major|X.Y.Z> --dry-run
```

What it does:

1. Verifies clean git tree.
2. Verifies current branch is `main`.
3. Computes next version (or uses explicit `X.Y.Z`).
4. Runs validation:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

5. Updates `Cargo.toml` version.
6. Commits with `chore(release): bump version to vX.Y.Z`.
7. Creates tag `vX.Y.Z`.
8. Pushes `main` and tag to `origin`.

Safety notes:

- Refuses to run from a dirty worktree.
- Refuses to run outside `main`.
- Uses argument-safe command execution (no `eval`).
- Supports dry-run mode for non-mutating verification.

### smoke-release.sh

Usage:

```bash
scripts/smoke-release.sh <vX.Y.Z>
scripts/smoke-release.sh <vX.Y.Z> --dry-run
```

What it does:

1. Resolves repository slug from `origin` remote.
2. Detects host OS/architecture and expected release asset name.
3. Downloads target asset and `SHA256SUMS` from GitHub Releases.
4. Verifies checksum for selected asset.
5. Extracts binary and runs `--version`.

Safety notes:

- Validates tag format (`vX.Y.Z`).
- Uses argument-safe command execution (no `eval`).
- Dry-run mode prints commands without downloading/executing.
- `origin` remote must be a GitHub URL ending with `.git` (SSH or HTTPS).

### update.sh

Usage:

```bash
scripts/update.sh [<update-args>...]
```

Examples:

```bash
scripts/update.sh
scripts/update.sh --yes
scripts/update.sh --no-self-update
```

Behavior:

- Delegates directly to:

```bash
rafaelcmm-ai-dotfiles update "$@"
```

- Preserves all CLI update and self-update behavior.

## When to use each script

- Use `release.sh` when preparing and publishing a new version tag.
- Use `smoke-release.sh` after tagging to verify one downloadable artifact on the current host platform.
- Use `update.sh` for a stable script entrypoint in local tooling/automation that should call `update`.

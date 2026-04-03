# rafaelcmm-ai-dotfiles

Rust CLI to install, update and debloat Rafael Monteiro AI configuration files for Claude, Copilot and Cursor.

## Features

- Installs platform configuration from the embedded `static/` tree.
- Merges `static/__shared__/` into each platform during install/update.
- Prefixes managed content with package version (`rafaelcmm-<version>-...`) for safe coexistence.
- Generates `~/.{claude|copilot|cursor}/_meta.md` from `static/_meta_template.md`.
- Non-destructive behavior: unmanaged files are never modified or removed.
- Managed content is identified by names starting with `rafaelcmm-` under `agents`, `rules`, `instructions`, and `skills`.
- Symlinks are never traversed during update/debloat operations.

## Commands

### Install

```bash
rafaelcmm-ai-dotfiles install
```

- Creates versioned managed files in:
  - `~/.claude/agents`, `~/.claude/rules`, `~/.claude/skills`
  - `~/.copilot/agents`, `~/.copilot/instructions`, `~/.copilot/skills`
  - `~/.cursor/agents`, `~/.cursor/rules`, `~/.cursor/skills`
- If an existing installation is detected (via `_meta.md`), exits with a message suggesting `update`.

### Update

```bash
rafaelcmm-ai-dotfiles update
```

- If installed version equals current package version, exits with `Configuration is already up to date.`
- If no previous installation is detected, `update` bootstraps a fresh installation.
- Otherwise:
  - Writes only files that changed.
  - Removes only stale managed files from older versions.
  - Keeps unmanaged files untouched.

### Debloat

```bash
rafaelcmm-ai-dotfiles debloat
```

- Removes only managed files created by this package.
- Removes `_meta.md` only when it is the generated metadata file.
- Leaves any custom/unmanaged user file untouched.

## Local development

### Build

```bash
cargo build
```

### Run

```bash
cargo run -- install
cargo run -- update
cargo run -- debloat
```

### Test coverage

The project includes tests for:

- Managed path versioning.
- Install behavior and generated metadata.
- Idempotent install fallback to update message.
- Update up-to-date detection.
- Debloat safety for unmanaged files.

Run tests with:

```bash
cargo test
```

## How files are versioned

Examples for version `1.0.0`:

- `static/.claude/agents/rust-specialist.md` → `~/.claude/agents/rafaelcmm-1.0.0-rust-specialist.md`
- `static/__shared__/skills/clean-code/SKILL.md` → `~/.copilot/skills/rafaelcmm-1.0.0-clean-code/SKILL.md`

## Creating new configuration files

1. Add platform-specific files under:
   - `static/.claude`
   - `static/.copilot`
   - `static/.cursor`
2. Add shared files under `static/__shared__`.
3. Keep content organized by managed roots:
   - `agents`
   - `rules` (or `instructions` for Copilot)
   - `skills/<skill-name>/SKILL.md`
4. Run `cargo test` to validate behavior.

## Release and distribution model

This project uses a tag-only stable release flow:

- Pushes to `main` run CI only (`fmt`, `clippy`, `test`, `build --release`).
- Releases are created only when a semantic version tag is pushed (for example `v1.2.3`).
- GitHub Releases provide pre-built binaries for supported platforms.

### Maintainer release checklist

1. Ensure local branch is up to date and clean.
2. Run local validation:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

3. Update version in `Cargo.toml`.
4. Commit version bump.
5. Create and push release tag:

```bash
git tag v1.0.1
git push origin main --tags
```

### Release helper script

This repository includes an automation script for the stable tag-only flow:

```bash
scripts/release.sh <patch|minor|major|X.Y.Z>
```

Examples:

```bash
scripts/release.sh patch
scripts/release.sh minor
scripts/release.sh 1.2.3
scripts/release.sh patch --dry-run
```

The script will:

1. Ensure your git tree is clean and on `main`.
2. Run `cargo fmt --check`, `cargo clippy`, and `cargo test`.
3. Bump `Cargo.toml` version.
4. Commit with `chore(release): bump version to vX.Y.Z`.
5. Create tag `vX.Y.Z`.
6. Push `main` and the tag to `origin`.

### Install on another machine with cargo-binstall

Use a release asset URL template. For Linux/macOS assets (`tar.gz`):

```bash
cargo binstall rafaelcmm-ai-dotfiles \
  --pkg-url "https://github.com/rafaelcmm/rafaelcmm-ai-dotfiles/releases/download/v{ version }/rafaelcmm-ai-dotfiles-{ target }.tar.gz" \
  --pkg-fmt tgz
```

For Windows assets (`zip`):

```powershell
cargo binstall rafaelcmm-ai-dotfiles --pkg-url "https://github.com/rafaelcmm/rafaelcmm-ai-dotfiles/releases/download/v{ version }/rafaelcmm-ai-dotfiles-{ target }.zip" --pkg-fmt zip
```

### Verify release artifact integrity

Download `SHA256SUMS` from the same release and verify checksums:

```bash
sha256sum -c SHA256SUMS
```

### Roll back to a previous version

Install using an older tag in the URL template (for example `v1.0.0`).

## Troubleshooting

- `HOME environment variable is not set`:
  - Export `HOME` in the current shell session.
- `refusing to operate outside HOME`:
  - Use a target path inside `HOME`, or pass `--allow-outside-home` only for controlled automation/testing.

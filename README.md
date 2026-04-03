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

## Private package distribution on GitHub

Recommended approach for a private installable CLI:

1. Publish GitHub Releases with Linux/macOS/Windows binaries built by GitHub Actions.
2. Install on any machine with one command using `cargo-binstall`:

```bash
cargo binstall --git https://github.com/rafaelcmm/rafaelcmm-ai-dotfiles rafaelcmm-ai-dotfiles
```

If you need npm-style global install semantics, ship a thin private npm wrapper package that downloads the release binary during `postinstall` and exposes the `rafaelcmm-ai-dotfiles` command.

## Troubleshooting

- `HOME environment variable is not set`:
  - Export `HOME` in the current shell session.
- `refusing to operate outside HOME`:
  - Use a target path inside `HOME`, or pass `--allow-outside-home` only for controlled automation/testing.

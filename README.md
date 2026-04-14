# rafaelcmm-ai-dotfiles

Rust CLI to install, update, and debloat Rafael Monteiro AI configuration files for Claude, Copilot, and Cursor.

## Documentation map

- Technical implementation details: [src/README.md](src/README.md)
- Script usage and release automation: [scripts/README.md](scripts/README.md)

## Prerequisites

- Rust toolchain installed (`cargo` available in `PATH`). Install guide: https://www.rust-lang.org/tools/install
- Network access to GitHub for update checks and release downloads.

## Test split

- `cargo test` runs CI-safe unit and offline high-level tests only.
- Networked GitHub integration tests are marked ignored and run explicitly by `scripts/release.sh` via `cargo test network_integration_ -- --ignored`.

## Installation

Install from a release tag with Cargo:

```bash
cargo install --locked --git https://github.com/rafaelcmm/rafaelcmm-ai-dotfiles.git --tag vX.Y.Z rafaelcmm-ai-dotfiles
```

Install from local source:

```bash
cargo install --path .
```

## Commands

Show version:

```bash
rafaelcmm-ai-dotfiles --version
```

Install managed configuration files:

```bash
rafaelcmm-ai-dotfiles install
```

Each platform root gets an `_meta.md` file whose YAML frontmatter tracks the version plus the files and directories bootstrapped by the CLI. `update` and `debloat` use that manifest as their source of truth.

Update managed configuration files (default behavior includes self-update check):

```bash
rafaelcmm-ai-dotfiles update
```

Update without self-update check:

```bash
rafaelcmm-ai-dotfiles update --no-self-update
```

Auto-confirm self-update prompt during update:

```bash
rafaelcmm-ai-dotfiles update --yes
```

Remove only managed files:

```bash
rafaelcmm-ai-dotfiles debloat
```

Convenience wrapper (equivalent to `update`):

```bash
scripts/update.sh
```

## Cursor deterministic workflow note

Cursor global behavior is most reliable when your workflow policy is set in Cursor Settings -> Rules -> User Rules.

After running `install` or `update`, copy the template from:

- `~/.cursor/instructions/user-rules-template.md`

and paste it into Cursor User Rules.

This keeps your workflow personal/global and avoids committing project-level rule files.
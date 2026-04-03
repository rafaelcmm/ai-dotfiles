# rafaelcmm-ai-dotfiles

Rust CLI to install, update, and debloat Rafael Monteiro AI configuration files for Claude, Copilot, and Cursor.

## Documentation map

- Technical implementation details: [src/README.md](src/README.md)
- Script usage and release automation: [scripts/README.md](scripts/README.md)

## Prerequisites

- Rust toolchain installed (`cargo` available in `PATH`). Install guide: https://www.rust-lang.org/tools/install
- Optional: `cargo-binstall` for faster binary installation and self-update flow. Install guide: https://github.com/cargo-bins/cargo-binstall#installation
- Network access to GitHub for update checks and release downloads.

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

Update managed configuration files (default behavior includes self-update check):

```bash
rafaelcmm-ai-dotfiles update
```

Update without self-update check:

```bash
rafaelcmm-ai-dotfiles update --no-self-update
```

Non-interactive update confirmation:

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

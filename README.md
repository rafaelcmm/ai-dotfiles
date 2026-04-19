# ai-dotfiles

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
cargo install --locked --git https://github.com/rafaelcmm/ai-dotfiles.git --tag vX.Y.Z ai-dotfiles
```

Install from local source:

```bash
cargo install --path .
```

## Commands

Show version:

```bash
ai-dotfiles --version
```

Install managed configuration files:

```bash
ai-dotfiles install
```

Installation now also bootstraps cross-tool shared files under each managed tool root:

- `~/.claude/AGENTS.md`
- `~/.claude/CLAUDE.md` (references `@AGENTS.md`)
- `~/.copilot/AGENTS.md`
- `~/.copilot/CLAUDE.md`
- `~/.cursor/AGENTS.md`
- `~/.cursor/CLAUDE.md`

Skills are installed canonically under `~/.claude/skills` to avoid duplication across tool roots.
Copilot can read Claude skills from this location, matching current interoperability guidance.

Each platform root gets an `_meta.md` file whose YAML frontmatter tracks the version plus the files and directories bootstrapped by the CLI. `update` and `debloat` use that manifest as their source of truth.

Update managed configuration files (default behavior includes self-update check):

```bash
ai-dotfiles update
```

Update without self-update check:

```bash
ai-dotfiles update --no-self-update
```

Auto-confirm self-update prompt during update:

```bash
ai-dotfiles update --yes
```

Remove only managed files:

```bash
ai-dotfiles debloat
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

## Hooks and MCP scaffolding

`install` and `update` provide safe starter scaffolds for hooks and MCP:

- `~/.claude/settings.json` (hooks + `mcpServers`)
- `~/.cursor/hooks.json`
- `~/.cursor/mcp.json`
- `~/.copilot/mcp.json`
- `~/.copilot/hooks/hooks.json`

Shared hook scripts are also installed automatically:

- `~/.claude/.hooks/*.sh`
- `~/.cursor/.hooks/*.sh`
- `~/.copilot/hooks/*.sh`
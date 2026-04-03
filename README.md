# rafaelcmm-ai-dotfiles

Rust CLI to install, update, and debloat Rafael Monteiro AI configuration files for Claude, Copilot, and Cursor.

## Documentation map

- Technical implementation details: [src/README.md](src/README.md)
- Script usage and release automation: [scripts/README.md](scripts/README.md)

## Prerequisites

- Rust toolchain installed (`cargo` available in `PATH`). Install guide: https://www.rust-lang.org/tools/install
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

## External skills bootstrap

This tool can bootstrap external skill folders during `install` and `update`.

- Source manifest: `static/external-skills.toml`
- Current bundled external sources:
	- `react-best-practices` from `vercel-labs/agent-skills`
	- `next-best-practices` from `vercel-labs/next-skills`
- Sources are pinned by commit SHA for reproducible installs.
- Downloaded external files are cached under:
	- `~/.cache/rafaelcmm-ai-dotfiles/external-skills`
- External source fetch is fail-open: if one source fails, install/update continues and prints a warning to stderr.
- A source is considered valid only if `SKILL.md` exists under the configured source path.

External skills are installed as managed files under `skills/rafaelcmm-<version>-<skill-id>/...`, so they participate in both update reconciliation and `debloat` cleanup just like embedded managed content.

Force refresh external skill cache:

```bash
rm -rf ~/.cache/rafaelcmm-ai-dotfiles/external-skills
rafaelcmm-ai-dotfiles update --no-self-update
```

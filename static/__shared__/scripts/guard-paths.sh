#!/usr/bin/env bash
# guard-paths.sh — Refuse edits to paths an agent has no business touching.
#
# Reads the hook event as JSON on stdin. Works across:
#   - Claude Code  (PreToolUse:  Edit|Write|MultiEdit)
#   - Cursor       (afterFileEdit — advisory only; see README)
#   - Copilot      (preToolUse:  edit|create)

set -u

if ! command -v jq >/dev/null 2>&1; then
  echo "guard-paths.sh: jq is required but not installed." >&2
  echo "Install with: brew install jq  |  apt install jq" >&2
  exit 2  # fail closed — refuse the write rather than silently allow
fi

INPUT="$(cat)"

# Pull the target path from whichever field the tool uses.
FILE="$(
  printf '%s' "$INPUT" | jq -r '
    .tool_input.file_path // .toolArgs.file_path // .file_path // empty
  ' 2>/dev/null
)"

# No path in payload → nothing to guard, allow.
[ -z "$FILE" ] && exit 0

# Resolve to absolute form so `../` tricks don't slip through.
# `realpath -m` tolerates non-existent files (new writes).
ABS="$(realpath -m "$FILE" 2>/dev/null || printf '%s' "$FILE")"

# Claude Code keeps per-project memory outside every repository by design, so the
# repo-root check below would block every memory write made from inside a repo.
# Allow exactly one shape: a Markdown file directly inside
# ~/.claude/projects/<project-slug>/memory/. ABS is already resolved, so `..` and
# symlinks cannot reach anything else through this branch.
# The root is resolved the same way as ABS, so a symlinked HOME (macOS /var) still matches.
MEMORY_ROOT="$(realpath -m "${HOME:-/nonexistent}/.claude/projects" 2>/dev/null)/"
if [ -n "${HOME:-}" ] && [[ "$ABS" == "$MEMORY_ROOT"* ]]; then
  MEMORY_REL="${ABS#"$MEMORY_ROOT"}"
  if [[ "$MEMORY_REL" =~ ^[^/]+/memory/[^/]+\.md$ ]]; then
    exit 0
  fi
fi

# If we're in a git repo, refuse writes outside it. This catches the
# occasional agent that decides to edit your ~/.zshrc.
REPO_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || true)"
if [ -n "$REPO_ROOT" ] && [[ "$ABS" != "$REPO_ROOT"* ]]; then
  echo "BLOCKED: $FILE is outside the repository root ($REPO_ROOT)." >&2
  exit 2
fi

# Paths matched against the basename or the repo-relative path.
REL="${ABS#"$REPO_ROOT"/}"

# Blocklist. Glob-style patterns, matched with bash's `==`.
BLOCKED_PATTERNS=(
  '.git/*'        '*/.git/*'
  '.ssh/*'        '*/.ssh/*'
  '*.pem'         '*.key'          '*id_rsa*'       '*id_ed25519*'
  'node_modules/*'   '*/node_modules/*'
  '.venv/*'          '*/.venv/*'      'venv/*'       '*/venv/*'
  '__pycache__/*'    '*/__pycache__/*'
)

for pat in "${BLOCKED_PATTERNS[@]}"; do
  # shellcheck disable=SC2053  # we want glob matching on the RHS
  if [[ "$REL" == $pat ]] || [[ "$ABS" == *"/$pat" ]]; then
    echo "BLOCKED: $FILE matches protected pattern '$pat'." >&2
    echo "If you really need to edit this, do it outside the agent." >&2
    exit 2
  fi
done

# Lockfiles get a softer treatment — warn but allow. Agents legitimately
# need to touch these during `npm install` etc., but silent edits are sus.
LOCKFILES=(
  'package-lock.json' 'yarn.lock' 'pnpm-lock.yaml'
  'Cargo.lock' 'poetry.lock' 'uv.lock' 'Gemfile.lock' 'go.sum'
)
BASENAME="$(basename "$ABS")"
for lf in "${LOCKFILES[@]}"; do
  if [ "$BASENAME" = "$lf" ]; then
    echo "NOTE: editing lockfile $BASENAME — make sure this is intentional." >&2
    # Do not exit 2; just surface the warning.
    break
  fi
done

exit 0
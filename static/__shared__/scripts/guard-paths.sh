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

# Resolves a path to absolute form with `..` and symlinks removed, tolerating
# components that do not exist yet (new writes). Uses `realpath -m` (GNU coreutils)
# and falls back to Python's non-strict os.path.realpath where `-m` is missing
# (stock macOS, busybox). Prints nothing and fails when neither is available.
resolve() {
  realpath -m -- "$1" 2>/dev/null && return 0
  command -v python3 >/dev/null 2>&1 &&
    python3 -I -c 'import os, sys; print(os.path.realpath(sys.argv[1]))' "$1" 2>/dev/null
}

# Every check below compares resolved paths; an unresolved target could carry `..`
# past them, so refuse the write instead.
ABS="$(resolve "$FILE")"
if [ -z "$ABS" ]; then
  echo "guard-paths.sh: cannot resolve $FILE (install coreutils for realpath -m, or python3)." >&2
  exit 2
fi
HOME_ABS=""
[ -n "${HOME:-}" ] && HOME_ABS="$(resolve "$HOME")"

# Writes that fall in one of the exemptions below skip the repository-root check.
# They still go through the protected-pattern blocklist at the end.
EXEMPT=false

# Claude Code keeps per-project memory outside every repository by design, so the
# repo-root check below would block every memory write made from inside a repo.
# Allow exactly one shape: a Markdown file directly inside
# ~/.claude/projects/<project-slug>/memory/. ABS is already resolved, so `..` and
# symlinks cannot reach anything else through this branch, and the root is resolved
# the same way, so a symlinked HOME (macOS /var) still matches.
if [ -n "$HOME_ABS" ] && [ "$HOME_ABS" != "/" ]; then
  MEMORY_ROOT="$HOME_ABS/.claude/projects/"
  if [[ "$ABS" == "$MEMORY_ROOT"* ]] &&
    [[ "${ABS#"$MEMORY_ROOT"}" =~ ^[^/]+/memory/[^/]+\.md$ ]]; then
    EXEMPT=true
  fi
  # Plan mode saves plans as Markdown files directly inside ~/.claude/plans/: the
  # same flat-file rule, so nothing else under ~/.claude becomes writable.
  PLANS_ROOT="$HOME_ABS/.claude/plans/"
  if [[ "$ABS" == "$PLANS_ROOT"* ]] && [[ "${ABS#"$PLANS_ROOT"}" =~ ^[^/]+\.md$ ]]; then
    EXEMPT=true
  fi
fi

# Claude Code gives each session a scratchpad for temporary files at
# <tmp>/claude-<uid>/<project-slug>/<session-id>/scratchpad/. Allow any file under a
# `scratchpad` directory at exactly that depth, for the current user only. <tmp> is
# each non-empty one of CLAUDE_CODE_TMPDIR and TMPDIR, plus /tmp. The claude-<uid> folder
# must be a real directory (not a symlink), owned by the current user and not writable by
# group or others: on a shared /tmp another account could otherwise pre-create or share
# it and collect or plant files there. `find -perm -mode` is POSIX, so this holds on
# GNU and BSD alike. A symlink
# planted inside the scratchpad resolves to its real target and no longer matches.
SCRATCH_UID="$(id -u 2>/dev/null || true)"
if [ "$EXEMPT" = false ] && [ -n "$SCRATCH_UID" ]; then
  for tmp_base in "${CLAUDE_CODE_TMPDIR:-}" "${TMPDIR:-}" /tmp; do
    [ -z "$tmp_base" ] && continue
    scratch_dir="$tmp_base/claude-$SCRATCH_UID"
    if [ ! -d "$scratch_dir" ] || [ -L "$scratch_dir" ] || [ ! -O "$scratch_dir" ]; then
      continue
    fi
    if [ -n "$(find "$scratch_dir" -maxdepth 0 \( -perm -0020 -o -perm -0002 \) 2>/dev/null)" ]; then
      continue
    fi
    scratch_root="$(resolve "$scratch_dir")"
    if [ -z "$scratch_root" ] || [ "$scratch_root" = "/" ]; then
      continue
    fi
    if [[ "$ABS" == "$scratch_root/"* ]] &&
      [[ "${ABS#"$scratch_root/"}" =~ ^[^/]+/[^/]+/scratchpad/.+ ]]; then
      EXEMPT=true
      break
    fi
  done
fi

# Claude Code exports CLAUDE_PROJECT_DIR, the directory the session was started in. A
# session started in a folder of several repositories works across all of them, so a
# write anywhere under that folder is in scope even when the agent's working directory
# sits in one repository. The folder is trusted only when it lies strictly inside
# $HOME, has no hidden component below $HOME (.ssh, .claude, .config, ...), and holds
# the working directory the hook payload reports (`cwd`, else the hook's own). That rules out $HOME itself and everything above
# it, system folders, and a value that does not describe this session.
PROJECT_ROOT=""
if [ "$EXEMPT" = false ] && [ -n "${CLAUDE_PROJECT_DIR:-}" ] &&
  [ -n "$HOME_ABS" ] && [ "$HOME_ABS" != "/" ]; then
  PROJECT_ROOT="$(resolve "$CLAUDE_PROJECT_DIR")"
  # The strip leaves PROJECT_ROOT unchanged when it is $HOME itself or outside $HOME; the
  # check below reads that as "not strictly inside $HOME".
  PROJECT_BELOW_HOME="${PROJECT_ROOT#"$HOME_ABS/"}"
  HOOK_CWD="$(printf '%s' "$INPUT" | jq -r '.cwd // empty' 2>/dev/null)"
  HOOK_CWD="$(resolve "${HOOK_CWD:-$PWD}")"
  if [ -z "$PROJECT_ROOT" ] || [ "$PROJECT_BELOW_HOME" = "$PROJECT_ROOT" ] ||
    [[ "/$PROJECT_BELOW_HOME" == */.* ]] ||
    { [ "$HOOK_CWD" != "$PROJECT_ROOT" ] && [[ "$HOOK_CWD" != "$PROJECT_ROOT/"* ]]; }; then
    PROJECT_ROOT=""
  fi
fi
if [ -n "$PROJECT_ROOT" ] && { [ "$ABS" = "$PROJECT_ROOT" ] || [[ "$ABS" == "$PROJECT_ROOT/"* ]]; }; then
  EXEMPT=true
fi

# In a git repo, refuse writes outside it unless an exemption above applies. This
# catches the occasional agent that decides to edit your ~/.zshrc. The trailing slash
# keeps a sibling such as <repo>-other from passing as inside <repo>.
REPO_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || true)"
if [ "$EXEMPT" = false ] && [ -n "$REPO_ROOT" ] &&
  [[ "$ABS" != "$REPO_ROOT" ]] && [[ "$ABS" != "$REPO_ROOT/"* ]]; then
  echo "BLOCKED: $FILE is outside the repository root ($REPO_ROOT)." >&2
  exit 2
fi

# Each pattern is tried against REL (repository-relative when the target is inside the
# repository) and as a suffix of the full resolved path (`*/$pat`), so a protected folder
# above the repository or project root, such as a project inside .ssh, is caught too.
# Key files are matched on the file name alone: in bash `[[ == ]]` a `*` also crosses
# `/`, so a path-wide pattern such as `*id_rsa*` would block a whole repository whose
# folder name merely contains it.
REL="${ABS#"$REPO_ROOT"/}"

# Blocklist. Glob-style patterns, matched with bash's `==`.
BLOCKED_PATTERNS=(
  '.git/*'        '*/.git/*'
  '.ssh/*'        '*/.ssh/*'
  '.gnupg/*'      '*/.gnupg/*'
  'node_modules/*'   '*/node_modules/*'
  '.venv/*'          '*/.venv/*'      'venv/*'       '*/venv/*'
  '__pycache__/*'    '*/__pycache__/*'
)

for pat in "${BLOCKED_PATTERNS[@]}"; do
  # shellcheck disable=SC2053  # we want glob matching on the RHS
  if [[ "$REL" == $pat ]] || [[ "$ABS" == */$pat ]]; then
    echo "BLOCKED: $FILE matches protected pattern '$pat'." >&2
    echo "If you really need to edit this, do it outside the agent." >&2
    exit 2
  fi
done

KEY_FILE_PATTERNS=('*.pem' '*.key' 'id_rsa*' 'id_ed25519*')
TARGET_NAME="${ABS##*/}"
for pat in "${KEY_FILE_PATTERNS[@]}"; do
  # shellcheck disable=SC2053  # we want glob matching on the RHS
  if [[ "$TARGET_NAME" == $pat ]]; then
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
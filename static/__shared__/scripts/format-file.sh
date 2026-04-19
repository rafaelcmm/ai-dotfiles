#!/usr/bin/env bash
# format-file.sh — Auto-format a file after the agent writes to it.
#
# Runs on PostToolUse / afterFileEdit / postToolUse. Detection is purely
# by extension + tool availability; no project config required.
#
# Philosophy: silent success, silent skip. Only speak up on real errors.

set -u

# No jq → can't parse the payload, skip silently. Formatting is optional so
# we fail open here (unlike the blocking hooks which fail closed).
command -v jq >/dev/null 2>&1 || exit 0

INPUT="$(cat)"

FILE="$(
  printf '%s' "$INPUT" | jq -r '
    .tool_input.file_path // .toolArgs.file_path // .file_path // empty
  ' 2>/dev/null
)"

# No path or file vanished → nothing to do.
[ -z "$FILE" ] && exit 0
[ -f "$FILE" ] || exit 0

# Run a formatter only if it's installed. `have` is a tiny helper.
have() { command -v "$1" >/dev/null 2>&1; }

# `>/dev/null 2>&1` on the formatter keeps hook output clean on success.
# Errors still go to stderr via the explicit redirect.
case "$FILE" in
  *.py)
    if   have ruff;   then ruff format "$FILE" >/dev/null 2>&1 || true
    elif have black;  then black -q "$FILE"     >/dev/null 2>&1 || true
    fi
    ;;
  *.js|*.jsx|*.ts|*.tsx|*.mjs|*.cjs|*.json|*.jsonc|*.css|*.scss|*.html|*.md|*.yaml|*.yml)
    if   have biome;    then biome format --write "$FILE" >/dev/null 2>&1 || true
    elif have prettier; then prettier --write --log-level=silent "$FILE" >/dev/null 2>&1 || true
    fi
    ;;
  *.go)
    have gofmt && gofmt -w "$FILE" >/dev/null 2>&1 || true
    ;;
  *.rs)
    have rustfmt && rustfmt --edition 2021 "$FILE" >/dev/null 2>&1 || true
    ;;
  *.sh|*.bash)
    have shfmt && shfmt -w "$FILE" >/dev/null 2>&1 || true
    ;;
  *.rb)
    have rubocop && rubocop -a --force-exclusion "$FILE" >/dev/null 2>&1 || true
    ;;
  *.tf|*.tfvars)
    have terraform && terraform fmt "$FILE" >/dev/null 2>&1 || true
    ;;
esac

# Always exit 0 — formatting failures shouldn't block the agent.
exit 0
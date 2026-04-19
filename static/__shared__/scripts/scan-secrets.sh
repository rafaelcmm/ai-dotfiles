#!/usr/bin/env bash
# scan-secrets.sh — Block writes/edits that contain obvious secrets.
#
# Reads the hook event as JSON on stdin. Works across:
#   - Claude Code  (PreToolUse:  Edit|Write|MultiEdit)
#   - Cursor       (afterFileEdit — advisory only; see README)
#   - Copilot      (preToolUse:  edit|create)
#
# Exit 0 = allow, exit 2 = block (stderr is shown to the agent).

set -u

if ! command -v jq >/dev/null 2>&1; then
  echo "scan-secrets.sh: jq is required but not installed." >&2
  echo "Install with: brew install jq  |  apt install jq" >&2
  exit 2  # fail closed — refuse the write rather than silently allow
fi

INPUT="$(cat)"

# Extract the content the agent is trying to write. Each tool uses slightly
# different field names, so we try the common ones and concatenate whatever
# we find. `jq -r // empty` keeps us quiet when a field is absent.
CONTENT="$(
  printf '%s' "$INPUT" | jq -r '
    [
      .tool_input.content       // empty,
      .tool_input.new_string    // empty,
      .tool_input.file_text     // empty,
      .toolArgs.content         // empty,
      .toolArgs.new_string      // empty,
      (.edits // [] | map(.new_string) | join("\n"))
    ] | join("\n")
  ' 2>/dev/null
)"

# Nothing to scan → allow.
[ -z "${CONTENT// /}" ] && exit 0

# Patterns are deliberately conservative to keep false positives low.
# Add more as your threat model demands, but resist the urge to get clever —
# an over-eager scanner gets disabled within a week.
PATTERNS=(
  'AKIA[0-9A-Z]{16}'                              # AWS access key
  'aws_secret_access_key[[:space:]]*=[[:space:]]*[A-Za-z0-9/+=]{40}'
  'sk-[a-zA-Z0-9]{32,}'                           # OpenAI / Anthropic-style
  'sk-ant-[a-zA-Z0-9_-]{32,}'                     # Anthropic specifically
  'ghp_[A-Za-z0-9]{36}'                           # GitHub personal token
  'gho_[A-Za-z0-9]{36}'                           # GitHub OAuth token
  'xox[baprs]-[A-Za-z0-9-]{10,}'                  # Slack token
  '-----BEGIN[[:space:]]+(RSA|EC|OPENSSH|PGP|DSA)?[[:space:]]*PRIVATE KEY-----'
  'eyJ[A-Za-z0-9_-]{20,}\.[A-Za-z0-9_-]{20,}\.[A-Za-z0-9_-]{20,}'  # JWT
)

for pat in "${PATTERNS[@]}"; do
  if printf '%s' "$CONTENT" | grep -Eq "$pat"; then
    echo "BLOCKED: content appears to contain a secret (pattern: $pat)." >&2
    echo "If this is a false positive, refactor the value out of the diff" >&2
    echo "or add the pattern to an allowlist." >&2
    exit 2
  fi
done

exit 0
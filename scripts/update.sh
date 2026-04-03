#!/usr/bin/env bash

# Prevent sourcing: this script is intended to run as an executable only.
if [[ ( -n "${BASH_SOURCE[0]-}" && "${BASH_SOURCE[0]}" != "$0" ) || ( -n "${ZSH_EVAL_CONTEXT-}" && "${ZSH_EVAL_CONTEXT}" == *:file ) ]]; then
  printf '[update] ERROR: do not source this script; run it as ./scripts/update.sh ...\n' >&2
  return 1 2>/dev/null || exit 1
fi

set -euo pipefail

if [[ $# -gt 0 && ("$1" == "-h" || "$1" == "--help") ]]; then
  cat <<'EOF'
Usage:
  scripts/update.sh [<update-args>...]

Examples:
  scripts/update.sh
  scripts/update.sh --yes
  scripts/update.sh --no-self-update

This is a thin wrapper around:
  rafaelcmm-ai-dotfiles update [args]
EOF
  exit 0
fi

exec rafaelcmm-ai-dotfiles update "$@"

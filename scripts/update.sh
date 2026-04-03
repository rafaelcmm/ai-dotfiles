#!/usr/bin/env bash
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

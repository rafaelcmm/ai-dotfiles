#!/usr/bin/env bash

# Release artifact smoke validation for the current host platform.
# - run_cmd executes argv directly (no eval).
# - Keep arguments passed as separate, quoted words at call sites.
# - Never build command arguments from unvalidated user-controlled input.
# - Download one release artifact plus SHA256SUMS for a tag
# - Verify artifact integrity via sha256sum
# - Extract and execute binary --version as a runtime smoke check
#
# Prerequisites:
# - git, curl, grep, sha256sum
# - tar for .tar.gz assets and unzip for .zip assets
#
# Notes:
# - This script validates one host-specific artifact per invocation.
# - Use --dry-run to inspect commands without downloading/executing.
#
# Security contract:
# - run_cmd executes commands via "$@" expansion (not eval).
# - All arguments are passed as individual parameters, preventing injection.
# - Never pass unvalidated user-controlled data to command constructors.

# Prevent sourcing: this script is intended to run as an executable only.
if [[ ( -n "${BASH_SOURCE[0]-}" && "${BASH_SOURCE[0]}" != "$0" ) || ( -n "${ZSH_EVAL_CONTEXT-}" && "${ZSH_EVAL_CONTEXT}" == *:file ) ]]; then
  printf '[smoke] ERROR: do not source this script; run it as ./scripts/smoke-release.sh ...\n' >&2
  return 1 2>/dev/null || exit 1
fi

set -euo pipefail

usage() {
  cat <<'EOF'
Usage:
  scripts/smoke-release.sh <vX.Y.Z> [--dry-run]

Examples:
  scripts/smoke-release.sh v1.0.1
  scripts/smoke-release.sh v1.0.1 --dry-run

What this script does:
  1. Detects current OS/arch and expected release asset name
  2. Downloads release asset and SHA256SUMS from GitHub Releases
  3. Verifies checksum for the selected asset
  4. Extracts binary and runs --version smoke check
EOF
}

log() {
  printf '[smoke] %s\n' "$*"
}

die() {
  printf '[smoke] ERROR: %s\n' "$*" >&2
  exit 1
}

run_cmd() {
  if [[ "$DRY_RUN" == "true" ]]; then
    printf '[dry-run]'
    printf ' %q' "$@"
    printf '\n'
  else
    "$@"
  fi
}

ensure_semver_tag() {
  [[ "$1" =~ ^v[0-9]+\.[0-9]+\.[0-9]+$ ]] || die "tag must be in vX.Y.Z format"
}

detect_target() {
  local os arch
  os="$(uname -s)"
  arch="$(uname -m)"

  case "$os" in
    Linux)
      case "$arch" in
        x86_64) TARGET="x86_64-unknown-linux-gnu"; ASSET_EXT="tar.gz"; BIN_NAME="ai-dotfiles" ;;
        *) die "unsupported Linux arch: $arch" ;;
      esac
      ;;
    Darwin)
      case "$arch" in
        arm64|aarch64) TARGET="aarch64-apple-darwin"; ASSET_EXT="tar.gz"; BIN_NAME="ai-dotfiles" ;;
        *) die "unsupported macOS arch: $arch" ;;
      esac
      ;;
    MINGW*|MSYS*|CYGWIN*)
      case "$arch" in
        x86_64) TARGET="x86_64-pc-windows-msvc"; ASSET_EXT="zip"; BIN_NAME="ai-dotfiles.exe" ;;
        *) die "unsupported Windows arch: $arch" ;;
      esac
      ;;
    *) die "unsupported OS: $os" ;;
  esac
}

resolve_repo_slug() {
  local remote
  remote="$(git remote get-url origin 2>/dev/null || true)"
  [[ -n "$remote" ]] || die "could not read origin remote"

  if [[ "$remote" =~ ^git@github.com:([^/]+)/([^/]+)\.git$ ]]; then
    REPO_SLUG="${BASH_REMATCH[1]}/${BASH_REMATCH[2]}"
    return
  fi

  if [[ "$remote" =~ ^https://github.com/([^/]+)/([^/]+)\.git$ ]]; then
    REPO_SLUG="${BASH_REMATCH[1]}/${BASH_REMATCH[2]}"
    return
  fi

  die "unsupported origin format: $remote"
}

download_artifacts() {
  local base_url
  base_url="https://github.com/${REPO_SLUG}/releases/download/${TAG}"
  ASSET_NAME="ai-dotfiles-${TARGET}.${ASSET_EXT}"

  log "repo:   ${REPO_SLUG}"
  log "tag:    ${TAG}"
  log "target: ${TARGET}"
  log "asset:  ${ASSET_NAME}"

  run_cmd curl -fsSL -o "$WORKDIR/${ASSET_NAME}" "${base_url}/${ASSET_NAME}"
  run_cmd curl -fsSL -o "$WORKDIR/SHA256SUMS" "${base_url}/SHA256SUMS"
}

verify_checksum() {
  if [[ "$DRY_RUN" == "true" ]]; then
    printf '[dry-run] grep %q %q > %q\n' "  ${ASSET_NAME}" "$WORKDIR/SHA256SUMS" "$WORKDIR/${ASSET_NAME}.sha"
    printf '[dry-run] (cd %q && sha256sum -c %q)\n' "$WORKDIR" "${ASSET_NAME}.sha"
    return
  fi

  grep "  ${ASSET_NAME}" "$WORKDIR/SHA256SUMS" > "$WORKDIR/${ASSET_NAME}.sha"
  (cd "$WORKDIR" && sha256sum -c "${ASSET_NAME}.sha")
}

extract_and_run() {
  run_cmd mkdir -p "$WORKDIR/unpack"

  if [[ "$ASSET_EXT" == "tar.gz" ]]; then
    run_cmd tar -xzf "$WORKDIR/${ASSET_NAME}" -C "$WORKDIR/unpack"
  else
    run_cmd unzip -q "$WORKDIR/${ASSET_NAME}" -d "$WORKDIR/unpack"
  fi

  local binary_path
  binary_path="$WORKDIR/unpack/$BIN_NAME"
  run_cmd "$binary_path" --version
}

main() {
  if [[ $# -lt 1 || $# -gt 2 ]]; then
    usage
    exit 1
  fi

  TAG="$1"
  DRY_RUN="false"

  if [[ $# -eq 2 ]]; then
    [[ "$2" == "--dry-run" ]] || die "second argument must be --dry-run"
    DRY_RUN="true"
  fi

  ensure_semver_tag "$TAG"
  detect_target
  resolve_repo_slug

  WORKDIR="$(mktemp -d)"
  trap 'rm -rf "$WORKDIR"' EXIT

  [[ "$DRY_RUN" == "true" ]] && log "mode: dry-run"

  download_artifacts
  verify_checksum
  extract_and_run

  log "smoke check completed"
}

main "$@"

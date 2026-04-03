#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage:
  scripts/release.sh <patch|minor|major|X.Y.Z> [--dry-run]

Examples:
  scripts/release.sh patch
  scripts/release.sh minor
  scripts/release.sh 1.2.3
  scripts/release.sh patch --dry-run

What this script does:
  1. Verifies clean git tree on main
  2. Computes next version (or uses explicit X.Y.Z)
  3. Runs cargo validation (fmt, clippy, test)
  4. Bumps Cargo.toml version
  5. Commits version bump and creates tag vX.Y.Z
  6. Pushes main and tag to origin
EOF
}

log() {
  printf '[release] %s\n' "$*"
}

safe_exit() {
  local code="$1"
  local bash_source="${BASH_SOURCE[0]-}"

  if [[ -n "$bash_source" && "$bash_source" != "$0" ]]; then
    return "$code"
  fi

  if [[ -n "${ZSH_VERSION-}" && "${ZSH_EVAL_CONTEXT-}" == *:file ]]; then
    return "$code"
  fi

  exit "$code"
}

die() {
  printf '[release] ERROR: %s\n' "$*" >&2
  safe_exit 1
}

run_cmd() {
  if [[ "$DRY_RUN" == "true" ]]; then
    printf '[dry-run] %s\n' "$*"
  else
    eval "$*"
  fi
}

ensure_clean_tree() {
  if [[ -n "$(git status --porcelain)" ]]; then
    die "working tree is not clean; commit or stash changes first"
  fi
}

ensure_on_main() {
  local branch
  branch="$(git rev-parse --abbrev-ref HEAD)"
  if [[ "$branch" != "main" ]]; then
    die "current branch is '$branch'; switch to 'main' first"
  fi
}

read_current_version() {
  awk '
    BEGIN { in_package=0 }
    /^\[package\]/ { in_package=1; next }
    /^\[/ && $0 != "[package]" { in_package=0 }
    in_package && /^version[[:space:]]*=[[:space:]]*"[0-9]+\.[0-9]+\.[0-9]+"/ {
      gsub(/version[[:space:]]*=[[:space:]]*"/, "", $0)
      gsub(/".*/, "", $0)
      print $0
      exit
    }
  ' Cargo.toml
}

bump_semver() {
  local current="$1"
  local kind="$2"
  local major minor patch

  IFS='.' read -r major minor patch <<< "$current"

  case "$kind" in
    patch)
      patch=$((patch + 1))
      ;;
    minor)
      minor=$((minor + 1))
      patch=0
      ;;
    major)
      major=$((major + 1))
      minor=0
      patch=0
      ;;
    *)
      die "invalid bump kind '$kind'"
      ;;
  esac

  printf '%s.%s.%s\n' "$major" "$minor" "$patch"
}

is_valid_semver() {
  [[ "$1" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]
}

update_cargo_version() {
  local new_version="$1"
  if [[ "$DRY_RUN" == "true" ]]; then
    log "would update Cargo.toml version to $new_version"
    return
  fi

  awk -v new_version="$new_version" '
    BEGIN { in_package=0; updated=0 }
    /^\[package\]/ { in_package=1; print; next }
    /^\[/ && $0 != "[package]" { in_package=0; print; next }
    in_package && !updated && /^version[[:space:]]*=[[:space:]]*"[0-9]+\.[0-9]+\.[0-9]+"/ {
      print "version = \"" new_version "\""
      updated=1
      next
    }
    { print }
    END {
      if (!updated) {
        exit 2
      }
    }
  ' Cargo.toml > Cargo.toml.tmp || die "failed to update Cargo.toml version"

  mv Cargo.toml.tmp Cargo.toml
}

run_checks() {
  run_cmd "cargo fmt --check"
  run_cmd "cargo clippy --all-targets --all-features -- -D warnings"
  run_cmd "cargo test"
}

commit_and_tag() {
  local next="$1"
  local tag="v$next"

  run_cmd "git add Cargo.toml"
  run_cmd "git commit -m \"chore(release): bump version to $tag\""
  run_cmd "git tag $tag"
}

push_release() {
  local next="$1"
  local tag="v$next"

  run_cmd "git push origin main"
  run_cmd "git push origin $tag"
}

main() {
  if [[ $# -lt 1 || $# -gt 2 ]]; then
    printf '[release] ERROR: missing or invalid arguments\n' >&2
    usage
    safe_exit 1
  fi

  local target="$1"
  DRY_RUN="false"

  if [[ $# -eq 2 ]]; then
    if [[ "$2" != "--dry-run" ]]; then
      usage
      safe_exit 1
    fi
    DRY_RUN="true"
  fi

  ensure_clean_tree
  ensure_on_main

  local current next
  current="$(read_current_version)"
  [[ -n "$current" ]] || die "could not read current version from Cargo.toml"

  if [[ "$target" == "major" || "$target" == "minor" || "$target" == "patch" ]]; then
    next="$(bump_semver "$current" "$target")"
  else
    is_valid_semver "$target" || die "explicit version must be in X.Y.Z format"
    next="$target"
  fi

  if [[ "$next" == "$current" ]]; then
    die "next version equals current version ($current)"
  fi

  if git rev-parse -q --verify "refs/tags/v$next" >/dev/null; then
    die "tag v$next already exists"
  fi

  log "current version: $current"
  log "next version:    $next"
  [[ "$DRY_RUN" == "true" ]] && log "mode:            dry-run"

  run_checks
  update_cargo_version "$next"
  commit_and_tag "$next"
  push_release "$next"

  log "release tag v$next pushed successfully"
}

main "$@"
